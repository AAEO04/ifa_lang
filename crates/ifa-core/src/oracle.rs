//! # Semantic Oracle
//!
//! Research tool for validating the semantic bridge between Ifá-Lang and Rust.

#[cfg(test)]
pub mod oracle {
    use crate::{parse, Interpreter, transpile_to_rust};
    use std::process::Command;
    use std::fs;
    use std::path::PathBuf;
    use std::io::Write;

    pub struct OracleResult {
        pub interpreter_stdout: String,
        pub transpiler_stdout: String,
    }

    pub fn verify_equivalence(source: &str) -> OracleResult {
        // 1. Run Interpreter
        let program = parse(source).expect("Failed to parse source");
        let mut interpreter = Interpreter::new();
        interpreter.execute(&program).expect("Interpreter execution failed");
        let int_stdout = interpreter.get_output().join("\n");

        // 2. Transpile and Compile
        let rust_code = transpile_to_rust(&program);
        
        // Use a temporary directory for compilation
        let tmp_dir = std::env::temp_dir().join("ifa_oracle");
        if !tmp_dir.exists() {
            fs::create_dir_all(&tmp_dir).unwrap();
        }

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
            println!("--- SEMANTIC MISMATCH ---");
            println!("SOURCE:\n{}", source);
            println!("INTERPRETER STDOUT:\n'{}'", int_stdout_final);
            println!("TRANSPILER STDOUT:\n'{}'", trans_stdout);
            panic!("equivalence violation");
        }

        OracleResult {
            interpreter_stdout: int_stdout_final,
            transpiler_stdout: trans_stdout,
        }
    }
}
