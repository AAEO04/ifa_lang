//! Test fixtures - reusable test data and setup

use ifa_core::*;
use std::collections::HashMap;

/// Create a default VM instance for testing
pub fn create_test_vm() -> IfaVM {
    IfaVM::new()
}

/// Create a VM with custom Opon size
pub fn create_vm_with_opon(size: usize) -> IfaVM {
    let opon = Opon::with_capacity(size);
    IfaVM::with_opon(opon)
}

/// Sample IfaValue instances for testing
pub fn sample_ifa_values() -> Vec<IfaValue> {
    vec![
        IfaValue::Int(42),
        IfaValue::Int(-42),
        IfaValue::Int(0),
        IfaValue::Float(3.14159),
        IfaValue::Float(-2.71828),
        IfaValue::Float(0.0),
        IfaValue::Str("hello".to_string()),
        IfaValue::Str("".to_string()),
        IfaValue::Str("🔥".to_string()),
        IfaValue::Bool(true),
        IfaValue::Bool(false),
        IfaValue::Null,
        IfaValue::List(vec![IfaValue::Int(1), IfaValue::Int(2), IfaValue::Int(3)]),
        IfaValue::List(vec![]),
        IfaValue::Map({
            let mut map = HashMap::new();
            map.insert("key".to_string(), IfaValue::Str("value".to_string()));
            map
        }),
    ]
}

/// Create a simple arithmetic program
pub fn simple_arithmetic_program() -> Program {
    Program {
        statements: vec![
            Statement::Expression(Expression::BinaryOp {
                left: Box::new(Expression::Literal(IfaValue::Int(1))),
                op: crate::ast::BinaryOperator::Add,
                right: Box::new(Expression::Literal(IfaValue::Int(2))),
            })
        ],
    }
}

/// Create a complex program with multiple statements
pub fn complex_program() -> Program {
    Program {
        statements: vec![
            Statement::VariableDeclaration {
                name: "x".to_string(),
                value: Expression::Literal(IfaValue::Int(10)),
            },
            Statement::VariableDeclaration {
                name: "y".to_string(),
                value: Expression::Literal(IfaValue::Int(20)),
            },
            Statement::Expression(Expression::BinaryOp {
                left: Box::new(Expression::Variable("x".to_string())),
                op: crate::ast::BinaryOperator::Add,
                right: Box::new(Expression::Variable("y".to_string())),
            }),
        ],
    }
}

/// Create bytecode for simple arithmetic
pub fn simple_arithmetic_bytecode() -> Bytecode {
    let mut bytecode = Bytecode::new();
    
    // Push 1, Push 2, Add
    bytecode.write_op(OpCode::PushInt(1));
    bytecode.write_op(OpCode::PushInt(2));
    bytecode.write_op(OpCode::Add);
    bytecode.write_op(OpCode::Return);
    
    bytecode
}

/// Create bytecode that will cause stack overflow
pub fn stack_overflow_bytecode() -> Bytecode {
    let mut bytecode = Bytecode::new();
    
    // Push many values to overflow stack
    for _ in 0..70000 {
        bytecode.write_op(OpCode::PushInt(1));
    }
    
    bytecode
}

/// Create bytecode that will cause stack underflow
pub fn stack_underflow_bytecode() -> Bytecode {
    let mut bytecode = Bytecode::new();
    
    // Try to pop from empty stack
    bytecode.write_op(OpCode::Pop);
    bytecode.write_op(OpCode::Return);
    
    bytecode
}

/// Test data for mathematical operations
pub mod math_test_data {
    use super::*;
    
    pub fn addition_test_cases() -> Vec<(IfaValue, IfaValue, IfaValue)> {
        vec![
            (IfaValue::Int(1), IfaValue::Int(2), IfaValue::Int(3)),
            (IfaValue::Int(-1), IfaValue::Int(1), IfaValue::Int(0)),
            (IfaValue::Float(1.5), IfaValue::Float(2.5), IfaValue::Float(4.0)),
            (IfaValue::Int(5), IfaValue::Float(2.5), IfaValue::Float(7.5)),
        ]
    }
    
    pub fn subtraction_test_cases() -> Vec<(IfaValue, IfaValue, IfaValue)> {
        vec![
            (IfaValue::Int(5), IfaValue::Int(3), IfaValue::Int(2)),
            (IfaValue::Int(1), IfaValue::Int(1), IfaValue::Int(0)),
            (IfaValue::Float(5.5), IfaValue::Float(2.5), IfaValue::Float(3.0)),
        ]
    }
    
    pub fn multiplication_test_cases() -> Vec<(IfaValue, IfaValue, IfaValue)> {
        vec![
            (IfaValue::Int(3), IfaValue::Int(4), IfaValue::Int(12)),
            (IfaValue::Int(-2), IfaValue::Int(3), IfaValue::Int(-6)),
            (IfaValue::Float(2.5), IfaValue::Float(4.0), IfaValue::Float(10.0)),
        ]
    }
    
    pub fn division_test_cases() -> Vec<(IfaValue, IfaValue, IfaValue)> {
        vec![
            (IfaValue::Int(10), IfaValue::Int(2), IfaValue::Int(5)),
            (IfaValue::Float(10.0), IfaValue::Float(4.0), IfaValue::Float(2.5)),
        ]
    }
}

/// Test data for string operations
pub mod string_test_data {
    use super::*;
    
    pub fn concatenation_test_cases() -> Vec<(String, String, String)> {
        vec![
            ("hello".to_string(), "world".to_string(), "helloworld".to_string()),
            ("".to_string(), "test".to_string(), "test".to_string()),
            ("🔥".to_string(), "🌟".to_string(), "🔥🌟".to_string()),
        ]
    }
}

/// Error test cases
pub mod error_test_data {
    use super::*;
    
    pub fn division_by_zero_cases() -> Vec<IfaValue> {
        vec![
            IfaValue::Int(0),
            IfaValue::Float(0.0),
        ]
    }
    
    pub fn invalid_operation_cases() -> Vec<(IfaValue, IfaValue)> {
        vec![
            (IfaValue::Str("hello".to_string()), IfaValue::Int(1)),
            (IfaValue::Bool(true), IfaValue::Float(1.0)),
            (IfaValue::Null, IfaValue::Int(1)),
        ]
    }
}
