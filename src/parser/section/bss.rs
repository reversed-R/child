use crate::parser::{Elf64Parser, ElfParseError, ElfSectionHeaderEntry};

impl<'a> Elf64Parser<'a> {
    pub(crate) fn section_bss(
        &'a self,
    ) -> Result<Option<&'a ElfSectionHeaderEntry>, ElfParseError> {
        self.get_section_with(|shdr| shdr.name == ".bss")
            .map(|opt| opt.map(|(shdr, _)| shdr))
    }
}
