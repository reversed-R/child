use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

use crate::{
    elf::elf64::{DT_SONAME, Elf64_Word},
    linker::symbol::ResolvedSym,
    parser::{
        Elf64Parser, ElfParseError,
        section::{
            dynamic::ElfSectionDynamic, rela_text::ElfSectionRelaText, symtab::ElfSectionSymtab,
        },
    },
};

mod dynamic;
mod output;
mod relocation;
mod section;
pub(crate) mod symbol;

pub(crate) struct Linker<'a> {
    objs: Vec<ElfObject<'a>>,
    shared_objs: Vec<ElfSharedObject<'a>>,
    resolved_syms: Vec<HashMap<usize, ResolvedSym>>,
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
    UnsupportedRelocationType {
        r_type: Elf64_Word,
    },
    UnsupportedSection {
        name: String,
    },
    EntryPointNotFound,
}

struct ElfObject<'a> {
    elf: Elf64Parser<'a>,
    symtab: ElfSectionSymtab,
    rela_text: Option<ElfSectionRelaText>,
}
struct ElfSharedObject<'a> {
    elf: Elf64Parser<'a>,
    symtab: ElfSectionSymtab,
    dynamic: ElfSectionDynamic,
    soname: Option<String>,
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
                let rela_text = elf.section_rela_text(&symtab, &strtab)?;

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
                let (dynamic, dyn_strtab) = elf.section_dynamic()?;
                let soname = dynamic
                    .dyns
                    .iter()
                    .find(|dyn_| dyn_.d_tag == DT_SONAME)
                    .map(|dyn_| dyn_strtab.get(unsafe { dyn_.d_un.d_val } as usize));

                Ok(ElfSharedObject {
                    elf,
                    symtab,
                    dynamic,
                    soname,
                })
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
                objs,
                shared_objs,
                resolved_syms: Vec::new(),
            })
        } else {
            Err(LinkerError::ParseError { errors })
        }
    }

    pub(crate) fn link(mut self) -> Result<Vec<u8>, LinkerError> {
        let dyn_syms = self.resolve_symbols()?;

        let mut sects = self.merge_and_arrange_sections(&dyn_syms)?;

        self.fill_plt(&mut sects, dyn_syms.len());

        self.relocate(&mut sects, &dyn_syms)?;

        self.output_elf(sects, &dyn_syms)
    }
}
