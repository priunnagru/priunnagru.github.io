fn main() {
    println!("cargo:rustc-check-cfg=cfg(debug_mode)");

    dotenvy::from_filename_override(".env").ok();

    let is_debug = std::env::var("DEBUG").unwrap_or_default() == "YES";

    if is_debug {
        println!("cargo:rustc-cfg=debug_mode");
    }

    println!("cargo:rerun-if-changed=.env");
    println!("cargo:rerun-if-changed=build.rs");
}
