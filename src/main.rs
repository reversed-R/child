use std::path::PathBuf;

use crate::parser::Elf64Parser;

mod elf;
mod parser;

fn main() {
    let args = std::env::args();

    if args.len() < 2 {
        panic_with_usage_message();
    }

    let mut obj_paths = Vec::new();
    let mut shared_obj_names = Vec::new();
    let mut shared_obj_base_paths = Vec::new();
    let mut dyn_linker = None;

    for mut arg in args.into_iter().skip(1) {
        if arg.starts_with("-") {
            if arg.starts_with("-l") {
                shared_obj_names.push(arg.split_off(2));
            } else if arg.starts_with("-L") {
                shared_obj_base_paths.push(arg.split_off(2));
            } else if arg.starts_with("-I") && dyn_linker.is_none() {
                dyn_linker = Some(arg.split_off(2));
            } else {
                panic_with_usage_message();
            }
        } else {
            obj_paths.push(arg);
        }
    }

    println!("obj_paths: {obj_paths:?}");
    println!("shared_obj_names: {shared_obj_names:?}");
    println!("shared_obj_base_paths: {shared_obj_base_paths:?}");
    println!("dyn_linker: {dyn_linker:?}");

    if obj_paths.is_empty() {
        panic!("At least 1 object file must be passed");
    }

    let obj_paths = obj_paths
        .into_iter()
        .map(|o| {
            let path = PathBuf::from(&o);
            if !path.exists() || !path.is_file() {
                panic!("No such file: {path:?}");
            }

            path
        })
        .collect::<Vec<_>>();
    let shared_obj_base_paths = shared_obj_base_paths
        .into_iter()
        .map(|p| {
            let path = PathBuf::from(&p);
            if !path.exists() || !path.is_dir() {
                panic!("No such directory: {path:?}");
            }

            path
        })
        .collect::<Vec<_>>();
    let shared_obj_paths = shared_obj_names
        .into_iter()
        .map(|so| {
            match shared_obj_base_paths.iter().find(|base| {
                let path = base.join(format!("lib{so}.so"));
                path.exists() && path.is_file()
            }) {
                Some(base) => base.join(format!("lib{so}.so")),
                None => {
                    panic!("Not found: lib{so}.so");
                }
            }
        })
        .collect::<Vec<_>>();

    let obj_bins = obj_paths
        .into_iter()
        .map(|path| {
            (
                std::fs::read(&path).unwrap_or_else(|_| panic!("Failed to read: {path:?}")),
                path,
            )
        })
        .collect::<Vec<_>>();

    for (bin, path) in obj_bins {
        let elf = Elf64Parser::new(&bin, path).unwrap();

        let strtab = elf.section_strtab().unwrap();
        println!("{elf:#?}");
        println!("-- .strtab --");
        println!("{strtab:#?}",);
        let symtab = elf.section_symtab(&strtab).unwrap();
        println!("-- .symtab --");
        println!("{symtab:#?}",);
        println!("-- .rela.text --");
        println!("{:#?}", elf.section_rela_text(&symtab, &strtab));
    }
}

fn panic_with_usage_message() -> ! {
    panic!(
        r#"Usage: child <object0>.o <object1>.o ...
    -l<shared-object0> -l<shared-object1> ...
    -L/base/path/to/shared/objects ...
    -I/path/to/dynamic/linker"#
    );
}
