mod diff;
mod emulator;
mod gdb;
mod interface;
mod patch;
mod program;

use emulator::Emulator;
use interface::Args;
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};
use paris::error;
use std::sync::{mpsc::channel, Arc, RwLock};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let workspace = Args::new();

    run_build_command(&workspace.build)?;

    let emulator = Arc::new(RwLock::new(Emulator::new(&workspace.emulator)?));

    // Kill emulator on ^C
    let emulator_clone = emulator.clone();
    ctrlc::set_handler(move || {
        emulator_clone.write().unwrap().try_kill();
        std::process::exit(0);
    })?;

    match hotload(&workspace) {
        Ok(()) => {}
        Err(error) => {
            error!("{}", error);
        }
    }

    // Kill emulator on exit
    if let Ok(mut emulator) = emulator.write() {
        emulator.try_kill();
    }
    Ok(())
}

fn hotload(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    println!("Waiting for GDB server...");
    let mut gdb = gdb::Client::new_blocking("[::1]:9123")?;

    // Parse ELF file
    let elf_file = std::fs::read(&args.elf)?;
    let program = program::Program::new(&elf_file)?;
    println!("Loaded {} items", program.items.len());

    // Watch for source changes
    println!("Watching {:?} for changes", args.src);
    let (tx, rx) = channel();
    let mut debouncer = new_debouncer(Duration::from_millis(10), tx)?;
    let watcher = debouncer.watcher();
    for path in &args.src {
        watcher.watch(path, RecursiveMode::Recursive)?;
    }

    // Everything is ready.
    println!("Ready for hotloading!");

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
        //println!("{}", program.items["main"].disassemble().unwrap());

        diff = diff::diff(&program, &new_program);

        if diff.is_empty() {
            println!("No changes (diff is empty)");
            continue;
        }

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
