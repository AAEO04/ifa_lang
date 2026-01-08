//! Integration tests for ifa-core
//! 
//! Tests the VM, bytecode execution, and IfaValue operations.

use ifa_core::*;

// =============================================================================
// IFAVALUE TESTS
// =============================================================================

mod value_tests {
    use super::*;
    
    #[test]
    fn test_int_arithmetic() {
        let a = IfaValue::Int(10);
        let b = IfaValue::Int(5);
        
        assert_eq!(a.clone() + b.clone(), IfaValue::Int(15));
        assert_eq!(a.clone() - b.clone(), IfaValue::Int(5));
        assert_eq!(a.clone() * b.clone(), IfaValue::Int(50));
    }
    
    #[test]
    fn test_float_arithmetic() {
        let a = IfaValue::Float(10.5);
        let b = IfaValue::Float(2.5);
        
        assert_eq!(a.clone() + b.clone(), IfaValue::Float(13.0));
        assert_eq!(a.clone() - b.clone(), IfaValue::Float(8.0));
        assert_eq!(a.clone() * b.clone(), IfaValue::Float(26.25));
    }
    
    #[test]
    fn test_mixed_arithmetic() {
        let int = IfaValue::Int(10);
        let float = IfaValue::Float(2.5);
        
        // Int + Float should promote to Float
        if let IfaValue::Float(result) = int.clone() + float.clone() {
            assert_eq!(result, 12.5);
        } else {
            panic!("Expected Float result");
        }
    }
    
    #[test]
    fn test_string_concat() {
        let a = IfaValue::Str("Hello, ".to_string());
        let b = IfaValue::Str("Ifá!".to_string());
        
        assert_eq!(a + b, IfaValue::Str("Hello, Ifá!".to_string()));
    }
    
    #[test]
    fn test_string_repeat() {
        let s = IfaValue::Str("Na".to_string());
        let n = IfaValue::Int(3);
        
        assert_eq!(s * n, IfaValue::Str("NaNaNa".to_string()));
    }
    
    #[test]
    fn test_list_concat() {
        let a = IfaValue::List(vec![IfaValue::Int(1), IfaValue::Int(2)]);
        let b = IfaValue::List(vec![IfaValue::Int(3), IfaValue::Int(4)]);
        
        let expected = IfaValue::List(vec![
            IfaValue::Int(1),
            IfaValue::Int(2),
            IfaValue::Int(3),
            IfaValue::Int(4),
        ]);
        
        assert_eq!(a + b, expected);
    }
    
    #[test]
    fn test_division_produces_float() {
        let a = IfaValue::Int(10);
        let b = IfaValue::Int(4);
        
        if let IfaValue::Float(result) = a / b {
            assert_eq!(result, 2.5);
        } else {
            panic!("Expected Float result from division");
        }
    }
    
    #[test]
    fn test_division_by_zero() {
        let a = IfaValue::Int(10);
        let b = IfaValue::Int(0);
        
        // Should return Null instead of panic
        assert_eq!(a / b, IfaValue::Null);
    }
    
    #[test]
    fn test_modulo() {
        let a = IfaValue::Int(17);
        let b = IfaValue::Int(5);
        
        assert_eq!(a % b, IfaValue::Int(2));
    }
    
    #[test]
    fn test_negation() {
        let a = IfaValue::Int(42);
        assert_eq!(-a, IfaValue::Int(-42));
        
        let b = IfaValue::Float(3.14);
        assert_eq!(-b, IfaValue::Float(-3.14));
    }
    
    #[test]
    fn test_logical_not() {
        assert_eq!(!IfaValue::Bool(true), IfaValue::Bool(false));
        assert_eq!(!IfaValue::Bool(false), IfaValue::Bool(true));
        assert_eq!(!IfaValue::Null, IfaValue::Bool(true));
        assert_eq!(!IfaValue::Int(0), IfaValue::Bool(true));
        assert_eq!(!IfaValue::Int(1), IfaValue::Bool(false));
    }
    
