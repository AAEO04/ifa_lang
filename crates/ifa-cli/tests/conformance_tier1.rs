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
            compile_cmd
                .arg("bytecode")
                .arg(path)
                .arg("-o")
                .arg(&bytecode_path)
                .arg("--force");
            let compile_output = compile_cmd.output().expect("Failed to compile to bytecode");
            assert!(
                compile_output.status.success(),
                "Failed to compile bytecode for {}.\nSTDOUT:\n{}\nSTDERR:\n{}",
                path.display(),
                String::from_utf8_lossy(&compile_output.stdout),
                String::from_utf8_lossy(&compile_output.stderr)
            );

            cmd.arg("runb").arg(&bytecode_path).arg("--force");
        }
        "build" => {
            // Transpile and build native binary
            let exe_name = if cfg!(windows) {
                "test_bin.exe"
            } else {
                "test_bin"
            };
            let exe_path = path.parent().unwrap().join(exe_name);

            let mut build_cmd = Command::new(&bin);
            build_cmd.arg("build").arg(path).arg("-o").arg(&exe_path);
            let build_output = build_cmd.output().expect("Failed to build native binary");
            assert!(
                build_output.status.success(),
                "Failed to build native binary for {}.\nSTDOUT:\n{}\nSTDERR:\n{}",
                path.display(),
                String::from_utf8_lossy(&build_output.stdout),
                String::from_utf8_lossy(&build_output.stderr)
            );

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
        let exe_name = if cfg!(windows) {
            "test_bin.exe"
        } else {
            "test_bin"
        };
        let _ = fs::remove_file(path.parent().unwrap().join(exe_name));
    }
}

fn discover_and_run(dir: &str, engines: Vec<&str>) {
    // Navigate to workspace root from crates/ifa-cli
    let base_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap() // goes to 'crates'
        .parent()
        .unwrap() // goes to 'ifa_lang'
        .to_path_buf();

    let base_path = base_path.join("tests").join("conformance").join(dir);
    if !base_path.exists() {
        return;
    }
    let entries = fs::read_dir(base_path).unwrap();
    for entry in entries {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("ifa") {
            println!("Running file: {}", path.display());
            for engine in &engines {
                println!("  Engine: {}", engine);
                run_test_file(&path, engine);
            }
        }
    }
}

#[test]
fn test_conformance_shared() {
    // Shared tests MUST pass on both the VM and AOT Transpiler
    discover_and_run("shared", vec!["vm", "build"]);
}

#[test]
fn test_tier1_ast() {
    // AST interpreter merged into babalawo; tests should be migrated if applicable
    // discover_and_run("ast", vec!["ast"]);
}

#[test]
fn test_tier1_vm() {
    discover_and_run("vm", vec!["vm"]);
}
