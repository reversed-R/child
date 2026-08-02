use std::collections::HashMap;

use crate::{
    elf::elf64::{
        ELF64_R_INFO, ELF64_ST_INFO, ELF64_ST_TYPE, ELFCLASS64, ELFDATA2LSB, ELFMAG0, ELFMAG1,
        ELFMAG2, ELFMAG3, ELFOSABI_LINUX, EM_X86_64, ET_EXEC, EV_CURRENT, Elf64_Addr, Elf64_Ehdr,
        Elf64_Half, Elf64_Off, Elf64_Phdr, Elf64_Rela, Elf64_Shdr, Elf64_Sym, Elf64_Word,
        Elf64_Xword, PF_R, PF_W, PF_X, PT_LOAD, R_X86_64_JUMP_SLOT, SHF_ALLOC, SHF_EXECINSTR,
        SHF_WRITE, SHN_UNDEF, SHT_DYNSYM, SHT_HASH, SHT_NOBITS, SHT_NULL, SHT_PROGBITS, SHT_RELA,
        SHT_STRTAB, STB_GLOBAL, STV_DEFAULT,
    },
    linker::{
        Linker, LinkerError,
        dynamic::GOT_ENTRY_BYTE_SIZE,
        section::{
            OutputSection, OutputSectionBytesKind, OutputSectionList, PAGE_SIZE, TEXT_BASE_ADDR,
        },
        symbol::ResolvedDynSym,
    },
};

// .shstrtab, .dynstr の中身と、名前からその中でのオフセットを引くためのテーブル
struct Strtab<'a> {
    bytes: Vec<u8>,
    offsets: HashMap<&'a str, Elf64_Word>,
}

