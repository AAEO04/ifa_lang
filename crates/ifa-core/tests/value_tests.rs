//! Comprehensive tests for IfaValue type system

use ifa_core::*;
use common::fixtures::*;
use common::helpers::*;

mod arithmetic_tests {
    use super::*;

    #[test]
    fn test_integer_addition() {
        let test_cases = math_test_data::addition_test_cases();
        for (a, b, expected) in test_cases {
            let result = a + b;
            assert_ifa_value_eq(&result, &expected);
        }
    }

    #[test]
    fn test_integer_subtraction() {
        let test_cases = math_test_data::subtraction_test_cases();
        for (a, b, expected) in test_cases {
            let result = a - b;
            assert_ifa_value_eq(&result, &expected);
        }
    }

    #[test]
    fn test_integer_multiplication() {
        let test_cases = math_test_data::multiplication_test_cases();
        for (a, b, expected) in test_cases {
            let result = a * b;
            assert_ifa_value_eq(&result, &expected);
        }
    }

    #[test]
    fn test_integer_division() {
        let test_cases = math_test_data::division_test_cases();
        for (a, b, expected) in test_cases {
            let result = a / b;
            assert_ifa_value_eq(&result, &expected);
        }
    }

    #[test]
    fn test_division_by_zero_errors() {
        let zero_cases = error_test_data::division_by_zero_cases();
        for divisor in zero_cases {
            let dividend = IfaValue::Int(10);
            let result = dividend / divisor.clone();
            assert_ifa_error(result, "division by zero");
        }
    }

    #[test]
    fn test_modulo_operation() {
        assert_eq!(IfaValue::Int(10) % IfaValue::Int(3), IfaValue::Int(1));
        assert_eq!(IfaValue::Int(10) % IfaValue::Int(5), IfaValue::Int(0));
        assert_eq!(IfaValue::Int(-10) % IfaValue::Int(3), IfaValue::Int(-1));
    }

    #[test]
    fn test_negation() {
        assert_eq!(-IfaValue::Int(5), IfaValue::Int(-5));
        assert_eq!(-IfaValue::Int(-5), IfaValue::Int(5));
        assert_eq!(-IfaValue::Float(3.14), IfaValue::Float(-3.14));
    }

    #[test]
    fn test_type_promotion() {
        // Int + Float should promote to Float
        let int_val = IfaValue::Int(5);
        let float_val = IfaValue::Float(2.5);
        let result = int_val + float_val;
        assert!(matches!(result, IfaValue::Float(7.5)));
    }

    #[test]
    fn test_invalid_arithmetic_operations() {
        let invalid_cases = error_test_data::invalid_operation_cases();
        for (a, b) in invalid_cases {
            let result = a + b.clone();
            assert_ifa_error(result, "type mismatch");
        }
    }

    #[test]
    fn test_arithmetic_performance() {
        benchmark_test!(arithmetic_performance, 10000, {
            let a = IfaValue::Int(42);
            let b = IfaValue::Int(24);
            let _ = a + b;
        });
    }
}

mod comparison_tests {
    use super::*;

    #[test]
    fn test_equality() {
        assert_eq!(IfaValue::Int(42), IfaValue::Int(42));
        assert_eq!(IfaValue::Str("hello".to_string()), IfaValue::Str("hello".to_string()));
        assert_eq!(IfaValue::Bool(true), IfaValue::Bool(true));
        assert_eq!(IfaValue::Null, IfaValue::Null);
    }

    #[test]
    fn test_inequality() {
        assert_ne!(IfaValue::Int(42), IfaValue::Int(24));
        assert_ne!(IfaValue::Str("hello".to_string()), IfaValue::Str("world".to_string()));
        assert_ne!(IfaValue::Bool(true), IfaValue::Bool(false));
    }

    #[test]
    fn test_less_than() {
        assert!(IfaValue::Int(5) < IfaValue::Int(10));
        assert!(IfaValue::Float(3.14) < IfaValue::Float(6.28));
        assert!(IfaValue::Str("apple".to_string()) < IfaValue::Str("banana".to_string()));
    }

