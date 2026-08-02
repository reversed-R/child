use std::collections::HashMap;

use crate::linker::{Linker, LinkerError, output::OUTPUT_ELF_HEADER_RESERVED_SIZE};

pub(super) const TEXT_BASE_ADDR: usize = 0x400000;
pub(super) const PAGE_SIZE: usize = 0x1000;

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
    pub(super) fn bytes_len(&self) -> usize {
        match &self.bytes {
            OutputSectionBytesKind::Bytes(bytes) => bytes.len(),
            OutputSectionBytesKind::NobitsLen(len) => *len,
        }
    }

    fn end_addr(&self) -> usize {
        self.base + self.bytes_len()
    }
}

// for normal section (has bytes)
//
// fill bytes with zeros to next alignment position,
// extend with merging section bytes,
// and record its offset.
fn push_bytes(
    bytes: &mut Vec<u8>,
    offsets: &mut HashMap<usize, usize>,
    obj_index: usize,
    align: usize,
    content: &[u8],
) {
    bytes.resize(bytes.len().next_multiple_of(align), 0);

    offsets.insert(obj_index, bytes.len());
    bytes.extend_from_slice(content);
}

// for no bytes section (e.g. .bss)
fn push_nobits(
    current_offset: &mut usize,
    offsets: &mut HashMap<usize, usize>,
    obj_index: usize,
    align: usize,
    len: usize,
) {
    *current_offset = current_offset.next_multiple_of(align);

    offsets.insert(obj_index, *current_offset);
    *current_offset += len;
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
        let mut bss_current_offset = 0;

        let mut text_section_offsets = HashMap::new();
        let mut data_section_offsets = HashMap::new();
        let mut rodata_section_offsets = HashMap::new();
        let mut bss_section_offsets = HashMap::new();

        for (obj_index, o) in self.objs.iter().enumerate() {
            if let Some((shdr, s_body)) = o
                .elf
                .section_text()
                .map_err(|e| LinkerError::ParseError { errors: vec![e] })?
            {
                push_bytes(
                    &mut text_section_bytes,
                    &mut text_section_offsets,
                    obj_index,
                    shdr.hdr.sh_addralign(),
                    s_body,
                );
            }

            if let Some((shdr, s_body)) = o
                .elf
                .section_data()
                .map_err(|e| LinkerError::ParseError { errors: vec![e] })?
            {
                push_bytes(
                    &mut data_section_bytes,
                    &mut data_section_offsets,
                    obj_index,
                    shdr.hdr.sh_addralign(),
                    s_body,
                );
            }

            if let Some((shdr, s_body)) = o
                .elf
                .section_rodata()
                .map_err(|e| LinkerError::ParseError { errors: vec![e] })?
            {
                push_bytes(
                    &mut rodata_section_bytes,
                    &mut rodata_section_offsets,
                    obj_index,
                    shdr.hdr.sh_addralign(),
                    s_body,
                );
            }

            if let Some(shdr) = o
                .elf
                .section_bss()
                .map_err(|e| LinkerError::ParseError { errors: vec![e] })?
            {
                push_nobits(
                    &mut bss_current_offset,
                    &mut bss_section_offsets,
                    obj_index,
                    shdr.hdr.sh_addralign(),
                    shdr.hdr.sh_size as usize,
                );
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

        // R-X segments
        // make a gap to load Ehdr and Phdrs
        sects.text.base = TEXT_BASE_ADDR + OUTPUT_ELF_HEADER_RESERVED_SIZE;
        // R-- segments
        sects.rodata.base = sects.text.end_addr().next_multiple_of(PAGE_SIZE);
        // RW- segments
        sects.data.base = sects.rodata.end_addr().next_multiple_of(PAGE_SIZE);
        sects.bss.base = sects.data.end_addr();

        Ok(sects)
    }
}
