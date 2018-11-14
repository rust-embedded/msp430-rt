use std::{env, fs, fs::File, io::Write, path::PathBuf};

fn main() {
    let target = env::var("TARGET").unwrap();
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    if target == "msp430-none-elf" {
        fs::copy(format!("bin/{}.a", target), out_dir.join("libmsp430-rt.a")).unwrap();
        println!("cargo:rustc-link-lib=static=msp430-rt");
    }

    // Put the linker script somewhere the linker can find it
    File::create(out_dir.join("link.x"))
        .unwrap()
        .write_all(include_bytes!("link.x"))
        .unwrap();
    println!("cargo:rustc-link-search={}", out_dir.display());

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=link.x");
}
