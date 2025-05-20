use std::env;
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    
    // Copy libfalcon.so to the output directory
    std::fs::copy("lib/libfalcon.so", out_dir.join("libfalcon.so"))
        .expect("Failed to copy libfalcon.so");

    println!("cargo:rustc-link-search=native={}", out_dir.display());
    println!("cargo:rustc-link-lib=dylib=falcon");
}
