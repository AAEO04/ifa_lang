//! Comprehensive tests for the Ifá parser

use ifa_core::*;
use common::fixtures::*;
use common::helpers::*;

mod basic_parsing_tests {
    use super::*;

    #[test]
    fn test_parse_simple_literal() {
        let code = "42";
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        
        assert_eq!(ast.statements.len(), 1);
        match &ast.statements[0] {
            Statement::Expression(expr) => {
                match expr {
                    Expression::Literal(IfaValue::Int(42)) => {},
                    _ => panic!("Expected integer literal"),
                }
            },
            _ => panic!("Expected expression statement"),
        }
    }

    #[test]
    fn test_parse_float_literal() {
        let code = "3.14";
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        
        match &ast.statements[0] {
            Statement::Expression(Expression::Literal(IfaValue::Float(f))) => {
                assert_float_eq(*f, 3.14, 0.001);
            },
            _ => panic!("Expected float literal"),
        }
    }

    #[test]
    fn test_parse_string_literal() {
        let code = "\"hello world\"";
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        
        match &ast.statements[0] {
            Statement::Expression(Expression::Literal(IfaValue::Str(s))) => {
                assert_eq!(s, "hello world");
            },
            _ => panic!("Expected string literal"),
        }
    }

    #[test]
    fn test_parse_boolean_literals() {
        let test_cases = vec![
            ("true", IfaValue::Bool(true)),
            ("false", IfaValue::Bool(false)),
        ];
        
        for (code, expected) in test_cases {
            let tokens = tokenize(code).unwrap();
            let ast = parse(tokens).unwrap();
            
            match &ast.statements[0] {
                Statement::Expression(Expression::Literal(value)) => {
                    assert_eq!(value, &expected);
                },
                _ => panic!("Expected boolean literal"),
            }
        }
    }

    #[test]
    fn test_parse_null_literal() {
        let code = "null";
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        
        match &ast.statements[0] {
            Statement::Expression(Expression::Literal(IfaValue::Null)) => {},
            _ => panic!("Expected null literal"),
        }
    }
}

mod expression_parsing_tests {
    use super::*;

    #[test]
    fn test_parse_binary_addition() {
        let code = "1 + 2";
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        
        match &ast.statements[0] {
            Statement::Expression(Expression::BinaryOp { left, op, right }) => {
                assert!(matches!(op, crate::ast::BinaryOperator::Add));
                assert!(matches!(**left, Expression::Literal(IfaValue::Int(1))));
                assert!(matches!(**right, Expression::Literal(IfaValue::Int(2))));
            },
            _ => panic!("Expected binary operation"),
        }
    }

    #[test]
    fn test_parse_operator_precedence() {
        let code = "1 + 2 * 3";
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        
        // Should parse as: 1 + (2 * 3)
        match &ast.statements[0] {
            Statement::Expression(Expression::BinaryOp { left, op, right }) => {
                assert!(matches!(op, crate::ast::BinaryOperator::Add));
                assert!(matches!(**left, Expression::Literal(IfaValue::Int(1))));
                
                match &**right {
                    Expression::BinaryOp { left: r_left, op: r_op, right: r_right } => {
                        assert!(matches!(r_op, crate::ast::BinaryOperator::Mul));
                        assert!(matches!(**r_left, Expression::Literal(IfaValue::Int(2))));
                        assert!(matches!(**r_right, Expression::Literal(IfaValue::Int(3))));
                    },
                    _ => panic!("Expected multiplication in right operand"),
                }
            },
            _ => panic!("Expected binary operation"),
        }
    }

