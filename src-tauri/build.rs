fn main() {
    // BASE_URL can be baked in at build time (see config.rs).
    println!("cargo:rerun-if-env-changed=BASE_URL");
    tauri_build::build()
}
