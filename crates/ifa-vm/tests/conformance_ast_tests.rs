#![cfg(feature = "std")]

use std::fs;
use std::path::{Path, PathBuf};

use ifa_types::IfaValue;
use ifa_vm::IfaVM;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/ifa-vm should have two parents")
        .to_path_buf()
}

fn parse_expectation(source: &str) -> Option<String> {
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("# expect:") {
            return Some(rest.trim().to_string());
        }
        if !trimmed.starts_with('#') {
            break;
        }
    }
    None
}

fn parse_expected_value(raw: &str) -> IfaValue {
    let s = raw.trim();
    if s == "ofo" || s.eq_ignore_ascii_case("null") {
        return IfaValue::null();
    }
    if s == "otito" || s.eq_ignore_ascii_case("true") {
        return IfaValue::bool(true);
    }
    if s == "eke" || s.eq_ignore_ascii_case("false") {
        return IfaValue::bool(false);
    }
    if let Some(stripped) = s.strip_prefix('"').and_then(|t| t.strip_suffix('"')) {
        return IfaValue::str(stripped);
    }
    if let Ok(i) = s.parse::<i64>() {
        return IfaValue::Int(i);
    }
    if let Ok(f) = s.parse::<f64>() {
        return IfaValue::Float(f);
    }
    panic!("Unsupported expect literal: {s}");
}

fn collect_ifa_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).expect("read_dir failed") {
        let entry = entry.expect("dir entry failed");
        let path = entry.path();
        if path.is_dir() {
            files.extend(collect_ifa_files(&path));
        } else if path.extension().and_then(|e| e.to_str()) == Some("ifa") {
            files.push(path);
        }
    }
    files.sort();
    files
}

#[test]
fn conformance_ast_defined_programs() {
    let root = repo_root();
    let dir = root.join("tests").join("conformance").join("ast");

    let files = collect_ifa_files(&dir);
    assert!(
        !files.is_empty(),
        "no conformance programs found under {}",
        dir.display()
    );

    for path in files {
        let source = fs::read_to_string(&path).expect("failed to read .ifa");
        let expect = parse_expectation(&source)
            .unwrap_or_else(|| panic!("missing '# expect:' directive in {}", path.display()));

        let relative_path = path.strip_prefix(&root).unwrap_or(&path);
        let path_str = relative_path.to_string_lossy().replace('\\', "/");
        let program = ifa_parser::parse(&source).unwrap_or_else(|e| {
            panic!("parse failed for {}: {e}", path.display());
        });
        let compiler = ifa_compiler::Compiler::new(&path_str);
        let bytecode = compiler.compile(&program).unwrap_or_else(|e| {
            panic!("compile failed for {}: {e}", path.display());
        });
        let mut vm = IfaVM::new();
        let got = vm.execute(&bytecode);

        if expect.contains("[at ") {
            let got_err = got.expect_err(&format!(
                "expected error containing '{}', but execution succeeded in {}",
                expect,
                path.display()
            ));
            let got_err_str = got_err.to_string();
            assert!(
                got_err_str.contains(&expect),
                "expected error containing '{}', got: '{}' in {}",
                expect,
                got_err_str,
                path.display()
            );
        } else {
            let got_val = got.unwrap_or_else(|e| {
                panic!("VM error for {}: {e}", path.display());
            });
            let expected_value = parse_expected_value(&expect);
            assert_eq!(
                got_val,
                expected_value,
                "wrong result for {}",
                path.display()
            );
        }
    }
}
