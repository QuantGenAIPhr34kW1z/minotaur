use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let fortran_dir = PathBuf::from(&manifest_dir).join("../fortran");
    let build_base = fortran_dir.join("build");

    // Build Fortran with fpm
    let status = Command::new("fpm")
        .arg("build")
        .arg("--profile")
        .arg("release")
        .current_dir(&fortran_dir)
        .status()
        .expect("Failed to run fpm build - is fpm installed?");

    if !status.success() {
        panic!("fpm build failed");
    }

    // Find the fpm build directory (gfortran_*_release pattern)
    let lib_dir = fs::read_dir(&build_base)
        .expect("Could not read fpm build directory")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .find(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("gfortran") && n.ends_with("release"))
                .unwrap_or(false)
        })
        .expect("Could not find fpm release build directory");

    // Tell cargo where to find the library
    println!("cargo:rustc-link-search=native={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=minotaur_fortran");

    // Link against gfortran runtime
    println!("cargo:rustc-link-lib=dylib=gfortran");

    // Rebuild if Fortran sources change
    for entry in fs::read_dir(fortran_dir.join("src")).unwrap() {
        if let Ok(e) = entry {
            println!("cargo:rerun-if-changed={}", e.path().display());
        }
    }
}
