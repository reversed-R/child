//! ELF64 definitions for x86_64 Linux (System-V ABI), ported from `<elf.h>`.
//! Only the ELF64 subset relevant to this target is kept; Elf32_*, other
//! architectures' relocations/notes, and Solaris/prelink-only tags are omitted.
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

/* Type for a 16-bit quantity.  */
pub type Elf64_Half = u16;

/* Types for signed and unsigned 32-bit quantities.  */
pub type Elf64_Word = u32;
pub type Elf64_Sword = i32;

/* Types for signed and unsigned 64-bit quantities.  */
pub type Elf64_Xword = u64;
pub type Elf64_Sxword = i64;

/* Type of addresses.  */
pub type Elf64_Addr = u64;

/* Type of file offsets.  */
pub type Elf64_Off = u64;

/* Type for section indices, which are 16-bit quantities.  */
pub type Elf64_Section = u16;

/* Type for version symbol information.  */
pub type Elf64_Versym = Elf64_Half;

/* The ELF file header.  This appears at the start of every ELF file.  */

pub const EI_NIDENT: usize = 16;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Elf64_Ehdr {
    pub e_ident: [u8; EI_NIDENT], /* Magic number and other info */
    pub e_type: Elf64_Half,       /* Object file type */
    pub e_machine: Elf64_Half,    /* Architecture */
    pub e_version: Elf64_Word,    /* Object file version */
    pub e_entry: Elf64_Addr,      /* Entry point virtual address */
    pub e_phoff: Elf64_Off,       /* Program header table file offset */
    pub e_shoff: Elf64_Off,       /* Section header table file offset */
    pub e_flags: Elf64_Word,      /* Processor-specific flags */
    pub e_ehsize: Elf64_Half,     /* ELF header size in bytes */
    pub e_phentsize: Elf64_Half,  /* Program header table entry size */
    pub e_phnum: Elf64_Half,      /* Program header table entry count */
    pub e_shentsize: Elf64_Half,  /* Section header table entry size */
    pub e_shnum: Elf64_Half,      /* Section header table entry count */
    pub e_shstrndx: Elf64_Half,   /* Section header string table index */
}

/* Fields in the e_ident array.  The EI_* consts are indices into the
array.  The consts under each EI_* const are the values the byte
may have.  */

pub const EI_MAG0: usize = 0; /* File identification byte 0 index */
pub const ELFMAG0: u8 = 0x7f; /* Magic number byte 0 */

pub const EI_MAG1: usize = 1; /* File identification byte 1 index */
pub const ELFMAG1: u8 = b'E'; /* Magic number byte 1 */

pub const EI_MAG2: usize = 2; /* File identification byte 2 index */
pub const ELFMAG2: u8 = b'L'; /* Magic number byte 2 */

pub const EI_MAG3: usize = 3; /* File identification byte 3 index */
pub const ELFMAG3: u8 = b'F'; /* Magic number byte 3 */

/* Conglomeration of the identification bytes, for easy testing as a word.  */
pub const ELFMAG: &[u8; 4] = b"\x7fELF";
pub const SELFMAG: usize = 4;

pub const EI_CLASS: usize = 4; /* File class byte index */
pub const ELFCLASSNONE: u8 = 0; /* Invalid class */
pub const ELFCLASS32: u8 = 1; /* 32-bit objects (rejected by this linker) */
pub const ELFCLASS64: u8 = 2; /* 64-bit objects */
pub const ELFCLASSNUM: u8 = 3;

pub const EI_DATA: usize = 5; /* Data encoding byte index */
pub const ELFDATANONE: u8 = 0; /* Invalid data encoding */
pub const ELFDATA2LSB: u8 = 1; /* 2's complement, little endian (x86_64) */
pub const ELFDATA2MSB: u8 = 2; /* 2's complement, big endian */
pub const ELFDATANUM: u8 = 3;

pub const EI_VERSION: usize = 6; /* File version byte index */
/* Value must be EV_CURRENT */

pub const EI_OSABI: usize = 7; /* OS ABI identification */
pub const ELFOSABI_NONE: u8 = 0; /* UNIX System V ABI */
pub const ELFOSABI_SYSV: u8 = 0; /* Alias.  */
pub const ELFOSABI_GNU: u8 = 3; /* Object uses GNU ELF extensions.  */
pub const ELFOSABI_LINUX: u8 = ELFOSABI_GNU; /* Compatibility alias.  */

pub const EI_ABIVERSION: usize = 8; /* ABI version */

pub const EI_PAD: usize = 9; /* Byte index of padding bytes */

/* Legal values for e_type (object file type).  */

pub const ET_NONE: Elf64_Half = 0; /* No file type */
pub const ET_REL: Elf64_Half = 1; /* Relocatable file */
pub const ET_EXEC: Elf64_Half = 2; /* Executable file */
pub const ET_DYN: Elf64_Half = 3; /* Shared object file */
pub const ET_CORE: Elf64_Half = 4; /* Core file */
pub const ET_NUM: Elf64_Half = 5; /* Number of defined types */

/* Legal values for e_machine (architecture).  */

pub const EM_NONE: Elf64_Half = 0; /* No machine */
pub const EM_X86_64: Elf64_Half = 62; /* AMD x86-64 architecture */

/* Legal values for e_version (version).  */

pub const EV_NONE: Elf64_Word = 0; /* Invalid ELF version */
pub const EV_CURRENT: Elf64_Word = 1; /* Current version */
pub const EV_NUM: Elf64_Word = 2;