    #[test]
    fn test_greater_than() {
        assert!(IfaValue::Int(10) > IfaValue::Int(5));
        assert!(IfaValue::Float(6.28) > IfaValue::Float(3.14));
        assert!(IfaValue::Str("banana".to_string()) > IfaValue::Str("apple".to_string()));
    }

    #[test]
    fn test_invalid_comparisons() {
        let result = IfaValue::Int(5) < IfaValue::Str("hello".to_string());
        assert_ifa_error(result, "cannot compare");
    }
}

mod string_tests {
    use super::*;

    #[test]
    fn test_string_concatenation() {
        let test_cases = string_test_data::concatenation_test_cases();
        for (a, b, expected) in test_cases {
            let result = IfaValue::Str(a) + IfaValue::Str(b);
            assert_ifa_value_eq(&result, &IfaValue::Str(expected));
        }
    }

    #[test]
    fn test_string_length() {
        assert_eq!(IfaValue::Str("hello".to_string()).len(), 5);
        assert_eq!(IfaValue::Str("".to_string()).len(), 0);
        assert_eq!(IfaValue::Str("🔥".to_string()).len(), 1); // Counting graphemes
    }

    #[test]
    fn test_string_slicing() {
        let s = IfaValue::Str("hello world".to_string());
        let slice = s.slice(0, 5).unwrap();
        assert_eq!(slice, IfaValue::Str("hello".to_string()));
    }

    #[test]
    fn test_string_indexing() {
        let s = IfaValue::Str("hello".to_string());
        let char = s.get(&IfaValue::Int(1)).unwrap();
        assert_eq!(char, IfaValue::Str("e".to_string()));
    }

    #[test]
    fn test_string_negative_indexing() {
        let s = IfaValue::Str("hello".to_string());
        let char = s.get(&IfaValue::Int(-1)).unwrap();
        assert_eq!(char, IfaValue::Str("o".to_string()));
    }

    #[test]
    fn test_string_index_out_of_bounds() {
        let s = IfaValue::Str("hello".to_string());
        let result = s.get(&IfaValue::Int(10));
        assert_ifa_error(result, "index out of bounds");
    }
}

mod list_tests {
    use super::*;

    #[test]
    fn test_list_creation() {
        let list = IfaValue::List(vec![
            IfaValue::Int(1),
            IfaValue::Int(2),
            IfaValue::Int(3),
        ]);
        assert_eq!(list.len(), 3);
    }

    #[test]
    fn test_list_indexing() {
        let list = IfaValue::List(vec![
            IfaValue::Int(10),
            IfaValue::Int(20),
            IfaValue::Int(30),
        ]);
        
        assert_eq!(list.get(&IfaValue::Int(0)).unwrap(), IfaValue::Int(10));
        assert_eq!(list.get(&IfaValue::Int(1)).unwrap(), IfaValue::Int(20));
        assert_eq!(list.get(&IfaValue::Int(-1)).unwrap(), IfaValue::Int(30));
    }

    #[test]
    fn test_list_slicing() {
        let list = IfaValue::List(vec![
            IfaValue::Int(1),
            IfaValue::Int(2),
            IfaValue::Int(3),
            IfaValue::Int(4),
            IfaValue::Int(5),
        ]);
        
        let slice = list.slice(1, 4).unwrap();
        assert_eq!(slice, IfaValue::List(vec![
            IfaValue::Int(2),
            IfaValue::Int(3),
            IfaValue::Int(4),
        ]));
    }

