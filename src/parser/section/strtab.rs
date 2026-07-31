use crate::{
    elf::elf64::Elf64_Shdr,
    parser::{Elf64Parser, ElfParseError},
};

#[derive(Debug)]
pub(crate) struct ElfSectionStrtab<'a> {
    shdr: &'a Elf64_Shdr,
    bin: &'a [u8],
}

impl<'a> Elf64Parser<'a> {
    pub(crate) fn section_strtab(&'a self) -> Result<ElfSectionStrtab<'a>, ElfParseError> {
        self.get_section_by_name(".strtab")?
            .ok_or(ElfParseError::SectionNotFound {
                name: ".strtab".into(),
            })
            .map(|(shdr, bin)| ElfSectionStrtab { shdr, bin })
    }
}

impl<'a> ElfSectionStrtab<'a> {
    pub(crate) fn get(&self, index: usize) -> String {
        for i in index..self.bin.len() {
            if self.bin[i] == 0 {
                return String::from_utf8(self.bin[index..i].to_vec()).expect("invalid utf8");
            }
        }

        String::from_utf8(self.bin[index..].to_vec()).expect("invalid utf8")
    }
}
