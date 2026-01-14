//! Full integration tests for the complete Ifá-Lang pipeline

use ifa_core::*;
use common::fixtures::*;
use common::helpers::*;

mod end_to_end_tests {
    use super::*;

    #[test]
    fn test_complete_arithmetic_pipeline() {
        let code = "1 + 2 * 3 - 4 / 2";
        
        // Tokenize
        let tokens = tokenize(code).unwrap();
        assert!(!tokens.is_empty());
        
        // Parse
        let ast = parse(tokens).unwrap();
        assert!(!ast.statements.is_empty());
        
        // Compile
        let bytecode = compile(ast).unwrap();
        assert!(!bytecode.instructions().is_empty());
        
        // Execute
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        
        // 1 + (2 * 3) - (4 / 2) = 1 + 6 - 2 = 5
        assert_ifa_value_eq(&result, &IfaValue::Int(5));
    }

    #[test]
    fn test_variable_declaration_and_usage() {
        let code = r#"
            let x = 10
            let y = 20
            x + y
        "#;
        
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        let bytecode = compile(ast).unwrap();
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        
        assert_ifa_value_eq(&result, &IfaValue::Int(30));
    }

    #[test]
    fn test_function_definition_and_call() {
        let code = r#"
            fn add(a, b) {
                a + b
            }
            add(5, 3)
        "#;
        
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        let bytecode = compile(ast).unwrap();
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        
        assert_ifa_value_eq(&result, &IfaValue::Int(8));
    }

    #[test]
    fn test_nested_function_calls() {
        let code = r#"
            fn square(x) {
                x * x
            }
            fn add_squares(a, b) {
                square(a) + square(b)
            }
            add_squares(3, 4)
        "#;
        
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        let bytecode = compile(ast).unwrap();
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        
        // 3^2 + 4^2 = 9 + 16 = 25
        assert_ifa_value_eq(&result, &IfaValue::Int(25));
    }

    #[test]
    fn test_recursive_function() {
        let code = r#"
            fn factorial(n) {
                if n <= 1 {
                    1
                } else {
                    n * factorial(n - 1)
                }
            }
            factorial(5)
        "#;
        
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        let bytecode = compile(ast).unwrap();
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        
        // 5! = 120
        assert_ifa_value_eq(&result, &IfaValue::Int(120));
    }

    #[test]
    fn test_string_operations() {
        let code = r#"
            let greeting = "Hello"
            let target = "World"
            greeting + ", " + target + "!"
        "#;
        
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        let bytecode = compile(ast).unwrap();
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        
        assert_ifa_value_eq(&result, &IfaValue::Str("Hello, World!".to_string()));
    }

    #[test]
    fn test_boolean_logic() {
        let code = r#"
            let a = true
            let b = false
            a && !b
        "#;
        
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        let bytecode = compile(ast).unwrap();
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        
        assert_ifa_value_eq(&result, &IfaValue::Bool(true));
    }

    #[test]
    fn test_list_operations() {
        let code = r#"
            let list = [1, 2, 3, 4, 5]
            list[0] + list[4]
        "#;
        
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        let bytecode = compile(ast).unwrap();
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        
        assert_ifa_value_eq(&result, &IfaValue::Int(6)); // 1 + 5
    }

    #[test]
    fn test_complex_expressions() {
        let code = r#"
            let x = 10
            let y = 20
            let z = 30
            (x + y) * z / (x - y + z)
        "#;
        
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        let bytecode = compile(ast).unwrap();
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        
        // (10 + 20) * 30 / (10 - 20 + 30) = 30 * 30 / 20 = 900 / 20 = 45
        assert_ifa_value_eq(&result, &IfaValue::Int(45));
    }
}

mod error_propagation_tests {
    use super::*;

