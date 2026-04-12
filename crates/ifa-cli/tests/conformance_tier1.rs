use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn get_ifa_binary() -> PathBuf {
    // Determine the path to the ifa binary built by cargo
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // Remove the executable name (the test runner)
    if path.ends_with("deps") {
        path.pop();
    }
    path.join("ifa")
}

fn parse_expected(content: &str) -> Option<String> {
    for line in content.lines() {
        if let Some(expected) = line.strip_prefix("# expect:") {
            return Some(expected.trim().to_string());
        }
    }
    None
}

fn run_test_file(path: &Path, engine: &str) {
    let content = fs::read_to_string(path).unwrap();
    let expected = match parse_expected(&content) {
        Some(s) => s,
        None => return, // Skip files without an expect header
    };

    let bin = get_ifa_binary();
    let mut cmd = Command::new(&bin);
    cmd.env("RUST_LOG", "error");

    match engine {
        "ast" => {
            cmd.arg("run").arg(path);
        }
        "vm" => {
            // First compile to bytecode
            let bytecode_path = path.with_extension("ifab");
            let mut compile_cmd = Command::new(&bin);
            compile_cmd.arg("bytecode").arg(path).arg("-o").arg(&bytecode_path);
            let compile_output = compile_cmd.output().expect("Failed to compile to bytecode");
            assert!(compile_output.status.success(), "Failed to compile bytecode for {}", path.display());

            cmd.arg("runb").arg(&bytecode_path);
        }
        "build" => {
            // Transpile and build native binary
            let exe_name = if cfg!(windows) { "test_bin.exe" } else { "test_bin" };
            let exe_path = path.parent().unwrap().join(exe_name);
            
            let mut build_cmd = Command::new(&bin);
            build_cmd.arg("build").arg(path).arg("-o").arg(&exe_path);
            let build_output = build_cmd.output().expect("Failed to build native binary");
            assert!(build_output.status.success(), "Failed to build native binary for {}", path.display());

            cmd = Command::new(&exe_path);
        }
        _ => panic!("Unknown engine: {}", engine),
    }

    let output = cmd.output().expect("Failed to execute ifa command");
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    assert!(
        stdout.contains(&expected) || stderr.contains(&expected),
        "[{}] Failed on {}: Expected to find '{}' in output.\nSTDOUT: '{}'\nSTDERR: '{}'",
        engine,
        path.display(),
        expected,
        stdout,
        stderr
    );

    // Cleanup bytecode and exe
    if engine == "vm" {
        let _ = fs::remove_file(path.with_extension("ifab"));
    } else if engine == "build" {
        let exe_name = if cfg!(windows) { "test_bin.exe" } else { "test_bin" };
        let _ = fs::remove_file(path.parent().unwrap().join(exe_name));
    }
}

fn discover_and_run(dir: &str, engines: Vec<&str>) {
    // Find workspace root by looking for Cargo.toml
    let mut base_path = std::env::current_dir().unwrap();
    while !base_path.join("Cargo.toml").exists() {
        if !base_path.pop() {
            panic!("Could not find workspace root");
        }
    }
    
    let base_path = base_path.join("tests").join("conformance").join(dir);
    if !base_path.exists() {
        return;
    }
    let entries = fs::read_dir(base_path).unwrap();
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("ifa") {
            for engine in &engines {
                run_test_file(&path, engine);
            }
        }
    }
}

#[test]
fn test_conformance_shared() {
    // Shared tests MUST pass on all three backends
    discover_and_run("shared", vec!["ast", "vm", "build"]);
}

#[test]
fn test_tier1_ast() {
    discover_and_run("ast", vec!["ast"]);
}

#[test]
fn test_tier1_vm() {
    discover_and_run("vm", vec!["vm"]);
}
