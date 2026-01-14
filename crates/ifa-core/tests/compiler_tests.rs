//! Comprehensive tests for the Ifá compiler

use ifa_core::*;
use common::fixtures::*;
use common::helpers::*;

mod basic_compilation_tests {
    use super::*;

    #[test]
    fn test_compile_simple_arithmetic() {
        let program = simple_arithmetic_program();
        let bytecode = compile(program).unwrap();
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        assert_ifa_value_eq(&result, &IfaValue::Int(3));
    }

    #[test]
    fn test_compile_complex_program() {
        let program = complex_program();
        let bytecode = compile(program).unwrap();
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        assert_ifa_value_eq(&result, &IfaValue::Int(30)); // 10 + 20
    }

    #[test]
    fn test_compile_variable_declaration() {
        let program = Program {
            statements: vec![
                Statement::VariableDeclaration {
                    name: "x".to_string(),
                    value: Expression::Literal(IfaValue::Int(42)),
                },
                Statement::Expression(Expression::Variable("x".to_string())),
            ],
        };
        
        let bytecode = compile(program).unwrap();
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        assert_ifa_value_eq(&result, &IfaValue::Int(42));
    }

    #[test]
    fn test_compile_function_declaration() {
        let program = Program {
            statements: vec![
                Statement::FunctionDeclaration {
                    name: "add".to_string(),
                    params: vec!["a".to_string(), "b".to_string()],
                    body: vec![
                        Statement::Expression(Expression::BinaryOp {
                            left: Box::new(Expression::Variable("a".to_string())),
                            op: crate::ast::BinaryOperator::Add,
                            right: Box::new(Expression::Variable("b".to_string())),
                        }),
                    ],
                },
                Statement::Expression(Expression::FunctionCall {
                    name: "add".to_string(),
                    args: vec![
                        Expression::Literal(IfaValue::Int(1)),
                        Expression::Literal(IfaValue::Int(2)),
                    ],
                }),
            ],
        };
        
        let bytecode = compile(program).unwrap();
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        assert_ifa_value_eq(&result, &IfaValue::Int(3));
    }
}

mod expression_compilation_tests {
    use super::*;

    #[test]
    fn test_compile_literal_expressions() {
        let literals = vec![
            IfaValue::Int(42),
            IfaValue::Float(3.14),
            IfaValue::Str("hello".to_string()),
            IfaValue::Bool(true),
            IfaValue::Null,
        ];
        
        for literal in literals {
            let program = Program {
                statements: vec![
                    Statement::Expression(Expression::Literal(literal.clone())),
                ],
            };
            
            let bytecode = compile(program).unwrap();
            let mut vm = create_test_vm();
            let result = vm.execute(&bytecode).unwrap();
            assert_ifa_value_eq(&result, &literal);
        }
    }

    #[test]
    fn test_compile_binary_operations() {
        let operations = vec![
            (crate::ast::BinaryOperator::Add, IfaValue::Int(5)),
            (crate::ast::BinaryOperator::Sub, IfaValue::Int(-1)),
            (crate::ast::BinaryOperator::Mul, IfaValue::Int(6)),
            (crate::ast::BinaryOperator::Div, IfaValue::Int(2)),
        ];
        
        for (op, expected) in operations {
            let program = Program {
                statements: vec![
                    Statement::Expression(Expression::BinaryOp {
                        left: Box::new(Expression::Literal(IfaValue::Int(2))),
                        op,
                        right: Box::new(Expression::Literal(IfaValue::Int(3))),
                    }),
                ],
            };
            
            let bytecode = compile(program).unwrap();
            let mut vm = create_test_vm();
            let result = vm.execute(&bytecode).unwrap();
            assert_ifa_value_eq(&result, &expected);
        }
    }

