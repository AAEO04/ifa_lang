use ifa_babalawo::*;
use ifa_parser::parse;

fn check(src: &str) -> Babalawo {
    let program = parse(src).expect("Failed to parse source");
    check_program(&program, "test.ifa")
}

#[test]
fn test_dynamic_typing_default() {
    // Dynamic typing should allow changing types
    let src = r#"
    ayanmo x = 10;
    x = "now string";
    "#;
    let baba = check(src);
    assert!(
        !baba.has_errors(),
        "Dynamic typing should not error on type change"
    );
}

#[test]
fn test_static_typing_mismatch_var_decl() {
    // Static typing should forbid mismatch at declaration
    let src = r#"
    ayanmo x: i32 = "hello";
    "#;
    let baba = check(src);
    assert!(
        baba.has_errors(),
        "Static typing should catch decl mismatch"
    );

    // Check specific error code (assuming TYPE_MISMATCH)
    let errors = &baba.diagnostics;
    assert!(errors.iter().any(|d| d.error.code == "TYPE_MISMATCH"));
}

#[test]
fn test_static_typing_mismatch_assignment() {
    // Static typing should forbid mismatch at assignment
    let src = r#"
    ayanmo x: i32 = 10;
    x = "now string";
    "#;
    let baba = check(src);
    assert!(
        baba.has_errors(),
        "Static typing should catch assignment mismatch"
    );
}

#[test]
fn test_static_typing_binary_op_pass() {
    // Should verify expression result type
    let src = r#"
    ayanmo x: i32 = 10 + 20;
    "#;
    let baba = check(src);
    assert!(!baba.has_errors(), "i32 = int + int should pass");
}

#[test]
fn test_static_typing_binary_op_fail() {
    // Should verify expression result type (int + int != str)
    let src = r#"
    ayanmo x: str = 10 + 20;
    "#;
    let baba = check(src);
    assert!(baba.has_errors(), "str = int + int should fail");
}

#[test]
fn test_static_typing_variable_inference() {
    // Should resolve variable types
    let src = r#"
    ayanmo x: i32 = 10;
    ayanmo y: i32 = x; 
    "#;
    let baba = check(src);
    assert!(!baba.has_errors(), "i32 = i32 var should pass");
}

#[test]
fn test_static_typing_variable_inference_mismatch() {
    // Should resolve variable types and find mismatch
    let src = r#"
    ayanmo x: str = "hello";
    ayanmo y: i32 = x; 
    "#;
    let baba = check(src);
    assert!(baba.has_errors(), "i32 = str var should fail");
}

#[test]
fn test_static_typing_complex_expr() {
    // (10 + 5) * 2 is still Int/i32
    let src = r#"
    ayanmo x: i32 = (10 + 5) * 2;
    "#;
    let baba = check(src);
    assert!(!baba.has_errors(), "Complex int math should pass");
}

#[test]
fn test_hybrid_assign_static_to_dynamic() {
    // Allowed: Dynamic variable can hold anything
    let src = r#"
    ayanmo s: i32 = 10;
    ayanmo d = s;
    "#;
    let baba = check(src);
    assert!(
        !baba.has_errors(),
        "Assigning static to dynamic should pass"
    );
}

#[test]
fn test_unsafe_ffi_bridge_is_flagged() {
    let src = r#"
    Coop.itumo("python");
    "#;
    let baba = check(src);
    assert!(
        baba.has_errors(),
        "unsafe FFI bridge summon should be flagged"
    );
    assert!(
        baba.diagnostics
            .iter()
            .any(|d| d.error.code == "TABOO_UNSAFE_FFI")
    );
}

#[test]
fn test_ffi_spawn_warnings_outside_strict() {
    // Without strict mode directive (abo;), using FFI/Spawn outside ailewu produces a warning
    let src = r#"
    Coop.js("console.log('hello')");
    Ogunda.run("ls");
    "#;
    let baba = check(src);
    assert!(
        !baba.has_errors(),
        "FFI/Spawn outside strict/ailewu should not produce errors"
    );
    assert!(
        baba.warning_count() >= 2,
        "Expected warnings for FFI and Spawn escapes"
    );
    assert!(
        baba.diagnostics
            .iter()
            .any(|d| d.error.code == "UNSAFE_ESCAPE_WARNING")
    );
}

#[test]
fn test_ffi_spawn_errors_inside_strict() {
    // With strict mode directive (abo;), using FFI/Spawn outside ailewu produces hard errors
    let src = r#"
    abo;
    Coop.js("console.log('hello')");
    Ogunda.bẹrẹ("dir");
    "#;
    let baba = check(src);
    assert!(
        baba.has_errors(),
        "FFI/Spawn inside strict outside ailewu must produce hard errors"
    );
    assert!(
        baba.diagnostics
            .iter()
            .any(|d| d.error.code == "UNAUTHORIZED_ESCAPE")
    );
}

#[test]
fn test_ffi_spawn_authorized_in_ailewu() {
    // Inside an ailewu (unsafe) block, FFI and Spawn are explicitly authorized
    let src = r#"
    abo;
    ailewu {
        Coop.js("console.log('hello')");
        Ogunda.run("ls");
    }
    "#;
    let baba = check(src);
    assert!(
        !baba.has_errors(),
        "FFI/Spawn inside ailewu should be authorized (no errors)"
    );
    // There will be a warning for entering the ailewu block itself, but no UNSAFE_ESCAPE_WARNING
    assert!(
        baba.warning_count() <= 1,
        "Only AILEWU_BLOCK warning is allowed"
    );
    assert!(
        !baba
            .diagnostics
            .iter()
            .any(|d| d.error.code == "UNSAFE_ESCAPE_WARNING")
    );
}
