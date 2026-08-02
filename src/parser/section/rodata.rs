use crate::parser::{Elf64Parser, ElfParseError, ElfSectionHeaderEntry};

impl<'a> Elf64Parser<'a> {
    pub(crate) fn section_rodata(
        &'a self,
    ) -> Result<Option<(&'a ElfSectionHeaderEntry, &'a [u8])>, ElfParseError> {
        self.get_section_with(|shdr| shdr.name == ".rodata")
    }
}