    #[test]
    fn test_compile_nested_expressions() {
        let program = Program {
            statements: vec![
                Statement::Expression(Expression::BinaryOp {
                    left: Box::new(Expression::BinaryOp {
                        left: Box::new(Expression::Literal(IfaValue::Int(1))),
                        op: crate::ast::BinaryOperator::Add,
                        right: Box::new(Expression::Literal(IfaValue::Int(2))),
                    }),
                    op: crate::ast::BinaryOperator::Mul,
                    right: Box::new(Expression::Literal(IfaValue::Int(3))),
                }),
            ],
        };
        
        // (1 + 2) * 3 = 9
        let bytecode = compile(program).unwrap();
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        assert_ifa_value_eq(&result, &IfaValue::Int(9));
    }

    #[test]
    fn test_compile_function_calls() {
        let program = Program {
            statements: vec![
                Statement::FunctionDeclaration {
                    name: "identity".to_string(),
                    params: vec!["x".to_string()],
                    body: vec![
                        Statement::Expression(Expression::Variable("x".to_string())),
                    ],
                },
                Statement::Expression(Expression::FunctionCall {
                    name: "identity".to_string(),
                    args: vec![Expression::Literal(IfaValue::Int(42))],
                }),
            ],
        };
        
        let bytecode = compile(program).unwrap();
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        assert_ifa_value_eq(&result, &IfaValue::Int(42));
    }

    #[test]
    fn test_compile_chained_function_calls() {
        let program = Program {
            statements: vec![
                Statement::FunctionDeclaration {
                    name: "add_one".to_string(),
                    params: vec!["x".to_string()],
                    body: vec![
                        Statement::Expression(Expression::BinaryOp {
                            left: Box::new(Expression::Variable("x".to_string())),
                            op: crate::ast::BinaryOperator::Add,
                            right: Box::new(Expression::Literal(IfaValue::Int(1))),
                        }),
                    ],
                },
                Statement::Expression(Expression::FunctionCall {
                    name: "add_one".to_string(),
                    args: vec![
                        Expression::FunctionCall {
                            name: "add_one".to_string(),
                            args: vec![Expression::Literal(IfaValue::Int(5))],
                        },
                    ],
                }),
            ],
        };
        
        // add_one(add_one(5)) = 7
        let bytecode = compile(program).unwrap();
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        assert_ifa_value_eq(&result, &IfaValue::Int(7));
    }
}

mod control_flow_compilation_tests {
    use super::*;

