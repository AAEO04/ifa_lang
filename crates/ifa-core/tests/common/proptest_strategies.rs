//! Proptest strategies for generating test data

use ifa_core::*;
use proptest::prelude::*;
use std::collections::HashMap;

/// Strategy for generating any IfaValue
pub fn any_ifa_value() -> impl Strategy<Value = IfaValue> {
    prop_oneof![
        any::<i64>().prop_map(IfaValue::Int),
        any::<f64>()
            .prop_filter("finite", |x| x.is_finite())
            .prop_map(IfaValue::Float),
        any::<string::String>()
            .prop_filter("valid utf-8", |s| s.is_char_boundary(0))
            .prop_map(IfaValue::Str),
        any::<bool>().prop_map(IfaValue::Bool),
        Just(IfaValue::Null),
        prop::collection::vec(any_ifa_value(), 0..10)
            .prop_map(IfaValue::List),
        prop::collection::hash_map(any::<string::String>(), any_ifa_value(), 0..10)
            .prop_map(|map| IfaValue::Map(map)),
    ]
}

/// Strategy for generating numeric IfaValues
pub fn any_number() -> impl Strategy<Value = IfaValue> {
    prop_oneof![
        any::<i64>().prop_map(IfaValue::Int),
        any::<f64>()
            .prop_filter("finite", |x| x.is_finite())
            .prop_map(IfaValue::Float),
    ]
}

/// Strategy for generating integers
pub fn any_int() -> impl Strategy<Value = IfaValue> {
    any::<i64>().prop_map(IfaValue::Int)
}

/// Strategy for generating floats
pub fn any_float() -> impl Strategy<Value = IfaValue> {
    any::<f64>()
        .prop_filter("finite", |x| x.is_finite())
        .prop_map(IfaValue::Float)
}

/// Strategy for generating strings
pub fn any_string() -> impl Strategy<Value = IfaValue> {
    any::<string::String>()
        .prop_filter("valid utf-8", |s| s.is_char_boundary(0))
        .prop_map(IfaValue::Str)
}

/// Strategy for generating booleans
pub fn any_bool() -> impl Strategy<Value = IfaValue> {
    any::<bool>().prop_map(IfaValue::Bool)
}

/// Strategy for generating lists
pub fn any_list() -> impl Strategy<Value = IfaValue> {
    prop::collection::vec(any_ifa_value(), 0..10)
        .prop_map(IfaValue::List)
}

/// Strategy for generating maps
pub fn any_map() -> impl Strategy<Value = IfaValue> {
    prop::collection::hash_map(any::<string::String>(), any_ifa_value(), 0..10)
        .prop_map(IfaValue::Map)
}

/// Strategy for generating valid arithmetic operands
pub fn any_arithmetic_operand() -> impl Strategy<Value = IfaValue> {
    prop_oneof![
        any::<i64>().prop_map(IfaValue::Int),
        any::<f64>()
            .prop_filter("finite", |x| x.is_finite())
            .prop_map(IfaValue::Float),
    ]
}

/// Strategy for generating non-zero numbers (for division)
pub fn any_non_zero_number() -> impl Strategy<Value = IfaValue> {
    prop_oneof![
        any::<i64>()
            .prop_filter("non-zero", |x| *x != 0)
            .prop_map(IfaValue::Int),
        any::<f64>()
            .prop_filter("finite and non-zero", |x| x.is_finite() && *x != 0.0)
            .prop_map(IfaValue::Float),
    ]
}

/// Strategy for generating valid indices
pub fn any_valid_index(list_size: usize) -> impl Strategy<Value = IfaValue> {
    (0..list_size as i64).prop_map(IfaValue::Int)
}

/// Strategy for generating valid string indices
pub fn any_string_index(string_len: usize) -> impl Strategy<Value = IfaValue> {
    (0..string_len as i64).prop_map(IfaValue::Int)
}

/// Strategy for generating valid slice ranges
pub fn any_slice_range(max_len: usize) -> impl Strategy<Value = (i64, i64)> {
    (0..=max_len as i64, 0..=max_len as i64)
        .prop_map(|(start, end)| {
            if start > end {
                (end, start)
            } else {
                (start, end)
            }
        })
}

/// Strategy for generating expressions
pub fn any_expression() -> impl Strategy<Value = crate::ast::Expression> {
    let leaf = prop_oneof![
        any_ifa_value().prop_map(crate::ast::Expression::Literal),
        any::<string::String>()
            .prop_filter("valid identifier", |s| {
                !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || c == '_')
            })
            .prop_map(crate::ast::Expression::Variable),
    ];
    
    prop::recursion(leaf, |inner| {
        prop_oneof![
            (inner.clone(), any_arithmetic_operand(), any::<crate::ast::BinaryOperator>())
                .prop_map(|(left, right, op)| crate::ast::Expression::BinaryOp {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                }),
            (any::<string::String>(), prop::collection::vec(inner, 0..5))
                .prop_map(|(name, args)| crate::ast::Expression::FunctionCall { name, args }),
        ]
    })
}