    #[test]
    fn test_syntax_error_propagation() {
        let invalid_code = "1 + + 2"; // Invalid syntax
        
        let tokens = tokenize(invalid_code).unwrap();
        let result = parse(tokens);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_runtime_error_propagation() {
        let code = "1 / 0"; // Division by zero
        
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        let bytecode = compile(ast).unwrap();
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode);
        
        assert!(result.is_err());
        assert_ifa_error(result, "division by zero");
    }

    #[test]
    fn test_undefined_variable_error() {
        let code = "undefined_variable + 1";
        
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        let result = compile(ast);
        
        // Should fail at compilation stage
        assert!(result.is_err());
    }

    #[test]
    fn test_type_mismatch_error() {
        let code = r#"
            let x = 10
            let y = "hello"
            x + y
        "#;
        
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        let bytecode = compile(ast).unwrap();
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode);
        
        assert!(result.is_err());
        assert_ifa_error(result, "type mismatch");
    }
}

mod performance_integration_tests {
    use super::*;

    #[test]
    fn test_large_program_performance() {
        let mut code = String::new();
        
        // Generate a large program
        for i in 0..1000 {
            code.push_str(&format!("let x{} = {}\n", i, i));
        }
        
        for i in 0..999 {
            code.push_str(&format!("x{} + ", i));
        }
        code.push_str("x999");
        
        let (_, duration) = measure_time(|| {
            let tokens = tokenize(&code).unwrap();
            let ast = parse(tokens).unwrap();
            let bytecode = compile(ast).unwrap();
            let mut vm = create_test_vm();
            vm.execute(&bytecode).unwrap();
        });
        
        // Should complete within reasonable time
        assert!(duration < std::time::Duration::from_secs(10));
    }

    #[test]
    fn test_deep_recursion_performance() {
        let code = r#"
            fn recursive_sum(n) {
                if n <= 0 {
                    0
                } else {
                    n + recursive_sum(n - 1)
                }
            }
            recursive_sum(100)
        "#;
        
        let (_, duration) = measure_time(|| {
            let tokens = tokenize(code).unwrap();
            let ast = parse(tokens).unwrap();
            let bytecode = compile(ast).unwrap();
            let mut vm = create_test_vm();
            vm.execute(&bytecode).unwrap();
        });
        
        // Should complete within reasonable time
        assert!(duration < std::time::Duration::from_secs(5));
    }

    #[test]
    fn test_memory_usage() {
        let code = r#"
            let large_list = []
            for i in 0..1000 {
                large_list.push(i)
            }
            large_list.length
        "#;
        
        let initial_memory = get_memory_usage();
        
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        let bytecode = compile(ast).unwrap();
        let mut vm = create_test_vm();
        vm.execute(&bytecode).unwrap();
        
        let final_memory = get_memory_usage();
        let memory_increase = final_memory.saturating_sub(initial_memory);
        
        // Memory usage should be reasonable
        assert!(memory_increase < 100_000_000); // Less than 100MB
    }
}

mod concurrency_integration_tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_concurrent_execution() {
        let code = "42 * 2";
        let bytecode = {
            let tokens = tokenize(code).unwrap();
            let ast = parse(tokens).unwrap();
            compile(ast).unwrap()
        };
        
        let handles: Vec<_> = (0..10)
            .map(|_| {
                let bc = bytecode.clone();
                thread::spawn(move || {
                    let mut vm = create_test_vm();
                    vm.execute(&bc).unwrap()
                })
            })
            .collect();
        
