use ifa_compiler::Compiler;
use ifa_parser::parse;
use ifa_types::OpCode;

#[test]
fn test_compile_var_decl() {
    let source = "let x = 42;";
    let program = parse(source).expect("Failed to parse");
    let compiler = Compiler::new("test");
    let bytecode = compiler.compile(&program).expect("Failed to compile");

    assert!(!bytecode.code.is_empty(), "Bytecode should not be empty");

    let push_int = bytecode.code.iter().find(|&&b| b == OpCode::PushInt as u8);
    assert!(push_int.is_some(), "Expected PushInt opcode in bytecode");

    let store_global = bytecode
        .code
        .iter()
        .find(|&&b| b == OpCode::StoreGlobal as u8);
    assert!(
        store_global.is_some(),
        "Expected StoreGlobal opcode in bytecode"
    );

    assert_eq!(
        *bytecode.code.last().unwrap(),
        OpCode::Halt as u8,
        "Bytecode must end with Halt"
    );
}

#[test]
fn test_compile_empty_program() {
    let program = parse("").unwrap();
    let compiler = Compiler::new("test");
    let bytecode = compiler.compile(&program).unwrap();

    assert_eq!(bytecode.code, vec![OpCode::Halt as u8]);
}
