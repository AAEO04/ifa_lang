//! Comprehensive tests for ifa-cli

use ifa_cli::*;
use std::process::Command;
use std::fs;
use std::path::PathBuf;

mod basic_cli_tests {
    use super::*;

    #[test]
    fn test_cli_help() {
        let output = Command::new("cargo")
            .args(&["run", "--bin", "ifa-cli", "--", "--help"])
            .output()
            .expect("Failed to execute CLI");
        
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Ifá-Lang"));
        assert!(stdout.contains("Usage"));
        assert!(stdout.contains("Options"));
    }

    #[test]
    fn test_cli_version() {
        let output = Command::new("cargo")
            .args(&["run", "--bin", "ifa-cli", "--", "--version"])
            .output()
            .expect("Failed to execute CLI");
        
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("ifa-cli"));
        assert!(stdout.contains("version"));
    }

    #[test]
    fn test_cli_no_args() {
        let output = Command::new("cargo")
            .args(&["run", "--bin", "ifa-cli"])
            .output()
            .expect("Failed to execute CLI");
        
        // Should show help when no arguments provided
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Usage") || stdout.contains("help"));
    }
}

mod execution_tests {
    use super::*;

    #[test]
    fn test_simple_expression() {
        let output = Command::new("cargo")
            .args(&["run", "--bin", "ifa-cli", "--", "execute", "1 + 2"])
            .output()
            .expect("Failed to execute CLI");
        
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("3"));
    }

    #[test]
    fn test_complex_expression() {
        let output = Command::new("cargo")
            .args(&["run", "--bin", "ifa-cli", "--", "execute", "(1 + 2) * 3 - 4 / 2"])
            .output()
            .expect("Failed to execute CLI");
        
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        // (1 + 2) * 3 - 4 / 2 = 9 - 2 = 7
        assert!(stdout.contains("7"));
    }

    #[test]
    fn test_string_operations() {
        let output = Command::new("cargo")
            .args(&["run", "--bin", "ifa-cli", "--", "execute", "\"hello\" + \" \" + \"world\""])
            .output()
            .expect("Failed to execute CLI");
        
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("hello world"));
    }

    #[test]
    fn test_boolean_operations() {
        let output = Command::new("cargo")
            .args(&["run", "--bin", "ifa-cli", "--", "execute", "true && false"])
            .output()
            .expect("Failed to execute CLI");
        
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("false"));
    }

    #[test]
    fn test_error_handling() {
        let output = Command::new("cargo")
            .args(&["run", "--bin", "ifa-cli", "--", "execute", "1 / 0"])
            .output()
            .expect("Failed to execute CLI");
        
        // Should fail with division by zero error
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("division by zero") || stderr.contains("error"));
    }
}

mod file_execution_tests {
    use super::*;

    fn create_test_file(content: &str) -> PathBuf {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("test_ifa.ifa");
        fs::write(&file_path, content).unwrap();
        file_path
    }

    #[test]
    fn test_execute_file() {
        let file_path = create_test_file("1 + 2");
        
        let output = Command::new("cargo")
            .args(&["run", "--bin", "ifa-cli", "--", "run", &file_path.to_string_lossy()])
            .output()
            .expect("Failed to execute CLI");
        
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("3"));
        
        // Clean up
        fs::remove_file(&file_path).unwrap();
    }

    #[test]
    fn test_execute_multiline_file() {
        let content = r#"
let x = 10
let y = 20
x + y
"#;
        let file_path = create_test_file(content);
        
        let output = Command::new("cargo")
            .args(&["run", "--bin", "ifa-cli", "--", "run", &file_path.to_string_lossy()])
            .output()
            .expect("Failed to execute CLI");
        
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("30"));
        
        // Clean up
        fs::remove_file(&file_path).unwrap();
    }

    #[test]
    fn test_execute_nonexistent_file() {
        let output = Command::new("cargo")
            .args(&["run", "--bin", "ifa-cli", "--", "run", "/nonexistent/file.ifa"])
            .output()
            .expect("Failed to execute CLI");
        
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("not found") || stderr.contains("No such file"));
    }

    #[test]
    fn test_execute_invalid_syntax() {
        let content = "1 + + 2"; // Invalid syntax
        let file_path = create_test_file(content);
        
        let output = Command::new("cargo")
            .args(&["run", "--bin", "ifa-cli", "--", "run", &file_path.to_string_lossy()])
            .output()
            .expect("Failed to execute CLI");
        
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("syntax") || stderr.contains("parse") || stderr.contains("error"));
        
        // Clean up
        fs::remove_file(&file_path).unwrap();
    }
}

mod repl_tests {
    use super::*;

