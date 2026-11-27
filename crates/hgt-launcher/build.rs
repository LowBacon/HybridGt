//! `HGT_RUNTIME_EXE=<path to victory.exe>` at build time puts the runtime inside HybridGt.exe
//! (unpacked on first start), so one file is all a player downloads. Without it the launcher
//! looks for a runtime on disk or builds one from source.

fn main() {
    println!("cargo:rerun-if-env-changed=HGT_RUNTIME_EXE");
    println!("cargo::rustc-check-cfg=cfg(embedded_runtime)");
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    if let Ok(path) = std::env::var("HGT_RUNTIME_EXE") {
        println!("cargo:rerun-if-changed={path}");
        std::fs::copy(&path, out.join("runtime.bin")).expect("HGT_RUNTIME_EXE: cannot read the runtime");
        println!("cargo:rustc-cfg=embedded_runtime");
    }
}
