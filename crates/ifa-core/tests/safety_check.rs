use ifa_core::interpreter::Interpreter;
use ifa_core::parser::parse;
use ifa_core::value::IfaValue;

#[test]
fn test_safety_violation_outside_ailewu() {
    let mut interp = Interpreter::new();

    interp.opon.set(0x100, IfaValue::Int(42));
    interp.env.define("ptr", IfaValue::Ptr(0x100));

    // Try to dereference it: *ptr
    let source = "
        var val = *ptr;
    ";

    let program = parse(source).unwrap();
    let result = interp.execute(&program);

    assert!(
        result.is_err(),
        "Should fail outside ailewu block case: {:?}",
        result
    );
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("Safety violation"),
        "Error was: {}",
        err
    );
}

#[test]
fn test_safety_compliance_inside_ailewu() {
    let mut interp = Interpreter::new();

    interp.opon.set(0x100, IfaValue::Int(42));
    interp.env.define("ptr", IfaValue::Ptr(0x100));

    // Dereference inside àìléwu
    let source = "
        àìléwu {
            var val = *ptr;
        }
    ";

    let program = parse(source).unwrap();
    let result = interp.execute(&program);

    assert!(
        result.is_ok(),
        "Should succeed inside ailewu block, got: {:?}",
        result.err()
    );

    // Verify value was read into environment
    // Note: Variable defined inside block might be scoped?
    // 'var' in Ifa usually defines in current scope.
    // 'ailewu' block executes in same scope unless it creates a new one.
    // In `core.rs` `Statement::Ailewu`, it executes body. It does NOT create `env.enter_scope()`.
    // So 'val' should be leak to 'env'.
    // Let's verify.
    // If Scoping is added later, this might break, but for now `Statement::Ailewu` implementation I wrote just executes loop.

    // Checking interpreter internal state requires access to Env.
    let val = interp.env.get("val");
    assert_eq!(val, Some(IfaValue::Int(42)));
}

#[test]
fn test_unsafe_write_violation() {
    let mut interp = Interpreter::new();
    interp.env.define("ptr", IfaValue::Ptr(0x200));

    let source = "
        *ptr = 99;
    ";

    let program = parse(source).unwrap();
    let result = interp.execute(&program);

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Safety violation"));
}

#[test]
fn test_unsafe_write_allowed() {
    let mut interp = Interpreter::new();
    interp.env.define("ptr", IfaValue::Ptr(0x200));

    let source = "
        àìléwu {
            *ptr = 99;
        }
    ";

    let program = parse(source).unwrap();
    let result = interp.execute(&program);

    assert!(result.is_ok());
    assert_eq!(interp.opon.get(0x200), Some(&IfaValue::Int(99)));
}