    #[test]
    fn test_list_push() {
        let mut list = IfaValue::List(vec![IfaValue::Int(1)]);
        list.push(IfaValue::Int(2)).unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list.get(&IfaValue::Int(1)).unwrap(), IfaValue::Int(2));
    }

    #[test]
    fn test_list_pop() {
        let mut list = IfaValue::List(vec![
            IfaValue::Int(1),
            IfaValue::Int(2),
        ]);
        
        let popped = list.pop().unwrap();
        assert_eq!(popped, IfaValue::Int(2));
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn test_list_pop_empty() {
        let mut list = IfaValue::List(vec![]);
        let result = list.pop();
        assert_ifa_error(result, "pop from empty list");
    }

    #[test]
    fn test_list_set() {
        let mut list = IfaValue::List(vec![IfaValue::Int(1), IfaValue::Int(2)]);
        list.set(&IfaValue::Int(0), IfaValue::Int(10)).unwrap();
        assert_eq!(list.get(&IfaValue::Int(0)).unwrap(), IfaValue::Int(10));
    }
}

mod map_tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_map_creation() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), IfaValue::Str("value".to_string()));
        let ifa_map = IfaValue::Map(map);
        
        assert_eq!(ifa_map.len(), 1);
    }

    #[test]
    fn test_map_access() {
        let mut map = HashMap::new();
        map.insert("name".to_string(), IfaValue::Str("Ifá".to_string()));
        let ifa_map = IfaValue::Map(map);
        
        let value = ifa_map.get(&IfaValue::Str("name".to_string())).unwrap();
        assert_eq!(value, IfaValue::Str("Ifá".to_string()));
    }

    #[test]
    fn test_map_missing_key() {
        let map = IfaValue::Map(HashMap::new());
        let result = map.get(&IfaValue::Str("missing".to_string()));
        assert_ifa_error(result, "key not found");
    }

    #[test]
    fn test_map_set() {
        let mut map = HashMap::new();
        map.insert("old".to_string(), IfaValue::Str("value".to_string()));
        let mut ifa_map = IfaValue::Map(map);
        
        ifa_map.set(&IfaValue::Str("new".to_string()), IfaValue::Str("value".to_string())).unwrap();
        assert_eq!(ifa_map.len(), 2);
    }
}

mod boolean_tests {
    use super::*;

    #[test]
    fn test_boolean_negation() {
        assert_eq!(!IfaValue::Bool(true), IfaValue::Bool(false));
        assert_eq!(!IfaValue::Bool(false), IfaValue::Bool(true));
    }

    #[test]
    fn test_truthy_values() {
        assert!(IfaValue::Int(1).is_truthy());
        assert!(IfaValue::Int(-1).is_truthy());
        assert!(IfaValue::Float(1.0).is_truthy());
        assert!(IfaValue::Float(-1.0).is_truthy());
        assert!(IfaValue::Str("hello".to_string()).is_truthy());
        assert!(IfaValue::Bool(true).is_truthy());
        
        assert!(!IfaValue::Int(0).is_truthy());
        assert!(!IfaValue::Float(0.0).is_truthy());
        assert!(!IfaValue::Str("".to_string()).is_truthy());
        assert!(!IfaValue::Bool(false).is_truthy());
        assert!(!IfaValue::Null.is_truthy());
    }

    #[test]
    fn test_empty_values() {
        assert!(IfaValue::List(vec![]).is_empty());
        assert!(IfaValue::Map(HashMap::new()).is_empty());
        assert!(IfaValue::Str("".to_string()).is_empty());
        
        assert!(!IfaValue::List(vec![IfaValue::Int(1)]).is_empty());
        assert!(!IfaValue::Str("hello".to_string()).is_empty());
    }
}

mod conversion_tests {
    use super::*;

    #[test]
    fn test_from_conversions() {
        assert_eq!(IfaValue::from(42i64), IfaValue::Int(42));
        assert_eq!(IfaValue::from(3.14f64), IfaValue::Float(3.14));
        assert_eq!(IfaValue::from("hello"), IfaValue::Str("hello".to_string()));
        assert_eq!(IfaValue::from(true), IfaValue::Bool(true));
        assert_eq!(IfaValue::from(()), IfaValue::Null);
    }

    #[test]
    fn test_from_vec() {
        let vec = vec![1, 2, 3];
        let ifa_vec = IfaValue::from(vec);
        assert_eq!(ifa_vec.len(), 3);
    }