    #[test]
    fn test_truthiness() {
        assert!(IfaValue::Bool(true).is_truthy());
        assert!(!IfaValue::Bool(false).is_truthy());
        assert!(IfaValue::Int(1).is_truthy());
        assert!(!IfaValue::Int(0).is_truthy());
        assert!(IfaValue::Str("hello".to_string()).is_truthy());
        assert!(!IfaValue::Str("".to_string()).is_truthy());
        assert!(!IfaValue::Null.is_truthy());
        assert!(IfaValue::List(vec![IfaValue::Int(1)]).is_truthy());
        assert!(!IfaValue::List(vec![]).is_truthy());
    }
    
    #[test]
    fn test_comparison() {
        assert!(IfaValue::Int(5) == IfaValue::Int(5));
        assert!(IfaValue::Int(5) != IfaValue::Int(10));
        assert!(IfaValue::Int(5) < IfaValue::Int(10));
        assert!(IfaValue::Int(10) > IfaValue::Int(5));
        
        // Cross-type comparison
        assert!(IfaValue::Int(5) == IfaValue::Float(5.0));
        assert!(IfaValue::Int(5) < IfaValue::Float(5.5));
    }
    
    #[test]
    fn test_negative_indexing() {
        let list = IfaValue::List(vec![
            IfaValue::Int(1),
            IfaValue::Int(2),
            IfaValue::Int(3),
        ]);
        
        assert_eq!(list.get(&IfaValue::Int(0)).unwrap(), IfaValue::Int(1));
        assert_eq!(list.get(&IfaValue::Int(-1)).unwrap(), IfaValue::Int(3));
        assert_eq!(list.get(&IfaValue::Int(-2)).unwrap(), IfaValue::Int(2));
    }
    
    #[test]
    fn test_string_negative_indexing() {
        let s = IfaValue::Str("Ifá".to_string());
        
        assert_eq!(s.get(&IfaValue::Int(0)).unwrap(), IfaValue::Str("I".to_string()));
        assert_eq!(s.get(&IfaValue::Int(-1)).unwrap(), IfaValue::Str("á".to_string()));
    }
    
    #[test]
    fn test_map_access() {
        let mut map = std::collections::HashMap::new();
        map.insert("name".to_string(), IfaValue::Str("Ifá".to_string()));
        map.insert("year".to_string(), IfaValue::Int(2026));
        
        let m = IfaValue::Map(map);
        
        assert_eq!(
            m.get(&IfaValue::Str("name".to_string())).unwrap(),
            IfaValue::Str("Ifá".to_string())
        );
        assert!(m.get(&IfaValue::Str("missing".to_string())).is_err());
    }
    
    #[test]
    fn test_slice() {
        let s = IfaValue::Str("Hello, World!".to_string());
        assert_eq!(s.slice(0, 5).unwrap(), IfaValue::Str("Hello".to_string()));
        assert_eq!(s.slice(7, 12).unwrap(), IfaValue::Str("World".to_string()));
    }
    
    #[test]
    fn test_list_slice() {
        let list = IfaValue::List(vec![
            IfaValue::Int(1),
            IfaValue::Int(2),
            IfaValue::Int(3),
            IfaValue::Int(4),
            IfaValue::Int(5),
        ]);
        
        let sliced = list.slice(1, 4).unwrap();
        let expected = IfaValue::List(vec![
            IfaValue::Int(2),
            IfaValue::Int(3),
            IfaValue::Int(4),
        ]);
        
        assert_eq!(sliced, expected);
    }
    
    #[test]
    fn test_type_name() {
        assert_eq!(IfaValue::Int(42).type_name(), "Int");
        assert_eq!(IfaValue::Float(3.14).type_name(), "Float");
        assert_eq!(IfaValue::Str("test".to_string()).type_name(), "Str");
        assert_eq!(IfaValue::Bool(true).type_name(), "Bool");
        assert_eq!(IfaValue::Null.type_name(), "Null");
    }
    
    #[test]
    fn test_display_yoruba() {
        // Boolean displays in Yoruba
        assert_eq!(format!("{}", IfaValue::Bool(true)), "òtítọ́");
        assert_eq!(format!("{}", IfaValue::Bool(false)), "èké");
        assert_eq!(format!("{}", IfaValue::Null), "àìsí");
    }
    
