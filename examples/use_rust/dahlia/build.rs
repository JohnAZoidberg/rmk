use std::path::PathBuf;
use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("--no-optional-locks")
        .args(args)
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn main() {
    let out = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let mut hash = [0u8; 8];
    if let Some(commit) = git(&["rev-parse", "HEAD"]) {
        for (byte, hex) in hash.iter_mut().zip(commit.as_bytes().chunks_exact(2)) {
            *byte = u8::from_str_radix(std::str::from_utf8(hex).unwrap(), 16).unwrap();
        }
    }
    let dirty = git(&["status", "--porcelain"]).is_none_or(|status| !status.is_empty());
    std::fs::write(
        out.join("build_info.rs"),
        format!(
            "pub const GIT_HASH: [u8; 8] = {hash:?};\npub const GIT_DIRTY: u8 = {};\n",
            u8::from(dirty)
        ),
    )
    .unwrap();
    println!("cargo:rerun-if-changed=.");
    for name in ["HEAD", "index", "refs"] {
        if let Some(path) = git(&["rev-parse", "--path-format=absolute", "--git-path", name]) {
            println!("cargo:rerun-if-changed={path}");
        }
    }
    // Changes in the local RMK crates also affect the firmware's dirty flag.
    for path in [
        "../../../rmk",
        "../../../rmk-types",
        "../../../rmk-config",
        "../../../rmk-macro",
    ] {
        println!("cargo:rerun-if-changed={path}/src");
        println!("cargo:rerun-if-changed={path}/Cargo.toml");
    }
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("none") {
        return;
    }
    std::fs::copy("memory.x", out.join("memory.x")).unwrap();
    println!("cargo:rustc-link-search={}", out.display());
    println!("cargo:rustc-link-arg=-Tlink.x");
}
