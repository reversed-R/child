use std::collections::HashMap;

use crate::{
    elf::elf64::{
        DF_1_NOW, DT_FLAGS_1, DT_HASH, DT_JMPREL, DT_NEEDED, DT_NULL, DT_PLTGOT, DT_PLTREL,
        DT_PLTRELSZ, DT_RELA, DT_STRSZ, DT_STRTAB, DT_SYMENT, DT_SYMTAB, ELF64_ST_INFO,
        ELF64_ST_TYPE, Elf64_Dyn, Elf64_Dyn_un, Elf64_Sxword, Elf64_Sym, Elf64_Xword, SHN_UNDEF,
        STB_GLOBAL, STV_DEFAULT,
    },
    linker::{
        Linker, LinkerError, Strtab,
        output::OutputFileLayout,
        section::{OutputSectionBytesKind, OutputSectionList},
        symbol::ResolvedDynSym,
    },
};

pub(super) const PLT_ENTRY_BYTE_SIZE: usize = 6;
pub(super) const GOT_ENTRY_BYTE_SIZE: usize = 8; // address size

// SYMTAB, STRTAB, STRSZ, SYMENT, HASH, PLTGOT, JMPREL,
// PLTRELSZ, PLTREL, FLAGS_1, NULL
pub(super) const DYNAMIC_FIXED_ENTRY_COUNT: usize = 11;

impl<'a> Linker<'a> {
    pub(super) fn fill_plt(&self, sects: &mut OutputSectionList, dynsym_len: usize) {
        if let OutputSectionBytesKind::Bytes(bytes) = &mut sects.plt.bytes {
            for i in 0..dynsym_len {
                let got_slot_addr = sects.got.base + i * GOT_ENTRY_BYTE_SIZE;
                // base address of rip relative addressing is the next instruction address
                let next_instr_addr = sects.plt.base + (i + 1) * PLT_ENTRY_BYTE_SIZE;
                let disp32 = (got_slot_addr as i64 - next_instr_addr as i64) as i32;

                // jmp qword ptr [rip + disp32]  (FF /4, ModRM=00 100 101)
                let mut instr_bytes = [0u8; PLT_ENTRY_BYTE_SIZE];
                instr_bytes[0] = 0xff;
                instr_bytes[1] = 0x25;
                instr_bytes[2..6].copy_from_slice(&disp32.to_le_bytes());

                bytes[i * PLT_ENTRY_BYTE_SIZE..(i + 1) * PLT_ENTRY_BYTE_SIZE]
                    .copy_from_slice(&instr_bytes);
            }
        }
    }

    pub(super) fn dynamic_metadata(
        &self,
        dyn_syms: &'a HashMap<String, ResolvedDynSym>,
    ) -> Result<(Vec<u8>, Vec<Elf64_Sym>, Strtab, DynsymHashTable), LinkerError> {
        let interp = self.generate_interp()?;

        let dynstr_names = dyn_syms
            .keys()
            .cloned()
            .chain(self.shared_objs.iter().filter_map(|so| so.soname.clone()))
            .collect::<Vec<_>>();
        let dynstr = Strtab::new(dynstr_names);

        let (dynsym, hash) = self.generate_dynsyms(dyn_syms, &dynstr);

        Ok((interp, dynsym, dynstr, hash))
    }

    pub(super) fn generate_dynsyms(
        &self,
        dyn_syms: &HashMap<String, ResolvedDynSym>,
        dynstr: &Strtab,
    ) -> (Vec<Elf64_Sym>, DynsymHashTable) {
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

        let syms = [
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
        .concat();

        (syms, hash)
    }

    fn generate_interp(&self) -> Result<Vec<u8>, LinkerError> {
        match &self.dyn_linker {
            Some(path) => {
                let mut bytes = path.as_bytes().to_vec();
                bytes.push(0);
                Ok(bytes)
            }
            None => Err(LinkerError::DynamicLinkerPathRequired),
        }
    }

    pub(super) fn generate_dynamic(
        &self,
        sects: &OutputSectionList,
        layout: &OutputFileLayout,
        dynstr: &Strtab,
    ) -> Vec<Elf64_Dyn> {
        self.shared_objs
            .iter()
            .map(|so| {
                Elf64_Dyn::new_with_val(
                    DT_NEEDED,
                    dynstr.offset_of(so.soname.as_deref().expect("shared object name not found"))
                        as Elf64_Xword,
                )
            })
            .chain([
                // 11 entries
                Elf64_Dyn::new_with_val(
                    DT_SYMTAB,
                    layout.addr_in_rodata_segment(sects, layout.dynsym) as Elf64_Xword,
                ),
                Elf64_Dyn::new_with_val(
                    DT_STRTAB,
                    layout.addr_in_rodata_segment(sects, layout.dynstr) as Elf64_Xword,
                ),
                Elf64_Dyn::new_with_val(DT_STRSZ, layout.dynstr_len as Elf64_Xword),
                Elf64_Dyn::new_with_val(DT_SYMENT, std::mem::size_of::<Elf64_Sym>() as Elf64_Xword),
                Elf64_Dyn::new_with_val(
                    DT_HASH,
                    layout.addr_in_rodata_segment(sects, layout.hash) as Elf64_Xword,
                ),
                Elf64_Dyn::new_with_val(DT_PLTGOT, sects.got.base as Elf64_Xword),
                Elf64_Dyn::new_with_val(
                    DT_JMPREL,
                    layout.addr_in_rodata_segment(sects, layout.rela_plt) as Elf64_Xword,
                ),
                Elf64_Dyn::new_with_val(DT_PLTRELSZ, layout.rela_plt_len as Elf64_Xword),
                Elf64_Dyn::new_with_val(DT_PLTREL, DT_RELA as Elf64_Xword),
                // In current implementaion,
                // lazy binding not supported, dynamic symbols will be resolved at load time.
                Elf64_Dyn::new_with_val(DT_FLAGS_1, DF_1_NOW),
                Elf64_Dyn::new_with_val(DT_NULL, 0),
            ])
            .collect()
    }
}

impl Elf64_Dyn {
    pub(super) fn as_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self as *const Self as *const u8,
                std::mem::size_of::<Self>(),
            )
        }
    }
}

pub(super) struct DynsymHashTable {
    nbucket: u32,
    nchain: u32,
    bucket: Vec<u32>,
    chain: Vec<u32>,
}

impl DynsymHashTable {
    pub(super) fn as_bytes(&self) -> Vec<u8> {
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

    pub(super) fn bytes_len(&self) -> usize {
        8 + (self.nbucket + self.nchain) as usize * 4
    }
}

impl Elf64_Dyn {
    fn new_with_val(d_tag: Elf64_Sxword, d_val: Elf64_Xword) -> Self {
        Elf64_Dyn {
            d_tag,
            d_un: Elf64_Dyn_un { d_val },
        }
    }
}
