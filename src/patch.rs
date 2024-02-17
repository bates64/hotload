use crate::diff::Diff;
use crate::gdb::{self, Client};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("not supported, restart the emulator")]
    NotSupported,

    #[error("GDB error: {0}")]
    Gdb(#[from] gdb::Error),
}

/// Apply a diff to a process.
pub fn apply(gdb: &mut Client, diff: &[Diff<'_, '_>]) -> Result<(), Error> {
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

                if old_item.section_name != new_item.section_name {
                    // TODO section changes?
                    return Err(Error::NotSupported);
                }

                gdb.write_memory(old_item.ram_addr, new_item.content)?;
            }
        }
    }
    Ok(())
}
