use std::collections::HashMap;

use crate::{
    elf::elf64::{
        ELFCLASS64, ELFDATA2LSB, ELFMAG0, ELFMAG1, ELFMAG2, ELFMAG3, ELFOSABI_LINUX, EM_X86_64,
        ET_EXEC, EV_CURRENT, Elf64_Addr, Elf64_Ehdr, Elf64_Half, Elf64_Off, Elf64_Phdr, Elf64_Shdr,
        Elf64_Word, Elf64_Xword, PF_R, PF_W, PF_X, PT_LOAD, SHF_ALLOC, SHF_EXECINSTR, SHF_WRITE,
        SHT_NOBITS, SHT_NULL, SHT_PROGBITS, SHT_STRTAB,
    },
    linker::{
        Linker, LinkerError,
        section::{
            OutputSection, OutputSectionBytesKind, OutputSectionList, PAGE_SIZE, TEXT_BASE_ADDR,
        },
    },
};

// .shstrtab の中身と、名前からその中でのオフセットを引くためのテーブル
struct Shstrtab {
    bytes: Vec<u8>,
    offsets: HashMap<&'static str, Elf64_Word>,
}

impl Shstrtab {
    fn new() -> Self {
        let mut bytes = vec![0u8]; // the 0 at offset = 0 is for NULL section
        let mut offsets = HashMap::new();

        for name in [".text", ".rodata", ".data", ".bss", ".shstrtab"] {
            offsets.insert(name, bytes.len() as Elf64_Word);
            bytes.extend_from_slice(name.as_bytes());
            bytes.push(0);
        }

        Self { bytes, offsets }
    }

    fn offset_of(&self, name: &str) -> Elf64_Word {
        self.offsets[name]
    }
}

struct OutputFileLayout {
    text: usize,
    rodata: usize,
    data: usize,
    shstrtab: usize,
    shdrs: usize,
}

// reserved size of Ehdr and Phdrs
// 3 means segments for text, rodata, data+bss
pub(super) const OUTPUT_ELF_HEADER_RESERVED_SIZE: usize =
    std::mem::size_of::<Elf64_Ehdr>() + std::mem::size_of::<Elf64_Phdr>() * 3;

impl OutputFileLayout {
    fn new(sects: &OutputSectionList, shstrtab_len: usize) -> Self {
        let text = OUTPUT_ELF_HEADER_RESERVED_SIZE;
        let rodata = (text + sects.text.bytes_len()).next_multiple_of(PAGE_SIZE);
        let data = (rodata + sects.rodata.bytes_len()).next_multiple_of(PAGE_SIZE);
        // .bss has no bytes in ELF file
        let shstrtab = data + sects.data.bytes_len();
        let shdrs = shstrtab + shstrtab_len;

        Self {
            text,
            rodata,
            data,
            shstrtab,
            shdrs,
        }
    }
}

impl<'a> Linker<'a> {
    pub(super) fn output_elf(&self, sects: OutputSectionList) -> Result<Vec<u8>, LinkerError> {
        let entry = self.find_entry_point(&sects)?;

        let shstrtab = Shstrtab::new();
        let layout = OutputFileLayout::new(&sects, shstrtab.bytes.len());

        let phdrs = self.generate_phdrs(&sects, &layout);
        let shdrs = self.generate_shdrs(&sects, &shstrtab, &layout);
        let ehdr = self.generate_ehdr(entry, &phdrs, &shdrs, &layout);

        /* -- write bytes to `out` (ELF file) -- */
        let mut out = vec![0u8; layout.shdrs + shdrs.len() * std::mem::size_of::<Elf64_Shdr>()];

        out[..std::mem::size_of::<Elf64_Ehdr>()].copy_from_slice(ehdr.as_bytes());

        let mut off = std::mem::size_of::<Elf64_Ehdr>();
        for phdr in &phdrs {
            out[off..off + std::mem::size_of::<Elf64_Phdr>()].copy_from_slice(phdr.as_bytes());
            off += std::mem::size_of::<Elf64_Phdr>();
        }

        write_section_bytes(&mut out, layout.text, &sects.text);
        write_section_bytes(&mut out, layout.rodata, &sects.rodata);
        write_section_bytes(&mut out, layout.data, &sects.data);
        // .bss has no bytes in ELF file.
        out[layout.shstrtab..layout.shstrtab + shstrtab.bytes.len()]
            .copy_from_slice(&shstrtab.bytes);

        let mut off = layout.shdrs;
        for shdr in &shdrs {
            out[off..off + std::mem::size_of::<Elf64_Shdr>()].copy_from_slice(shdr.as_bytes());
            off += std::mem::size_of::<Elf64_Shdr>();
        }

        Ok(out)
    }

