//! Property-based tests using proptest for ifa-core
//!
//! These tests verify that invariants hold across randomly generated inputs.

use ifa_core::IfaValue;
use proptest::prelude::*;

// Strategy for generating IfaValue
fn any_ifa_value() -> impl Strategy<Value = IfaValue> {
    prop_oneof![
        any::<i64>().prop_map(IfaValue::Int),
        any::<f64>()
            .prop_filter("finite", |x| x.is_finite())
            .prop_map(IfaValue::Float),
        ".*".prop_map(IfaValue::Str),
        any::<bool>().prop_map(IfaValue::Bool),
        Just(IfaValue::Null),
    ]
}

fn any_number() -> impl Strategy<Value = IfaValue> {
    prop_oneof![
        any::<i64>().prop_map(IfaValue::Int),
        any::<f64>()
            .prop_filter("finite", |x| x.is_finite())
            .prop_map(IfaValue::Float),
    ]
}

proptest! {
    // =========================================================================
    // ARITHMETIC PROPERTIES
    // =========================================================================

    #[test]
    fn prop_addition_commutative(a in any::<i64>(), b in any::<i64>()) {
        let va = IfaValue::Int(a);
        let vb = IfaValue::Int(b);

        // a + b == b + a
        let result1 = va.clone() + vb.clone();
        let result2 = vb + va;
        prop_assert_eq!(result1, result2);
    }

    #[test]
    fn prop_multiplication_commutative(a in any::<i64>(), b in any::<i64>()) {
        let va = IfaValue::Int(a);
        let vb = IfaValue::Int(b);

        // a * b == b * a
        let result1 = va.clone() * vb.clone();
        let result2 = vb * va;
        prop_assert_eq!(result1, result2);
    }

    #[test]
    fn prop_add_zero_identity(a in any::<i64>()) {
        let va = IfaValue::Int(a);
        let zero = IfaValue::Int(0);

        // a + 0 == a
        let result = va.clone() + zero;
        prop_assert_eq!(result, va);
    }

    #[test]
    fn prop_mul_one_identity(a in any::<i64>()) {
        let va = IfaValue::Int(a);
        let one = IfaValue::Int(1);

        // a * 1 == a
        let result = va.clone() * one;
        prop_assert_eq!(result, va);
    }

    #[test]
    fn prop_double_negation(a in any::<i64>()) {
        let va = IfaValue::Int(a);

        // --a == a
        let result = -(-va.clone());
        prop_assert_eq!(result, va);
    }

    // =========================================================================
    // STRING PROPERTIES
    // =========================================================================

    #[test]
    fn prop_string_concat_associative(
        a in "\\PC*",
        b in "\\PC*",
        c in "\\PC*"
    ) {
        let va = IfaValue::Str(a.clone());
        let vb = IfaValue::Str(b.clone());
        let vc = IfaValue::Str(c.clone());

        // (a + b) + c == a + (b + c)
        let result1 = (va.clone() + vb.clone()) + vc.clone();
        let result2 = va + (vb + vc);
        prop_assert_eq!(result1, result2);
    }

    #[test]
    fn prop_string_repeat_length(s in "\\PC{0,10}", n in 0..10i64) {
        let vs = IfaValue::Str(s.clone());
        let vn = IfaValue::Int(n);

        if let IfaValue::Str(result) = vs * vn {
            // Length should be original * n
            prop_assert_eq!(result.len(), s.len() * (n as usize));
        }
    }

    // =========================================================================
    // LIST PROPERTIES
    // =========================================================================

    #[test]
    fn prop_list_concat_length(
        a in prop::collection::vec(any::<i64>(), 0..10),
        b in prop::collection::vec(any::<i64>(), 0..10)
    ) {
        let va = IfaValue::List(a.iter().map(|x| IfaValue::Int(*x)).collect());
        let vb = IfaValue::List(b.iter().map(|x| IfaValue::Int(*x)).collect());

        if let IfaValue::List(result) = va.clone() + vb.clone() {
            // Length should be sum of lengths
            prop_assert_eq!(result.len(), a.len() + b.len());
        }
    }

    // =========================================================================
    // TRUTHINESS PROPERTIES
    // =========================================================================

    #[test]
    fn prop_double_not(v in any_ifa_value()) {
        // !!v == v.is_truthy()
        let double_not = !(!v.clone());
        if let IfaValue::Bool(result) = double_not {
            prop_assert_eq!(result, v.is_truthy());
        }
    }

    // =========================================================================
    // INDEXING PROPERTIES
    // =========================================================================

    #[test]
    fn prop_negative_index_equivalence(
        items in prop::collection::vec(any::<i64>(), 1..10)
    ) {
        let list = IfaValue::List(items.iter().map(|x| IfaValue::Int(*x)).collect());
        let len = items.len() as i64;

        // list[-1] == list[len-1]
        let last_neg = list.get(&IfaValue::Int(-1));
        let last_pos = list.get(&IfaValue::Int(len - 1));

        if let (Ok(a), Ok(b)) = (last_neg, last_pos) {
            prop_assert_eq!(a, b);
        }
    }

    // =========================================================================
    // COMPARISON PROPERTIES
    // =========================================================================

    #[test]
    fn prop_equality_reflexive(a in any::<i64>()) {
        let va = IfaValue::Int(a);
        prop_assert_eq!(va.clone(), va);
    }

    #[test]
    fn prop_equality_symmetric(a in any::<i64>(), b in any::<i64>()) {
        let va = IfaValue::Int(a);
        let vb = IfaValue::Int(b);

        prop_assert_eq!(va == vb, vb == va);
    }

    #[test]
    fn prop_less_than_irreflexive(a in any::<i64>()) {
        let va = IfaValue::Int(a);
        prop_assert!(!(va.clone() < va));
    }
}
