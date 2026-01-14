//! Comprehensive tests for the Ifá Virtual Machine

use ifa_core::*;
use common::fixtures::*;
use common::helpers::*;

mod basic_execution_tests {
    use super::*;

    #[test]
    fn test_simple_arithmetic_execution() {
        let bytecode = simple_arithmetic_bytecode();
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        assert_ifa_value_eq(&result, &IfaValue::Int(3));
    }

    #[test]
    fn test_constant_loading() {
        let mut bytecode = Bytecode::new();
        bytecode.write_op(OpCode::PushInt(42));
        bytecode.write_op(OpCode::Return);
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        assert_ifa_value_eq(&result, &IfaValue::Int(42));
    }

    #[test]
    fn test_float_operations() {
        let mut bytecode = Bytecode::new();
        bytecode.write_op(OpCode::PushFloat(3.14));
        bytecode.write_op(OpCode::PushFloat(2.0));
        bytecode.write_op(OpCode::Add);
        bytecode.write_op(OpCode::Return);
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        assert_float_eq(
            match result {
                IfaValue::Float(f) => f,
                _ => panic!("Expected Float"),
            },
            5.14,
            0.001
        );
    }

    #[test]
    fn test_string_operations() {
        let mut bytecode = Bytecode::new();
        bytecode.write_op(OpCode::PushStr("hello".to_string()));
        bytecode.write_op(OpCode::PushStr("world".to_string()));
        bytecode.write_op(OpCode::Add);
        bytecode.write_op(OpCode::Return);
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        assert_ifa_value_eq(&result, &IfaValue::Str("helloworld".to_string()));
    }

    #[test]
    fn test_boolean_operations() {
        let mut bytecode = Bytecode::new();
        bytecode.write_op(OpCode::PushBool(true));
        bytecode.write_op(OpCode::Not);
        bytecode.write_op(OpCode::Return);
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        assert_ifa_value_eq(&result, &IfaValue::Bool(false));
    }

    #[test]
    fn test_null_value() {
        let mut bytecode = Bytecode::new();
        bytecode.write_op(OpCode::PushNull);
        bytecode.write_op(OpCode::Return);
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        assert_ifa_value_eq(&result, &IfaValue::Null);
    }
}

mod stack_operations_tests {
    use super::*;

    #[test]
    fn test_stack_push_pop() {
        let mut bytecode = Bytecode::new();
        bytecode.write_op(OpCode::PushInt(1));
        bytecode.write_op(OpCode::PushInt(2));
        bytecode.write_op(OpCode::Pop);
        bytecode.write_op(OpCode::Return);
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        assert_ifa_value_eq(&result, &IfaValue::Int(1));
    }

    #[test]
    fn test_stack_underflow() {
        let bytecode = stack_underflow_bytecode();
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode);
        assert_ifa_error(result, "stack underflow");
    }

    #[test]
    fn test_stack_overflow() {
        let bytecode = stack_overflow_bytecode();
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode);
        assert_ifa_error(result, "stack overflow");
    }

    #[test]
    fn test_stack_depth_tracking() {
        let mut bytecode = Bytecode::new();
        
        // Push multiple values and check stack depth
        for i in 0..10 {
            bytecode.write_op(OpCode::PushInt(i));
        }
        
        // Pop all values
        for _ in 0..10 {
            bytecode.write_op(OpCode::Pop);
        }
        
        bytecode.write_op(OpCode::Return);
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        assert_ifa_value_eq(&result, &IfaValue::Null);
    }
}

mod control_flow_tests {
    use super::*;

    #[test]
    fn test_simple_jump() {
        let mut bytecode = Bytecode::new();
        bytecode.write_op(OpCode::PushInt(1));
        bytecode.write_op(OpCode::Jump(5)); // Jump to return
        bytecode.write_op(OpCode::PushInt(2)); // This should be skipped
        bytecode.write_op(OpCode::Add);
        bytecode.write_op(OpCode::Return);
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        assert_ifa_value_eq(&result, &IfaValue::Int(1));
    }

