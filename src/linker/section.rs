use std::collections::HashMap;

use crate::linker::{Linker, LinkerError};

const TEXT_BASE_ADDR: usize = 0x400000;
const PAGE_SIZE: usize = 0x1000;

pub(super) struct OutputSectionList {
    pub(super) text: OutputSection,
    pub(super) data: OutputSection,
}

pub(super) struct OutputSection {
    pub(super) base: usize,
    pub(super) bytes: Vec<u8>,

    // 複数のオブジェクトファイルにあった元々のセクションのマージ後のオフセット
    // obj_index -> offset
    pub(super) section_offsets: HashMap<usize, usize>,
}

impl<'a> Linker<'a> {
    pub(super) fn merge_sections(&mut self) -> Result<(OutputSection, OutputSection), LinkerError> {
        let mut text_section_bytes = vec![];
        let mut data_section_bytes = vec![];

        let mut text_section_offsets = HashMap::new();
        let mut data_section_offsets = HashMap::new();

        let mut text_current_offset = 0;
        let mut data_current_offset = 0;

        for (obj_index, o) in self.objs.iter().enumerate() {
            let opt_text = o
                .elf
                .section_text()
                .map_err(|e| LinkerError::ParseError { errors: vec![e] })?;
            if let Some((_, s_body)) = opt_text {
                text_section_offsets.insert(obj_index, text_current_offset);
                text_section_bytes.extend_from_slice(s_body);
                text_current_offset += s_body.len();
            }

            let opt_data = o
                .elf
                .section_data()
                .map_err(|e| LinkerError::ParseError { errors: vec![e] })?;
            if let Some((_, s_body)) = opt_data {
                data_section_offsets.insert(obj_index, data_current_offset);
                data_section_bytes.extend_from_slice(s_body);
                data_current_offset += s_body.len();
            }
        }

        let text_section = OutputSection {
            base: TEXT_BASE_ADDR,
            bytes: text_section_bytes,
            section_offsets: text_section_offsets,
        };
        let data_section = OutputSection {
            base: (text_section.base + text_section.bytes.len()).next_multiple_of(PAGE_SIZE),
            bytes: data_section_bytes,
            section_offsets: data_section_offsets,
        };
        // .text (R-X) と .data (RW) は本来書き込み権限が違うセグメントに属するべき
        // ページ境界をまたいで配置しておく
        // TODO:
        // program header の生成

        Ok((text_section, data_section))
    }

    pub(super) fn arrange_sections(
        &self,
        text_section: OutputSection,
        data_section: OutputSection,
    ) -> Result<OutputSectionList, LinkerError> {
        Ok(OutputSectionList {
            text: text_section,
            data: data_section,
        })
    }
}
