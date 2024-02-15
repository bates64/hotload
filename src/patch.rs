use crate::diff::Diff;
use crate::gdb::Gdb;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("not supported, restart the emulator")]
    NotSupported,
}

/// Apply a diff to a process.
pub fn apply(gdb: &mut Gdb, diff: &[Diff<'_, '_>]) -> Result<(), Error> {
    for change in diff {
        match change {
            Diff::Add(_) => return Err(Error::NotSupported),
            Diff::Remove(_) => return Err(Error::NotSupported),
            Diff::Change(old_item, new_item) => {
                if old_item.size() != new_item.size() {
                    // TODO resizes
                    return Err(Error::NotSupported);
                }

                if old_item.ram_addr != new_item.ram_addr {
                    // TODO moves
                    return Err(Error::NotSupported);
                }

                /*
                // TODO
                // print hex of old item
                for (i, byte) in old_item.content.iter().enumerate() {
                    if i % 16 == 0 {
                        print!("\n{:08x}  ", old_item.rom_addr + i as u64);
                    }
                    print!("{:02x} ", byte);
                }
                println!();
                */

                println!("{}", new_item.disassemble().unwrap());
            }
        }
    }
    Ok(())
}
