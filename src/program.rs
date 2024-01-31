use goblin::elf::{section_header, Elf};
use paris::warn;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Program<'a> {
    pub items: HashMap<&'a str, Item<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item<'a> {
    section_name: Option<&'a str>,
    ram_addr: u64,
    rom_addr: u64,
    size: u64,
}

impl<'a> From<&Elf<'a>> for Program<'a> {
    fn from(elf: &Elf<'a>) -> Self {
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

            // TODO: consider tracking st_type (sym::STT_* consts)

            // TODO: read the content of the item

            let item = Item {
                section_name,
                ram_addr: sym.st_value,
                rom_addr: section.sh_offset + sym_offset,
                size: sym.st_size,
            };

            if let Some(name) = name {
                items.insert(name, item);
            } else {
                warn!("Symbol at index {} has no name, ignoring it", sym.st_name);
            }
        }

        Self { items }
    }
}
