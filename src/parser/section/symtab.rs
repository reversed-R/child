use crate::{
    elf::elf64::Elf64_Sym,
    linker::ResolvedSym,
    parser::{Elf64Parser, ElfParseError, section::strtab::ElfSectionStrtab},
};

#[derive(Debug)]
pub(crate) struct ElfSectionSymtab {
    pub(crate) syms: Vec<ElfSym>,
}

#[derive(Debug)]
pub(crate) struct ElfSym {
    pub(crate) name: String,
    pub(crate) sym: Elf64_Sym,
    pub(crate) resolved_sym: Option<ResolvedSym>,
}

impl<'a> Elf64Parser<'a> {
    pub(crate) fn section_symtab(
        &self,
        strtab: &ElfSectionStrtab,
    ) -> Result<ElfSectionSymtab, ElfParseError> {
        let (_, bin) =
            self.get_section_by_name(".symtab")?
                .ok_or(ElfParseError::SectionNotFound {
                    name: ".symtab".into(),
                })?;

        Ok(ElfSectionSymtab {
            syms: (0..bin.len() / std::mem::size_of::<Elf64_Sym>())
                .map(|i| {
                    let sym_bytes: [u8; std::mem::size_of::<Elf64_Sym>()] =
                        bin[std::mem::size_of::<Elf64_Sym>() * i
                            ..std::mem::size_of::<Elf64_Sym>() * (i + 1)]
                            .try_into()
                            .unwrap();
                    let sym: Elf64_Sym = unsafe { std::mem::transmute(sym_bytes) };

                    ElfSym {
                        sym,
                        name: strtab.get(sym.st_name as usize),
                        resolved_sym: None,
                    }
                })
                .collect(),
        })
    }
}

impl ElfSectionSymtab {
    pub(crate) fn get(&self, index: usize) -> Option<&ElfSym> {
        self.syms.get(index)
    }
}
