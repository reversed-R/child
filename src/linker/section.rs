use std::collections::HashMap;

use crate::linker::{Linker, LinkerError};

pub(super) struct Section {
    base: usize,
    bytes: Vec<u8>,

    // 複数のオブジェクトファイルにあった元々のセクションのマージ後のオフセット
    // (obj_index, sect_index) -> offset
    section_offsets: HashMap<(usize, usize), usize>,
}

impl<'a> Linker<'a> {
    pub(super) fn merge_sections(&mut self) -> Result<(Section, Section), LinkerError> {
        let mut text_section_bytes = vec![];
        let mut data_section_bytes = vec![];

        let mut text_section_offsets = HashMap::new();
        let mut data_section_offsets = HashMap::new();

        let mut text_current_offset = 0;
        let mut data_current_offset = 0;

        for (obj_index, o) in self.objs.iter().enumerate() {
            for (sect_index, shdr) in o.elf.shdrs.iter().enumerate() {
                match shdr.name.as_str() {
                    ".text" => {
                        text_section_offsets.insert((obj_index, sect_index), text_current_offset);
                        let s_body = o
                            .elf
                            .get_section_body(shdr)
                            .map_err(|e| LinkerError::ParseError { errors: vec![e] })?;
                        text_section_bytes.extend_from_slice(s_body);
                        text_current_offset += s_body.len();
                    }
                    ".data" => {
                        data_section_offsets.insert((obj_index, sect_index), data_current_offset);
                        let s_body = o
                            .elf
                            .get_section_body(shdr)
                            .map_err(|e| LinkerError::ParseError { errors: vec![e] })?;
                        data_section_bytes.extend_from_slice(s_body);
                        data_current_offset += s_body.len();
                    }
                    _ => {}
                }
            }
        }

        let text_section = Section {
            base: 0, // TODO:
            bytes: text_section_bytes,
            section_offsets: text_section_offsets,
        };
        let data_section = Section {
            base: 0, // TODO:
            bytes: data_section_bytes,
            section_offsets: data_section_offsets,
        };

        Ok((text_section, data_section))
    }

    pub(super) fn arrange_sections(
        &self,
        text_section: Section,
        data_section: Section,
    ) -> Result<(), LinkerError> {
        todo!()
    }
}