    /// "_start" があればそれを、なければ "main" をエントリポイントとして使う。
    /// TODO: crt0/_start の生成に対応したら "main" へのフォールバックは無くす
    fn find_entry_point(&self, sects: &OutputSectionList) -> Result<usize, LinkerError> {
        for name in ["_start", "main"] {
            for (obj_index, obj) in self.objs.iter().enumerate() {
                if let Some(sym_index) = obj
                    .symtab
                    .syms
                    .iter()
                    .position(|s| s.name == name && s.sym.is_resolved_index())
                {
                    return self.symbol_address(obj_index, sym_index, sects);
                }
            }
        }

        Err(LinkerError::EntryPointNotFound)
    }

    fn generate_ehdr(
        &self,
        entry: usize,
        phdrs: &[Elf64_Phdr],
        shdrs: &[Elf64_Shdr],
        layout: &OutputFileLayout,
    ) -> Elf64_Ehdr {
        Elf64_Ehdr {
            e_ident: [
                ELFMAG0,
                ELFMAG1,
                ELFMAG2,
                ELFMAG3,
                ELFCLASS64,
                ELFDATA2LSB,
                EV_CURRENT as u8,
                ELFOSABI_LINUX,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
                0,
            ],
            e_type: ET_EXEC,
            e_machine: EM_X86_64,
            e_version: EV_CURRENT,
            e_entry: entry as Elf64_Addr,
            e_phoff: std::mem::size_of::<Elf64_Ehdr>() as Elf64_Off,
            e_shoff: layout.shdrs as Elf64_Off,
            e_flags: 0,
            e_ehsize: std::mem::size_of::<Elf64_Ehdr>() as Elf64_Half,
            e_phentsize: std::mem::size_of::<Elf64_Phdr>() as Elf64_Half,
            e_phnum: phdrs.len() as Elf64_Half,
            e_shentsize: std::mem::size_of::<Elf64_Shdr>() as Elf64_Half,
            e_shnum: shdrs.len() as Elf64_Half,
            // In current implementaion,
            // shstrtab is placed at the last of sections.
            e_shstrndx: (shdrs.len() - 1) as Elf64_Half,
        }
    }

    fn generate_phdrs(
        &self,
        sects: &OutputSectionList,
        layout: &OutputFileLayout,
    ) -> Vec<Elf64_Phdr> {
        // Ehdr + Phdrs + .text
        let text = Elf64_Phdr {
            p_type: PT_LOAD,
            p_flags: PF_R | PF_X,
            p_offset: 0,
            p_vaddr: TEXT_BASE_ADDR as Elf64_Addr,
            p_paddr: TEXT_BASE_ADDR as Elf64_Addr,
            p_filesz: (layout.text + sects.text.bytes_len()) as Elf64_Xword,
            p_memsz: (layout.text + sects.text.bytes_len()) as Elf64_Xword,
            p_align: PAGE_SIZE as Elf64_Xword,
        };
        let rodata = Elf64_Phdr {
            p_type: PT_LOAD,
            p_flags: PF_R,
            p_offset: layout.rodata as Elf64_Off,
            p_vaddr: sects.rodata.base as Elf64_Addr,
            p_paddr: sects.rodata.base as Elf64_Addr,
            p_filesz: sects.rodata.bytes_len() as Elf64_Xword,
            p_memsz: sects.rodata.bytes_len() as Elf64_Xword,
            p_align: PAGE_SIZE as Elf64_Xword,
        };
        let data = Elf64_Phdr {
            p_type: PT_LOAD,
            p_flags: PF_R | PF_W,
            p_offset: layout.data as Elf64_Off,
            p_vaddr: sects.data.base as Elf64_Addr,
            p_paddr: sects.data.base as Elf64_Addr,
            p_filesz: sects.data.bytes_len() as Elf64_Xword,
            // .bss has no bytes in ELF file,
            // so set p_memsz larger than p_filesz by .bss size.
            // OS loader will fill this gap with zero.
            p_memsz: (sects.data.bytes_len() + sects.bss.bytes_len()) as Elf64_Xword,
            p_align: PAGE_SIZE as Elf64_Xword,
        };

        vec![text, rodata, data]
    }

