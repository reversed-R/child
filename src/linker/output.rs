use std::collections::HashMap;

use crate::{
    elf::elf64::{
        ELF64_R_INFO, ELFCLASS64, ELFDATA2LSB, ELFMAG0, ELFMAG1, ELFMAG2, ELFMAG3, ELFOSABI_LINUX,
        EM_X86_64, ET_EXEC, EV_CURRENT, Elf64_Addr, Elf64_Dyn, Elf64_Ehdr, Elf64_Half, Elf64_Off,
        Elf64_Phdr, Elf64_Rela, Elf64_Shdr, Elf64_Sym, Elf64_Word, Elf64_Xword, PF_R, PF_W, PF_X,
        PT_DYNAMIC, PT_INTERP, PT_LOAD, R_X86_64_JUMP_SLOT, SHF_ALLOC, SHF_EXECINSTR, SHF_WRITE,
        SHT_DYNAMIC, SHT_DYNSYM, SHT_HASH, SHT_NOBITS, SHT_NULL, SHT_PROGBITS, SHT_RELA,
        SHT_STRTAB,
    },
    linker::{
        Linker, LinkerError,
        dynamic::{DYNAMIC_FIXED_ENTRY_COUNT, DynsymHashTable, GOT_ENTRY_BYTE_SIZE},
        section::{
            OutputSection, OutputSectionBytesKind, OutputSectionList, PAGE_SIZE, TEXT_BASE_ADDR,
        },
        symbol::ResolvedDynSym,
    },
};

// .shstrtab, .dynstr の中身と、名前からその中でのオフセットを引くためのテーブル
pub(super) struct Strtab {
    pub(super) bytes: Vec<u8>,
    offsets: HashMap<String, Elf64_Word>,
}

impl Strtab {
    pub(super) fn new(strs: Vec<String>) -> Self {
        let mut bytes = vec![0u8]; // the 0 at offset = 0 is for NULL section
        let mut offsets = HashMap::new();

        for name in strs.into_iter() {
            offsets.entry(name.clone()).or_insert_with(|| {
                let off = bytes.len() as Elf64_Word;
                bytes.extend_from_slice(name.as_bytes());
                bytes.push(0);
                off
            });
        }

        Self { bytes, offsets }
    }

    pub(super) fn offset_of(&self, name: &str) -> Elf64_Word {
        self.offsets[name]
    }
}

pub(super) struct OutputFileLayout {
    pub(super) text: usize,
    pub(super) plt: usize,
    pub(super) rodata: usize,
    pub(super) interp: usize,
    pub(super) dynsym: usize,
    pub(super) dynstr: usize,
    pub(super) hash: usize,
    pub(super) rela_plt: usize,
    pub(super) dynamic: usize,
    pub(super) data: usize,
    pub(super) shstrtab: usize,
    pub(super) shdrs: usize,

    pub(super) interp_len: usize,
    pub(super) dynsym_len: usize,
    pub(super) dynstr_len: usize,
    pub(super) hash_len: usize,
    pub(super) rela_plt_len: usize,
    pub(super) dynamic_len: usize,
}

// reserved size of Ehdr and Phdrs
// 5 means: PT_LOAD(text) + PT_LOAD(rodata) + PT_LOAD(data) + PT_INTERP + PT_DYNAMIC
pub(super) const OUTPUT_ELF_HEADER_RESERVED_SIZE: usize =
    std::mem::size_of::<Elf64_Ehdr>() + std::mem::size_of::<Elf64_Phdr>() * 5;

impl OutputFileLayout {
    fn new(
        sects: &OutputSectionList,
        interp_len: usize,
        dynsym_len: usize,
        dynstr_len: usize,
        hash_len: usize,
        rela_plt_len: usize,
        dynamic_len: usize,
        shstrtab_len: usize,
    ) -> Self {
        // segments loaded on memory in execution time:
        // R-X
        let text = OUTPUT_ELF_HEADER_RESERVED_SIZE;
        let plt = text + sects.text.bytes_len();
        // R--
        let rodata = (plt + sects.plt.bytes_len()).next_multiple_of(PAGE_SIZE);
        let interp = rodata + sects.rodata.bytes_len();
        let dynsym = interp + interp_len;
        let dynstr = dynsym + dynsym_len;
        let hash = dynstr + dynstr_len;
        let rela_plt = hash + hash_len;
        let dynamic = rela_plt + rela_plt_len;
        // RW-
        let data = (dynamic + dynamic_len).next_multiple_of(PAGE_SIZE);
        // .bss and .got has no bytes in ELF file

        // only in ELF file (not loaded):
        let shstrtab = data + sects.data.bytes_len();
        let shdrs = shstrtab + shstrtab_len;

        Self {
            text,
            plt,
            rodata,
            interp,
            dynsym,
            dynstr,
            hash,
            rela_plt,
            dynamic,
            data,
            shstrtab,
            shdrs,

            interp_len,
            dynsym_len,
            dynstr_len,
            hash_len,
            rela_plt_len,
            dynamic_len,
        }
    }