    #[test]
    fn test_overflow_promotion() {
        let max = IfaValue::Int(i64::MAX);
        let one = IfaValue::Int(1);
        
        // Should promote to Float instead of overflow
        if let IfaValue::Float(_) = max + one {
            // Expected
        } else {
            panic!("Should promote to Float on overflow");
        }
    }
}

// =============================================================================
// BYTECODE TESTS
// =============================================================================

mod bytecode_tests {
    use super::*;
    
    #[test]
    fn test_opcode_roundtrip() {
        let opcodes = [
            OpCode::PushNull,
            OpCode::PushInt,
            OpCode::Add,
            OpCode::Sub,
            OpCode::Mul,
            OpCode::Div,
            OpCode::Halt,
        ];
        
        for op in opcodes {
            let byte = op as u8;
            let decoded = OpCode::from_byte(byte).unwrap();
            assert_eq!(decoded, op);
        }
    }
    
    #[test]
    fn test_unknown_opcode() {
        let result = OpCode::from_byte(0xFE);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_bytecode_serialize() {
        let mut bc = Bytecode::new("test.ifa");
        bc.code = vec![
            OpCode::PushInt as u8,
            0, 0, 0, 0, 0, 0, 0, 5,
            OpCode::Halt as u8,
        ];
        bc.strings = vec!["hello".to_string(), "world".to_string()];
        
        let bytes = bc.to_bytes();
        let decoded = Bytecode::from_bytes(&bytes).unwrap();
        
        assert_eq!(decoded.code, bc.code);
        assert_eq!(decoded.strings, bc.strings);
        assert_eq!(decoded.source_name, "test.ifa");
    }
    
    #[test]
    fn test_invalid_magic() {
        let bad_bytes = vec![0x00, 0x00, 0x00, 0x00, 0x01];
        let result = Bytecode::from_bytes(&bad_bytes);
        assert!(result.is_err());
    }
}

// =============================================================================
// OPON (MEMORY) TESTS
// =============================================================================

mod opon_tests {
    use super::*;
    use crate::opon::{Opon, OponSize};
    
    #[test]
    fn test_memory_set_get() {
        let mut opon = Opon::new(OponSize::Kekere);
        
        assert!(opon.set(0, IfaValue::Int(42)));
        assert_eq!(opon.get(0), Some(&IfaValue::Int(42)));
    }
    
    #[test]
    fn test_memory_bounds() {
        let mut opon = Opon::new(OponSize::Kekere); // 256 slots
        
        // Should succeed
        assert!(opon.set(255, IfaValue::Int(1)));
        
        // Should fail (out of bounds for fixed-size)
        // Note: Kekere has 256 slots, so 256 is out of bounds
    }
    
    #[test]
    fn test_flight_recorder() {
        let mut opon = Opon::new(OponSize::Kekere);
        
        opon.record("Ìrosù", "fọ̀", &IfaValue::Str("Hello".to_string()));
        opon.record("Ọ̀bàrà", "fikun", &IfaValue::Int(42));
        
        let history = opon.get_history();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].spirit, "Ìrosù");
        assert_eq!(history[1].spirit, "Ọ̀bàrà");
    }
    
    #[test]
    fn test_circular_buffer() {
        let mut opon = Opon::new(OponSize::Kekere);
        
        // Fill beyond capacity (256)
        for i in 0..300 {
            opon.record_msg("Test", "event", &format!("{}", i));
        }
        
        let history = opon.get_history();
        assert_eq!(history.len(), 256); // Capped
        
        // Oldest should have been overwritten
        assert_eq!(history[0].value, "44"); // 300 - 256 = 44
    }
    
    #[test]
    fn test_memory_used() {
        let mut opon = Opon::new(OponSize::Kekere);
        
        assert_eq!(opon.memory_used(), 0);
        
        opon.set(0, IfaValue::Int(1));
        opon.set(10, IfaValue::Int(2));
        
        assert_eq!(opon.memory_used(), 2);
    }
}