/* Section header.  */

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Elf64_Shdr {
    pub sh_name: Elf64_Word,       /* Section name (string tbl index) */
    pub sh_type: Elf64_Word,       /* Section type */
    pub sh_flags: Elf64_Xword,     /* Section flags */
    pub sh_addr: Elf64_Addr,       /* Section virtual addr at execution */
    pub sh_offset: Elf64_Off,      /* Section file offset */
    pub sh_size: Elf64_Xword,      /* Section size in bytes */
    pub sh_link: Elf64_Word,       /* Link to another section */
    pub sh_info: Elf64_Word,       /* Additional section information */
    pub sh_addralign: Elf64_Xword, /* Section alignment */
    pub sh_entsize: Elf64_Xword,   /* Entry size if section holds table */
}

/* Special section indices.  */

pub const SHN_UNDEF: Elf64_Section = 0; /* Undefined section */
pub const SHN_LORESERVE: Elf64_Section = 0xff00; /* Start of reserved indices */
pub const SHN_ABS: Elf64_Section = 0xfff1; /* Associated symbol is absolute */
pub const SHN_COMMON: Elf64_Section = 0xfff2; /* Associated symbol is common */
pub const SHN_XINDEX: Elf64_Section = 0xffff; /* Index is in extra table.  */
pub const SHN_HIRESERVE: Elf64_Section = 0xffff; /* End of reserved indices */

/* Legal values for sh_type (section type).  */

pub const SHT_NULL: Elf64_Word = 0; /* Section header table entry unused */
pub const SHT_PROGBITS: Elf64_Word = 1; /* Program data */
pub const SHT_SYMTAB: Elf64_Word = 2; /* Symbol table */
pub const SHT_STRTAB: Elf64_Word = 3; /* String table */
pub const SHT_RELA: Elf64_Word = 4; /* Relocation entries with addends */
pub const SHT_HASH: Elf64_Word = 5; /* Symbol hash table */
pub const SHT_DYNAMIC: Elf64_Word = 6; /* Dynamic linking information */
pub const SHT_NOTE: Elf64_Word = 7; /* Notes */
pub const SHT_NOBITS: Elf64_Word = 8; /* Program space with no data (bss) */
pub const SHT_REL: Elf64_Word = 9; /* Relocation entries, no addends */
pub const SHT_SHLIB: Elf64_Word = 10; /* Reserved */
pub const SHT_DYNSYM: Elf64_Word = 11; /* Dynamic linker symbol table */
pub const SHT_INIT_ARRAY: Elf64_Word = 14; /* Array of constructors */
pub const SHT_FINI_ARRAY: Elf64_Word = 15; /* Array of destructors */
pub const SHT_PREINIT_ARRAY: Elf64_Word = 16; /* Array of pre-constructors */
pub const SHT_GROUP: Elf64_Word = 17; /* Section group */
pub const SHT_SYMTAB_SHNDX: Elf64_Word = 18; /* Extended section indices */
pub const SHT_RELR: Elf64_Word = 19; /* RELR relative relocations */
pub const SHT_NUM: Elf64_Word = 20; /* Number of defined types.  */
pub const SHT_LOOS: Elf64_Word = 0x60000000; /* Start OS-specific.  */
pub const SHT_GNU_ATTRIBUTES: Elf64_Word = 0x6ffffff5; /* Object attributes.  */
pub const SHT_GNU_HASH: Elf64_Word = 0x6ffffff6; /* GNU-style hash table.  */
pub const SHT_GNU_verdef: Elf64_Word = 0x6ffffffd; /* Version definition section.  */
pub const SHT_GNU_verneed: Elf64_Word = 0x6ffffffe; /* Version needs section.  */
pub const SHT_GNU_versym: Elf64_Word = 0x6fffffff; /* Version symbol table.  */
pub const SHT_HIOS: Elf64_Word = 0x6fffffff; /* End OS-specific type */
pub const SHT_LOPROC: Elf64_Word = 0x70000000; /* Start of processor-specific */
pub const SHT_HIPROC: Elf64_Word = 0x7fffffff; /* End of processor-specific */
pub const SHT_LOUSER: Elf64_Word = 0x80000000; /* Start of application-specific */
pub const SHT_HIUSER: Elf64_Word = 0x8fffffff; /* End of application-specific */

/* Legal values for sh_flags (section flags).  */

pub const SHF_WRITE: Elf64_Xword = 1 << 0; /* Writable */
pub const SHF_ALLOC: Elf64_Xword = 1 << 1; /* Occupies memory during execution */
pub const SHF_EXECINSTR: Elf64_Xword = 1 << 2; /* Executable */
pub const SHF_MERGE: Elf64_Xword = 1 << 4; /* Might be merged */
pub const SHF_STRINGS: Elf64_Xword = 1 << 5; /* Contains nul-terminated strings */
pub const SHF_INFO_LINK: Elf64_Xword = 1 << 6; /* `sh_info' contains SHT index */
pub const SHF_LINK_ORDER: Elf64_Xword = 1 << 7; /* Preserve order after combining */
pub const SHF_OS_NONCONFORMING: Elf64_Xword = 1 << 8; /* Non-standard OS specific handling required */
pub const SHF_GROUP: Elf64_Xword = 1 << 9; /* Section is member of a group.  */
pub const SHF_TLS: Elf64_Xword = 1 << 10; /* Section hold thread-local data.  */
pub const SHF_COMPRESSED: Elf64_Xword = 1 << 11; /* Section with compressed data. */
pub const SHF_MASKOS: Elf64_Xword = 0x0ff00000; /* OS-specific.  */
pub const SHF_MASKPROC: Elf64_Xword = 0xf0000000; /* Processor-specific */
pub const SHF_GNU_RETAIN: Elf64_Xword = 1 << 21; /* Not to be GCed by linker.  */
pub const SHF_EXCLUDE: Elf64_Xword = 1 << 31; /* Section is excluded unless referenced or allocated */

