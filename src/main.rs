mod program;

use clap::Parser;
use std::path::PathBuf;

/// Hot code loading (dynamic software updating) for Nintendo 64 development
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// ELF file to watch
    elf: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Parse ELF file
    let elf_file = std::fs::read(args.elf)?;
    let elf = goblin::elf::Elf::parse(&elf_file)?;
    let program = program::Program::from(&elf);

    println!("Loaded {} items", program.items.len());

    Ok(())
}
