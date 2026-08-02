use crate::linker::{
    Linker,
    section::{OutputSectionBytesKind, OutputSectionList},
};

pub(super) const PLT_ENTRY_BYTE_SIZE: usize = 6;
pub(super) const GOT_ENTRY_BYTE_SIZE: usize = 8; // address size

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
}