    // convert output ELF file offset to R-- segment virtual address.
    pub(super) fn addr_in_rodata_segment(
        &self,
        sects: &OutputSectionList,
        file_offset: usize,
    ) -> usize {
        sects.rodata.base + (file_offset - self.rodata)
    }
}

impl<'a> Linker<'a> {
    pub(super) fn output_elf(
        &self,
        sects: OutputSectionList,
        dyn_syms: &HashMap<String, ResolvedDynSym>,
        interp: Vec<u8>,
        dynstr: Strtab,
        dynsym: Vec<Elf64_Sym>,
        hash: DynsymHashTable,
    ) -> Result<Vec<u8>, LinkerError> {
        let entry = self.find_entry_point(&sects)?;

        let shstrtab = Strtab::new(vec![
            ".text".into(),
            ".plt".into(),
            ".rodata".into(),
            ".interp".into(),
            ".dynsym".into(),
            ".dynstr".into(),
            ".hash".into(),
            ".rela.plt".into(),
            ".dynamic".into(),
            ".data".into(),
            ".bss".into(),
            ".got".into(),
            ".shstrtab".into(),
        ]);

        let rela_plt = self.generate_rela_plt(&sects, dyn_syms.len());

        let layout = OutputFileLayout::new(
            &sects,
            interp.len(),
            dynsym.len() * std::mem::size_of::<Elf64_Sym>(),
            dynstr.bytes.len(),
            hash.bytes_len(),
            rela_plt.len() * std::mem::size_of::<Elf64_Rela>(),
            (self.shared_objs.len() + DYNAMIC_FIXED_ENTRY_COUNT) * std::mem::size_of::<Elf64_Dyn>(),
            shstrtab.bytes.len(),
        );

        let phdrs = self.generate_phdrs(&sects, &layout);
        let shdrs = self.generate_shdrs(&sects, &shstrtab, &layout);
        let ehdr = self.generate_ehdr(entry, &phdrs, &shdrs, &layout);
        let dynamic = self.generate_dynamic(&sects, &layout, &dynstr);

        /* -- write bytes to `out` (ELF file) -- */
        let mut out = vec![0u8; layout.shdrs + shdrs.len() * std::mem::size_of::<Elf64_Shdr>()];

        out[..std::mem::size_of::<Elf64_Ehdr>()].copy_from_slice(ehdr.as_bytes());

        let mut off = std::mem::size_of::<Elf64_Ehdr>();
        for phdr in &phdrs {
            out[off..off + std::mem::size_of::<Elf64_Phdr>()].copy_from_slice(phdr.as_bytes());
            off += std::mem::size_of::<Elf64_Phdr>();
        }

        write_section_bytes(&mut out, layout.text, &sects.text);
        write_section_bytes(&mut out, layout.plt, &sects.plt);
        write_section_bytes(&mut out, layout.rodata, &sects.rodata);

        out[layout.interp..layout.interp + interp.len()].copy_from_slice(&interp);

        let mut off = layout.dynsym;
        for sym in &dynsym {
            out[off..off + std::mem::size_of::<Elf64_Sym>()].copy_from_slice(sym.as_bytes());
            off += std::mem::size_of::<Elf64_Sym>();
        }
        out[layout.dynstr..layout.dynstr + dynstr.bytes.len()].copy_from_slice(&dynstr.bytes);
        out[layout.hash..layout.hash + hash.bytes_len()].copy_from_slice(&hash.as_bytes());

        let mut off = layout.rela_plt;
        for rela in &rela_plt {
            out[off..off + std::mem::size_of::<Elf64_Rela>()].copy_from_slice(rela.as_bytes());
            off += std::mem::size_of::<Elf64_Rela>();
        }

        let mut off = layout.dynamic;
        for d in &dynamic {
            out[off..off + std::mem::size_of::<Elf64_Dyn>()].copy_from_slice(d.as_bytes());
            off += std::mem::size_of::<Elf64_Dyn>();
        }

        write_section_bytes(&mut out, layout.data, &sects.data);
        // .bss and .got have no bytes in ELF file.

        out[layout.shstrtab..layout.shstrtab + shstrtab.bytes.len()]
            .copy_from_slice(&shstrtab.bytes);

        let mut off = layout.shdrs;
        for shdr in &shdrs {
            out[off..off + std::mem::size_of::<Elf64_Shdr>()].copy_from_slice(shdr.as_bytes());
            off += std::mem::size_of::<Elf64_Shdr>();
        }

        Ok(out)
    }