/// Strategy for generating statements
pub fn any_statement() -> impl Strategy<Value = crate::ast::Statement> {
    prop_oneof![
        any_expression().prop_map(crate::ast::Statement::Expression),
        (any::<string::String>(), any_expression())
            .prop_map(|(name, value)| crate::ast::Statement::VariableDeclaration { name, value }),
        (any::<string::String>(), prop::collection::vec(any::<string::String>(), 0..3), any_expression())
            .prop_map(|(name, params, body)| crate::ast::Statement::FunctionDeclaration {
                name,
                params,
                body: vec![crate::ast::Statement::Expression(body)],
            }),
    ]
}

/// Strategy for generating programs
pub fn any_program() -> impl Strategy<Value = crate::ast::Program> {
    prop::collection::vec(any_statement(), 1..10)
        .prop_map(crate::ast::Program)
}

/// Strategy for generating bytecode
pub fn any_bytecode() -> impl Strategy<Value = Bytecode> {
    // Generate a sequence of valid opcodes
    prop::collection::vec(any_opcode(), 1..50)
        .prop_map(|opcodes| {
            let mut bytecode = Bytecode::new();
            for opcode in opcodes {
                bytecode.write_op(opcode);
            }
            bytecode
        })
}

/// Strategy for generating opcodes
pub fn any_opcode() -> impl Strategy<Value = OpCode> {
    prop_oneof![
        // Simple opcodes
        Just(OpCode::Add),
        Just(OpCode::Sub),
        Just(OpCode::Mul),
        Just(OpCode::Div),
        Just(OpCode::Mod),
        Just(OpCode::Neg),
        Just(OpCode::Not),
        Just(OpCode::Pop),
        Just(OpCode::Return),
        
        // Opcodes with arguments
        any::<i64>().prop_map(OpCode::PushInt),
        any::<f64>()
            .prop_filter("finite", |x| x.is_finite())
            .prop_map(OpCode::PushFloat),
        any::<string::String>().prop_map(OpCode::PushStr),
        any::<bool>().prop_map(OpCode::PushBool),
        Just(OpCode::PushNull),
        
        // Control flow
        any::<usize>().prop_map(OpCode::Jump),
        any::<usize>().prop_map(OpCode::JumpIfTrue),
        any::<usize>().prop_map(OpCode::JumpIfFalse),
        
        // Variable operations
        any::<string::String>().prop_map(OpCode::LoadVar),
        any::<string::String>().prop_map(OpCode::StoreVar),
        
        // Function operations
        any::<usize>().prop_map(|ip| OpCode::Call { ip }),
        any::<u8>().prop_map(OpCode::Ret),
    ]
}

/// Strategy for generating valid arithmetic pairs
pub fn any_arithmetic_pair() -> impl Strategy<Value = (IfaValue, IfaValue)> {
    prop_oneof![
        (any::<i64>(), any::<i64>())
            .prop_map(|(a, b)| (IfaValue::Int(a), IfaValue::Int(b))),
        (any::<f64>(), any::<f64>())
            .prop_filter("both finite", |(a, b)| a.is_finite() && b.is_finite())
            .prop_map(|(a, b)| (IfaValue::Float(a), IfaValue::Float(b))),
        (any::<i64>(), any::<f64>())
            .prop_filter("float finite", |(_, b)| b.is_finite())
            .prop_map(|(a, b)| (IfaValue::Int(a), IfaValue::Float(b))),
    ]
}

/// Strategy for generating comparable pairs
pub fn any_comparable_pair() -> impl Strategy<Value = (IfaValue, IfaValue)> {
    prop_oneof![
        (any::<i64>(), any::<i64>())
            .prop_map(|(a, b)| (IfaValue::Int(a), IfaValue::Int(b))),
        (any::<f64>(), any::<f64>())
            .prop_filter("both finite", |(a, b)| a.is_finite() && b.is_finite())
            .prop_map(|(a, b)| (IfaValue::Float(a), IfaValue::Float(b))),
        (any::<string::String>(), any::<string::String>())
            .prop_map(|(a, b)| (IfaValue::Str(a), IfaValue::Str(b))),
        (any::<bool>(), any::<bool>())
            .prop_map(|(a, b)| (IfaValue::Bool(a), IfaValue::Bool(b))),
    ]
}

/// Strategy for generating lists with same type elements
pub fn any_homogeneous_list() -> impl Strategy<Value = IfaValue> {
    prop_oneof![
        prop::collection::vec(any::<i64>(), 0..10)
            .prop_map(|v| IfaValue::List(v.into_iter().map(IfaValue::Int).collect())),
        prop::collection::vec(any::<f64>(), 0..10)
            .prop_filter("all finite", |v| v.iter().all(|x| x.is_finite()))
            .prop_map(|v| IfaValue::List(v.into_iter().map(IfaValue::Float).collect())),
        prop::collection::vec(any::<string::String>(), 0..10)
            .prop_map(|v| IfaValue::List(v.into_iter().map(IfaValue::Str).collect())),
        prop::collection::vec(any::<bool>(), 0..10)
            .prop_map(|v| IfaValue::List(v.into_iter().map(IfaValue::Bool).collect())),
    ]
}

/// Strategy for generating valid map keys
pub fn any_map_key() -> impl Strategy<Value = String> {
    any::<string::String>()
        .prop_filter("non-empty", |s| !s.is_empty())
        .prop_filter("valid characters", |s| {
            s.chars().all(|c| c.is_alphanumeric() || c == '_')
        })
}
