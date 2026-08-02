use std::collections::{HashMap, HashSet, hash_map::Entry};

use crate::elf::elf64::{ELF64_ST_BIND, SHN_UNDEF, STB_GLOBAL};

use crate::linker::{Linker, LinkerError};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) enum ResolvedObjIndexKind {
    Obj(usize),
    Shared(usize),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ResolvedSym {
    pub(super) obj_index: ResolvedObjIndexKind,
    pub(super) sym_index: usize,
}

pub(super) struct ResolvedDynSym {
    pub(super) shared_obj_index: usize,
    pub(super) sym_index: usize,
}

impl<'a> Linker<'a> {
    pub(super) fn resolve_symbols(
        &mut self,
    ) -> Result<HashMap<String, ResolvedDynSym>, LinkerError> {
        let mut resolved_syms = vec![HashMap::<usize, ResolvedSym>::new(); self.objs.len()];
        let mut duplicated_syms = HashSet::new();
        let mut missing_syms = HashSet::new();
        let mut dyn_syms = HashMap::new();

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
                                            dyn_syms.insert(
                                                s.name.clone(),
                                                ResolvedDynSym {
                                                    shared_obj_index: o_index,
                                                    sym_index: s_index,
                                                },
                                            );
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
            for (obj_index, (o, resolved)) in self.objs.iter_mut().zip(&resolved_syms).enumerate() {
                for (sym_index, sym) in o.symtab.syms.iter_mut().enumerate().skip(1) {
                    if sym.sym.st_shndx == SHN_UNDEF {
                        sym.resolved_sym = Some(resolved.get(&sym_index).unwrap().clone());
                    } else {
                        // for bug prevension, symbol points itself.
                        sym.resolved_sym = Some(ResolvedSym {
                            obj_index: ResolvedObjIndexKind::Obj(obj_index),
                            sym_index,
                        })
                    }
                }
            }

            self.resolved_syms = resolved_syms;

            Ok(dyn_syms)
        } else {
            Err(LinkerError::SymbolResolveFailed {
                duplicated_syms,
                missing_syms,
            })
        }
    }
}
