mod diff;
mod gdb;
mod patch;
mod program;

use clap::Parser;
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};
use paris::error;
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
    let args = Args::parse();

    run_build_command(&args.build)?;

    // Spawn emulator
    let _ = std::process::Command::new("sh")
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
    let elf = goblin::elf::Elf::parse(&elf_file)?;
    let program = program::Program::from(&elf);
    println!("Loaded {} items", program.items.len());

    // Watch for source changes
    let (tx, rx) = std::sync::mpsc::channel();
    let mut debouncer = new_debouncer(Duration::from_millis(10), tx)?;
    let watcher = debouncer.watcher();
    for path in args.src {
        watcher.watch(&path, RecursiveMode::Recursive)?;
    }

    for result in rx {
        result?; // Propagate errors

        // Rebuild the project
        if let Err(error) = run_build_command(&args.build) {
            error!("{}", error);
            continue;
        }

        // Reload the program
        let elf_file = std::fs::read(&args.elf)?;
        let elf = goblin::elf::Elf::parse(&elf_file)?;
        let new_program = program::Program::from(&elf);
        println!("New program! Loaded {} items", new_program.items.len());

        let diff = diff::diff(&program, &new_program);
        patch::apply(&mut gdb, &diff)?;
    }

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