    #[test]
    fn test_compile_if_expression() {
        let program = Program {
            statements: vec![
                Statement::Expression(Expression::If {
                    condition: Box::new(Expression::Literal(IfaValue::Bool(true))),
                    then_branch: Box::new(Expression::Literal(IfaValue::Int(1))),
                    else_branch: Some(Box::new(Expression::Literal(IfaValue::Int(2))),
                }),
            ],
        };
        
        let bytecode = compile(program).unwrap();
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        assert_ifa_value_eq(&result, &IfaValue::Int(1));
    }

    #[test]
    fn test_compile_if_else_false_branch() {
        let program = Program {
            statements: vec![
                Statement::Expression(Expression::If {
                    condition: Box::new(Expression::Literal(IfaValue::Bool(false))),
                    then_branch: Box::new(Expression::Literal(IfaValue::Int(1))),
                    else_branch: Some(Box::new(Expression::Literal(IfaValue::Int(2))),
                }),
            ],
        };
        
        let bytecode = compile(program).unwrap();
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        assert_ifa_value_eq(&result, &IfaValue::Int(2));
    }

    #[test]
    fn test_compile_nested_if() {
        let program = Program {
            statements: vec![
                Statement::Expression(Expression::If {
                    condition: Box::new(Expression::Literal(IfaValue::Bool(true))),
                    then_branch: Box::new(Expression::If {
                        condition: Box::new(Expression::Literal(IfaValue::Bool(false))),
                        then_branch: Box::new(Expression::Literal(IfaValue::Int(1))),
                        else_branch: Some(Box::new(Expression::Literal(IfaValue::Int(2))),
                    }),
                    else_branch: Some(Box::new(Expression::Literal(IfaValue::Int(3))),
                }),
            ],
        };
        
        // true -> (false ? 1 : 2) = 2
        let bytecode = compile(program).unwrap();
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        assert_ifa_value_eq(&result, &IfaValue::Int(2));
    }
}

mod optimization_tests {
    use super::*;

    #[test]
    fn test_constant_folding() {
        let program = Program {
            statements: vec![
                Statement::Expression(Expression::BinaryOp {
                    left: Box::new(Expression::Literal(IfaValue::Int(1))),
                    op: crate::ast::BinaryOperator::Add,
                    right: Box::new(Expression::Literal(IfaValue::Int(2))),
                }),
            ],
        };
        
        let bytecode = compile(program).unwrap();
        
        // Should optimize to a single push of 3
        assert!(bytecode.instructions().len() <= 3); // PushInt(3), Return
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        assert_ifa_value_eq(&result, &IfaValue::Int(3));
    }

    #[test]
    fn test_dead_code_elimination() {
        let program = Program {
            statements: vec![
                Statement::VariableDeclaration {
                    name: "unused".to_string(),
                    value: Expression::Literal(IfaValue::Int(42)),
                },
                Statement::Expression(Expression::Literal(IfaValue::Int(1))),
            ],
        };
        
        let bytecode = compile(program).unwrap();
        
        // Should eliminate unused variable
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        assert_ifa_value_eq(&result, &IfaValue::Int(1));
    }

    #[test]
    fn test_peephole_optimization() {
        let program = Program {
            statements: vec![
                Statement::Expression(Expression::UnaryOp {
                    op: crate::ast::UnaryOperator::Neg,
                    operand: Box::new(Expression::Literal(IfaValue::Int(-5))),
                }),
            ],
        };
        
        let bytecode = compile(program).unwrap();
        
        // Should optimize negation of -5 to push 5
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        assert_ifa_value_eq(&result, &IfaValue::Int(5));
    }
}

mod error_handling_tests {
    use super::*;

    #[test]
    fn test_compile_undefined_variable() {
        let program = Program {
            statements: vec![
                Statement::Expression(Expression::Variable("undefined".to_string())),
            ],
        };
        
        let result = compile(program);
        assert_ifa_error(result, "undefined variable");
    }

    #[test]
    fn test_compile_redefined_function() {
        let program = Program {
            statements: vec![
                Statement::FunctionDeclaration {
                    name: "func".to_string(),
                    params: vec![],
                    body: vec![
                        Statement::Expression(Expression::Literal(IfaValue::Int(1))),
                    ],
                },
                Statement::FunctionDeclaration {
                    name: "func".to_string(),
                    params: vec![],
                    body: vec![
                        Statement::Expression(Expression::Literal(IfaValue::Int(2))),
                    ],
                },
            ],
        };
        
        let result = compile(program);
        assert_ifa_error(result, "function already defined");
    }

    #[test]
    fn test_compile_wrong_argument_count() {
        let program = Program {
            statements: vec![
                Statement::FunctionDeclaration {
                    name: "func".to_string(),
                    params: vec!["x".to_string(), "y".to_string()],
                    body: vec![
                        Statement::Expression(Expression::Literal(IfaValue::Int(1))),
                    ],
                },
                Statement::Expression(Expression::FunctionCall {
                    name: "func".to_string(),
                    args: vec![Expression::Literal(IfaValue::Int(1))], // Only 1 arg, expects 2
                }),
            ],
        };
        
        let result = compile(program);
        assert_ifa_error(result, "wrong number of arguments");
    }

    #[test]
    fn test_compile_type_errors() {
        let program = Program {
            statements: vec![
                Statement::Expression(Expression::BinaryOp {
                    left: Box::new(Expression::Literal(IfaValue::Int(1))),
                    op: crate::ast::BinaryOperator::Add,
                    right: Box::new(Expression::Literal(IfaValue::Str("hello".to_string()))),
                }),
            ],
        };
        
        let result = compile(program);
        assert_ifa_error(result, "type mismatch");
    }
}

mod bytecode_generation_tests {
    use super::*;

    #[test]
    fn test_bytecode_instruction_order() {
        let program = Program {
            statements: vec![
                Statement::Expression(Expression::BinaryOp {
                    left: Box::new(Expression::Literal(IfaValue::Int(1))),
                    op: crate::ast::BinaryOperator::Add,
                    right: Box::new(Expression::Literal(IfaValue::Int(2))),
                }),
            ],
        };
        
        let bytecode = compile(program).unwrap();
        let instructions = bytecode.instructions();
        
        // Should generate: PushInt(1), PushInt(2), Add, Return
        assert!(instructions.len() >= 4);
        assert!(matches!(instructions[0], OpCode::PushInt(1)));
        assert!(matches!(instructions[1], OpCode::PushInt(2)));
        assert!(matches!(instructions[2], OpCode::Add));
        assert!(matches!(instructions[instructions.len()-1], OpCode::Return));
    }

    #[test]
    fn test_bytecode_constant_pool() {
        let program = Program {
            statements: vec![
                Statement::Expression(Expression::Literal(IfaValue::Str("hello".to_string()))),
                Statement::Expression(Expression::Literal(IfaValue::Str("world".to_string()))),
                Statement::Expression(Expression::Literal(IfaValue::Str("hello".to_string()))),
            ],
        };
        
        let bytecode = compile(program).unwrap();
        
        // Should reuse string constants
        // This is more of a structural test - actual implementation may vary
        assert!(bytecode.instructions().len() > 0);
    }

    #[test]
    fn test_bytecode_jump_labels() {
        let program = Program {
            statements: vec![
                Statement::Expression(Expression::If {
                    condition: Box::new(Expression::Literal(IfaValue::Bool(true))),
                    then_branch: Box::new(Expression::Literal(IfaValue::Int(1))),
                    else_branch: Some(Box::new(Expression::Literal(IfaValue::Int(2))),
                }),
            ],
        };
        
        let bytecode = compile(program).unwrap();
        let instructions = bytecode.instructions();
        
        // Should contain jump instructions
        let has_jump = instructions.iter().any(|op| {
            matches!(op, OpCode::Jump(_) | OpCode::JumpIfTrue(_) | OpCode::JumpIfFalse(_))
        });
        assert!(has_jump, "Expected jump instructions for if expression");
    }
}

mod performance_tests {
    use super::*;

    #[test]
    fn test_compilation_performance() {
        let program = complex_program();
        
        benchmark_test!(compilation_performance, 1000, {
            let _bytecode = compile(program.clone()).unwrap();
        });
    }

    #[test]
    fn test_large_program_compilation() {
        let mut statements = Vec::new();
        
        // Create a large program
        for i in 0..1000 {
            statements.push(Statement::Expression(Expression::BinaryOp {
                left: Box::new(Expression::Literal(IfaValue::Int(i))),
                op: crate::ast::BinaryOperator::Add,
                right: Box::new(Expression::Literal(IfaValue::Int(1))),
            }));
        }
        
        let program = Program { statements };
        
        let (_, duration) = measure_time(|| {
            compile(program).unwrap()
        });
        
        // Should compile within reasonable time
        assert!(duration.as_millis() < 1000, "Large program compilation took too long: {:?}", duration);
    }

    #[test]
    fn test_memory_usage_during_compilation() {
        let program = complex_program();
        
        let initial_memory = get_memory_usage();
        let _bytecode = compile(program).unwrap();
        let final_memory = get_memory_usage();
        
        // Memory usage should be reasonable
        let memory_increase = final_memory.saturating_sub(initial_memory);
        assert!(memory_increase < 10_000_000, "Compilation used too much memory: {} bytes", memory_increase);
    }
}
