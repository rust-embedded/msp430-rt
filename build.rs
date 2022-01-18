use std::{env, fs, fs::File, io::Write, path::PathBuf};

fn main() {
    let target = env::var("TARGET").unwrap();
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());

    if target == "msp430-none-elf" {
        fs::copy(format!("bin/{}.a", target), out_dir.join("libmsp430-rt.a")).unwrap();
        println!("cargo:rustc-link-lib=static=msp430-rt");
    }

    // Put the linker script somewhere the linker can find it
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let link_x = include_bytes!("link.x.in");
    if env::var_os("CARGO_FEATURE_DEVICE").is_some() {
        let mut f = File::create(out.join("link.x")).unwrap();

        f.write_all(link_x).unwrap();

        // *IMPORTANT*: The weak aliases (i.e. `PROVIDED`) must come *after* `EXTERN(__INTERRUPTS)`.
        // Otherwise the linker will ignore user defined interrupts and always populate the table
        // with the weak aliases.
        writeln!(
            f,
            r#"
/* Provides weak aliases (cf. PROVIDED) for device specific interrupt handlers */
/* This will usually be provided by a device crate generated using svd2rust (see `device.x`) */
INCLUDE device.x"#
        )
        .unwrap();
    } else {
        let mut f = File::create(out.join("link.x")).unwrap();
        f.write_all(link_x).unwrap();
    };

    println!("cargo:rustc-link-search={}", out_dir.display());

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=link.x.in");
}
