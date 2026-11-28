//! Two things at build time, both offline:
//!
//! - The executable's icon and Windows version details: an `.ico` drawn from the brand mark
//!   (`src/mark.rs`, the same pixels as the window icon) and a version resource, compiled by
//!   the Windows SDK's `rc.exe` and linked in. Without the SDK the build goes on without them
//!   (a warning says so); `HGT_RC` names an `rc.exe` elsewhere.
//! - `HGT_RUNTIME_EXE=<path to victory.exe>` puts the runtime inside HybridGt.exe (unpacked on
//!   first start), so one file is all a player downloads.

use std::path::{Path, PathBuf};

include!("src/mark.rs");

fn main() {
    println!("cargo:rerun-if-env-changed=HGT_RUNTIME_EXE");
    println!("cargo:rerun-if-env-changed=HGT_RC");
    println!("cargo:rerun-if-changed=src/mark.rs");
    println!("cargo::rustc-check-cfg=cfg(embedded_runtime)");
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("OUT_DIR"));
    if let Ok(path) = std::env::var("HGT_RUNTIME_EXE") {
        println!("cargo:rerun-if-changed={path}");
        std::fs::copy(&path, out.join("runtime.bin")).expect("HGT_RUNTIME_EXE: cannot read the runtime");
        println!("cargo:rustc-cfg=embedded_runtime");
    }
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        match resources(&out) {
            Ok(res) => println!("cargo:rustc-link-arg-bins={}", res.display()),
            Err(why) => println!("cargo:warning=no icon or version details in the executable: {why}"),
        }
    }
}

/// An `.ico` of 16 to 256 pixels: 32-bit images with their alpha, stored bottom up as Windows
/// reads them, each with the (empty) one-bit mask an icon image carries.
fn ico() -> Vec<u8> {
    let sizes = [16u32, 20, 24, 32, 40, 48, 64, 96, 128, 256];
    let images: Vec<Vec<u8>> = sizes
        .iter()
        .map(|&n| {
            let (rgba, _) = mark_pixels(n);
            let mask_row = n.div_ceil(32) * 4;
            let mut b = Vec::new();
            for v in [40u32, n, n * 2] {
                b.extend(v.to_le_bytes());
            }
            b.extend(1u16.to_le_bytes());
            b.extend(32u16.to_le_bytes());
            for v in [0u32, n * n * 4 + mask_row * n, 0, 0, 0, 0] {
                b.extend(v.to_le_bytes());
            }
            for y in (0..n).rev() {
                for x in 0..n {
                    let i = ((y * n + x) * 4) as usize;
                    b.extend([rgba[i + 2], rgba[i + 1], rgba[i], rgba[i + 3]]);
                }
            }
            b.resize(b.len() + (mask_row * n) as usize, 0);
            b
        })
        .collect();
    let mut f = Vec::new();
    for v in [0u16, 1, sizes.len() as u16] {
        f.extend(v.to_le_bytes());
    }
    let mut offset = 6 + 16 * sizes.len() as u32;
    for (n, img) in sizes.iter().zip(&images) {
        let side = if *n >= 256 { 0 } else { *n as u8 };
        f.extend([side, side, 0, 0]);
        f.extend(1u16.to_le_bytes());
        f.extend(32u16.to_le_bytes());
        f.extend((img.len() as u32).to_le_bytes());
        f.extend(offset.to_le_bytes());
        offset += img.len() as u32;
    }
    for img in images {
        f.extend(img);
    }
    f
}

/// The Windows SDK's newest x64 `rc.exe`, from `HGT_RC` or the Windows Kits folder.
fn find_rc() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("HGT_RC").map(PathBuf::from) {
        return p.is_file().then_some(p);
    }
    let base = std::env::var_os("ProgramFiles(x86)").map(PathBuf::from)?.join("Windows Kits").join("10").join("bin");
    let mut versions: Vec<PathBuf> = std::fs::read_dir(&base).ok()?.flatten().map(|e| e.path().join("x64").join("rc.exe")).filter(|p| p.is_file()).collect();
    versions.sort();
    versions.pop()
}

fn resources(out: &Path) -> Result<PathBuf, String> {
    let rc = find_rc().ok_or("the Windows SDK's rc.exe was not found (set HGT_RC)")?;
    std::fs::write(out.join("hybridgt.ico"), ico()).map_err(|e| e.to_string())?;
    let version = std::env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".into());
    let mut parts: Vec<u16> = version.split(['.', '-']).take(3).map(|p| p.parse().unwrap_or(0)).collect();
    parts.resize(4, 0);
    let numeric = parts.iter().map(u16::to_string).collect::<Vec<_>>().join(",");
    let script = format!(
        r#"#pragma code_page(65001)
1 ICON "hybridgt.ico"
1 VERSIONINFO
FILEVERSION {numeric}
PRODUCTVERSION {numeric}
FILEFLAGSMASK 0x3f
FILEFLAGS 0x0
FILEOS 0x40004
FILETYPE 0x1
FILESUBTYPE 0x0
BEGIN
  BLOCK "StringFileInfo"
  BEGIN
    BLOCK "040904B0"
    BEGIN
      VALUE "CompanyName", "HybridGt project"
      VALUE "FileDescription", "HybridGt launcher"
      VALUE "FileVersion", "{version}"
      VALUE "InternalName", "HybridGt"
      VALUE "LegalCopyright", "GPL-3.0-or-later. Contains no game data."
      VALUE "OriginalFilename", "HybridGt.exe"
      VALUE "ProductName", "HybridGt"
      VALUE "ProductVersion", "{version}"
    END
  END
  BLOCK "VarFileInfo"
  BEGIN
    VALUE "Translation", 0x409, 1200
  END
END
"#
    );
    let rc_file = out.join("hybridgt.rc");
    std::fs::write(&rc_file, script).map_err(|e| e.to_string())?;
    let res = out.join("hybridgt.res");
    let status = std::process::Command::new(&rc)
        .current_dir(out)
        .args(["/nologo", "/fo"])
        .arg(&res)
        .arg(&rc_file)
        .status()
        .map_err(|e| format!("{}: {e}", rc.display()))?;
    if !status.success() {
        return Err(format!("{} failed ({status})", rc.display()));
    }
    Ok(res)
}
