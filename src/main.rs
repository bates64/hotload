mod diff;
mod emulator;
mod gdb;
mod patch;
mod program;

use clap::Parser;
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};
use paris::error; // TODO: stop using paris, use a ratatui widget
use std::sync::mpsc::channel;
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

    run_build_command(&args.build)?;

    emulator::spawn(&args.emulator);

    // Wait for port to open
    println!("Waiting for GDB server...");
    loop {
        if std::net::TcpStream::connect("[::1]:9123").is_ok() {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }

    // Connect to GDB server
    let mut gdb = gdb::Client::new("[::1]:9123")?;

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

    let mut diff;

    // On filesystem change, rebuild the project
    for result in rx {
        result?;

        // Rebuild the project
        if let Err(error) = run_build_command(&args.build) {
            error!("{}", error);
            continue;
        }

        // Reload the program
        let elf_file = std::fs::read(&args.elf)?;
        let new_program = program::Program::new(&elf_file)?;
        println!("New program! Loaded {} items", new_program.items.len());
        println!("{}", program.items["main"].disassemble().unwrap());

        diff = diff::diff(&program, &new_program);

        for diff in &diff {
            println!("{}", diff);
        }

        if let Err(error) = patch::apply(&mut gdb, &diff) {
            error!("{}", error);
            continue;
        }

        // TODO: solve lifetimes
        //program = new_program;
    }

    emulator::try_kill();
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

/// Set up a panic handler that kills the emulator before exiting.
fn setup_panic_handler() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        emulator::try_kill();
        original_hook(panic_info);
    }));
}
