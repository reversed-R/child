use crate::{
    elf::elf64::{DT_NULL, Elf64_Dyn, SHT_DYNAMIC},
    parser::{Elf64Parser, ElfParseError, section::strtab::ElfSectionStrtab},
};

pub(crate) struct ElfSectionDynamic {
    pub(crate) sect_index: usize,
    pub(crate) dyns: Vec<Elf64_Dyn>,
}

impl<'a> Elf64Parser<'a> {
    pub(crate) fn section_dynamic(
        &'a self,
    ) -> Result<(ElfSectionDynamic, ElfSectionStrtab<'a>), ElfParseError> {
        let (dynamic_shdr, bin) = self
            .get_section_with(|shdr| shdr.hdr.sh_type == SHT_DYNAMIC)?
            .ok_or(ElfParseError::SectionNotFound {
                name: ".dynamic".into(),
            })?;

        let strtab = self.section_strtab(dynamic_shdr.hdr.sh_link as usize)?;

        let mut dyns = Vec::new();
        for i in 0..bin.len() / std::mem::size_of::<Elf64_Dyn>() {
            let sym_bytes: [u8; std::mem::size_of::<Elf64_Dyn>()] = bin
                [std::mem::size_of::<Elf64_Dyn>() * i..std::mem::size_of::<Elf64_Dyn>() * (i + 1)]
                .try_into()
                .unwrap();
            let dyn_: Elf64_Dyn = unsafe { std::mem::transmute(sym_bytes) };

            if dyn_.d_tag == DT_NULL {
                break;
            }

            dyns.push(dyn_);
        }

        Ok((
            ElfSectionDynamic {
                sect_index: dynamic_shdr.index,
                dyns,
            },
            strtab,
        ))
    }
}
