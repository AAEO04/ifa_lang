use ifa_bytecode::OpCode;

#[test]
fn test_load_store_stack_effects() {
    // Load operations: 1 in, 1 out
    assert_eq!(OpCode::Load8.stack_effect(), Some((1, 1)));
    assert_eq!(OpCode::Load16.stack_effect(), Some((1, 1)));
    assert_eq!(OpCode::Load32.stack_effect(), Some((1, 1)));

    // Store operations: 2 in, 0 out
    assert_eq!(OpCode::Store8.stack_effect(), Some((2, 0)));
    assert_eq!(OpCode::Store16.stack_effect(), Some((2, 0)));
    assert_eq!(OpCode::Store32.stack_effect(), Some((2, 0)));
}

#[test]
fn test_arithmetic_stack_effects() {
    // Binary ops: 2 in, 1 out
    assert_eq!(OpCode::Add.stack_effect(), Some((2, 1)));
    assert_eq!(OpCode::Concat.stack_effect(), Some((2, 1)));
    assert_eq!(OpCode::Sub.stack_effect(), Some((2, 1)));

    // Unary ops: 1 in, 1 out
    assert_eq!(OpCode::Neg.stack_effect(), Some((1, 1)));
    assert_eq!(OpCode::Not.stack_effect(), Some((1, 1)));
}

#[test]
fn test_stack_effect_descriptions() {
    let load = OpCode::Load8.stack_effect_description();
    let store = OpCode::Store8.stack_effect_description();
    let add = OpCode::Add.stack_effect_description();
    let concat = OpCode::Concat.stack_effect_description();

    assert!(load.contains("[addr]") && load.contains("[value]"));
    assert!(store.contains("[addr, value]") && store.contains("[]"));
    assert!(add.contains("[a, b]") && add.contains("[a+b]"));
    assert!(concat.contains("[lhs, rhs]") && concat.contains("[lhs++rhs]"));
}
