use crate::{
    linker::{ElfObject, Linker, LinkerError, symbol::ResolvedObjIndexKind},
    parser::section::rela_text::ElfRela,
};

impl<'a> Linker<'a> {
    pub(super) fn relocate(&self) -> Result<(), LinkerError> {
        for o in &self.objs {
            if let Some(relas) = &o.rela_text {
                for rela in &relas.relas {
                    self.patch_relocation(o, rela)?;
                }
            }
        }

        Ok(())
    }

    fn patch_relocation(&self, obj: &ElfObject, rela: &ElfRela) -> Result<(), LinkerError> {
        let sym_index = rela.rela.r_sym() as usize;
        let sym = &obj.symtab.syms[sym_index];
        let resolved = sym.resolved_sym.as_ref().unwrap();
        match &resolved.obj_index {
            ResolvedObjIndexKind::Obj(obj_index) => {
                let addr = self.sym_addrs[*obj_index][&resolved.sym_index];
                // TODO: addr で書き換える
            }
            ResolvedObjIndexKind::Shared(_shared_obj_index) => {
                // TODO:
                //
                // let addr = self.dynsym_addrs[*shared_obj_index][&resolved.sym_index];
            }
        }

        Ok(())
    }
}