// =============================================================================
// VM TESTS
// =============================================================================

mod vm_tests {
    use super::*;
    
    #[test]
    fn test_simple_addition() {
        let mut vm = IfaVM::new();
        
        let mut bc = Bytecode::new("test");
        bc.code = vec![
            OpCode::PushInt as u8,
            0, 0, 0, 0, 0, 0, 0, 5, // Push 5
            OpCode::PushInt as u8,
            0, 0, 0, 0, 0, 0, 0, 3, // Push 3
            OpCode::Add as u8,
            OpCode::Halt as u8,
        ];
        
        let result = vm.execute(&bc).unwrap();
        assert_eq!(result, IfaValue::Int(8));
    }
    
    #[test]
    fn test_subtraction() {
        let mut vm = IfaVM::new();
        
        let mut bc = Bytecode::new("test");
        bc.code = vec![
            OpCode::PushInt as u8,
            0, 0, 0, 0, 0, 0, 0, 10,
            OpCode::PushInt as u8,
            0, 0, 0, 0, 0, 0, 0, 4,
            OpCode::Sub as u8,
            OpCode::Halt as u8,
        ];
        
        let result = vm.execute(&bc).unwrap();
        assert_eq!(result, IfaValue::Int(6));
    }
    
    #[test]
    fn test_multiplication() {
        let mut vm = IfaVM::new();
        
        let mut bc = Bytecode::new("test");
        bc.code = vec![
            OpCode::PushInt as u8,
            0, 0, 0, 0, 0, 0, 0, 6,
            OpCode::PushInt as u8,
            0, 0, 0, 0, 0, 0, 0, 7,
            OpCode::Mul as u8,
            OpCode::Halt as u8,
        ];
        
        let result = vm.execute(&bc).unwrap();
        assert_eq!(result, IfaValue::Int(42));
    }
    
    #[test]
    fn test_stack_operations() {
        let mut vm = IfaVM::new();
        
        vm.push(IfaValue::Int(1)).unwrap();
        vm.push(IfaValue::Int(2)).unwrap();
        vm.push(IfaValue::Int(3)).unwrap();
        
        assert_eq!(vm.pop().unwrap(), IfaValue::Int(3));
        assert_eq!(vm.pop().unwrap(), IfaValue::Int(2));
        assert_eq!(vm.pop().unwrap(), IfaValue::Int(1));
        assert!(vm.pop().is_err()); // Stack underflow
    }
    
    #[test]
    fn test_comparison_ops() {
        let mut vm = IfaVM::new();
        
        let mut bc = Bytecode::new("test");
        bc.code = vec![
            OpCode::PushInt as u8,
            0, 0, 0, 0, 0, 0, 0, 5,
            OpCode::PushInt as u8,
            0, 0, 0, 0, 0, 0, 0, 10,
            OpCode::Lt as u8, // 5 < 10
            OpCode::Halt as u8,
        ];
        
        let result = vm.execute(&bc).unwrap();
        assert_eq!(result, IfaValue::Bool(true));
    }
    
    #[test]
    fn test_boolean_ops() {
        let mut vm = IfaVM::new();
        
        let mut bc = Bytecode::new("test");
        bc.code = vec![
            OpCode::PushTrue as u8,
            OpCode::PushFalse as u8,
            OpCode::Or as u8, // true || false
            OpCode::Halt as u8,
        ];
        
        let result = vm.execute(&bc).unwrap();
        assert_eq!(result, IfaValue::Bool(true));
    }
}

// =============================================================================
// ERROR TESTS
// =============================================================================

mod error_tests {
    use super::*;
    use crate::error::IfaError;
    
    #[test]
    fn test_error_proverbs() {
        let div_zero = IfaError::DivisionByZero("test".to_string());
        assert!(!div_zero.proverb().is_empty());
        
        let type_err = IfaError::TypeError {
            expected: "Int".to_string(),
            got: "Str".to_string(),
        };
        assert!(!type_err.proverb().is_empty());
        
        let stack_err = IfaError::StackUnderflow;
        assert!(!stack_err.proverb().is_empty());
    }
}
