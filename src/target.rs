use std::process::Child;

/// The target system, e.g. an emulator process.
#[derive(Debug)]
pub struct Target {
    child: Child,
}

impl Target {
    pub fn new(command: &str) -> std::io::Result<Self> {
        Ok(Target {
            child: std::process::Command::new("sh")
                .arg("-c")
                .arg(command)
                .spawn()?,
        })
    }

    pub fn try_kill(&mut self) {
        let _ = self.child.kill();
    }
}
