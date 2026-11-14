fn main() {
    let variants = ["ANSI", "ISO", "JIS", "NUMPAD", "MACROPAD"];
    assert_eq!(
        variants
            .iter()
            .filter(|v| std::env::var_os(format!("CARGO_FEATURE_{v}")).is_some())
            .count(),
        1,
        "select exactly one module feature"
    );
    let out = std::env::var_os("OUT_DIR").unwrap();
    std::fs::write(
        std::path::Path::new(&out).join("memory.x"),
        include_bytes!("memory.x"),
    )
    .unwrap();
    println!(
        "cargo:rustc-link-search={}",
        std::path::Path::new(&out).display()
    );
    println!("cargo:rerun-if-changed=memory.x");
    for arg in ["--nmagic", "-Tlink.x", "-Tdefmt.x"] {
        println!("cargo:rustc-link-arg={arg}");
    }
}