    #[test]
    fn test_type_name() {
        assert_eq!(IfaValue::Int(42).type_name(), "Int");
        assert_eq!(IfaValue::Float(3.14).type_name(), "Float");
        assert_eq!(IfaValue::Str("hello".to_string()).type_name(), "Str");
        assert_eq!(IfaValue::Bool(true).type_name(), "Bool");
        assert_eq!(IfaValue::Null.type_name(), "Null");
        assert_eq!(IfaValue::List(vec![]).type_name(), "List");
        assert_eq!(IfaValue::Map(HashMap::new()).type_name(), "Map");
    }
}

mod edge_case_tests {
    use super::*;

    #[test]
    fn test_large_integers() {
        let large = IfaValue::Int(i64::MAX);
        let small = IfaValue::Int(1);
        let result = large + small;
        assert_ifa_error(result, "overflow");
    }

    #[test]
    fn test_min_integers() {
        let min = IfaValue::Int(i64::MIN);
        let neg_one = IfaValue::Int(-1);
        let result = min + neg_one;
        assert_ifa_error(result, "overflow");
    }

    #[test]
    fn test_infinity_floats() {
        let inf = IfaValue::Float(f64::INFINITY);
        let normal = IfaValue::Float(1.0);
        let result = inf + normal;
        assert!(matches!(result, IfaValue::Float(f) if f.is_infinite()));
    }

    #[test]
    fn test_nan_floats() {
        let nan = IfaValue::Float(f64::NAN);
        let normal = IfaValue::Float(1.0);
        let result = nan + normal;
        assert!(matches!(result, IfaValue::Float(f) if f.is_nan()));
    }

    #[test]
    fn test_unicode_strings() {
        let unicode = IfaValue::Str("🔥🌟✨".to_string());
        assert_eq!(unicode.len(), 3);
        
        let emoji = unicode.get(&IfaValue::Int(0)).unwrap();
        assert_eq!(emoji, IfaValue::Str("🔥".to_string()));
    }

    #[test]
    fn test_nested_structures() {
        let inner_list = IfaValue::List(vec![
            IfaValue::Int(1),
            IfaValue::Int(2),
        ]);
        
        let outer_map = IfaValue::Map({
            let mut map = HashMap::new();
            map.insert("numbers".to_string(), inner_list);
            map
        });
        
        let numbers = outer_map.get(&IfaValue::Str("numbers".to_string())).unwrap();
        assert_eq!(numbers.len(), 2);
    }

    #[test]
    fn test_deep_nesting() {
        // Create a deeply nested structure
        let mut current = IfaValue::Int(0);
        
        for _ in 0..100 {
            let mut map = HashMap::new();
            map.insert("value".to_string(), current);
            current = IfaValue::Map(map);
        }
        
        // Should be able to access the nested value
        let mut current_ref = &current;
        for _ in 0..100 {
            current_ref = current_ref.get(&IfaValue::Str("value".to_string())).unwrap();
        }
        
        assert_eq!(current_ref, &IfaValue::Int(0));
    }
}

mod serialization_tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_json_serialization() {
        let value = IfaValue::Int(42);
        let json = serde_json::to_string(&value).unwrap();
        let deserialized: IfaValue = serde_json::from_str(&json).unwrap();
        assert_eq!(value, deserialized);
    }

    #[test]
    fn test_json_serialization_complex() {
        let complex = IfaValue::List(vec![
            IfaValue::Int(1),
            IfaValue::Str("hello".to_string()),
            IfaValue::Bool(true),
        ]);
        
        let json = serde_json::to_string(&complex).unwrap();
        let deserialized: IfaValue = serde_json::from_str(&json).unwrap();
        assert_eq!(complex, deserialized);
    }

    #[test]
    fn test_json_serialization_map() {
        let mut map = HashMap::new();
        map.insert("key".to_string(), IfaValue::Int(42));
        let ifa_map = IfaValue::Map(map);
        
        let json = serde_json::to_string(&ifa_map).unwrap();
        let deserialized: IfaValue = serde_json::from_str(&json).unwrap();
        assert_eq!(ifa_map, deserialized);
    }
}
