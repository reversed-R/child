use std::path::PathBuf;

use crate::elf::elf64::{
    EI_CLASS, EI_DATA, EI_MAG0, EI_MAG3, EI_VERSION, ELFCLASS64, ELFDATA2LSB, ELFMAG, EM_X86_64,
    EV_CURRENT, Elf64_Ehdr, Elf64_Shdr,
};

mod section;

#[derive(Debug)]
pub(crate) struct Elf64Parser<'a> {
    bin: &'a [u8],
    path: PathBuf,
    hdr: Elf64_Ehdr,
    shdrs: Vec<ElfSectionHeaderEntry>,
}

#[derive(Debug)]
struct ElfSectionHeaderEntry {
    name: String,
    hdr: Elf64_Shdr,
}

#[derive(Debug)]
pub(crate) enum ElfParseError {
    TooShort,
    InvalidShstrIndex,
    InvalidShstrEnd,
    InvalidMagic,
    InvalidClass,
    InvalidEndian,
    InvalidVersion,
    InvalidMachine,
    SectionNotFound { name: String },
    SectionNotFoundByIndex { index: usize },
    SymbolNotFound { index: usize },
}

impl<'a> Elf64Parser<'a> {
    pub(crate) fn new(bin: &'a [u8], path: PathBuf) -> Result<Self, ElfParseError> {
        /* load elf header */
        if bin.len() < std::mem::size_of::<Elf64_Ehdr>() {
            return Err(ElfParseError::TooShort);
        }

        let hdr_bytes: [u8; std::mem::size_of::<Elf64_Ehdr>()] =
            bin[..std::mem::size_of::<Elf64_Ehdr>()].try_into().unwrap();
        let hdr: Elf64_Ehdr = unsafe { std::mem::transmute(hdr_bytes) };

        if &hdr.e_ident[EI_MAG0..=EI_MAG3] != ELFMAG {
            return Err(ElfParseError::InvalidMagic);
        }
        if hdr.e_ident[EI_CLASS] != ELFCLASS64 {
            return Err(ElfParseError::InvalidClass);
        }
        if hdr.e_ident[EI_DATA] != ELFDATA2LSB {
            return Err(ElfParseError::InvalidEndian);
        }
        if hdr.e_ident[EI_VERSION] != EV_CURRENT as u8 || hdr.e_version != EV_CURRENT {
            return Err(ElfParseError::InvalidVersion);
        }
        if hdr.e_machine != EM_X86_64 {
            return Err(ElfParseError::InvalidMachine);
        }

        /* load section headers */
        if (bin.len() as u64)
            < hdr.e_shoff + (std::mem::size_of::<Elf64_Shdr>() as u64) * hdr.e_shnum as u64
        {
            return Err(ElfParseError::TooShort);
        }

        let shdrs = (0..hdr.e_shnum)
            .map(|i| {
                let begin = hdr.e_shoff as usize + std::mem::size_of::<Elf64_Shdr>() * i as usize;
                let shdr_bytes: [u8; std::mem::size_of::<Elf64_Shdr>()] = bin
                    [begin..begin + std::mem::size_of::<Elf64_Shdr>()]
                    .try_into()
                    .unwrap();
                let shdr: Elf64_Shdr = unsafe { std::mem::transmute(shdr_bytes) };
                shdr
            })
            .collect::<Vec<_>>();

        if hdr.e_shstrndx >= hdr.e_shnum {
            return Err(ElfParseError::InvalidShstrIndex);
        }

        let shstr = &shdrs[hdr.e_shstrndx as usize];
        if bin.len() < (shstr.sh_offset + shstr.sh_size) as usize {
            return Err(ElfParseError::TooShort);
        }
        let shstrtab = &bin[shstr.sh_offset as usize..(shstr.sh_offset + shstr.sh_size) as usize];
        let shdrs = shdrs
            .into_iter()
            .map(|shdr| {
                if shstrtab.len() < shdr.sh_name as usize {
                    Err(ElfParseError::TooShort)
                } else {
                    for i in shdr.sh_name as usize..shstrtab.len() {
                        if shstrtab[i] == 0 {
                            let sh_name =
                                String::from_utf8(shstrtab[shdr.sh_name as usize..i].to_vec())
                                    .expect("invalid utf8");
                            return Ok(ElfSectionHeaderEntry {
                                name: sh_name,
                                hdr: shdr,
                            });
                        }
                    }

                    Err(ElfParseError::InvalidShstrEnd)
                }
            })
            .collect::<Result<_, _>>()?;

        Ok(Self {
            bin,
            path,
            hdr,
            shdrs,
        })
    }

    fn get_section_by_name(
        &'a self,
        name: &str,
    ) -> Option<Result<(&'a Elf64_Shdr, &'a [u8]), ElfParseError>> {
        self.shdrs
            .iter()
            .find(|shdr| shdr.name == name)
            .map(|shdr| {
                if self.bin.len() < (shdr.hdr.sh_offset + shdr.hdr.sh_size) as usize {
                    Err(ElfParseError::TooShort)
                } else {
                    Ok((
                        &shdr.hdr,
                        &self.bin[shdr.hdr.sh_offset as usize
                            ..(shdr.hdr.sh_offset + shdr.hdr.sh_size) as usize],
                    ))
                }
            })
    }
}