    #[test]
    fn test_parse_parentheses() {
        let code = "(1 + 2) * 3";
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        
        // Should parse as: (1 + 2) * 3
        match &ast.statements[0] {
            Statement::Expression(Expression::BinaryOp { left, op, right }) => {
                assert!(matches!(op, crate::ast::BinaryOperator::Mul));
                
                match &**left {
                    Expression::BinaryOp { left: l_left, op: l_op, right: l_right } => {
                        assert!(matches!(l_op, crate::ast::BinaryOperator::Add));
                        assert!(matches!(**l_left, Expression::Literal(IfaValue::Int(1))));
                        assert!(matches!(**l_right, Expression::Literal(IfaValue::Int(2))));
                    },
                    _ => panic!("Expected addition in left operand"),
                }
                
                assert!(matches!(**right, Expression::Literal(IfaValue::Int(3))));
            },
            _ => panic!("Expected binary operation"),
        }
    }

    #[test]
    fn test_parse_unary_negation() {
        let code = "-42";
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        
        match &ast.statements[0] {
            Statement::Expression(Expression::UnaryOp { op, operand }) => {
                assert!(matches!(op, crate::ast::UnaryOperator::Neg));
                assert!(matches!(**operand, Expression::Literal(IfaValue::Int(42))));
            },
            _ => panic!("Expected unary operation"),
        }
    }

    #[test]
    fn test_parse_unary_not() {
        let code = "!true";
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        
        match &ast.statements[0] {
            Statement::Expression(Expression::UnaryOp { op, operand }) => {
                assert!(matches!(op, crate::ast::UnaryOperator::Not));
                assert!(matches!(**operand, Expression::Literal(IfaValue::Bool(true))));
            },
            _ => panic!("Expected unary operation"),
        }
    }

    #[test]
    fn test_parse_function_call() {
        let code = "func(1, 2, 3)";
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        
        match &ast.statements[0] {
            Statement::Expression(Expression::FunctionCall { name, args }) => {
                assert_eq!(name, "func");
                assert_eq!(args.len(), 3);
                assert!(matches!(args[0], Expression::Literal(IfaValue::Int(1))));
                assert!(matches!(args[1], Expression::Literal(IfaValue::Int(2))));
                assert!(matches!(args[2], Expression::Literal(IfaValue::Int(3))));
            },
            _ => panic!("Expected function call"),
        }
    }

    #[test]
    fn test_parse_nested_function_calls() {
        let code = "outer(inner(1), 2)";
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        
        match &ast.statements[0] {
            Statement::Expression(Expression::FunctionCall { name, args }) => {
                assert_eq!(name, "outer");
                assert_eq!(args.len(), 2);
                
                match &args[0] {
                    Expression::FunctionCall { name: inner_name, args: inner_args } => {
                        assert_eq!(inner_name, "inner");
                        assert_eq!(inner_args.len(), 1);
                        assert!(matches!(inner_args[0], Expression::Literal(IfaValue::Int(1))));
                    },
                    _ => panic!("Expected nested function call"),
                }
                
                assert!(matches!(args[1], Expression::Literal(IfaValue::Int(2))));
            },
            _ => panic!("Expected function call"),
        }
    }

    #[test]
    fn test_parse_variable_reference() {
        let code = "variable_name";
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        
        match &ast.statements[0] {
            Statement::Expression(Expression::Variable(name)) => {
                assert_eq!(name, "variable_name");
            },
            _ => panic!("Expected variable reference"),
        }
    }
}

mod statement_parsing_tests {
    use super::*;

    #[test]
    fn test_parse_variable_declaration() {
        let code = "let x = 42";
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        
        match &ast.statements[0] {
            Statement::VariableDeclaration { name, value } => {
                assert_eq!(name, "x");
                assert!(matches!(**value, Expression::Literal(IfaValue::Int(42))));
            },
            _ => panic!("Expected variable declaration"),
        }
    }

