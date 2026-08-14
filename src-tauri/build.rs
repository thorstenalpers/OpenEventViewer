fn main() {
    // `tauri_build` emits its own rerun-if-changed set, which switches off cargo's default of
    // re-running on any file change in the package — so replacing the icon rebuilt the binary but
    // relinked the *cached* resource, and both debug and release kept showing the previous one.
    println!("cargo:rerun-if-changed=icons/icon.ico");
    tauri_build::build()
}
