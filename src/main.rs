mod program;

use clap::Parser;
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};
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
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

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
        std::process::Command::new("sh")
            .arg("-c")
            .arg(&args.build)
            .status()
            .expect("Failed to execute build system");

        // Reload the program
        let elf_file = std::fs::read(&args.elf)?;
        let elf = goblin::elf::Elf::parse(&elf_file)?;
        let new_program = program::Program::from(&elf);
        println!("New program! Loaded {} items", new_program.items.len());
    }

    Ok(())
}
