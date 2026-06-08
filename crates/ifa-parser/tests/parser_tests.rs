use ifa_parser::parse;
use ifa_types::ast::{Expression, Statement};

#[test]
fn test_parse_var_decl() {
    let source = "let x = 42;";
    let program = parse(source).expect("Failed to parse var decl");

    assert_eq!(program.statements.len(), 1);

    if let Statement::VarDecl { name, value, .. } = &program.statements[0] {
        assert_eq!(name, "x");
        if let Expression::Int(i) = value {
            assert_eq!(*i, 42);
        } else {
            panic!("Expected Int literal, got {:?}", value);
        }
    } else {
        panic!("Expected VarDecl, got {:?}", program.statements[0]);
    }
}

#[test]
fn test_parse_error() {
    let source = "let x = ;"; // Invalid syntax
    let result = parse(source);
    assert!(result.is_err(), "Expected parse error for invalid syntax");
}
