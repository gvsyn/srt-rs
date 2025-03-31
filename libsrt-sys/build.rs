use bindgen;
use cmake;

use std::{env, path::Path, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if cfg!(unix) {
        let mut cfg = cmake::Config::new("libsrt");
        cfg.define("ENABLE_APPS", "OFF");
        cfg.define("ENABLE_BONDING", "ON");
        cfg.define("CMAKE_POLICY_VERSION_MINIMUM", "3.5");
        #[cfg(feature = "static")]
        {
            cfg.define("ENABLE_SHARED", "OFF");
        }
        let dst = cfg.build();
        let dst_dir = Path::new(&dst);

        let lib_dirs = ["lib", "lib64"]
            .iter()
            .map(|dir| dst_dir.join(dir))
            .filter_map(|dir| if dst_dir.exists() { Some(dir) } else { None })
            .collect::<Vec<_>>();

        if lib_dirs.is_empty() {
            panic!("No lib dir in {:?}", dst);
        }
        for dir in lib_dirs {
            println!("cargo:rustc-link-search={}", dir.display());
        }
        println!("cargo:rustc-link-lib=static=srt");
    } else if cfg!(windows) {
        let dst = cmake::Config::new("libsrt")
            .generator("Visual Studio 16 2019")
            .cxxflag("/EHs")
            .define("ENABLE_STDCXX_SYNC", "ON")
            .define("ENABLE_APPS", "OFF")
            .build();
        let mut lib_dir = PathBuf::from(dst.clone());
        lib_dir.push("lib");
        let mut bin_dir = PathBuf::from(dst);
        bin_dir.push("bin");
        println!("cargo:rustc-link-search={}", lib_dir.display());
        println!("cargo:rustc-link-search={}", bin_dir.display());
        println!("cargo:rustc-link-lib=srt");
    }

    let mut include_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    include_path.push("include");
    include_path.push("srt");

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        .clang_arg(format!("--include-directory={}", include_path.display()))
        .size_t_is_usize(true)
        .whitelist_function("srt_.*")
        .whitelist_type("SRT.*")
        .whitelist_var("SRT.*")
        .bitfield_enum("SRT_EPOLL_OPT")
        .default_enum_style(bindgen::EnumVariation::NewType { is_bitfield: false })
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
    Ok(())
}