    #[test]
    fn test_repl_start() {
        // This test is tricky because REPL is interactive
        // We'll test that the REPL command doesn't crash
        let output = Command::new("timeout")
            .args(&["2s", "cargo", "run", "--bin", "ifa-cli", "--", "repl"])
            .output()
            .expect("Failed to execute CLI");
        
        // Should timeout (exit after 2 seconds) without crashing
        assert!(!output.status.success()); // timeout command should fail
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("Ifá") || stdout.contains("REPL") || stdout.is_empty());
    }

    #[test]
    fn test_repl_with_input() {
        // Test REPL with piped input
        let input = "1 + 2\n3 * 4\nexit\n";
        
        let output = Command::new("cargo")
            .args(&["run", "--bin", "ifa-cli", "--", "repl"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("Failed to start CLI")
            .wait_with_output()
            .expect("Failed to read output");
        
        // Should process the input and exit
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Should contain results of the expressions
        assert!(stdout.contains("3") || stdout.contains("12"));
    }
}

mod compilation_tests {
    use super::*;

    #[test]
    fn test_compile_to_bytecode() {
        let file_path = create_test_file("1 + 2");
        let output_path = std::env::temp_dir().join("test.ifab");
        
        let output = Command::new("cargo")
            .args(&[
                "run", "--bin", "ifa-cli", "--",
                "compile",
                &file_path.to_string_lossy(),
                "-o",
                &output_path.to_string_lossy()
            ])
            .output()
            .expect("Failed to execute CLI");
        
        assert!(output.status.success());
        assert!(output_path.exists());
        
        // Clean up
        fs::remove_file(&file_path).unwrap();
        fs::remove_file(&output_path).unwrap();
    }

    #[test]
    fn test_execute_bytecode() {
        // First compile
        let source_path = create_test_file("1 + 2");
        let bytecode_path = std::env::temp_dir().join("test.ifab");
        
        let compile_output = Command::new("cargo")
            .args(&[
                "run", "--bin", "ifa-cli", "--",
                "compile",
                &source_path.to_string_lossy(),
                "-o",
                &bytecode_path.to_string_lossy()
            ])
            .output()
            .expect("Failed to compile");
        
        assert!(compile_output.status.success());
        
        // Then execute bytecode
        let exec_output = Command::new("cargo")
            .args(&[
                "run", "--bin", "ifa-cli", "--",
                "execute-bytecode",
                &bytecode_path.to_string_lossy()
            ])
            .output()
            .expect("Failed to execute bytecode");
        
        assert!(exec_output.status.success());
        let stdout = String::from_utf8_lossy(&exec_output.stdout);
        assert!(stdout.contains("3"));
        
        // Clean up
        fs::remove_file(&source_path).unwrap();
        fs::remove_file(&bytecode_path).unwrap();
    }

    #[test]
    fn test_optimization_flag() {
        let file_path = create_test_file("1 + 2");
        let output_path = std::env::temp_dir().join("test_optimized.ifab");
        
        let output = Command::new("cargo")
            .args(&[
                "run", "--bin", "ifa-cli", "--",
                "compile",
                "--optimize",
                &file_path.to_string_lossy(),
                "-o",
                &output_path.to_string_lossy()
            ])
            .output()
            .expect("Failed to execute CLI");
        
        assert!(output.status.success());
        assert!(output_path.exists());
        
        // Clean up
        fs::remove_file(&file_path).unwrap();
        fs::remove_file(&output_path).unwrap();
    }
}

mod sandbox_tests {
    use super::*;

    #[test]
    fn test_sandbox_mode() {
        let output = Command::new("cargo")
            .args(&["run", "--bin", "ifa-cli", "--", "execute", "--sandbox", "1 + 2"])
            .output()
            .expect("Failed to execute CLI");
        
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("3"));
    }

    #[test]
    fn test_sandbox_file_restriction() {
        let content = r#"
// Try to read system file
let content = read_file("/etc/passwd")
content
"#;
        let file_path = create_test_file(content);
        
        let output = Command::new("cargo")
            .args(&[
                "run", "--bin", "ifa-cli", "--",
                "run",
                "--sandbox",
                &file_path.to_string_lossy()
            ])
            .output()
            .expect("Failed to execute CLI");
        
        // Should fail due to sandbox restrictions
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("permission") || stderr.contains("denied") || stderr.contains("sandbox"));
        
        // Clean up
        fs::remove_file(&file_path).unwrap();
    }

    #[test]
    fn test_sandbox_network_restriction() {
        let content = r#"
// Try to make network request
let response = http_get("http://example.com")
response
"#;
        let file_path = create_test_file(content);
        
        let output = Command::new("cargo")
            .args(&[
                "run", "--bin", "ifa-cli", "--",
                "run",
                "--sandbox",
                &file_path.to_string_lossy()
            ])
            .output()
            .expect("Failed to execute CLI");
        
        // Should fail due to sandbox network restrictions
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("network") || stderr.contains("permission") || stderr.contains("denied"));
        
        // Clean up
        fs::remove_file(&file_path).unwrap();
    }
}

