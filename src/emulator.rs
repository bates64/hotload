use std::process::Child;
use std::sync::{OnceLock, RwLock, TryLockResult};

/// The emulator process we are controlling.
/// This must be global because the panic handler needs to be able to kill the emulator.
// TODO: is this idomatic?
static EMULATOR: OnceLock<RwLock<Child>> = OnceLock::new();

pub fn spawn(command: &str) {
    if EMULATOR.get().is_some() {
        panic!("emulator already started");
    }

    EMULATOR.get_or_init(|| {
        RwLock::new(
            std::process::Command::new("sh")
                .arg("-c")
                .arg(command)
                .spawn()
                .expect("failed to start emulator"),
        )
    });
}

pub fn try_kill() {
    if let Some(emulator) = EMULATOR.get() {
        if let TryLockResult::Ok(mut emulator) = emulator.try_write() {
            let _ = emulator.kill();
        }
    }
}