    fn generate_shdrs(
        &self,
        sects: &OutputSectionList,
        shstrtab: &Shstrtab,
        layout: &OutputFileLayout,
    ) -> Vec<Elf64_Shdr> {
        let null = Elf64_Shdr {
            sh_name: 0,
            sh_type: SHT_NULL,
            sh_flags: 0,
            sh_addr: 0,
            sh_offset: 0,
            sh_size: 0,
            sh_link: 0,
            sh_info: 0,
            sh_addralign: 0,
            sh_entsize: 0,
        };
        let text = Elf64_Shdr {
            sh_name: shstrtab.offset_of(".text"),
            sh_type: SHT_PROGBITS,
            sh_flags: SHF_ALLOC | SHF_EXECINSTR,
            sh_addr: sects.text.base as Elf64_Addr,
            sh_offset: layout.text as Elf64_Off,
            sh_size: sects.text.bytes_len() as Elf64_Xword,
            sh_link: 0,
            sh_info: 0,
            sh_addralign: 1,
            sh_entsize: 0,
        };
        let rodata = Elf64_Shdr {
            sh_name: shstrtab.offset_of(".rodata"),
            sh_type: SHT_PROGBITS,
            sh_flags: SHF_ALLOC,
            sh_addr: sects.rodata.base as Elf64_Addr,
            sh_offset: layout.rodata as Elf64_Off,
            sh_size: sects.rodata.bytes_len() as Elf64_Xword,
            sh_link: 0,
            sh_info: 0,
            sh_addralign: 1,
            sh_entsize: 0,
        };
        let data = Elf64_Shdr {
            sh_name: shstrtab.offset_of(".data"),
            sh_type: SHT_PROGBITS,
            sh_flags: SHF_ALLOC | SHF_WRITE,
            sh_addr: sects.data.base as Elf64_Addr,
            sh_offset: layout.data as Elf64_Off,
            sh_size: sects.data.bytes_len() as Elf64_Xword,
            sh_link: 0,
            sh_info: 0,
            sh_addralign: 1,
            sh_entsize: 0,
        };
        let bss = Elf64_Shdr {
            sh_name: shstrtab.offset_of(".bss"),
            sh_type: SHT_NOBITS,
            sh_flags: SHF_ALLOC | SHF_WRITE,
            sh_addr: sects.bss.base as Elf64_Addr,
            // .bss has no bytes in ELF file (SHT_NOBITS).
            // but we need to set virtual offset.
            sh_offset: (layout.data + sects.data.bytes_len()) as Elf64_Off,
            sh_size: sects.bss.bytes_len() as Elf64_Xword,
            sh_link: 0,
            sh_info: 0,
            sh_addralign: 1,
            sh_entsize: 0,
        };
        let shstrtab_shdr = Elf64_Shdr {
            sh_name: shstrtab.offset_of(".shstrtab"),
            sh_type: SHT_STRTAB,
            sh_flags: 0,
            sh_addr: 0,
            sh_offset: layout.shstrtab as Elf64_Off,
            sh_size: shstrtab.bytes.len() as Elf64_Xword,
            sh_link: 0,
            sh_info: 0,
            sh_addralign: 1,
            sh_entsize: 0,
        };

        // In current implementaion,
        // shstrtab is placed at the last of sections.
        vec![null, text, rodata, data, bss, shstrtab_shdr]
    }
}

fn write_section_bytes(out: &mut [u8], offset: usize, sect: &OutputSection) {
    if let OutputSectionBytesKind::Bytes(bytes) = &sect.bytes {
        out[offset..offset + bytes.len()].copy_from_slice(bytes);
    }
}

impl Elf64_Ehdr {
    fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self as *const Self as *const u8,
                std::mem::size_of::<Self>(),
            )
        }
    }
}

impl Elf64_Phdr {
    fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self as *const Self as *const u8,
                std::mem::size_of::<Self>(),
            )
        }
    }
}

impl Elf64_Shdr {
    fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self as *const Self as *const u8,
                std::mem::size_of::<Self>(),
            )
        }
    }
}
