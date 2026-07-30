use crate::{
    elf::elf64::{Elf64_Rela, SHT_RELA},
    parser::{Elf64Parser, ElfParseError},
};

#[derive(Debug)]
pub(crate) struct ElfRela {
    name: String,
    rela: Elf64_Rela,
}

impl<'a> Elf64Parser<'a> {
    pub(crate) fn section_rela_text(&self) -> Result<Vec<ElfRela>, ElfParseError> {
        let (_, bin) =
            self.get_section_by_type(SHT_RELA)
                .ok_or(ElfParseError::SectionNotFound {
                    name: ".rela.text".into(),
                })??;

        let strtab = self.section_strtab()?;

        Ok((0..bin.len() / std::mem::size_of::<Elf64_Rela>())
            .map(|i| {
                let rela_bytes: [u8; std::mem::size_of::<Elf64_Rela>()] =
                    bin[std::mem::size_of::<Elf64_Rela>() * i
                        ..std::mem::size_of::<Elf64_Rela>() * (i + 1)]
                        .try_into()
                        .unwrap();
                let rela: Elf64_Rela = unsafe { std::mem::transmute(rela_bytes) };

                ElfRela {
                    rela,
                    name: strtab.get(((rela.r_info >> 32) & 0xff_ff_ff_ff) as usize),
                }
            })
            .collect())
    }
}
