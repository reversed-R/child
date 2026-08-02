use std::collections::HashMap;

use crate::{
    elf::elf64::{R_X86_64_PC32, R_X86_64_PLT32},
    linker::{
        ElfObject, Linker, LinkerError,
        dynamic::PLT_ENTRY_BYTE_SIZE,
        section::{OutputSectionBytesKind, OutputSectionList},
        symbol::{ResolvedDynSym, ResolvedObjIndexKind},
    },
    parser::{ElfParseError, section::rela_text::ElfRela},
};

impl<'a> Linker<'a> {
    pub(super) fn relocate(
        &self,
        sects: &mut OutputSectionList,
        dyn_syms: &HashMap<String, ResolvedDynSym>,
    ) -> Result<(), LinkerError> {
        for (obj_index, o) in self.objs.iter().enumerate() {
            if let Some(relas) = &o.rela_text {
                for rela in &relas.relas {
                    self.patch_relocation(obj_index, o, rela, sects, dyn_syms)?;
                }
            }
        }

        Ok(())
    }

    fn patch_relocation(
        &self,
        rela_obj_index: usize,
        rela_obj: &ElfObject,
        rela: &ElfRela,
        sects: &mut OutputSectionList,
        dyn_syms: &HashMap<String, ResolvedDynSym>,
    ) -> Result<(), LinkerError> {
        // R_X86_64_PC32 / R_X86_64_PLT32
        // PC相対アドレス: S + A - P (4 byte)
        match rela.rela.r_type() {
            R_X86_64_PC32 | R_X86_64_PLT32 => {}
            r_type => return Err(LinkerError::UnsupportedRelocationType { r_type }),
        }

        let sym_index = rela.rela.r_sym() as usize;
        let sym = &rela_obj.symtab.syms[sym_index];
        let resolved = sym.resolved_sym.as_ref().unwrap();

        // P: パッチする箇所そのものの配置確定後のアドレス
        // .rela.text は常に .text セクションを対象にするので .text 側の
        // base/section_offsets を見ればよい
        // section_offsets は
        // rela_obj_index のオブジェクトの .text が、マージ後の .text の
        // どこから始まるかを持っているので、そこに r_offset
        // (元々のオブジェクト自身の .text 内でのパッチ位置)を足す
        let place_offset =
            sects.text.section_offsets[&rela_obj_index] + rela.rela.r_offset as usize;
        let place_addr = sects.text.base + place_offset;

        // S: 参照先シンボルの、配置確定後の最終アドレス
        let sym_addr = match &resolved.obj_index {
            ResolvedObjIndexKind::Obj(o_index) => {
                self.symbol_address(*o_index, resolved.sym_index, sects)?
            }
            ResolvedObjIndexKind::Shared(_) => {
                let dyn_index = dyn_syms[&sym.name].dyn_index;
                sects.plt.base + dyn_index * PLT_ENTRY_BYTE_SIZE
            }
        };

        // PC相対アドレス: S + A - P
        // A: append
        let value = (sym_addr as i64 + rela.rela.r_addend - place_addr as i64) as i32;

        if let OutputSectionBytesKind::Bytes(bytes) = &mut sects.text.bytes {
            bytes[place_offset..place_offset + 4].copy_from_slice(&value.to_le_bytes());
        }

        Ok(())
    }

    // (obj_index, sym_index) で指定したシンボルが、配置確定後の出力ファイル上で
    // 最終的にどのアドレスに置かれるかを求める
    pub(super) fn symbol_address(
        &self,
        obj_index: usize,
        sym_index: usize,
        sects: &OutputSectionList,
    ) -> Result<usize, LinkerError> {
        let obj = &self.objs[obj_index];
        let sym = &obj.symtab.syms[sym_index].sym;

        let sect_name = obj
            .elf
            .shdrs
            .get(sym.st_shndx as usize)
            .ok_or(LinkerError::ParseError {
                errors: vec![ElfParseError::SectionNotFoundByIndex {
                    index: sym.st_shndx as usize,
                }],
            })?
            .name
            .as_str();

        let output_sect = match sect_name {
            ".text" => &sects.text,
            ".data" => &sects.data,
            ".rodata" => &sects.rodata,
            ".bss" => &sects.bss,
            other => {
                return Err(LinkerError::UnsupportedSection {
                    name: other.to_string(),
                });
            }
        };

        Ok(output_sect.base + output_sect.section_offsets[&obj_index] + sym.st_value as usize)
    }
}
