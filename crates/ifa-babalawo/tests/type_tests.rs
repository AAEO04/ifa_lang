use ifa_babalawo::*;
use ifa_core::parser::parse;

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
