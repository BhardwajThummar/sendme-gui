// No commands are exposed to JavaScript; the app calls the Swift side from Rust.
fn main() {
    tauri_plugin::Builder::new(&[]).ios_path("ios").build();
}
