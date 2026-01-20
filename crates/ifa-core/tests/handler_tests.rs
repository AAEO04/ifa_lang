//! # Handler Tests
//!
//! Tests for Odù domain handlers using interpreter-level testing.
//! Uses ayanmo (variable declaration) statements to capture results.

use ifa_core::{Interpreter, IfaValue, parser::parse};

/// Helper to run Ifá code and get environment value
fn run_and_get(code: &str, var: &str) -> Result<IfaValue, String> {
    let program = parse(code).map_err(|e| e.to_string())?;
    let mut interp = Interpreter::new();
    interp.execute(&program).map_err(|e| e.to_string())?;
    interp.env.get(var).ok_or_else(|| format!("Variable {} not found", var))
}

// =============================================================================
// Ọ̀sá (Concurrency) Handler Tests
// =============================================================================

#[test]
fn test_osa_threads() {
    let result = run_and_get("ayanmo t = Osa.threads();", "t").unwrap();
    if let IfaValue::Int(n) = result {
        assert!(n >= 1, "Should have at least 1 thread");
    } else {
        panic!("Expected Int, got {:?}", result);
    }
}

#[test]
fn test_osa_sum() {
    let result = run_and_get("ayanmo s = Osa.sum([1, 2, 3, 4, 5]);", "s").unwrap();
    assert_eq!(result, IfaValue::Int(15));
}

#[test]
fn test_osa_product() {
    let result = run_and_get("ayanmo p = Osa.product([2, 3, 4]);", "p").unwrap();
    assert_eq!(result, IfaValue::Int(24));
}

#[test]
fn test_osa_min() {
    let result = run_and_get("ayanmo m = Osa.min([5, 2, 8, 1, 9]);", "m").unwrap();
    assert_eq!(result, IfaValue::Int(1));
}

#[test]
fn test_osa_max() {
    let result = run_and_get("ayanmo m = Osa.max([5, 2, 8, 1, 9]);", "m").unwrap();
    assert_eq!(result, IfaValue::Int(9));
}

#[test]
fn test_osa_sort() {
    let result = run_and_get("ayanmo s = Osa.sort([3, 1, 4, 1, 5]);", "s").unwrap();
    if let IfaValue::List(items) = result {
        let nums: Vec<i64> = items.iter().filter_map(|v| {
            if let IfaValue::Int(n) = v { Some(*n) } else { None }
        }).collect();
        assert_eq!(nums, vec![1, 1, 3, 4, 5]);
    } else {
        panic!("Expected List");
    }
}

// =============================================================================
// Ìrẹtẹ̀ (Crypto) Handler Tests
// =============================================================================

#[test]
fn test_irete_sha256() {
    let result = run_and_get(r#"ayanmo h = Irete.sha256("hello");"#, "h").unwrap();
    if let IfaValue::Str(hash) = result {
        assert_eq!(hash.len(), 64, "SHA256 hex should be 64 chars");
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    } else {
        panic!("Expected Str");
    }
}

#[test]
fn test_irete_sha256_known() {
    // SHA256("hello") known value
    let result = run_and_get(r#"ayanmo h = Irete.sha256("hello");"#, "h").unwrap();
    if let IfaValue::Str(hash) = result {
        assert_eq!(hash, "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824");
    } else {
        panic!("Expected Str");
    }
}

#[test]
fn test_irete_base64_encode() {
    let result = run_and_get(r#"ayanmo e = Irete.encode_base64("hello");"#, "e").unwrap();
    if let IfaValue::Str(encoded) = result {
        assert_eq!(encoded, "aGVsbG8=");
    } else {
        panic!("Expected Str");
    }
}

#[test]
fn test_irete_base64_decode() {
    let result = run_and_get(r#"ayanmo d = Irete.decode_base64("aGVsbG8=");"#, "d").unwrap();
    if let IfaValue::Str(decoded) = result {
        assert_eq!(decoded, "hello");
    } else {
        panic!("Expected Str");
    }
}

#[test]
fn test_irete_uuid() {
    let result = run_and_get("ayanmo u = Irete.uuid();", "u").unwrap();
    if let IfaValue::Str(uuid) = result {
        assert_eq!(uuid.len(), 36, "UUID should be 36 chars with dashes");
        assert!(uuid.contains('-'));
    } else {
        panic!("Expected Str");
    }
}

#[test]
fn test_irete_random_bytes() {
    let result = run_and_get("ayanmo r = Irete.random_bytes(16);", "r").unwrap();
    if let IfaValue::Str(hex) = result {
        assert_eq!(hex.len(), 32, "16 bytes = 32 hex chars");
    } else {
        panic!("Expected Str");
    }
}

// =============================================================================
// Basic Language Tests
// =============================================================================

#[test]
fn test_arithmetic() {
    assert_eq!(run_and_get("ayanmo x = 1 + 2;", "x").unwrap(), IfaValue::Int(3));
    assert_eq!(run_and_get("ayanmo x = 10 - 4;", "x").unwrap(), IfaValue::Int(6));
    assert_eq!(run_and_get("ayanmo x = 3 * 4;", "x").unwrap(), IfaValue::Int(12));
    assert_eq!(run_and_get("ayanmo x = 15 / 3;", "x").unwrap(), IfaValue::Int(5));
}

#[test]
fn test_variable_declaration() {
    let result = run_and_get("ayanmo x = 42;", "x").unwrap();
    assert_eq!(result, IfaValue::Int(42));
}

#[test]
fn test_list_creation() {
    let result = run_and_get("ayanmo l = [1, 2, 3];", "l").unwrap();
    if let IfaValue::List(items) = result {
        assert_eq!(items.len(), 3);
    } else {
        panic!("Expected List");
    }
}

#[test]
fn test_comparison() {
    assert_eq!(run_and_get("ayanmo x = 5 > 3;", "x").unwrap(), IfaValue::Bool(true));
    assert_eq!(run_and_get("ayanmo x = 2 < 1;", "x").unwrap(), IfaValue::Bool(false));
    assert_eq!(run_and_get("ayanmo x = 4 == 4;", "x").unwrap(), IfaValue::Bool(true));
}

#[test]
fn test_string_concat() {
    let result = run_and_get(r#"ayanmo s = "Hello" + " World";"#, "s").unwrap();
    assert_eq!(result, IfaValue::Str("Hello World".to_string()));
}


