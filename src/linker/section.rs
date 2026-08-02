use std::collections::HashMap;

use crate::linker::{Linker, LinkerError};

const TEXT_BASE_ADDR: usize = 0x400000;
const PAGE_SIZE: usize = 0x1000;

pub(super) struct OutputSectionList {
    pub(super) text: OutputSection,
    pub(super) rodata: OutputSection,
    pub(super) data: OutputSection,
    pub(super) bss: OutputSection,
}

pub(super) enum OutputSectionBytesKind {
    Bytes(Vec<u8>),
    NobitsLen(usize),
}

pub(super) struct OutputSection {
    pub(super) base: usize,
    pub(super) bytes: OutputSectionBytesKind,

    // 複数のオブジェクトファイルにあった元々のセクションのマージ後のオフセット
    // obj_index -> offset
    pub(super) section_offsets: HashMap<usize, usize>,
}

impl OutputSection {
    fn bytes_len(&self) -> usize {
        match &self.bytes {
            OutputSectionBytesKind::Bytes(bytes) => bytes.len(),
            OutputSectionBytesKind::NobitsLen(len) => *len,
        }
    }

    fn end_addr(&self) -> usize {
        self.base + self.bytes_len()
    }
}

impl<'a> Linker<'a> {
    pub(super) fn merge_and_arrange_sections(&mut self) -> Result<OutputSectionList, LinkerError> {
        let sects = self.merge_sections()?;

        self.arrange_sections(sects)
    }

    fn merge_sections(&mut self) -> Result<OutputSectionList, LinkerError> {
        let mut text_section_bytes = vec![];
        let mut data_section_bytes = vec![];
        let mut rodata_section_bytes = vec![];

        let mut text_section_offsets = HashMap::new();
        let mut data_section_offsets = HashMap::new();
        let mut rodata_section_offsets = HashMap::new();
        let mut bss_section_offsets = HashMap::new();

        let mut text_current_offset = 0;
        let mut data_current_offset = 0;
        let mut rodata_current_offset = 0;
        let mut bss_current_offset = 0;

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

            let opt_rodata = o
                .elf
                .section_rodata()
                .map_err(|e| LinkerError::ParseError { errors: vec![e] })?;
            if let Some((_, s_body)) = opt_rodata {
                rodata_section_offsets.insert(obj_index, rodata_current_offset);
                rodata_section_bytes.extend_from_slice(s_body);
                rodata_current_offset += s_body.len();
            }

            let opt_bss = o
                .elf
                .section_bss()
                .map_err(|e| LinkerError::ParseError { errors: vec![e] })?;
            if let Some(shdr) = opt_bss {
                bss_section_offsets.insert(obj_index, bss_current_offset);
                bss_current_offset += shdr.hdr.sh_size as usize;
            }
        }

        let text_section = OutputSection {
            base: 0,
            bytes: OutputSectionBytesKind::Bytes(text_section_bytes),
            section_offsets: text_section_offsets,
        };
        let data_section = OutputSection {
            base: 0,
            bytes: OutputSectionBytesKind::Bytes(data_section_bytes),
            section_offsets: data_section_offsets,
        };
        let rodata_section = OutputSection {
            base: 0,
            bytes: OutputSectionBytesKind::Bytes(rodata_section_bytes),
            section_offsets: rodata_section_offsets,
        };
        let bss_section = OutputSection {
            base: 0,
            bytes: OutputSectionBytesKind::NobitsLen(bss_current_offset),
            section_offsets: bss_section_offsets,
        };

        Ok(OutputSectionList {
            text: text_section,
            rodata: rodata_section,
            data: data_section,
            bss: bss_section,
        })
    }

    fn arrange_sections(
        &self,
        mut sects: OutputSectionList,
    ) -> Result<OutputSectionList, LinkerError> {
        // page is devided between segments
        // which have different (Read-Write-Execute) permissions.
        // TODO:
        // program header の生成

        // R-X segments
        sects.text.base = TEXT_BASE_ADDR;
        // R-- segments
        sects.rodata.base = sects.text.end_addr().next_multiple_of(PAGE_SIZE);
        // RW- segments
        sects.data.base = sects.rodata.end_addr().next_multiple_of(PAGE_SIZE);
        sects.bss.base = sects.data.end_addr();

        Ok(sects)
    }
}
