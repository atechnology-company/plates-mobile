fn main() {
    // Configure environment variables for Tauri build
    println!("cargo:rerun-if-changed=tauri.conf.json");
    println!("cargo:rerun-if-changed=capabilities");
    
    // Build the Tauri application
    tauri_build::build();
}
