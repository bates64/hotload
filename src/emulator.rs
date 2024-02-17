use std::process::Child;

/// An emulator process.
#[derive(Debug)]
pub struct Emulator {
    child: Child,
}

impl Emulator {
    pub fn new(command: &str) -> std::io::Result<Self> {
        Ok(Emulator {
            child: std::process::Command::new("sh")
                .arg("-c")
                .arg(command)
                .spawn()?,
        })
    }
}

impl Drop for Emulator {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}
