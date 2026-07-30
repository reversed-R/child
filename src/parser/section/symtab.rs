use crate::{
    elf::elf64::{Elf64_Sym, SHT_SYMTAB},
    parser::{Elf64Parser, ElfParseError},
};

#[derive(Debug)]
pub(crate) struct ElfSym {
    name: String,
    sym: Elf64_Sym,
}

impl<'a> Elf64Parser<'a> {
    pub(crate) fn section_symtab(&self) -> Result<Vec<ElfSym>, ElfParseError> {
        let (_, bin) =
            self.get_section_by_type(SHT_SYMTAB)
                .ok_or(ElfParseError::SectionNotFound {
                    name: ".symtab".into(),
                })??;

        let strtab = self.section_strtab()?;

        Ok((0..bin.len() / std::mem::size_of::<Elf64_Sym>())
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
                }
            })
            .collect())
    }
}