/* Section compression header.  Used when SHF_COMPRESSED is set.  */

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Elf64_Chdr {
    pub ch_type: Elf64_Word, /* Compression format.  */
    pub ch_reserved: Elf64_Word,
    pub ch_size: Elf64_Xword,      /* Uncompressed data size.  */
    pub ch_addralign: Elf64_Xword, /* Uncompressed data alignment.  */
}

/* Legal values for ch_type (compression algorithm).  */
pub const ELFCOMPRESS_ZLIB: Elf64_Word = 1; /* ZLIB/DEFLATE algorithm.  */
pub const ELFCOMPRESS_ZSTD: Elf64_Word = 2; /* Zstandard algorithm.  */

/* Section group handling.  */
pub const GRP_COMDAT: u32 = 0x1; /* Mark group as COMDAT.  */

/* Symbol table entry.  */

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Elf64_Sym {
    pub st_name: Elf64_Word,     /* Symbol name (string tbl index) */
    pub st_info: u8,             /* Symbol type and binding */
    pub st_other: u8,            /* Symbol visibility */
    pub st_shndx: Elf64_Section, /* Section index */
    pub st_value: Elf64_Addr,    /* Symbol value */
    pub st_size: Elf64_Xword,    /* Symbol size */
}

impl Elf64_Sym {
    pub fn is_resolved_index(&self) -> bool {
        self.st_shndx != SHN_UNDEF && self.st_shndx < SHN_LORESERVE
    }
}

/* The syminfo section if available contains additional information about
every dynamic symbol.  */

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Elf64_Syminfo {
    pub si_boundto: Elf64_Half, /* Direct bindings, symbol bound to */
    pub si_flags: Elf64_Half,   /* Per symbol flags */
}

/* Possible values for si_boundto.  */
pub const SYMINFO_BT_SELF: Elf64_Half = 0xffff; /* Symbol bound to self */
pub const SYMINFO_BT_PARENT: Elf64_Half = 0xfffe; /* Symbol bound to parent */
pub const SYMINFO_BT_LOWRESERVE: Elf64_Half = 0xff00; /* Beginning of reserved entries */

/* Possible bitmasks for si_flags.  */
pub const SYMINFO_FLG_DIRECT: Elf64_Half = 0x0001; /* Direct bound symbol */
pub const SYMINFO_FLG_PASSTHRU: Elf64_Half = 0x0002; /* Pass-through symbol for translator */
pub const SYMINFO_FLG_COPY: Elf64_Half = 0x0004; /* Symbol is a copy-reloc */
pub const SYMINFO_FLG_LAZYLOAD: Elf64_Half = 0x0008; /* Symbol bound to object to be lazy loaded */

/* Syminfo version values.  */
pub const SYMINFO_NONE: Elf64_Half = 0;
pub const SYMINFO_CURRENT: Elf64_Half = 1;
pub const SYMINFO_NUM: Elf64_Half = 2;

/* How to extract and insert information held in the st_info field.
Both Elf32_Sym and Elf64_Sym use the same one-byte st_info field.  */

