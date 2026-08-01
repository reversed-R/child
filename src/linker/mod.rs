use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use crate::{
    linker::symbol::ResolvedSym,
    parser::{
        Elf64Parser, ElfParseError,
        section::{rela_text::ElfSectionRelaText, symtab::ElfSectionSymtab},
    },
};

mod relocation;
mod section;
pub(crate) mod symbol;

pub(crate) struct Linker<'a> {
    objs: Vec<ElfObject<'a>>,
    shared_objs: Vec<ElfSharedObject<'a>>,
    resolved_syms: Vec<HashMap<usize, ResolvedSym>>,
    // resolved_syms[obj_index][symbol_index]
    sym_addrs: Vec<HashMap<usize, usize>>,
    // sym_addrs[obj_index][symbol_index]
    dynsym_addrs: Vec<HashMap<usize, usize>>,
}

#[derive(Debug)]
pub(crate) enum LinkerError {
    SymbolResolveFailed {
        duplicated_syms: HashSet<String>,
        missing_syms: HashSet<String>,
    },
    ParseError {
        errors: Vec<ElfParseError>,
    },
}

struct ElfObject<'a> {
    elf: Elf64Parser<'a>,
    symtab: ElfSectionSymtab,
    rela_text: Option<ElfSectionRelaText>,
}
struct ElfSharedObject<'a> {
    elf: Elf64Parser<'a>,
    symtab: ElfSectionSymtab,
}

impl<'a> Linker<'a> {
    pub(crate) fn new(
        objs: Vec<(&'a [u8], PathBuf)>,
        shared_objs: Vec<(&'a [u8], PathBuf)>,
    ) -> Result<Self, LinkerError> {
        let objs = objs
            .into_iter()
            .map(|(bin, path)| {
                let elf = Elf64Parser::new(bin, path)?;

                let (symtab, strtab) = elf.section_symtab()?;
                println!("-- .symtab --");
                println!("{symtab:#?}",);

                let rela_text = elf.section_rela_text(&symtab, &strtab)?;
                println!("-- .rela.text --");
                println!("{rela_text:#?}");

                Ok(ElfObject {
                    elf,
                    symtab,
                    rela_text,
                })
            })
            .collect::<Vec<Result<_, _>>>();

        let shared_objs = shared_objs
            .into_iter()
            .map(|(bin, path)| {
                let elf = Elf64Parser::new(bin, path)?;

                let (symtab, _) = elf.section_symtab()?;
                // println!("-- .symtab --");
                // println!("{symtab:#?}",);

                Ok(ElfSharedObject { elf, symtab })
            })
            .collect::<Vec<Result<_, _>>>();

        let mut errors = Vec::new();
        let objs = objs
            .into_iter()
            .filter_map(|res| match res {
                Ok(o) => Some(o),
                Err(e) => {
                    errors.push(e);
                    None
                }
            })
            .collect::<Vec<_>>();
        let shared_objs = shared_objs
            .into_iter()
            .filter_map(|res| match res {
                Ok(o) => Some(o),
                Err(e) => {
                    errors.push(e);
                    None
                }
            })
            .collect::<Vec<_>>();

        if errors.is_empty() {
            Ok(Self {
                sym_addrs: vec![HashMap::new(); objs.len()],
                dynsym_addrs: vec![HashMap::new(); shared_objs.len()],
                objs,
                shared_objs,
                resolved_syms: Vec::new(),
            })
        } else {
            Err(LinkerError::ParseError { errors })
        }
    }

    pub(crate) fn link(mut self) -> Result<(), LinkerError> {
        self.resolve_symbols()?;

        let (text_section, data_section) = self.merge_sections()?;

        self.arrange_sections(text_section, data_section)?;

        self.relocate()?;

        todo!()
    }
}
