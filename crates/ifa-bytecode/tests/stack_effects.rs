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
    assert_eq!(OpCode::Sub.stack_effect(), Some((2, 1)));

    // Unary ops: 1 in, 1 out
    assert_eq!(OpCode::Neg.stack_effect(), Some((1, 1)));
    assert_eq!(OpCode::Not.stack_effect(), Some((1, 1)));
}

#[test]
fn test_stack_effect_descriptions() {
    assert_eq!(OpCode::Load8.stack_effect_description(), "[addr] → [value]");
    assert_eq!(
        OpCode::Store8.stack_effect_description(),
        "[addr, value] → []"
    );
    assert_eq!(OpCode::Add.stack_effect_description(), "[a, b] → [a+b]");
}
