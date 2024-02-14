mod diff;
mod gdb;
mod patch;
mod program;

use clap::Parser;
use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};
use paris::error;
use ratatui::{
    prelude::{CrosstermBackend, Stylize, Terminal},
    widgets::Paragraph,
};
use std::io::stdout;
use std::sync::mpsc::{channel, TryRecvError};
use std::{path::PathBuf, time::Duration};

/// Hot code loading (dynamic software updating) for Nintendo 64 development
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Build system command to run (e.g. `make`, `ninja`, `libdragon build`)
    #[clap(short, long)]
    build: String,

    /// ELF file that is output from build command
    #[clap(short, long)]
    elf: PathBuf,

    /// Source files and/or directories to recursively watch for changes
    #[clap(short, long)]
    src: Vec<PathBuf>,

    // TODO: support attaching to existing emulator
    /// Emulator command (e.g. `ares rom.z64`)
    #[clap(short = 'x', long)]
    emulator: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_panic_handler();

    let args = Args::parse();

    // Build the project
    run_build_command(&args.build)?;

    // Spawn emulator
    let mut emulator = std::process::Command::new("sh")
        .arg("-c")
        .arg(&args.emulator)
        .spawn()?;

    // Wait for port to open
    // TODO: make this better
    for _ in 0..10 {
        if std::net::TcpStream::connect("[::1]:9123").is_ok() {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    // Connect to GDB server
    let mut gdb = gdb::Gdb::new("[::1]:9123")?;

    // Parse ELF file
    let elf_file = std::fs::read(&args.elf)?;
    let program = program::Program::new(&elf_file)?;
    println!("Loaded {} items", program.items.len());
    program.items["main"].print_hex();

    // Watch for source changes
    println!("Watching {:?} for changes", args.src);
    let (tx, rx) = channel();
    let mut debouncer = new_debouncer(Duration::from_millis(10), tx)?;
    let watcher = debouncer.watcher();
    for path in args.src {
        watcher.watch(&path, RecursiveMode::Recursive)?;
    }

    // Everything is ready.

    // Initialise TUI
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    loop {
        terminal.draw(|frame| {
            let area = frame.size();
            frame.render_widget(
                Paragraph::new("Hello Ratatui! (press 'q' to quit)")
                    .white()
                    .on_blue(),
                area,
            );
        })?;

        // Check for exit
        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                match (key.kind, key.code) {
                    (KeyEventKind::Press, KeyCode::Char('q')) => break,
                    (KeyEventKind::Press, KeyCode::Esc) => break,
                    (KeyEventKind::Press, KeyCode::Char('c'))
                        if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                    {
                        break
                    }
                    _ => {}
                }
            }
        }

        // Check for filesystem change; if so, rebuild the project
        match rx.try_recv() {
            Ok(Ok(_)) => {
                // TODO: thread

                // Rebuild the project
                if let Err(error) = run_build_command(&args.build) {
                    error!("{}", error);
                    continue;
                }

                // Reload the program
                let elf_file = std::fs::read(&args.elf)?;
                let new_program = program::Program::new(&elf_file)?;
                println!("New program! Loaded {} items", new_program.items.len());
                program.items["main"].print_hex();

                let diff = diff::diff(&program, &new_program);
                if let Err(error) = patch::apply(&mut gdb, &diff) {
                    error!("{}", error);
                    continue;
                }

                // TODO: solve lifetimes
                //program = new_program;
            }
            Ok(Err(error)) => {
                // TODO: do this with ratatui
                error!("fs error: {}", error);
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                error!("filesystem died");
                break;
            }
        }
    }

    emulator.kill()?;
    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}

fn run_build_command(command: &str) -> Result<(), Box<dyn std::error::Error>> {
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .status()?;
    if !status.success() {
        return Err("Build failed".into());
    }
    Ok(())
}

/// Set up a panic handler that restores the terminal state before exiting.
// https://ratatui.rs/how-to/develop-apps/panic-hooks/
// TODO: also close emulator
fn setup_panic_handler() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen).unwrap();
        crossterm::terminal::disable_raw_mode().unwrap();
        original_hook(panic_info);
    }));
}
