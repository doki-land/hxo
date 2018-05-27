use hxo_types::{Error, Result, Span};
use std::{
    env, fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Default)]
pub struct RustWasmCompiler;

impl RustWasmCompiler {
    pub fn new() -> Self {
        Self
    }

    /// 编译 Rust 代码为 WASM
    pub fn compile(&self, source: &str) -> Result<Vec<u8>> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
        let temp_dir = env::temp_dir().join(format!("hxo_rust_{}_{}", std::process::id(), timestamp));
        let project_dir = temp_dir.join("wasm_project");
        let src_dir = project_dir.join("src");

        fs::create_dir_all(&src_dir).map_err(Error::io_error)?;

        let cargo_toml = r#"[package]
name = "wasm_project"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
"#;
        fs::write(project_dir.join("Cargo.toml"), cargo_toml).map_err(Error::io_error)?;
        fs::write(src_dir.join("lib.rs"), source).map_err(Error::io_error)?;

        let output = Command::new("cargo")
            .arg("build")
            .arg("--target")
            .arg("wasm32-unknown-unknown")
            .arg("--release")
            .current_dir(&project_dir)
            .output()
            .map_err(Error::io_error)?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            // Cleanup on failure
            let _ = fs::remove_dir_all(&temp_dir);
            return Err(Error::external_error("cargo build".to_string(), stderr, Span::default()));
        }

        let wasm_path = project_dir.join("target").join("wasm32-unknown-unknown").join("release").join("wasm_project.wasm");

        let wasm_bytes = fs::read(&wasm_path).map_err(Error::io_error)?;

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);

        Ok(wasm_bytes)
    }
}