pub const fn ELF64_ST_BIND(val: u8) -> u8 {
    val >> 4
}
pub const fn ELF64_ST_TYPE(val: u8) -> u8 {
    val & 0xf
}
pub const fn ELF64_ST_INFO(bind: u8, r#type: u8) -> u8 {
    (bind << 4) + (r#type & 0xf)
}

/* Legal values for ST_BIND subfield of st_info (symbol binding).  */

pub const STB_LOCAL: u8 = 0; /* Local symbol */
pub const STB_GLOBAL: u8 = 1; /* Global symbol */
pub const STB_WEAK: u8 = 2; /* Weak symbol */
pub const STB_NUM: u8 = 3; /* Number of defined types.  */
pub const STB_LOOS: u8 = 10; /* Start of OS-specific */
pub const STB_GNU_UNIQUE: u8 = 10; /* Unique symbol.  */
pub const STB_HIOS: u8 = 12; /* End of OS-specific */
pub const STB_LOPROC: u8 = 13; /* Start of processor-specific */
pub const STB_HIPROC: u8 = 15; /* End of processor-specific */

/* Legal values for ST_TYPE subfield of st_info (symbol type).  */

pub const STT_NOTYPE: u8 = 0; /* Symbol type is unspecified */
pub const STT_OBJECT: u8 = 1; /* Symbol is a data object */
pub const STT_FUNC: u8 = 2; /* Symbol is a code object */
pub const STT_SECTION: u8 = 3; /* Symbol associated with a section */
pub const STT_FILE: u8 = 4; /* Symbol's name is file name */
pub const STT_COMMON: u8 = 5; /* Symbol is a common data object */
pub const STT_TLS: u8 = 6; /* Symbol is thread-local data object*/
pub const STT_NUM: u8 = 7; /* Number of defined types.  */
pub const STT_LOOS: u8 = 10; /* Start of OS-specific */
pub const STT_GNU_IFUNC: u8 = 10; /* Symbol is indirect code object */
pub const STT_HIOS: u8 = 12; /* End of OS-specific */
pub const STT_LOPROC: u8 = 13; /* Start of processor-specific */
pub const STT_HIPROC: u8 = 15; /* End of processor-specific */

/* Symbol table indices are found in the hash buckets and chain table
of a symbol hash table section.  This special index value indicates
the end of a chain, meaning no further symbols are found in that bucket.  */

pub const STN_UNDEF: Elf64_Word = 0; /* End of a chain.  */

/* How to extract and insert information held in the st_other field.  */

pub const fn ELF64_ST_VISIBILITY(o: u8) -> u8 {
    o & 0x03
}

/* Symbol visibility specification encoded in the st_other field.  */
pub const STV_DEFAULT: u8 = 0; /* Default symbol visibility rules */
pub const STV_INTERNAL: u8 = 1; /* Processor specific hidden class */
pub const STV_HIDDEN: u8 = 2; /* Sym unavailable in other modules */
pub const STV_PROTECTED: u8 = 3; /* Not preemptible, not exported */

/* Relocation table entry without addend (in section of type SHT_REL).  */

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Elf64_Rel {
    pub r_offset: Elf64_Addr, /* Address */
    pub r_info: Elf64_Xword,  /* Relocation type and symbol index */
}

/* Relocation table entry with addend (in section of type SHT_RELA).  */

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Elf64_Rela {
    pub r_offset: Elf64_Addr,   /* Address */
    pub r_info: Elf64_Xword,    /* Relocation type and symbol index */
    pub r_addend: Elf64_Sxword, /* Addend */
}

/* How to extract and insert information held in the r_info field.  */

pub const fn ELF64_R_SYM(info: Elf64_Xword) -> Elf64_Word {
    (info >> 32) as Elf64_Word
}
pub const fn ELF64_R_TYPE(info: Elf64_Xword) -> Elf64_Word {
    (info & 0xffffffff) as Elf64_Word
}
pub const fn ELF64_R_INFO(sym: Elf64_Word, r#type: Elf64_Word) -> Elf64_Xword {
    ((sym as Elf64_Xword) << 32) + (r#type as Elf64_Xword)
}

/* Program segment header.  */

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Elf64_Phdr {
    pub p_type: Elf64_Word,    /* Segment type */
    pub p_flags: Elf64_Word,   /* Segment flags */
    pub p_offset: Elf64_Off,   /* Segment file offset */
    pub p_vaddr: Elf64_Addr,   /* Segment virtual address */
    pub p_paddr: Elf64_Addr,   /* Segment physical address */
    pub p_filesz: Elf64_Xword, /* Segment size in file */
    pub p_memsz: Elf64_Xword,  /* Segment size in memory */
    pub p_align: Elf64_Xword,  /* Segment alignment */
}

/* Special value for e_phnum.  This indicates that the real number of
program headers is too large to fit into e_phnum.  Instead the real
value is in the field sh_info of section 0.  */

pub const PN_XNUM: Elf64_Half = 0xffff;

/* Legal values for p_type (segment type).  */

pub const PT_NULL: Elf64_Word = 0; /* Program header table entry unused */
pub const PT_LOAD: Elf64_Word = 1; /* Loadable program segment */
pub const PT_DYNAMIC: Elf64_Word = 2; /* Dynamic linking information */
pub const PT_INTERP: Elf64_Word = 3; /* Program interpreter */
pub const PT_NOTE: Elf64_Word = 4; /* Auxiliary information */
pub const PT_SHLIB: Elf64_Word = 5; /* Reserved */
pub const PT_PHDR: Elf64_Word = 6; /* Entry for header table itself */
pub const PT_TLS: Elf64_Word = 7; /* Thread-local storage segment */
pub const PT_NUM: Elf64_Word = 8; /* Number of defined types */
pub const PT_LOOS: Elf64_Word = 0x60000000; /* Start of OS-specific */
pub const PT_GNU_EH_FRAME: Elf64_Word = 0x6474e550; /* GCC .eh_frame_hdr segment */
pub const PT_GNU_STACK: Elf64_Word = 0x6474e551; /* Indicates stack executability */
pub const PT_GNU_RELRO: Elf64_Word = 0x6474e552; /* Read-only after relocation */
pub const PT_GNU_PROPERTY: Elf64_Word = 0x6474e553; /* GNU property */
pub const PT_GNU_SFRAME: Elf64_Word = 0x6474e554; /* SFrame segment.  */
pub const PT_HIOS: Elf64_Word = 0x6fffffff; /* End of OS-specific */
pub const PT_LOPROC: Elf64_Word = 0x70000000; /* Start of processor-specific */
pub const PT_HIPROC: Elf64_Word = 0x7fffffff; /* End of processor-specific */

/* Legal values for p_flags (segment flags).  */

pub const PF_X: Elf64_Word = 1 << 0; /* Segment is executable */
pub const PF_W: Elf64_Word = 1 << 1; /* Segment is writable */
pub const PF_R: Elf64_Word = 1 << 2; /* Segment is readable */
pub const PF_MASKOS: Elf64_Word = 0x0ff00000; /* OS-specific */
pub const PF_MASKPROC: Elf64_Word = 0xf0000000; /* Processor-specific */

/* Note section contents.  Each entry in the note section begins with
a header of a fixed form.  */

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Elf64_Nhdr {
    pub n_namesz: Elf64_Word, /* Length of the note's name.  */
    pub n_descsz: Elf64_Word, /* Length of the note's descriptor.  */
    pub n_type: Elf64_Word,   /* Type of the note.  */
}

/* Known names of notes.  */

/* Note entries for GNU systems have this name.  */
pub const ELF_NOTE_GNU: &str = "GNU";

/* Defined note types for GNU systems.  */

/* ABI information.  The descriptor consists of words:
   word 0: OS descriptor
   word 1: major version of the ABI
   word 2: minor version of the ABI
   word 3: subminor version of the ABI
*/
pub const NT_GNU_ABI_TAG: Elf64_Word = 1;
pub const ELF_NOTE_ABI: Elf64_Word = NT_GNU_ABI_TAG; /* Old name.  */

/* Known OSes.  These values can appear in word 0 of an
NT_GNU_ABI_TAG note section entry.  */
pub const ELF_NOTE_OS_LINUX: Elf64_Word = 0;

/* Synthetic hwcap information.  The descriptor begins with two words:
word 0: number of entries
word 1: bitmask of enabled entries
Then follow variable-length entries, one byte followed by a
'\0'-terminated hwcap name string.  The byte gives the bit
number to test if enabled, (1U << bit) & bitmask.  */
pub const NT_GNU_HWCAP: Elf64_Word = 2;

/* Build ID bits as generated by ld --build-id.
The descriptor consists of any nonzero number of bytes.  */
pub const NT_GNU_BUILD_ID: Elf64_Word = 3;

/* Program property.  */
pub const NT_GNU_PROPERTY_TYPE_0: Elf64_Word = 5;

/* Note section name of program property.   */
pub const NOTE_GNU_PROPERTY_SECTION_NAME: &str = ".note.gnu.property";

/* Values used in GNU .note.gnu.property notes (NT_GNU_PROPERTY_TYPE_0).  */

/* Stack size.  */
pub const GNU_PROPERTY_STACK_SIZE: Elf64_Word = 1;
/* No copy relocation on protected data symbol.  */
pub const GNU_PROPERTY_NO_COPY_ON_PROTECTED: Elf64_Word = 2;

/* A 4-byte unsigned integer property: A bit is set if it is set in all
relocatable inputs.  */
pub const GNU_PROPERTY_UINT32_AND_LO: Elf64_Word = 0xb0000000;
pub const GNU_PROPERTY_UINT32_AND_HI: Elf64_Word = 0xb0007fff;

/* A 4-byte unsigned integer property: A bit is set if it is set in any
relocatable inputs.  */
pub const GNU_PROPERTY_UINT32_OR_LO: Elf64_Word = 0xb0008000;
pub const GNU_PROPERTY_UINT32_OR_HI: Elf64_Word = 0xb000ffff;

/* The needed properties by the object file.  */
pub const GNU_PROPERTY_1_NEEDED: Elf64_Word = GNU_PROPERTY_UINT32_OR_LO;

/* Set if the object file requires canonical function pointers and
cannot be used with copy relocation.  */
pub const GNU_PROPERTY_1_NEEDED_INDIRECT_EXTERN_ACCESS: Elf64_Word = 1 << 0;

/* Processor-specific semantics, lo */
pub const GNU_PROPERTY_LOPROC: Elf64_Word = 0xc0000000;
/* Processor-specific semantics, hi */
pub const GNU_PROPERTY_HIPROC: Elf64_Word = 0xdfffffff;
/* Application-specific semantics, lo */
pub const GNU_PROPERTY_LOUSER: Elf64_Word = 0xe0000000;
/* Application-specific semantics, hi */
pub const GNU_PROPERTY_HIUSER: Elf64_Word = 0xffffffff;

/* The x86 instruction sets indicated by the corresponding bits are
used in program.  Their support in the hardware is optional.  */
pub const GNU_PROPERTY_X86_ISA_1_USED: Elf64_Word = 0xc0010002;
/* The x86 instruction sets indicated by the corresponding bits are
used in program and they must be supported by the hardware.   */
pub const GNU_PROPERTY_X86_ISA_1_NEEDED: Elf64_Word = 0xc0008002;
/* X86 processor-specific features used in program.  */
pub const GNU_PROPERTY_X86_FEATURE_1_AND: Elf64_Word = 0xc0000002;

/* GNU_PROPERTY_X86_ISA_1_BASELINE: CMOV, CX8 (cmpxchg8b), FPU (fld),
MMX, OSFXSR (fxsave), SCE (syscall), SSE and SSE2.  */
pub const GNU_PROPERTY_X86_ISA_1_BASELINE: Elf64_Word = 1 << 0;
/* GNU_PROPERTY_X86_ISA_1_V2: GNU_PROPERTY_X86_ISA_1_BASELINE,
CMPXCHG16B (cmpxchg16b), LAHF-SAHF (lahf), POPCNT (popcnt), SSE3,
SSSE3, SSE4.1 and SSE4.2.  */
pub const GNU_PROPERTY_X86_ISA_1_V2: Elf64_Word = 1 << 1;
/* GNU_PROPERTY_X86_ISA_1_V3: GNU_PROPERTY_X86_ISA_1_V2, AVX, AVX2, BMI1,
BMI2, F16C, FMA, LZCNT, MOVBE, XSAVE.  */
pub const GNU_PROPERTY_X86_ISA_1_V3: Elf64_Word = 1 << 2;
/* GNU_PROPERTY_X86_ISA_1_V4: GNU_PROPERTY_X86_ISA_1_V3, AVX512F,
AVX512BW, AVX512CD, AVX512DQ and AVX512VL.  */
pub const GNU_PROPERTY_X86_ISA_1_V4: Elf64_Word = 1 << 3;

/* This indicates that all executable sections are compatible with
IBT.  */
pub const GNU_PROPERTY_X86_FEATURE_1_IBT: Elf64_Word = 1 << 0;
/* This indicates that all executable sections are compatible with
SHSTK.  */
pub const GNU_PROPERTY_X86_FEATURE_1_SHSTK: Elf64_Word = 1 << 1;

/* Legal values for the note segment descriptor types for object files.  */

pub const NT_VERSION: Elf64_Word = 1; /* Contains a version string.  */

/* Dynamic section entry.  */

#[repr(C)]
#[derive(Clone, Copy)]
pub union Elf64_Dyn_un {
    pub d_val: Elf64_Xword, /* Integer value */
    pub d_ptr: Elf64_Addr,  /* Address value */
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Elf64_Dyn {
    pub d_tag: Elf64_Sxword, /* Dynamic entry type */
    pub d_un: Elf64_Dyn_un,
}

/* Legal values for d_tag (dynamic entry type).  */

pub const DT_NULL: Elf64_Sxword = 0; /* Marks end of dynamic section */
pub const DT_NEEDED: Elf64_Sxword = 1; /* Name of needed library */
pub const DT_PLTRELSZ: Elf64_Sxword = 2; /* Size in bytes of PLT relocs */
pub const DT_PLTGOT: Elf64_Sxword = 3; /* Processor defined value */
pub const DT_HASH: Elf64_Sxword = 4; /* Address of symbol hash table */
pub const DT_STRTAB: Elf64_Sxword = 5; /* Address of string table */
pub const DT_SYMTAB: Elf64_Sxword = 6; /* Address of symbol table */
pub const DT_RELA: Elf64_Sxword = 7; /* Address of Rela relocs */
pub const DT_RELASZ: Elf64_Sxword = 8; /* Total size of Rela relocs */
pub const DT_RELAENT: Elf64_Sxword = 9; /* Size of one Rela reloc */
pub const DT_STRSZ: Elf64_Sxword = 10; /* Size of string table */
pub const DT_SYMENT: Elf64_Sxword = 11; /* Size of one symbol table entry */
pub const DT_INIT: Elf64_Sxword = 12; /* Address of init function */
pub const DT_FINI: Elf64_Sxword = 13; /* Address of termination function */
pub const DT_SONAME: Elf64_Sxword = 14; /* Name of shared object */
pub const DT_RPATH: Elf64_Sxword = 15; /* Library search path (deprecated) */
pub const DT_SYMBOLIC: Elf64_Sxword = 16; /* Start symbol search here */
pub const DT_REL: Elf64_Sxword = 17; /* Address of Rel relocs */
pub const DT_RELSZ: Elf64_Sxword = 18; /* Total size of Rel relocs */
pub const DT_RELENT: Elf64_Sxword = 19; /* Size of one Rel reloc */
pub const DT_PLTREL: Elf64_Sxword = 20; /* Type of reloc in PLT */
pub const DT_DEBUG: Elf64_Sxword = 21; /* For debugging; unspecified */
pub const DT_TEXTREL: Elf64_Sxword = 22; /* Reloc might modify .text */
pub const DT_JMPREL: Elf64_Sxword = 23; /* Address of PLT relocs */
pub const DT_BIND_NOW: Elf64_Sxword = 24; /* Process relocations of object */
pub const DT_INIT_ARRAY: Elf64_Sxword = 25; /* Array with addresses of init fct */
pub const DT_FINI_ARRAY: Elf64_Sxword = 26; /* Array with addresses of fini fct */
pub const DT_INIT_ARRAYSZ: Elf64_Sxword = 27; /* Size in bytes of DT_INIT_ARRAY */
pub const DT_FINI_ARRAYSZ: Elf64_Sxword = 28; /* Size in bytes of DT_FINI_ARRAY */
pub const DT_RUNPATH: Elf64_Sxword = 29; /* Library search path */
pub const DT_FLAGS: Elf64_Sxword = 30; /* Flags for the object being loaded */
pub const DT_PREINIT_ARRAY: Elf64_Sxword = 32; /* Array with addresses of preinit fct*/
pub const DT_PREINIT_ARRAYSZ: Elf64_Sxword = 33; /* size in bytes of DT_PREINIT_ARRAY */
pub const DT_SYMTAB_SHNDX: Elf64_Sxword = 34; /* Address of SYMTAB_SHNDX section */
pub const DT_RELRSZ: Elf64_Sxword = 35; /* Total size of RELR relative relocations */
pub const DT_RELR: Elf64_Sxword = 36; /* Address of RELR relative relocations */
pub const DT_RELRENT: Elf64_Sxword = 37; /* Size of one RELR relative relocation */
pub const DT_NUM: Elf64_Sxword = 38; /* Number used */
pub const DT_LOOS: Elf64_Sxword = 0x6000000d; /* Start of OS-specific */
pub const DT_HIOS: Elf64_Sxword = 0x6ffff000; /* End of OS-specific */
pub const DT_LOPROC: Elf64_Sxword = 0x70000000; /* Start of processor-specific */
pub const DT_HIPROC: Elf64_Sxword = 0x7fffffff; /* End of processor-specific */

/* GNU extensions used on Linux.  */
pub const DT_GNU_HASH: Elf64_Sxword = 0x6ffffef5; /* GNU-style hash table.  */
pub const DT_TLSDESC_PLT: Elf64_Sxword = 0x6ffffef6;
pub const DT_TLSDESC_GOT: Elf64_Sxword = 0x6ffffef7;

/* The versioning entry types.  */
pub const DT_VERSYM: Elf64_Sxword = 0x6ffffff0;

pub const DT_RELACOUNT: Elf64_Sxword = 0x6ffffff9;
pub const DT_RELCOUNT: Elf64_Sxword = 0x6ffffffa;

pub const DT_FLAGS_1: Elf64_Sxword = 0x6ffffffb; /* State flags, see DF_1_* below.  */
pub const DT_VERDEF: Elf64_Sxword = 0x6ffffffc; /* Address of version definition table */
pub const DT_VERDEFNUM: Elf64_Sxword = 0x6ffffffd; /* Number of version definitions */
pub const DT_VERNEED: Elf64_Sxword = 0x6ffffffe; /* Address of table with needed versions */
pub const DT_VERNEEDNUM: Elf64_Sxword = 0x6fffffff; /* Number of needed versions */

/* x86-64 d_tag values.  */
pub const DT_X86_64_PLT: Elf64_Sxword = DT_LOPROC + 0;
pub const DT_X86_64_PLTSZ: Elf64_Sxword = DT_LOPROC + 1;
pub const DT_X86_64_PLTENT: Elf64_Sxword = DT_LOPROC + 3;
pub const DT_X86_64_NUM: Elf64_Sxword = 4;

/* Values of `d_un.d_val' in the DT_FLAGS entry.  */
pub const DF_ORIGIN: Elf64_Xword = 0x00000001; /* Object may use DF_ORIGIN */
pub const DF_SYMBOLIC: Elf64_Xword = 0x00000002; /* Symbol resolutions starts here */
pub const DF_TEXTREL: Elf64_Xword = 0x00000004; /* Object contains text relocations */
pub const DF_BIND_NOW: Elf64_Xword = 0x00000008; /* No lazy binding for this object */
pub const DF_STATIC_TLS: Elf64_Xword = 0x00000010; /* Module uses the static TLS model */

/* State flags selectable in the `d_un.d_val' element of the DT_FLAGS_1
entry in the dynamic section, as interpreted by glibc's ld.so.  */
pub const DF_1_NOW: Elf64_Xword = 0x00000001; /* Set RTLD_NOW for this object.  */
pub const DF_1_GLOBAL: Elf64_Xword = 0x00000002; /* Set RTLD_GLOBAL for this object.  */
pub const DF_1_GROUP: Elf64_Xword = 0x00000004; /* Set RTLD_GROUP for this object.  */
pub const DF_1_NODELETE: Elf64_Xword = 0x00000008; /* Set RTLD_NODELETE for this object.*/
pub const DF_1_INITFIRST: Elf64_Xword = 0x00000020; /* Set RTLD_INITFIRST for this object*/
pub const DF_1_NOOPEN: Elf64_Xword = 0x00000040; /* Set RTLD_NOOPEN for this object.  */
pub const DF_1_ORIGIN: Elf64_Xword = 0x00000080; /* $ORIGIN must be handled.  */
pub const DF_1_DIRECT: Elf64_Xword = 0x00000100; /* Direct binding enabled.  */
pub const DF_1_INTERPOSE: Elf64_Xword = 0x00000400; /* Object is used to interpose.  */
pub const DF_1_NODEFLIB: Elf64_Xword = 0x00000800; /* Ignore default lib search path.  */
pub const DF_1_NODUMP: Elf64_Xword = 0x00001000; /* Object can't be dldump'ed.  */
pub const DF_1_PIE: Elf64_Xword = 0x08000000; /* Object is a position-independent executable.  */

/* Version definition sections.  */

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Elf64_Verdef {
    pub vd_version: Elf64_Half, /* Version revision */
    pub vd_flags: Elf64_Half,   /* Version information */
    pub vd_ndx: Elf64_Half,     /* Version Index */
    pub vd_cnt: Elf64_Half,     /* Number of associated aux entries */
    pub vd_hash: Elf64_Word,    /* Version name hash value */
    pub vd_aux: Elf64_Word,     /* Offset in bytes to verdaux array */
    pub vd_next: Elf64_Word,    /* Offset in bytes to next verdef entry */
}

/* Legal values for vd_version (version revision).  */
pub const VER_DEF_NONE: Elf64_Half = 0; /* No version */
pub const VER_DEF_CURRENT: Elf64_Half = 1; /* Current version */
pub const VER_DEF_NUM: Elf64_Half = 2; /* Given version number */

/* Legal values for vd_flags (version information flags).  */
pub const VER_FLG_BASE: Elf64_Half = 0x1; /* Version definition of file itself */
pub const VER_FLG_WEAK: Elf64_Half = 0x2; /* Weak version identifier.  Also used by vna_flags below.  */

/* Versym symbol index values.  */
pub const VER_NDX_LOCAL: Elf64_Half = 0; /* Symbol is local.  */
pub const VER_NDX_GLOBAL: Elf64_Half = 1; /* Symbol is global.  */
pub const VER_NDX_LORESERVE: Elf64_Half = 0xff00; /* Beginning of reserved entries.  */
pub const VER_NDX_ELIMINATE: Elf64_Half = 0xff01; /* Symbol is to be eliminated.  */

/* Auxiliary version information.  */

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Elf64_Verdaux {
    pub vda_name: Elf64_Word, /* Version or dependency names */
    pub vda_next: Elf64_Word, /* Offset in bytes to next verdaux entry */
}

/* Version dependency section.  */

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Elf64_Verneed {
    pub vn_version: Elf64_Half, /* Version of structure */
    pub vn_cnt: Elf64_Half,     /* Number of associated aux entries */
    pub vn_file: Elf64_Word,    /* Offset of filename for this dependency */
    pub vn_aux: Elf64_Word,     /* Offset in bytes to vernaux array */
    pub vn_next: Elf64_Word,    /* Offset in bytes to next verneed entry */
}

/* Legal values for vn_version (version revision).  */
pub const VER_NEED_NONE: Elf64_Half = 0; /* No version */
pub const VER_NEED_CURRENT: Elf64_Half = 1; /* Current version */
pub const VER_NEED_NUM: Elf64_Half = 2; /* Given version number */

/* Auxiliary needed version information.  */

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Elf64_Vernaux {
    pub vna_hash: Elf64_Word,  /* Hash value of dependency name */
    pub vna_flags: Elf64_Half, /* Dependency specific information */
    pub vna_other: Elf64_Half, /* Unused */
    pub vna_name: Elf64_Word,  /* Dependency name string offset */
    pub vna_next: Elf64_Word,  /* Offset in bytes to next vernaux entry */
}

/* AMD x86-64 relocations.  */
pub const R_X86_64_NONE: Elf64_Word = 0; /* No reloc */
pub const R_X86_64_64: Elf64_Word = 1; /* Direct 64 bit  */
pub const R_X86_64_PC32: Elf64_Word = 2; /* PC relative 32 bit signed */
pub const R_X86_64_GOT32: Elf64_Word = 3; /* 32 bit GOT entry */
pub const R_X86_64_PLT32: Elf64_Word = 4; /* 32 bit PLT address */
pub const R_X86_64_COPY: Elf64_Word = 5; /* Copy symbol at runtime */
pub const R_X86_64_GLOB_DAT: Elf64_Word = 6; /* Create GOT entry */
pub const R_X86_64_JUMP_SLOT: Elf64_Word = 7; /* Create PLT entry */
pub const R_X86_64_RELATIVE: Elf64_Word = 8; /* Adjust by program base */
pub const R_X86_64_GOTPCREL: Elf64_Word = 9; /* 32 bit signed PC relative offset to GOT */
pub const R_X86_64_32: Elf64_Word = 10; /* Direct 32 bit zero extended */
pub const R_X86_64_32S: Elf64_Word = 11; /* Direct 32 bit sign extended */
pub const R_X86_64_16: Elf64_Word = 12; /* Direct 16 bit zero extended */
pub const R_X86_64_PC16: Elf64_Word = 13; /* 16 bit sign extended pc relative */
pub const R_X86_64_8: Elf64_Word = 14; /* Direct 8 bit sign extended  */
pub const R_X86_64_PC8: Elf64_Word = 15; /* 8 bit sign extended pc relative */
pub const R_X86_64_DTPMOD64: Elf64_Word = 16; /* ID of module containing symbol */
pub const R_X86_64_DTPOFF64: Elf64_Word = 17; /* Offset in module's TLS block */
pub const R_X86_64_TPOFF64: Elf64_Word = 18; /* Offset in initial TLS block */
pub const R_X86_64_TLSGD: Elf64_Word = 19; /* 32 bit signed PC relative offset to two GOT entries for GD symbol */
pub const R_X86_64_TLSLD: Elf64_Word = 20; /* 32 bit signed PC relative offset to two GOT entries for LD symbol */
pub const R_X86_64_DTPOFF32: Elf64_Word = 21; /* Offset in TLS block */
pub const R_X86_64_GOTTPOFF: Elf64_Word = 22; /* 32 bit signed PC relative offset to GOT entry for IE symbol */
pub const R_X86_64_TPOFF32: Elf64_Word = 23; /* Offset in initial TLS block */
pub const R_X86_64_PC64: Elf64_Word = 24; /* PC relative 64 bit */
pub const R_X86_64_GOTOFF64: Elf64_Word = 25; /* 64 bit offset to GOT */
pub const R_X86_64_GOTPC32: Elf64_Word = 26; /* 32 bit signed pc relative offset to GOT */
pub const R_X86_64_GOT64: Elf64_Word = 27; /* 64-bit GOT entry offset */
pub const R_X86_64_GOTPCREL64: Elf64_Word = 28; /* 64-bit PC relative offset to GOT entry */
pub const R_X86_64_GOTPC64: Elf64_Word = 29; /* 64-bit PC relative offset to GOT */
pub const R_X86_64_GOTPLT64: Elf64_Word = 30; /* like GOT64, says PLT entry needed */
pub const R_X86_64_PLTOFF64: Elf64_Word = 31; /* 64-bit GOT relative offset to PLT entry */
pub const R_X86_64_SIZE32: Elf64_Word = 32; /* Size of symbol plus 32-bit addend */
pub const R_X86_64_SIZE64: Elf64_Word = 33; /* Size of symbol plus 64-bit addend */
pub const R_X86_64_GOTPC32_TLSDESC: Elf64_Word = 34; /* GOT offset for TLS descriptor.  */
pub const R_X86_64_TLSDESC_CALL: Elf64_Word = 35; /* Marker for call through TLS descriptor.  */
pub const R_X86_64_TLSDESC: Elf64_Word = 36; /* TLS descriptor.  */
pub const R_X86_64_IRELATIVE: Elf64_Word = 37; /* Adjust indirectly by program base */
pub const R_X86_64_RELATIVE64: Elf64_Word = 38; /* 64-bit adjust by program base */
/* 39 Reserved was R_X86_64_PC32_BND */
/* 40 Reserved was R_X86_64_PLT32_BND */
pub const R_X86_64_GOTPCRELX: Elf64_Word = 41; /* Load from 32 bit signed pc relative offset to GOT entry without REX prefix, relaxable.  */
pub const R_X86_64_REX_GOTPCRELX: Elf64_Word = 42; /* Load from 32 bit signed pc relative offset to GOT entry with REX prefix, relaxable.  */
pub const R_X86_64_NUM: Elf64_Word = 43;

/* x86-64 sh_type values.  */
pub const SHT_X86_64_UNWIND: Elf64_Word = 0x70000001; /* Unwind information.  */