    fn find_entry_point(&self, sects: &OutputSectionList) -> Result<usize, LinkerError> {
        for (obj_index, obj) in self.objs.iter().enumerate() {
            if let Some(sym_index) = obj
                .symtab
                .syms
                .iter()
                .position(|s| s.name == "_start" && s.sym.is_resolved_index())
            {
                return self.symbol_address(obj_index, sym_index, sects);
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
        // Ehdr + Phdrs + .text + .plt
        let text = Elf64_Phdr {
            p_type: PT_LOAD,
            p_flags: PF_R | PF_X,
            p_offset: 0,
            p_vaddr: TEXT_BASE_ADDR as Elf64_Addr,
            p_paddr: TEXT_BASE_ADDR as Elf64_Addr,
            p_filesz: (layout.text + sects.text.bytes_len() + sects.plt.bytes_len()) as Elf64_Xword,
            p_memsz: (layout.text + sects.text.bytes_len() + sects.plt.bytes_len()) as Elf64_Xword,
            p_align: PAGE_SIZE as Elf64_Xword,
        };
        // .rodata + .interp + .dynsym + .dynstr + .hash + .rela.plt + .dynamic
        // metadata used by dynamic linker also are included in this segment.
        let rodata_end = layout.dynamic + layout.dynamic_len;
        let rodata = Elf64_Phdr {
            p_type: PT_LOAD,
            p_flags: PF_R,
            p_offset: layout.rodata as Elf64_Off,
            p_vaddr: sects.rodata.base as Elf64_Addr,
            p_paddr: sects.rodata.base as Elf64_Addr,
            p_filesz: (rodata_end - layout.rodata) as Elf64_Xword,
            p_memsz: (rodata_end - layout.rodata) as Elf64_Xword,
            p_align: PAGE_SIZE as Elf64_Xword,
        };
        // .data + .bss + .got
        let data = Elf64_Phdr {
            p_type: PT_LOAD,
            p_flags: PF_R | PF_W,
            p_offset: layout.data as Elf64_Off,
            p_vaddr: sects.data.base as Elf64_Addr,
            p_paddr: sects.data.base as Elf64_Addr,
            p_filesz: sects.data.bytes_len() as Elf64_Xword,
            // .bss and .got have no bytes in ELF file,
            // so set p_memsz larger than p_filesz by .bss size + .got size.
            // OS loader will fill this gap with zero.
            p_memsz: (sects.data.bytes_len() + sects.bss.bytes_len() + sects.got.bytes_len())
                as Elf64_Xword,
            p_align: PAGE_SIZE as Elf64_Xword,
        };

        let interp_addr = layout.addr_in_rodata_segment(sects, layout.interp);
        let interp = Elf64_Phdr {
            p_type: PT_INTERP,
            p_flags: PF_R,
            p_offset: layout.interp as Elf64_Off,
            p_vaddr: interp_addr as Elf64_Addr,
            p_paddr: interp_addr as Elf64_Addr,
            p_filesz: layout.interp_len as Elf64_Xword,
            p_memsz: layout.interp_len as Elf64_Xword,
            p_align: 1,
        };

        let dynamic_addr = layout.addr_in_rodata_segment(sects, layout.dynamic);
        let dynamic = Elf64_Phdr {
            p_type: PT_DYNAMIC,
            p_flags: PF_R,
            p_offset: layout.dynamic as Elf64_Off,
            p_vaddr: dynamic_addr as Elf64_Addr,
            p_paddr: dynamic_addr as Elf64_Addr,
            p_filesz: layout.dynamic_len as Elf64_Xword,
            p_memsz: layout.dynamic_len as Elf64_Xword,
            p_align: 8,
        };

        vec![text, rodata, data, interp, dynamic]
    }

    fn generate_shdrs(
        &self,
        sects: &OutputSectionList,
        shstrtab: &Strtab,
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
        let plt = Elf64_Shdr {
            sh_name: shstrtab.offset_of(".plt"),
            sh_type: SHT_PROGBITS,
            sh_flags: SHF_ALLOC | SHF_EXECINSTR,
            sh_addr: sects.plt.base as Elf64_Addr,
            sh_offset: layout.plt as Elf64_Off,
            sh_size: sects.plt.bytes_len() as Elf64_Xword,
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
        let interp = Elf64_Shdr {
            sh_name: shstrtab.offset_of(".interp"),
            sh_type: SHT_PROGBITS,
            sh_flags: SHF_ALLOC,
            sh_addr: layout.addr_in_rodata_segment(sects, layout.interp) as Elf64_Addr,
            sh_offset: layout.interp as Elf64_Off,
            sh_size: layout.interp_len as Elf64_Xword,
            sh_link: 0,
            sh_info: 0,
            sh_addralign: 1,
            sh_entsize: 0,
        };
        let dynsym = Elf64_Shdr {
            sh_name: shstrtab.offset_of(".dynsym"),
            sh_type: SHT_DYNSYM,
            sh_flags: SHF_ALLOC,
            sh_addr: layout.addr_in_rodata_segment(sects, layout.dynsym) as Elf64_Addr,
            sh_offset: layout.dynsym as Elf64_Off,
            sh_size: layout.dynsym_len as Elf64_Xword,
            sh_link: 6, // section index of .dynstr
            sh_info: 0,
            sh_addralign: 8,
            sh_entsize: std::mem::size_of::<Elf64_Sym>() as Elf64_Xword,
        };
        let dynstr = Elf64_Shdr {
            sh_name: shstrtab.offset_of(".dynstr"),
            sh_type: SHT_STRTAB,
            sh_flags: SHF_ALLOC,
            sh_addr: layout.addr_in_rodata_segment(sects, layout.dynstr) as Elf64_Addr,
            sh_offset: layout.dynstr as Elf64_Off,
            sh_size: layout.dynstr_len as Elf64_Xword,
            sh_link: 0,
            sh_info: 0,
            sh_addralign: 1,
            sh_entsize: 0,
        };
        let hash = Elf64_Shdr {
            sh_name: shstrtab.offset_of(".hash"),
            sh_type: SHT_HASH,
            sh_flags: SHF_ALLOC,
            sh_addr: layout.addr_in_rodata_segment(sects, layout.hash) as Elf64_Addr,
            sh_offset: layout.hash as Elf64_Off,
            sh_size: layout.hash_len as Elf64_Xword,
            sh_link: 5, // section index of .dynsym
            sh_info: 0,
            sh_addralign: 4,
            sh_entsize: 0,
        };
        let rela_plt = Elf64_Shdr {
            sh_name: shstrtab.offset_of(".rela.plt"),
            sh_type: SHT_RELA,
            sh_flags: SHF_ALLOC,
            sh_addr: layout.addr_in_rodata_segment(sects, layout.rela_plt) as Elf64_Addr,
            sh_offset: layout.rela_plt as Elf64_Off,
            sh_size: layout.rela_plt_len as Elf64_Xword,
            sh_link: 5, // section index of .dynsym
            sh_info: 2, // section index of .plt
            sh_addralign: 8,
            sh_entsize: std::mem::size_of::<Elf64_Rela>() as Elf64_Xword,
        };
        let dynamic = Elf64_Shdr {
            sh_name: shstrtab.offset_of(".dynamic"),
            sh_type: SHT_DYNAMIC,
            sh_flags: SHF_ALLOC | SHF_WRITE,
            sh_addr: layout.addr_in_rodata_segment(sects, layout.dynamic) as Elf64_Addr,
            sh_offset: layout.dynamic as Elf64_Off,
            sh_size: layout.dynamic_len as Elf64_Xword,
            sh_link: 6, // section index of .dynstr
            sh_info: 0,
            sh_addralign: 8,
            sh_entsize: std::mem::size_of::<Elf64_Dyn>() as Elf64_Xword,
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
        let got = Elf64_Shdr {
            sh_name: shstrtab.offset_of(".got"),
            sh_type: SHT_NOBITS,
            sh_flags: SHF_ALLOC | SHF_WRITE,
            sh_addr: sects.got.base as Elf64_Addr,
            // .got has no bytes in ELF file (SHT_NOBITS).
            // but we need to set virtual offset.
            sh_offset: (layout.data + sects.data.bytes_len() + sects.bss.bytes_len()) as Elf64_Off,
            sh_size: sects.got.bytes_len() as Elf64_Xword,
            sh_link: 0,
            sh_info: 0,
            sh_addralign: 8,
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
        vec![
            null,
            text,
            plt,
            rodata,
            interp,
            dynsym,
            dynstr,
            hash,
            rela_plt,
            dynamic,
            data,
            bss,
            got,
            shstrtab_shdr,
        ]
    }

    fn generate_rela_plt(&self, sects: &OutputSectionList, dynsyms_len: usize) -> Vec<Elf64_Rela> {
        (0..dynsyms_len)
            .map(|i| Elf64_Rela {
                r_offset: (sects.got.base + i * GOT_ENTRY_BYTE_SIZE) as Elf64_Addr,
                r_info: ELF64_R_INFO((i + 1) as u32, R_X86_64_JUMP_SLOT),
                r_addend: 0,
            })
            .collect()
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

impl Elf64_Sym {
    fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self as *const Self as *const u8,
                std::mem::size_of::<Self>(),
            )
        }
    }
}

impl Elf64_Rela {
    fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self as *const Self as *const u8,
                std::mem::size_of::<Self>(),
            )
        }
    }
}
