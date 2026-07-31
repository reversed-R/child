use std::{
    collections::{HashMap, HashSet, hash_map::Entry},
    path::PathBuf,
};

use crate::{
    elf::elf64::{ELF64_ST_BIND, SHN_UNDEF, STB_GLOBAL},
    parser::{
        Elf64Parser, ElfParseError,
        section::{rela_text::ElfSectionRelaText, symtab::ElfSectionSymtab},
    },
};

pub(crate) struct Linker<'a> {
    objs: Vec<ElfObject<'a>>,
    shared_objs: Vec<ElfSharedObject<'a>>,
    resolved_syms: Vec<ResolvedSym>,
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

#[derive(Debug, Clone)]
enum ResolvedObjIndexKind {
    Obj(usize),
    Shared(usize),
}

#[derive(Debug, Clone)]
pub(crate) struct ResolvedSym {
    obj_index: ResolvedObjIndexKind,
    sym_index: usize,
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
                // println!("{elf:#?}");

                let strtab = elf.section_strtab()?;
                println!("-- .strtab --");
                println!("{strtab:#?}",);

                let symtab = elf.section_symtab(&strtab)?;
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
                // println!("{elf:#?}");

                let strtab = elf.section_strtab()?;
                println!("-- .strtab --");
                println!("{strtab:#?}",);

                let symtab = elf.section_symtab(&strtab)?;
                println!("-- .symtab --");
                println!("{symtab:#?}",);

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
            .collect();
        let shared_objs = shared_objs
            .into_iter()
            .filter_map(|res| match res {
                Ok(o) => Some(o),
                Err(e) => {
                    errors.push(e);
                    None
                }
            })
            .collect();

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

    pub(crate) fn resolve_symbols(&mut self) -> Result<(), LinkerError> {
        let mut resolved_syms = vec![HashMap::<usize, ResolvedSym>::new(); self.objs.len()];
        let mut duplicated_syms = HashSet::new();
        let mut missing_syms = HashSet::new();

        for (obj_index, obj) in self.objs.iter().enumerate() {
            for (sym_index, sym) in obj.symtab.syms.iter().enumerate().skip(1) {
                if sym.sym.st_shndx == SHN_UNDEF {
                    let mut found = false;
                    for (o_index, o) in self.objs.iter().enumerate() {
                        if obj_index == o_index {
                            continue;
                        }

                        for (s_index, s) in o.symtab.syms.iter().enumerate().skip(1) {
                            if sym.name == s.name
                                && ELF64_ST_BIND(s.sym.st_info) == STB_GLOBAL
                                && s.sym.is_resolved_index()
                            {
                                match resolved_syms[obj_index].entry(sym_index) {
                                    Entry::Vacant(e) => {
                                        e.insert(ResolvedSym {
                                            obj_index: ResolvedObjIndexKind::Obj(o_index),
                                            sym_index: s_index,
                                        });
                                        found = true;
                                    }
                                    Entry::Occupied(_) => {
                                        duplicated_syms.insert(sym.name.clone());
                                    }
                                }
                            }
                        }
                    }

                    if !found {
                        for (o_index, o) in self.shared_objs.iter().enumerate() {
                            for (s_index, s) in o.symtab.syms.iter().skip(1).enumerate() {
                                if sym.name == s.name
                                    && ELF64_ST_BIND(s.sym.st_info) == STB_GLOBAL
                                    && s.sym.is_resolved_index()
                                {
                                    match resolved_syms[obj_index].entry(sym_index) {
                                        Entry::Vacant(e) => {
                                            e.insert(ResolvedSym {
                                                obj_index: ResolvedObjIndexKind::Shared(o_index),
                                                sym_index: s_index,
                                            });
                                            found = true;
                                        }
                                        Entry::Occupied(_) => {
                                            duplicated_syms.insert(sym.name.clone());
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if !found {
                        missing_syms.insert(sym.name.clone());
                    }
                }
            }
        }

        if duplicated_syms.is_empty() && missing_syms.is_empty() {
            for (o, resolved) in self.objs.iter_mut().zip(&mut resolved_syms).skip(1) {
                for (sym_index, sym) in o.symtab.syms.iter_mut().enumerate() {
                    sym.resolved_sym = Some(resolved.remove(&sym_index).unwrap());
                }
            }

            Ok(())
        } else {
            Err(LinkerError::SymbolResolveFailed {
                duplicated_syms,
                missing_syms,
            })
        }
    }
}