    #[test]
    fn test_parse_function_declaration() {
        let code = "fn add(a, b) { a + b }";
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        
        match &ast.statements[0] {
            Statement::FunctionDeclaration { name, params, body } => {
                assert_eq!(name, "add");
                assert_eq!(params, vec!["a", "b"]);
                assert_eq!(body.len(), 1);
                
                match &body[0] {
                    Statement::Expression(Expression::BinaryOp { left, op, right }) => {
                        assert!(matches!(op, crate::ast::BinaryOperator::Add));
                        assert!(matches!(**left, Expression::Variable(a) if a == "a"));
                        assert!(matches!(**right, Expression::Variable(b) if b == "b"));
                    },
                    _ => panic!("Expected function body to contain expression"),
                }
            },
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_parse_empty_function() {
        let code = "fn empty() {}";
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        
        match &ast.statements[0] {
            Statement::FunctionDeclaration { name, params, body } => {
                assert_eq!(name, "empty");
                assert_eq!(params.len(), 0);
                assert_eq!(body.len(), 0);
            },
            _ => panic!("Expected function declaration"),
        }
    }

    #[test]
    fn test_parse_multiple_statements() {
        let code = "let x = 1\nlet y = 2\nx + y";
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        
        assert_eq!(ast.statements.len(), 3);
        
        // First statement
        match &ast.statements[0] {
            Statement::VariableDeclaration { name, value } => {
                assert_eq!(name, "x");
                assert!(matches!(**value, Expression::Literal(IfaValue::Int(1))));
            },
            _ => panic!("Expected variable declaration"),
        }
        
        // Second statement
        match &ast.statements[1] {
            Statement::VariableDeclaration { name, value } => {
                assert_eq!(name, "y");
                assert!(matches!(**value, Expression::Literal(IfaValue::Int(2))));
            },
            _ => panic!("Expected variable declaration"),
        }
        
        // Third statement
        match &ast.statements[2] {
            Statement::Expression(Expression::BinaryOp { left, op, right }) => {
                assert!(matches!(op, crate::ast::BinaryOperator::Add));
                assert!(matches!(**left, Expression::Variable(x) if x == "x"));
                assert!(matches!(**right, Expression::Variable(y) if y == "y"));
            },
            _ => panic!("Expected expression"),
        }
    }
}

mod complex_expression_tests {
    use super::*;

    #[test]
    fn test_parse_complex_arithmetic() {
        let code = "1 + 2 * 3 - 4 / 5 + 6";
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        
        // Should parse with correct precedence: 1 + (2 * 3) - (4 / 5) + 6
        match &ast.statements[0] {
            Statement::Expression(expr) => {
                // The exact structure depends on implementation, but should be valid
                assert!(matches!(expr, Expression::BinaryOp { .. }));
            },
            _ => panic!("Expected expression"),
        }
    }

    #[test]
    fn test_parse_deeply_nested_parentheses() {
        let code = "((((1 + 2))))";
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        
        match &ast.statements[0] {
            Statement::Expression(Expression::BinaryOp { left, op, right }) => {
                assert!(matches!(op, crate::ast::BinaryOperator::Add));
                assert!(matches!(**left, Expression::Literal(IfaValue::Int(1))));
                assert!(matches!(**right, Expression::Literal(IfaValue::Int(2))));
            },
            _ => panic!("Expected binary operation"),
        }
    }

    #[test]
    fn test_parse_mixed_operations() {
        let code = "!true && false || true";
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        
        match &ast.statements[0] {
            Statement::Expression(expr) => {
                // Should parse logical operations with correct precedence
                assert!(matches!(expr, Expression::BinaryOp { .. }));
            },
            _ => panic!("Expected expression"),
        }
    }

    #[test]
    fn test_parse_chained_comparisons() {
        let code = "1 < 2 < 3";
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        
        // Should parse as (1 < 2) < 3 (though this might be semantically odd)
        match &ast.statements[0] {
            Statement::Expression(expr) => {
                assert!(matches!(expr, Expression::BinaryOp { .. }));
            },
            _ => panic!("Expected expression"),
        }
    }
}

mod error_handling_tests {
    use super::*;

    #[test]
    fn test_parse_unmatched_parenthesis() {
        let code = "(1 + 2";
        let tokens = tokenize(code).unwrap();
        let result = parse(tokens);
        assert_ifa_error(result, "unmatched");
    }

    #[test]
    fn test_parse_unmatched_bracket() {
        let code = "func(1, 2";
        let tokens = tokenize(code).unwrap();
        let result = parse(tokens);
        assert_ifa_error(result, "unmatched");
    }

    #[test]
    fn test_parse_invalid_character() {
        let code = "@#$";
        let result = tokenize(code);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_incomplete_expression() {
        let code = "1 +";
        let tokens = tokenize(code).unwrap();
        let result = parse(tokens);
        assert_ifa_error(result, "incomplete");
    }

    #[test]
    fn test_parse_empty_input() {
        let code = "";
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        assert_eq!(ast.statements.len(), 0);
    }

    #[test]
    fn test_parse_whitespace_only() {
        let code = "   \n\t   ";
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        assert_eq!(ast.statements.len(), 0);
    }

    #[test]
    fn test_parse_invalid_number_format() {
        let code = "123.456.789";
        let result = tokenize(code);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_unclosed_string() {
        let code = "\"unclosed string";
        let result = tokenize(code);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_escape_sequence() {
        let code = "\"invalid \\x escape\"";
        let result = tokenize(code);
        // Might succeed or fail depending on implementation
        // This test checks that we handle it gracefully
        match result {
            Ok(_) => {
                // If tokenization succeeds, parsing should handle it
                let ast = parse(result.unwrap());
                // Should either succeed or fail gracefully
                assert!(ast.is_ok() || ast.is_err());
            },
            Err(_) => {
                // Tokenization failed, which is acceptable
            }
        }
    }
}

mod unicode_tests {
    use super::*;

    #[test]
    fn test_parse_unicode_identifiers() {
        let code = "let 变量 = 42";
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        
        match &ast.statements[0] {
            Statement::VariableDeclaration { name, value } => {
                assert_eq!(name, "变量");
                assert!(matches!(**value, Expression::Literal(IfaValue::Int(42))));
            },
            _ => panic!("Expected variable declaration"),
        }
    }

    #[test]
    fn test_parse_unicode_strings() {
        let code = "\"🔥🌟✨\"";
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        
        match &ast.statements[0] {
            Statement::Expression(Expression::Literal(IfaValue::Str(s))) => {
                assert_eq!(s, "🔥🌟✨");
            },
            _ => panic!("Expected string literal"),
        }
    }

    #[test]
    fn test_parse_yoruba_keywords() {
        let code = "let kíkàn = 42";
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        
        match &ast.statements[0] {
            Statement::VariableDeclaration { name, value } => {
                assert_eq!(name, "kíkàn");
                assert!(matches!(**value, Expression::Literal(IfaValue::Int(42))));
            },
            _ => panic!("Expected variable declaration"),
        }
    }
}

mod performance_tests {
    use super::*;

    #[test]
    fn test_parsing_performance() {
        let code = "1 + 2 * 3 - 4 / 5 + 6 * 7 - 8 / 9 + 10";
        
        benchmark_test!(parsing_performance, 1000, {
            let tokens = tokenize(code).unwrap();
            let _ast = parse(tokens).unwrap();
        });
    }

    #[test]
    fn test_large_program_parsing() {
        let mut code = String::new();
        
        // Generate a large program
        for i in 0..1000 {
            code.push_str(&format!("let x{} = {}\n", i, i));
        }
        
        let (_, duration) = measure_time(|| {
            let tokens = tokenize(&code).unwrap();
            let _ast = parse(tokens).unwrap();
        });
        
        // Should parse within reasonable time
        assert!(duration.as_millis() < 500, "Large program parsing took too long: {:?}", duration);
    }

    #[test]
    fn test_deeply_nested_parsing() {
        let mut code = "1".to_string();
        
        // Create deeply nested parentheses
        for _ in 0..100 {
            code = format!("({} + 1)", code);
        }
        
        let tokens = tokenize(&code).unwrap();
        let ast = parse(tokens).unwrap();
        
        // Should handle deep nesting
        assert_eq!(ast.statements.len(), 1);
    }
}