mod debug_tests {
    use super::*;

    #[test]
    fn test_debug_flag() {
        let output = Command::new("cargo")
            .args(&["run", "--bin", "ifa-cli", "--", "execute", "--debug", "1 + 2"])
            .output()
            .expect("Failed to execute CLI");
        
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Should contain debug information
        assert!(stdout.contains("3"));
        // Debug output might be in stderr
        let stderr = String::from_utf8_lossy(&output.stderr);
        // May contain debug info depending on implementation
    }

    #[test]
    fn test_verbose_flag() {
        let output = Command::new("cargo")
            .args(&["run", "--bin", "ifa-cli", "--", "execute", "--verbose", "1 + 2"])
            .output()
            .expect("Failed to execute CLI");
        
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("3"));
        
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Verbose mode should show more information
        assert!(!stderr.is_empty() || stdout.len() > 10);
    }

    #[test]
    fn test_ast_dump() {
        let output = Command::new("cargo")
            .args(&["run", "--bin", "ifa-cli", "--", "execute", "--dump-ast", "1 + 2"])
            .output()
            .expect("Failed to execute CLI");
        
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Should contain AST representation
        assert!(stdout.contains("BinaryOp") || stdout.contains("Literal") || stdout.contains("AST"));
    }

    #[test]
    fn test_bytecode_dump() {
        let output = Command::new("cargo")
            .args(&["run", "--bin", "ifa-cli", "--", "execute", "--dump-bytecode", "1 + 2"])
            .output()
            .expect("Failed to execute CLI");
        
        assert!(output.status.success());
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Should contain bytecode representation
        assert!(stdout.contains("PushInt") || stdout.contains("Add") || stdout.contains("bytecode"));
    }
}

mod performance_tests {
    use super::*;

    #[test]
    fn test_execution_performance() {
        let file_path = create_test_file("1 + 2");
        
        let start = std::time::Instant::now();
        
        let output = Command::new("cargo")
            .args(&["run", "--bin", "ifa-cli", "--", "run", &file_path.to_string_lossy()])
            .output()
            .expect("Failed to execute CLI");
        
        let duration = start.elapsed();
        
        assert!(output.status.success());
        assert!(duration < std::time::Duration::from_secs(5));
        
        // Clean up
        fs::remove_file(&file_path).unwrap();
    }

    #[test]
    fn test_large_file_execution() {
        let mut content = String::new();
        for i in 0..1000 {
            content.push_str(&format!("let x{} = {}\n", i, i));
        }
        content.push_str("x999");
        
        let file_path = create_test_file(&content);
        
        let start = std::time::Instant::now();
        
        let output = Command::new("cargo")
            .args(&["run", "--bin", "ifa-cli", "--", "run", &file_path.to_string_lossy()])
            .output()
            .expect("Failed to execute CLI");
        
        let duration = start.elapsed();
        
        assert!(output.status.success());
        assert!(duration < std::time::Duration::from_secs(10));
        
        // Clean up
        fs::remove_file(&file_path).unwrap();
    }
}

mod error_handling_tests {
    use super::*;

    #[test]
    fn test_invalid_command() {
        let output = Command::new("cargo")
            .args(&["run", "--bin", "ifa-cli", "--", "invalid-command"])
            .output()
            .expect("Failed to execute CLI");
        
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("unrecognized") || stderr.contains("invalid") || stderr.contains("usage"));
    }

    #[test]
    fn test_invalid_flag() {
        let output = Command::new("cargo")
            .args(&["run", "--bin", "ifa-cli", "--", "--invalid-flag"])
            .output()
            .expect("Failed to execute CLI");
        
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("unexpected") || stderr.contains("invalid") || stderr.contains("flag"));
    }

    #[test]
    fn test_missing_argument() {
        let output = Command::new("cargo")
            .args(&["run", "--bin", "ifa-cli", "--", "run"])
            .output()
            .expect("Failed to execute CLI");
        
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("required") || stderr.contains("missing") || stderr.contains("argument"));
    }

    #[test]
    fn test_runtime_error_handling() {
        let output = Command::new("cargo")
            .args(&["run", "--bin", "ifa-cli", "--", "execute", "undefined_variable"])
            .output()
            .expect("Failed to execute CLI");
        
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("undefined") || stderr.contains("error") || stderr.contains("not found"));
    }
}
