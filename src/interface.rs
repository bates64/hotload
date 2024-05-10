use clap::Parser;
use paris::error;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;

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

    /// Safe functions, high up in the call stack, where updating can occur e.g. the game step / frame function
    #[clap(short, long)]
    pub checkpoints: Vec<Checkpoint>,
}

/// An update-safe point in the code where dynamic software updating can occur
#[derive(Parser, Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Symbol name of the function
    pub function: String,
}

// For clap to parse Vec<Checkpoint> as comma-separated values
impl FromStr for Checkpoint {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Checkpoint {
            function: s.to_string(),
        })
    }
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
