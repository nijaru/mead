use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");

    // Use pkg-config to find SVT-AV1
    let lib = pkg_config::Config::new()
        .probe("SvtAv1Enc")
        .expect("SVT-AV1 not found. Install with: brew install svt-av1");

    // Tell cargo to link against SVT-AV1
    println!("cargo:rustc-link-lib=SvtAv1Enc");

    // Get include path from pkg-config
    let include_path = lib
        .include_paths
        .first()
        .expect("No include path from pkg-config");

    // Generate bindings
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg(format!("-I{}", include_path.display()))
        // Only generate bindings for SVT-AV1 types
        .allowlist_type("Eb.*")
        .allowlist_function("svt_.*")
        .allowlist_var("SVT_.*")
        .allowlist_var("EB_.*")
        .allowlist_var("ENC_.*")
        .allowlist_var("DEFAULT")
        .allowlist_var("MAX_.*")
        .allowlist_var("HIERARCHICAL_.*")
        .allowlist_var("REF_.*")
        // Use core instead of std
        .use_core()
        // Add derives
        .derive_debug(true)
        .derive_default(true)
        .derive_eq(true)
        .derive_partialeq(true)
        // Generate
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Failed to generate bindings");

    // Write bindings to OUT_DIR
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Failed to write bindings");
}
