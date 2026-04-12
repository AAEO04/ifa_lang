#![cfg(feature = "std")]

use std::fs;
use std::path::{Path, PathBuf};

use ifa_core::compiler::Compiler;
use ifa_core::error::IfaError;
use ifa_core::parser::parse;
use ifa_core::vm::IfaVM;
use ifa_types::IfaValue;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/ifa-core should have two parents")
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
        // Stop after initial comment block
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

fn execute_to_completion(
    vm: &mut IfaVM,
    bytecode: &ifa_core::bytecode::Bytecode,
) -> Result<IfaValue, IfaError> {
    match vm.execute(bytecode) {
        Ok(value) => Ok(value),
        Err(IfaError::Yielded) => loop {
            match vm.resume_execution(bytecode) {
                Ok(value) => break Ok(value),
                Err(IfaError::Yielded) => continue,
                Err(err) => break Err(err),
            }
        },
        Err(err) => Err(err),
    }
}

#[test]
fn conformance_vm_defined_programs() {
    let root = repo_root();
    let dir = root.join("tests").join("conformance").join("vm");

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
        let expected_value = parse_expected_value(&expect);

        let program = parse(&source).unwrap_or_else(|e| {
            panic!("parse failed for {}: {e}", path.display());
        });
        let compiler = Compiler::new(&path.display().to_string());
        let bytecode = compiler.compile(&program).unwrap_or_else(|e| {
            panic!("compile failed for {}: {e}", path.display());
        });

        let mut vm = IfaVM::new();
        match execute_to_completion(&mut vm, &bytecode) {
            Ok(got) => assert_eq!(got, expected_value, "wrong result for {}", path.display()),
            Err(err) => panic!(
                "vm error for {}: {err} (code={:?})",
                path.display(),
                err.error_code()
            ),
        }
    }
}

#[test]
fn conformance_vm_errors_are_catchable() {
    // Sanity: the VM recovery mechanism should catch runtime errors raised inside gbiyanju
    // and allow deterministic "error becomes value" behavior at source level.
    let source = r#"
    # expect: 1
    gbiyanju {
        ayanmo x = 1 / 0;
        pada 0;
    } gba (e) {
        pada 1;
    }
    "#;

    let expect = parse_expectation(source).unwrap();
    let expected_value = parse_expected_value(&expect);

    let program = parse(source).expect("parse failed");
    let compiler = Compiler::new("conformance_vm_errors_are_catchable");
    let bytecode = compiler.compile(&program).expect("compile failed");
    let mut vm = IfaVM::new();
    let got = vm.execute(&bytecode).expect("vm failed");
    assert_eq!(got, expected_value);
}

#[test]
fn conformance_vm_tailcall_emits_and_executes() {
    // This test verifies both emission (compiler) and execution (VM does not panic on TailCall).
    let source = r#"
    ese g(x) { pada x + 1; }
    ese f(x) { pada g(x); }
    pada f(41);
    "#;

    let program = parse(source).expect("parse failed");
    let compiler = Compiler::new("conformance_vm_tailcall_emits_and_executes");
    let bytecode = compiler.compile(&program).expect("compile failed");

    // Emission check
    assert!(
        bytecode
            .code
            .iter()
            .any(|b| *b == ifa_core::OpCode::TailCall as u8),
        "expected TailCall opcode byte to be present"
    );

    // Execution check
    let mut vm = IfaVM::new();
    let got = vm.execute(&bytecode).expect("vm failed");
    assert_eq!(got, IfaValue::Int(42));
}

#[test]
fn conformance_vm_repl_globals_persist_across_executes() {
    // VM REPL contract: globals persist across REPL submissions in a single session.
    let source1 = r#"
    ayanmo x = 3;
    "#;
    let source2 = r#"
    pada x + 1;
    "#;

    let program1 = parse(source1).expect("parse failed");
    let program2 = parse(source2).expect("parse failed");
    let compiler = Compiler::new("conformance_vm_repl_globals_persist_across_executes");
    let bytecode1 = compiler.compile(&program1).expect("compile failed");
    let compiler2 = Compiler::new("conformance_vm_repl_globals_persist_across_executes");
    let bytecode2 = compiler2.compile(&program2).expect("compile failed");

    let mut vm = IfaVM::new();
    let _ = vm.execute(&bytecode1).expect("vm failed");
    let got = vm.execute(&bytecode2).expect("vm failed");
    assert_eq!(got, IfaValue::Int(4));
}

#[test]
fn conformance_vm_closure_captures_outer() {
    let source = r#"
    ese make_adder(x) {
        ese add(y) { pada x + y; }
        pada add;
    }

    ayanmo f = make_adder(5);
    pada f(3);
    "#;

    let program = parse(source).expect("parse failed");
    let compiler = Compiler::new("conformance_vm_closure_captures_outer");
    let bytecode = compiler.compile(&program).expect("compile failed");
    let mut vm = IfaVM::new();
    let got = vm.execute(&bytecode).expect("vm failed");
    assert_eq!(got, IfaValue::Int(8));
}

#[test]
fn conformance_vm_match_compiles_and_executes() {
    let source = r#"
    # expect: 20
    ayanmo x = 2;
    ayanmo out = 0;
    yàn(x) {
        1 => { out = 10; }
        2 => { out = 20; }
        _ => { out = 30; }
    }
    pada out;
    "#;

    let expect = parse_expectation(source).unwrap();
    let expected_value = parse_expected_value(&expect);

    let program = parse(source).expect("parse failed");
    let compiler = Compiler::new("conformance_vm_match_compiles_and_executes");
    let bytecode = compiler.compile(&program).expect("compile failed");
    let mut vm = IfaVM::new();
    let got = vm.execute(&bytecode).expect("vm failed");
    assert_eq!(got, expected_value);
}

