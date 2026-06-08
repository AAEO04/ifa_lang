//! # Semantic Oracle
//!
//! Research tool for validating the semantic bridge between Ifá-Lang and Rust.

use crate::{parse, transpile_to_rust};
use std::fs;
use std::process::Command;

pub struct OracleResult {
    pub interpreter_stdout: String,
    pub transpiler_stdout: String,
}

pub fn verify_equivalence(source: &str) -> OracleResult {
    let program = parse(source).expect("Failed to parse source");

    // Use a temporary directory for compilation and running
    let tmp_dir = std::env::temp_dir().join("ifa_oracle");
    if !tmp_dir.exists() {
        fs::create_dir_all(&tmp_dir).unwrap();
    }

    let source_file = tmp_dir.join("temp_source.ifa");
    fs::write(&source_file, source).expect("Failed to write source file");

    // 1. Run VM via CLI process
    let vm_output = Command::new("cargo")
        .arg("run")
        .arg("-p")
        .arg("ifa-cli")
        .arg("--")
        .arg("run")
        .arg(&source_file)
        .output()
        .expect("Failed to run cargo run ifa");
    let int_stdout = String::from_utf8_lossy(&vm_output.stdout).to_string();

    // 2. Transpile and Compile
    let rust_code = transpile_to_rust(&program);

    let rust_file = tmp_dir.join("test_output.rs");
    let bin_file = if cfg!(windows) {
        tmp_dir.join("test_output.exe")
    } else {
        tmp_dir.join("test_output")
    };

    fs::write(&rust_file, rust_code).expect("Failed to write Rust source");

    // Compile with rustc
    let status = Command::new("rustc")
        .arg(&rust_file)
        .arg("-o")
        .arg(&bin_file)
        .status()
        .expect("Failed to run rustc");

    if !status.success() {
        panic!("Generated Rust failed to compile");
    }

    // 3. Run Binary
    let output = Command::new(&bin_file)
        .output()
        .expect("Failed to run generated binary");

    let trans_stdout_raw = String::from_utf8_lossy(&output.stdout).to_string();

    // Cleanup "Àṣẹ! (Success)" and leading/trailing whitespace
    let trans_stdout = trans_stdout_raw
        .replace("\nÀṣẹ! (Success)", "")
        .trim()
        .to_string();

    let int_stdout_final = int_stdout.trim().to_string();

    if int_stdout_final != trans_stdout {
        panic!(
            "--- SEMANTIC MISMATCH ---\nSOURCE:\n{}\nVM STDOUT:\n'{}'\nTRANSPILER STDOUT:\n'{}'\nequivalence violation",
            source, int_stdout_final, trans_stdout
        );
    }

    OracleResult {
        interpreter_stdout: int_stdout_final,
        transpiler_stdout: trans_stdout,
    }
}
