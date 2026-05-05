fn main() {
    #[cfg(not(debug_assertions))]
    tauri_build::build();
}
