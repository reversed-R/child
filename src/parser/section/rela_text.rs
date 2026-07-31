use crate::{
    elf::elf64::{ELF64_R_SYM, ELF64_ST_TYPE, Elf64_Rela, STT_SECTION},
    parser::{
        Elf64Parser, ElfParseError,
        section::{strtab::ElfSectionStrtab, symtab::ElfSectionSymtab},
    },
};

#[derive(Debug)]
pub(crate) struct ElfRela {
    name: String,
    rela: Elf64_Rela,
}

impl<'a> Elf64Parser<'a> {
    pub(crate) fn section_rela_text(
        &self,
        symtab: &ElfSectionSymtab,
        strtab: &ElfSectionStrtab,
    ) -> Result<Vec<ElfRela>, ElfParseError> {
        let (_, bin) =
            self.get_section_by_name(".rela.text")
                .ok_or(ElfParseError::SectionNotFound {
                    name: ".rela.text".into(),
                })??;

        (0..bin.len() / std::mem::size_of::<Elf64_Rela>())
            .map(|i| {
                let rela_bytes: [u8; std::mem::size_of::<Elf64_Rela>()] =
                    bin[std::mem::size_of::<Elf64_Rela>() * i
                        ..std::mem::size_of::<Elf64_Rela>() * (i + 1)]
                        .try_into()
                        .unwrap();
                let rela: Elf64_Rela = unsafe { std::mem::transmute(rela_bytes) };
                let index = ELF64_R_SYM(rela.r_info) as usize;
                let sym = symtab
                    .get(index)
                    .ok_or(ElfParseError::SymbolNotFound { index })?;
                let name = if ELF64_ST_TYPE(sym.sym.st_info) == STT_SECTION {
                    self.shdrs
                        .get(sym.sym.st_shndx as usize)
                        .ok_or(ElfParseError::SectionNotFoundByIndex {
                            index: sym.sym.st_shndx as usize,
                        })?
                        .name
                        .clone()
                } else {
                    strtab.get(sym.sym.st_name as usize)
                };

                Ok(ElfRela { rela, name })
            })
            .collect()
    }
}
