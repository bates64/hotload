use clap::Parser;
use paris::error;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Hot code loading (dynamic software updating) for Nintendo 64 development
#[derive(Parser, Debug, Serialize, Deserialize)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Build system command to run (e.g. `make`, `ninja`, `libdragon build`)
    #[clap(short, long)]
    pub build: String,

    /// ELF file that is output from build command
    #[clap(short, long)]
    pub elf: PathBuf,

    /// Source files and/or directories to recursively watch for changes
    #[clap(short, long)]
    pub src: Vec<PathBuf>,

    // TODO: support attaching to existing emulator
    /// Emulator command (e.g. `ares rom.z64`)
    #[clap(short = 'x', long)]
    pub emulator: String,
}

impl Args {
    /// Creates a new Args (arguments) struct.
    /// If there is a hotload.toml file in the current directory, uses it. Otherwise, uses command line arguments.
    /// Exits with an error if there is a problem parsing the arguments or the hotload.toml file.
    pub fn new() -> Self {
        if let Ok(toml) = std::fs::read_to_string("hotload.toml") {
            match toml::from_str(&toml) {
                Ok(args) => return args,
                Err(error) => {
                    error!("Error parsing hotload.toml: {}", error);
                }
            }
        }

        Self::parse()
    }
}