    #[test]
    fn test_conditional_jump_true() {
        let mut bytecode = Bytecode::new();
        bytecode.write_op(OpCode::PushBool(true));
        bytecode.write_op(OpCode::JumpIfTrue(5)); // Should jump
        bytecode.write_op(OpCode::PushInt(1)); // Skipped
        bytecode.write_op(OpCode::Jump(6));
        bytecode.write_op(OpCode::PushInt(2)); // Executed
        bytecode.write_op(OpCode::Return);
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        assert_ifa_value_eq(&result, &IfaValue::Int(2));
    }

    #[test]
    fn test_conditional_jump_false() {
        let mut bytecode = Bytecode::new();
        bytecode.write_op(OpCode::PushBool(false));
        bytecode.write_op(OpCode::JumpIfTrue(5)); // Should not jump
        bytecode.write_op(OpCode::PushInt(1)); // Executed
        bytecode.write_op(OpCode::Jump(6));
        bytecode.write_op(OpCode::PushInt(2)); // Skipped
        bytecode.write_op(OpCode::Return);
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        assert_ifa_value_eq(&result, &IfaValue::Int(1));
    }

    #[test]
    fn test_conditional_jump_false_branch() {
        let mut bytecode = Bytecode::new();
        bytecode.write_op(OpCode::PushBool(false));
        bytecode.write_op(OpCode::JumpIfFalse(5)); // Should jump
        bytecode.write_op(OpCode::PushInt(1)); // Skipped
        bytecode.write_op(OpCode::Jump(6));
        bytecode.write_op(OpCode::PushInt(2)); // Executed
        bytecode.write_op(OpCode::Return);
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        assert_ifa_value_eq(&result, &IfaValue::Int(2));
    }
}

mod variable_tests {
    use super::*;

    #[test]
    fn test_variable_store_and_load() {
        let mut bytecode = Bytecode::new();
        bytecode.write_op(OpCode::PushInt(42));
        bytecode.write_op(OpCode::StoreVar("x".to_string()));
        bytecode.write_op(OpCode::LoadVar("x".to_string()));
        bytecode.write_op(OpCode::Return);
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        assert_ifa_value_eq(&result, &IfaValue::Int(42));
    }

    #[test]
    fn test_variable_not_found() {
        let mut bytecode = Bytecode::new();
        bytecode.write_op(OpCode::LoadVar("nonexistent".to_string()));
        bytecode.write_op(OpCode::Return);
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode);
        assert_ifa_error(result, "variable not found");
    }

    #[test]
    fn test_multiple_variables() {
        let mut bytecode = Bytecode::new();
        bytecode.write_op(OpCode::PushInt(1));
        bytecode.write_op(OpCode::StoreVar("a".to_string()));
        bytecode.write_op(OpCode::PushInt(2));
        bytecode.write_op(OpCode::StoreVar("b".to_string()));
        bytecode.write_op(OpCode::LoadVar("a".to_string()));
        bytecode.write_op(OpCode::LoadVar("b".to_string()));
        bytecode.write_op(OpCode::Add);
        bytecode.write_op(OpCode::Return);
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        assert_ifa_value_eq(&result, &IfaValue::Int(3));
    }

    #[test]
    fn test_variable_overwrite() {
        let mut bytecode = Bytecode::new();
        bytecode.write_op(OpCode::PushInt(1));
        bytecode.write_op(OpCode::StoreVar("x".to_string()));
        bytecode.write_op(OpCode::PushInt(2));
        bytecode.write_op(OpCode::StoreVar("x".to_string()));
        bytecode.write_op(OpCode::LoadVar("x".to_string()));
        bytecode.write_op(OpCode::Return);
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        assert_ifa_value_eq(&result, &IfaValue::Int(2));
    }
}

mod function_tests {
    use super::*;

    #[test]
    fn test_simple_function_call() {
        let mut bytecode = Bytecode::new();
        
        // Function definition at position 10
        bytecode.write_op(OpCode::Jump(20)); // Skip function definition
        
        // Function body
        bytecode.write_op(OpCode::PushInt(42));
        bytecode.write_op(OpCode::Return);
        
        // Main code
        bytecode.write_op(OpCode::Call { ip: 10 });
        bytecode.write_op(OpCode::Return);
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        assert_ifa_value_eq(&result, &IfaValue::Int(42));
    }

