fn main() {
    // Set the library search path
    println!("cargo:rustc-link-search=native=.");
    
    // Platform-specific library linking
    #[cfg(target_os = "windows")]
    println!("cargo:rustc-link-lib=dylib=libfalcon");
    
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    println!("cargo:rustc-link-lib=dylib=falcon");
    
    // Platform-specific runtime path settings
    #[cfg(target_os = "macos")]
    println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path");
    
    #[cfg(target_os = "linux")]
    println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
    
    // Watch for library changes
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    println!("cargo:rerun-if-changed=libfalcon.so");
    
    #[cfg(target_os = "windows")]
    println!("cargo:rerun-if-changed=libfalcon.dll");
} 