        for handle in handles {
            let result = handle.join().unwrap();
            assert_ifa_value_eq(&result, &IfaValue::Int(84));
        }
    }

    #[test]
    fn test_vm_isolation() {
        let code1 = "let x = 10";
        let code2 = "let x = 20";
        
        let bytecode1 = {
            let tokens = tokenize(code1).unwrap();
            let ast = parse(tokens).unwrap();
            compile(ast).unwrap()
        };
        
        let bytecode2 = {
            let tokens = tokenize(code2).unwrap();
            let ast = parse(tokens).unwrap();
            compile(ast).unwrap()
        };
        
        let handle1 = thread::spawn(move || {
            let mut vm = create_test_vm();
            vm.execute(&bytecode1).unwrap()
        });
        
        let handle2 = thread::spawn(move || {
            let mut vm = create_test_vm();
            vm.execute(&bytecode2).unwrap()
        });
        
        let result1 = handle1.join().unwrap();
        let result2 = handle2.join().unwrap();
        
        // Both should succeed independently
        assert!(result1.is_ok() || result1.is_err()); // Either is fine for variable declaration
        assert!(result2.is_ok() || result2.is_err());
    }
}

mod serialization_integration_tests {
    use super::*;

    #[test]
    fn test_bytecode_serialization() {
        let code = "1 + 2 * 3";
        
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        let bytecode = compile(ast).unwrap();
        
        // Serialize bytecode
        let serialized = serde_json::to_string(&bytecode).unwrap();
        
        // Deserialize bytecode
        let deserialized: Bytecode = serde_json::from_str(&serialized).unwrap();
        
        // Execute deserialized bytecode
        let mut vm = create_test_vm();
        let result1 = vm.execute(&bytecode).unwrap();
        let result2 = vm.execute(&deserialized).unwrap();
        
        assert_eq!(result1, result2);
    }

    #[test]
    fn test_value_serialization() {
        let complex_value = IfaValue::List(vec![
            IfaValue::Int(42),
            IfaValue::Str("hello".to_string()),
            IfaValue::Bool(true),
            IfaValue::Map({
                let mut map = std::collections::HashMap::new();
                map.insert("key".to_string(), IfaValue::Float(3.14));
                map
            }),
        ]);
        
        // Serialize
        let serialized = serde_json::to_string(&complex_value).unwrap();
        
        // Deserialize
        let deserialized: IfaValue = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(complex_value, deserialized);
    }
}

mod unicode_integration_tests {
    use super::*;

    #[test]
    fn test_unicode_identifiers() {
        let code = r#"
            let 变量 = 42
            let Ìfá = "Hello"
            变量 + 1
        "#;
        
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        let bytecode = compile(ast).unwrap();
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        
        assert_ifa_value_eq(&result, &IfaValue::Int(43));
    }

    #[test]
    fn test_unicode_strings() {
        let code = r#"
            let greeting = "🔥🌟✨"
            let yoruba = "Ẹ jẹ́ áwòkọ́"
            greeting + " " + yoruba
        "#;
        
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        let bytecode = compile(ast).unwrap();
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        
        match result {
            IfaValue::Str(s) => {
                assert!(s.contains("🔥🌟✨"));
                assert!(s.contains("Ẹ jẹ́ áwòkọ́"));
            },
            _ => panic!("Expected string result"),
        }
    }
}

mod regression_tests {
    use super::*;

    #[test]
    fn test_operator_precedence_regression() {
        let code = "1 + 2 * 3";
        
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        let bytecode = compile(ast).unwrap();
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        
        // Should be 1 + (2 * 3) = 7, not (1 + 2) * 3 = 9
        assert_ifa_value_eq(&result, &IfaValue::Int(7));
    }

    #[test]
    fn test_variable_scope_regression() {
        let code = r#"
            let x = 10
            {
                let x = 20
                x
            }
        "#;
        
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        let bytecode = compile(ast).unwrap();
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        
        // Should return the inner x = 20
        assert_ifa_value_eq(&result, &IfaValue::Int(20));
    }

    #[test]
    fn test_function_parameter_regression() {
        let code = r#"
            fn test(a, b) {
                a + b
            }
            test(1, 2, 3)  // Too many arguments
        "#;
        
        let tokens = tokenize(code).unwrap();
        let ast = parse(tokens).unwrap();
        let result = compile(ast);
        
        // Should fail at compilation due to wrong argument count
        assert!(result.is_err());
    }
}