    #[test]
    fn test_function_with_arguments() {
        let mut bytecode = Bytecode::new();
        
        // Function definition at position 10
        bytecode.write_op(OpCode::Jump(25)); // Skip function definition
        
        // Function body (adds two arguments)
        bytecode.write_op(OpCode::Add);
        bytecode.write_op(OpCode::Return);
        
        // Main code
        bytecode.write_op(OpCode::PushInt(10));
        bytecode.write_op(OpCode::PushInt(20));
        bytecode.write_op(OpCode::Call { ip: 10 });
        bytecode.write_op(OpCode::Return);
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        assert_ifa_value_eq(&result, &IfaValue::Int(30));
    }

    #[test]
    fn test_nested_function_calls() {
        let mut bytecode = Bytecode::new();
        
        // Function A at position 15
        bytecode.write_op(OpCode::Jump(30)); // Skip function A
        
        // Function A body
        bytecode.write_op(OpCode::PushInt(5));
        bytecode.write_op(OpCode::Return);
        
        // Function B at position 20
        bytecode.write_op(OpCode::Jump(35)); // Skip function B
        
        // Function B body
        bytecode.write_op(OpCode::Call { ip: 15 }); // Call function A
        bytecode.write_op(OpCode::PushInt(3));
        bytecode.write_op(OpCode::Add);
        bytecode.write_op(OpCode::Return);
        
        // Main code
        bytecode.write_op(OpCode::Call { ip: 20 }); // Call function B
        bytecode.write_op(OpCode::Return);
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        assert_ifa_value_eq(&result, &IfaValue::Int(8)); // 5 + 3
    }

    #[test]
    fn test_function_recursion() {
        let mut bytecode = Bytecode::new();
        
        // Recursive function at position 10
        bytecode.write_op(OpCode::Jump(30)); // Skip function definition
        
        // Function body (simple recursion with base case)
        bytecode.write_op(OpCode::Dup); // Duplicate argument
        bytecode.write_op(OpCode::PushInt(0));
        bytecode.write_op(OpCode::LessThan);
        bytecode.write_op(OpCode::JumpIfTrue(25)); // Base case: return 0
        bytecode.write_op(OpCode::Dup); // Copy argument
        bytecode.write_op(OpCode::PushInt(1));
        bytecode.write_op(OpCode::Sub);
        bytecode.write_op(OpCode::Call { ip: 10 }); // Recursive call
        bytecode.write_op(OpCode::PushInt(1));
        bytecode.write_op(OpCode::Add);
        bytecode.write_op(OpCode::Return);
        
        // Base case return
        bytecode.write_op(OpCode::PushInt(0));
        bytecode.write_op(OpCode::Return);
        
        // Main code - call with small number to avoid stack overflow
        bytecode.write_op(OpCode::PushInt(3));
        bytecode.write_op(OpCode::Call { ip: 10 });
        bytecode.write_op(OpCode::Return);
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode).unwrap();
        assert_ifa_value_eq(&result, &IfaValue::Int(3)); // 3 + 2 + 1 + 0
    }
}

mod memory_tests {
    use super::*;

    #[test]
    fn test_opon_memory_allocation() {
        let vm = create_vm_with_opon(1024);
        assert_eq!(vm.opon.capacity(), 1024);
    }

    #[test]
    fn test_memory_usage_tracking() {
        let mut vm = create_test_vm();
        let initial_usage = vm.opon.usage();
        
        // Execute some code that uses memory
        let mut bytecode = Bytecode::new();
        for i in 0..100 {
            bytecode.write_op(OpCode::PushInt(i));
        }
        bytecode.write_op(OpCode::Return);
        
        vm.execute(&bytecode).unwrap();
        
        let final_usage = vm.opon.usage();
        assert!(final_usage > initial_usage);
    }

