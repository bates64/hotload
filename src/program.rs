use goblin::elf::{section_header, Elf};
use paris::warn;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Program<'a> {
    pub items: HashMap<&'a str, Item<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item<'a> {
    pub section_name: Option<&'a str>,
    pub ram_addr: u64,
    pub rom_addr: u64,
    pub content: &'a [u8],
}

impl<'a> Program<'a> {
    pub fn new(bytes: &'a [u8]) -> Result<Self, goblin::error::Error> {
        let elf = Elf::parse(bytes)?;

        let mut items = HashMap::new();

        for sym in &elf.syms {
            let name = elf.strtab.get_at(sym.st_name);

            // Ignore symbols that are not in a section
            match sym.st_shndx as u32 {
                section_header::SHN_UNDEF
                | section_header::SHN_LOPROC
                | section_header::SHN_HIPROC
                | section_header::SHN_ABS
                | section_header::SHN_COMMON
                | section_header::SHN_HIRESERVE => continue,
                _ => {}
            };

            let section = &elf.section_headers[sym.st_shndx];
            let section_name = elf.shdr_strtab.get_at(section.sh_name);

            let sym_offset = sym.st_value - section.sh_addr;
            let rom_addr = section.sh_offset + sym_offset;

            // TODO: consider tracking st_type (sym::STT_* consts)

            let item = Item {
                section_name,
                ram_addr: sym.st_value,
                rom_addr,
                content: &bytes[rom_addr as usize..(rom_addr + sym.st_size) as usize],
            };

            if let Some(name) = name {
                items.insert(name, item);
            } else {
                warn!("Symbol at index {} has no name, ignoring it", sym.st_name);
            }
        }

        Ok(Self { items })
    }
}

impl<'a> Item<'a> {
    pub fn size(&self) -> usize {
        self.content.len()
    }

    pub fn print_hex(&self) {
        for (i, byte) in self.content.iter().enumerate() {
            if i % 16 == 0 {
                print!("\n{:08x}  ", self.rom_addr + i as u64);
            }
            print!("{:02x} ", byte);
        }
        println!();
    }
}