#[test]
fn conformance_vm_return_from_catch_still_runs_finally() {
    let source = r#"
    ayanmo y = 0;
    ese f() {
        gbiyanju {
            ayanmo _boom = 1 / 0;
        } gba (e) {
            pada 1;
        } nipari {
            y = 2;
        }
    }
    ayanmo _r = f();
    pada y;
    "#;

    let program = parse(source).expect("parse failed");
    let compiler = Compiler::new("conformance_vm_return_from_catch_still_runs_finally");
    let bytecode = compiler.compile(&program).expect("compile failed");
    let mut vm = IfaVM::new();
    let got = vm.execute(&bytecode).expect("vm failed");
    assert_eq!(got, IfaValue::Int(2));
}

#[test]
fn conformance_vm_nested_finally_runs_outermost_before_return_completes() {
    let source = r#"
    ayanmo y = 0;
    ese f() {
        gbiyanju {
            gbiyanju {
                ayanmo _boom = 1 / 0;
            } gba (e) {
                pada 7;
            } nipari {
                y = 1;
            }
        } gba (outer) {
            pada 9;
        } nipari {
            y = 2;
        }
    }
    ayanmo _r = f();
    pada y;
    "#;

    let program = parse(source).expect("parse failed");
    let compiler =
        Compiler::new("conformance_vm_nested_finally_runs_outermost_before_return_completes");
    let bytecode = compiler.compile(&program).expect("compile failed");
    let mut vm = IfaVM::new();
    let got = vm.execute(&bytecode).expect("vm failed");
    assert_eq!(got, IfaValue::Int(2));
}

#[test]
fn conformance_vm_propagate_error_unwraps_ok_and_throws_err() {
    let mut ok_vm = IfaVM::new();
    ok_vm.set_global("okv", IfaValue::ok(IfaValue::Int(41)));
    let mut ok_bytecode =
        ifa_core::bytecode::Bytecode::new("conformance_vm_propagate_error_unwraps_ok");
    ok_bytecode.strings.push("okv".to_string());
    ok_bytecode.code = vec![
        ifa_core::OpCode::LoadGlobal as u8,
        0,
        0,
        ifa_core::OpCode::PropagateError as u8,
        ifa_core::OpCode::Return as u8,
    ];
    let ok_got = ok_vm.execute(&ok_bytecode).expect("vm failed");
    assert_eq!(ok_got, IfaValue::Int(41));

    let mut err_vm = IfaVM::new();
    err_vm.set_global("failv", IfaValue::err(IfaValue::str("boom")));
    let mut err_bytecode =
        ifa_core::bytecode::Bytecode::new("conformance_vm_propagate_error_throws_err");
    err_bytecode.strings.push("failv".to_string());
    err_bytecode.code = vec![
        ifa_core::OpCode::TryBegin as u8,
        5,
        0,
        0,
        0,
        ifa_core::OpCode::LoadGlobal as u8,
        0,
        0,
        ifa_core::OpCode::PropagateError as u8,
        ifa_core::OpCode::TryEnd as u8,
        ifa_core::OpCode::PushInt as u8,
        7,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        ifa_core::OpCode::Return as u8,
    ];
    let err_got = err_vm.execute(&err_bytecode).expect("vm failed");
    assert_eq!(err_got, IfaValue::Int(7));
}

#[test]
fn conformance_vm_import_executes_module() {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    let root = std::env::temp_dir().join(format!(
        "ifa_vm_import_{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
    ));
    fs::create_dir_all(&root).unwrap();

    let module_path = root.join("a.ifa");
    let main_path = root.join("main.ifa");

    fs::write(
        &module_path,
        r#"
        fi ese foo(x) { pada x + 1; }
        "#,
    )
    .unwrap();

    fs::write(
        &main_path,
        r#"
        iba a;
        pada a.foo(2);
        "#,
    )
    .unwrap();

    let source = fs::read_to_string(&main_path).unwrap();
    let program = parse(&source).expect("parse failed");
    let compiler = Compiler::new(main_path.to_string_lossy().as_ref());
    let bytecode = compiler.compile(&program).expect("compile failed");

    let mut vm = IfaVM::with_file(&main_path);
    let got = vm.execute(&bytecode).expect("vm failed");
    assert_eq!(got, IfaValue::Int(3));
}

#[test]
fn conformance_vm_import_reloads_on_change() {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    let root = std::env::temp_dir().join(format!(
        "ifa_vm_import_reload_{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
    ));
    fs::create_dir_all(&root).unwrap();

    let module_path = root.join("a.ifa");
    let main_path = root.join("main.ifa");

    fs::write(
        &module_path,
        r#"
        fi ese foo() { pada 1; }
        "#,
    )
    .unwrap();

    fs::write(
        &main_path,
        r#"
        iba a;
        pada a.foo();
        "#,
    )
    .unwrap();

    let source = fs::read_to_string(&main_path).unwrap();
    let program = parse(&source).expect("parse failed");
    let compiler = Compiler::new(main_path.to_string_lossy().as_ref());
    let bytecode = compiler.compile(&program).expect("compile failed");

    let mut vm = IfaVM::with_file(&main_path);
    let got1 = vm.execute(&bytecode).expect("vm failed");
    assert_eq!(got1, IfaValue::Int(1));

    fs::write(
        &module_path,
        r#"
        fi ese foo() { pada 2; }
        "#,
    )
    .unwrap();

    let got2 = vm.execute(&bytecode).expect("vm failed");
    assert_eq!(got2, IfaValue::Int(2));
}