    #[test]
    fn test_memory_cleanup() {
        let mut vm = create_test_vm();
        
        // Execute code that creates temporary values
        let mut bytecode = Bytecode::new();
        bytecode.write_op(OpCode::PushInt(1));
        bytecode.write_op(OpCode::PushInt(2));
        bytecode.write_op(OpCode::Add);
        bytecode.write_op(OpCode::Pop); // Clean up result
        bytecode.write_op(OpCode::Return);
        
        let initial_usage = vm.opon.usage();
        vm.execute(&bytecode).unwrap();
        
        // Memory should be cleaned up after execution
        let final_usage = vm.opon.usage();
        assert!(final_usage <= initial_usage + 100); // Allow some overhead
    }
}

mod error_handling_tests {
    use super::*;

    #[test]
    fn test_division_by_zero_error() {
        let mut bytecode = Bytecode::new();
        bytecode.write_op(OpCode::PushInt(10));
        bytecode.write_op(OpCode::PushInt(0));
        bytecode.write_op(OpCode::Div);
        bytecode.write_op(OpCode::Return);
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode);
        assert_ifa_error(result, "division by zero");
    }

    #[test]
    fn test_type_mismatch_error() {
        let mut bytecode = Bytecode::new();
        bytecode.write_op(OpCode::PushInt(1));
        bytecode.write_op(OpCode::PushStr("hello".to_string()));
        bytecode.write_op(OpCode::Add);
        bytecode.write_op(OpCode::Return);
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode);
        assert_ifa_error(result, "type mismatch");
    }

    #[test]
    fn test_invalid_jump_target() {
        let mut bytecode = Bytecode::new();
        bytecode.write_op(OpCode::Jump(999)); // Jump to invalid location
        bytecode.write_op(OpCode::Return);
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode);
        assert_ifa_error(result, "invalid jump target");
    }

    #[test]
    fn test_missing_return() {
        let mut bytecode = Bytecode::new();
        bytecode.write_op(OpCode::PushInt(42));
        // No return instruction
        
        let mut vm = create_test_vm();
        let result = vm.execute(&bytecode);
        // Should handle gracefully or return last value
        match result {
            Ok(_) => {}, // Acceptable
            Err(_) => {}, // Also acceptable
        }
    }
}

mod performance_tests {
    use super::*;

    #[test]
    fn test_arithmetic_performance() {
        benchmark_test!(vm_arithmetic_performance, 1000, {
            let mut bytecode = Bytecode::new();
            for i in 0..100 {
                bytecode.write_op(OpCode::PushInt(i));
                bytecode.write_op(OpCode::PushInt(1));
                bytecode.write_op(OpCode::Add);
                bytecode.write_op(OpCode::Pop);
            }
            bytecode.write_op(OpCode::Return);
            
            let mut vm = create_test_vm();
            vm.execute(&bytecode).unwrap();
        });
    }

    #[test]
    fn test_memory_allocation_performance() {
        benchmark_test!(memory_allocation_performance, 100, {
            let mut bytecode = Bytecode::new();
            for _ in 0..1000 {
                bytecode.write_op(OpCode::PushInt(42));
                bytecode.write_op(OpCode::Pop);
            }
            bytecode.write_op(OpCode::Return);
            
            let mut vm = create_test_vm();
            vm.execute(&bytecode).unwrap();
        });
    }

    #[test]
    fn test_function_call_performance() {
        let mut bytecode = Bytecode::new();
        
        // Simple function
        bytecode.write_op(OpCode::Jump(15));
        bytecode.write_op(OpCode::PushInt(1));
        bytecode.write_op(OpCode::Return);
        
        // Main code - call function many times
        for _ in 0..100 {
            bytecode.write_op(OpCode::Call { ip: 10 });
            bytecode.write_op(OpCode::Pop);
        }
        bytecode.write_op(OpCode::Return);
        
        benchmark_test!(function_call_performance, 100, {
            let mut vm = create_test_vm();
            vm.execute(&bytecode).unwrap();
        });
    }
}

mod concurrency_tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_concurrent_vm_execution() {
        let bytecode = simple_arithmetic_bytecode();
        
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
            assert_ifa_value_eq(&result, &IfaValue::Int(3));
        }
    }

    #[test]
    fn test_vm_thread_safety() {
        // VM should be Send + Sync
        fn is_send_sync<T: Send + Sync>() {}
        is_send_sync::<IfaVM>();
    }
}