impl<'a> Strtab<'a> {
    fn new(strs: Vec<&'a str>) -> Self {
        let mut bytes = vec![0u8]; // the 0 at offset = 0 is for NULL section
        let mut offsets = HashMap::new();

        for name in strs.into_iter() {
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
    plt: usize,
    rodata: usize,
    data: usize,
    dynsym: usize,
    dynstr: usize,
    hash: usize,
    rela_plt: usize,
    shstrtab: usize,
    shdrs: usize,

    dynsym_len: usize,
    dynstr_len: usize,
    hash_len: usize,
    rela_plt_len: usize,
}

// reserved size of Ehdr and Phdrs
// 3 means segments for text, rodata, data+bss
pub(super) const OUTPUT_ELF_HEADER_RESERVED_SIZE: usize =
    std::mem::size_of::<Elf64_Ehdr>() + std::mem::size_of::<Elf64_Phdr>() * 3;

impl OutputFileLayout {
    fn new(
        sects: &OutputSectionList,
        dynsym_len: usize,
        dynstr_len: usize,
        hash_len: usize,
        rela_plt_len: usize,
        shstrtab_len: usize,
    ) -> Self {
        // segments loaded on memory in execution time:
        // R-X
        let text = OUTPUT_ELF_HEADER_RESERVED_SIZE;
        let plt = text + sects.text.bytes_len();
        // R--
        let rodata = (plt + sects.plt.bytes_len()).next_multiple_of(PAGE_SIZE);
        // RW-
        let data = (rodata + sects.rodata.bytes_len()).next_multiple_of(PAGE_SIZE);
        // .bss and .got has no bytes in ELF file

        // only in ELF file:
        let dynsym = data + sects.data.bytes_len();
        let dynstr = dynsym + dynsym_len;
        let hash = dynstr + dynstr_len;
        let rela_plt = hash + hash_len;
        let shstrtab = rela_plt + rela_plt_len;
        let shdrs = shstrtab + shstrtab_len;

        Self {
            text,
            plt,
            rodata,
            data,
            dynsym,
            dynstr,
            hash,
            rela_plt,
            shstrtab,
            shdrs,

            dynsym_len,
            dynstr_len,
            hash_len,
            rela_plt_len,
        }
    }
}

impl<'a> Linker<'a> {
    pub(super) fn output_elf(
        &self,
        sects: OutputSectionList,
        dyn_syms: &HashMap<String, ResolvedDynSym>,
    ) -> Result<Vec<u8>, LinkerError> {
        let entry = self.find_entry_point(&sects)?;

        let shstrtab = Strtab::new(vec![
            ".text",
            ".plt",
            ".rodata",
            ".data",
            ".bss",
            ".got",
            ".dynsym",
            ".dynstr",
            ".hash",
            ".rela.plt",
            ".shstrtab",
        ]);

        let (dynsym, dynstr, hash) = self.generate_dynsyms(dyn_syms);
        let layout = OutputFileLayout::new(
            &sects,
            dynsym.len() * std::mem::size_of::<Elf64_Sym>(), // at 0 is NULL entry
            dynstr.bytes.len(),
            hash.bytes_len(),
            dyn_syms.len() * std::mem::size_of::<Elf64_Rela>(),
            shstrtab.bytes.len(),
        );

        let phdrs = self.generate_phdrs(&sects, &layout);
        let shdrs = self.generate_shdrs(&sects, &shstrtab, &layout);
        let ehdr = self.generate_ehdr(entry, &phdrs, &shdrs, &layout);
        let rela_plt = self.generate_rela_plt(&sects, dyn_syms.len());

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
        write_section_bytes(&mut out, layout.data, &sects.data);
        // .bss has no bytes in ELF file.
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
        // .rodata
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

        vec![text, rodata, data]
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
            sh_addralign: 1,
            sh_entsize: 0,
        };
        let dynsym = Elf64_Shdr {
            sh_name: shstrtab.offset_of(".dynsym"),
            sh_type: SHT_DYNSYM,
            sh_flags: 0,
            sh_addr: 0,
            sh_offset: layout.dynsym as Elf64_Off,
            sh_size: layout.dynsym_len as Elf64_Xword,
            sh_link: 0,
            sh_info: 0,
            sh_addralign: 1,
            sh_entsize: 0,
        };
        let dynstr = Elf64_Shdr {
            sh_name: shstrtab.offset_of(".dynstr"),
            sh_type: SHT_STRTAB,
            sh_flags: 0,
            sh_addr: 0,
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
            sh_flags: 0,
            sh_addr: 0,
            sh_offset: layout.hash as Elf64_Off,
            sh_size: layout.hash_len as Elf64_Xword,
            sh_link: 0,
            sh_info: 0,
            sh_addralign: 1,
            sh_entsize: 0,
        };
        let rela_plt = Elf64_Shdr {
            sh_name: shstrtab.offset_of(".rela.plt"),
            sh_type: SHT_RELA,
            sh_flags: 0,
            sh_addr: 0,
            sh_offset: layout.rela_plt as Elf64_Off,
            sh_size: layout.rela_plt_len as Elf64_Xword,
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
        vec![
            null,
            text,
            plt,
            rodata,
            data,
            bss,
            got,
            dynsym,
            dynstr,
            hash,
            rela_plt,
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

    fn generate_dynsyms(
        &self,
        dyn_syms: &'a HashMap<String, ResolvedDynSym>,
    ) -> (Vec<Elf64_Sym>, Strtab<'a>, DynsymHashTable) {
        let dynstr = Strtab::new(dyn_syms.keys().map(|name| name.as_str()).collect());

        // simplified implementaion.
        // this linker does not output .so,
        // so no one want to search symbols fast.
        let n = dyn_syms.len() as u32 + 1;
        let hash = DynsymHashTable {
            nbucket: 1,
            nchain: n,
            bucket: vec![if dyn_syms.is_empty() { 0 } else { 1 }],
            chain: (0..n)
                .map(|i| if i == 0 { 0 } else { (i + 1) % n })
                .collect(),
        };

        let mut dyn_syms_vec = dyn_syms.iter().collect::<Vec<_>>();
        dyn_syms_vec.sort_by_key(|(_, sym)| sym.dyn_index);

        (
            [
                // NULL entry
                vec![Elf64_Sym {
                    st_name: 0,
                    st_info: 0,
                    st_other: 0,
                    st_shndx: 0,
                    st_value: 0,
                    st_size: 0,
                }],
                dyn_syms_vec
                    .iter()
                    .map(|(name, sym)| {
                        let st_type = ELF64_ST_TYPE(
                            self.shared_objs[sym.shared_obj_index].symtab.syms[sym.sym_index]
                                .sym
                                .st_info,
                        );
                        Elf64_Sym {
                            st_name: dynstr.offset_of(name.as_str()),
                            st_info: ELF64_ST_INFO(STB_GLOBAL, st_type),
                            st_other: STV_DEFAULT,
                            st_shndx: SHN_UNDEF,
                            st_value: 0,
                            st_size: 0,
                        }
                    })
                    .collect(),
            ]
            .concat(),
            dynstr,
            hash,
        )
    }
}

struct DynsymHashTable {
    nbucket: u32,
    nchain: u32,
    bucket: Vec<u32>,
    chain: Vec<u32>,
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

impl DynsymHashTable {
    fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![0; self.bytes_len()];

        bytes[0..4].copy_from_slice(&self.nbucket.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.nchain.to_le_bytes());
        bytes[8..8 + self.nbucket as usize * 4].copy_from_slice(
            &self
                .bucket
                .iter()
                .flat_map(|b| b.to_le_bytes())
                .collect::<Vec<_>>(),
        );
        bytes[8 + self.nbucket as usize * 4..].copy_from_slice(
            &self
                .chain
                .iter()
                .flat_map(|b| b.to_le_bytes())
                .collect::<Vec<_>>(),
        );

        bytes
    }

    fn bytes_len(&self) -> usize {
        8 + (self.nbucket + self.nchain) as usize * 4
    }
}
