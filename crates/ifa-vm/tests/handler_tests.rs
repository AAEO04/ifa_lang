//! # Handler Tests
//!
//! Tests for Odù domain handlers using interpreter-level testing.
//! Uses ayanmo (variable declaration) statements to capture results.

use ifa_std::vm_registry::StdRegistry;
use ifa_vm::{IfaVM, IfaValue};

/// Helper to run Ifá code and get environment value
fn run_and_get(code: &str, var: &str) -> Result<IfaValue, String> {
    let bytecode = ifa_compiler::compile(code).map_err(|e| e.to_string())?;
    let mut registry = StdRegistry::new();
    let mut caps = ifa_types::capability::CapabilitySet::new();
    caps.grant(ifa_types::capability::Ofun::Crypto);
    caps.grant(ifa_types::capability::Ofun::Random);
    caps.grant(ifa_types::capability::Ofun::Time);
    caps.grant(ifa_types::capability::Ofun::Stdio);
    caps.grant(ifa_types::capability::Ofun::ReadFiles {
        root: std::path::PathBuf::from("."),
    });
    caps.grant(ifa_types::capability::Ofun::WriteFiles {
        root: std::path::PathBuf::from("."),
    });
    registry.set_capabilities(caps);

    let mut vm = IfaVM::new().with_registry(Box::new(registry));
    vm.execute(&bytecode).map_err(|e| e.to_string())?;
    vm.get_global(var)
        .cloned()
        .ok_or_else(|| format!("Variable {} not found", var))
}

// =============================================================================
// Cpu (Parallelism) Handler Tests
// =============================================================================

#[test]
fn test_cpu_threads() {
    let result = run_and_get("ayanmo t = Cpu.threads();", "t").unwrap();
    if let IfaValue::Int(n) = result {
        assert!(n >= 1, "Should have at least 1 thread");
    } else {
        panic!("Expected Int, got {:?}", result);
    }
}

#[test]
fn test_cpu_sum() {
    let code = r#"
ayanmo view = Cpu.alloc_buffer(5);
Cpu.write_buffer(view, [1, 2, 3, 4, 5]);
ayanmo s = Cpu.par_reduce(view, "sum");
"#;
    let result = run_and_get(code, "s").unwrap();
    if let IfaValue::Float(f) = result {
        assert_eq!(f, 15.0);
    } else {
        panic!("Expected Float, got {:?}", result);
    }
}

#[test]
fn test_cpu_product() {
    let code = r#"
ayanmo view = Cpu.alloc_buffer(3);
Cpu.write_buffer(view, [2, 3, 4]);
ayanmo p = Cpu.par_reduce(view, "product");
"#;
    let result = run_and_get(code, "p").unwrap();
    if let IfaValue::Float(f) = result {
        assert_eq!(f, 24.0);
    } else {
        panic!("Expected Float, got {:?}", result);
    }
}

#[test]
fn test_cpu_min() {
    let code = r#"
ayanmo view = Cpu.alloc_buffer(5);
Cpu.write_buffer(view, [5, 2, 8, 1, 9]);
ayanmo m = Cpu.par_reduce(view, "min");
"#;
    let result = run_and_get(code, "m").unwrap();
    if let IfaValue::Float(f) = result {
        assert_eq!(f, 1.0);
    } else {
        panic!("Expected Float, got {:?}", result);
    }
}

#[test]
fn test_cpu_max() {
    let code = r#"
ayanmo view = Cpu.alloc_buffer(5);
Cpu.write_buffer(view, [5, 2, 8, 1, 9]);
ayanmo m = Cpu.par_reduce(view, "max");
"#;
    let result = run_and_get(code, "m").unwrap();
    if let IfaValue::Float(f) = result {
        assert_eq!(f, 9.0);
    } else {
        panic!("Expected Float, got {:?}", result);
    }
}

// =============================================================================
// Ìrẹtẹ̀ (Crypto) Handler Tests
// =============================================================================

#[test]
fn test_irete_sha256() {
    let result = run_and_get(r#"ayanmo h = Irete.sha256_hex("hello");"#, "h").unwrap();
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
    let result = run_and_get(r#"ayanmo h = Irete.sha256_hex("hello");"#, "h").unwrap();
    if let IfaValue::Str(hash) = result {
        assert_eq!(
            hash,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824".into()
        );
    } else {
        panic!("Expected Str");
    }
}

#[test]
fn test_irete_sha256_raw() {
    let result = run_and_get(r#"ayanmo h = Irete.sha256("hello");"#, "h").unwrap();
    if let IfaValue::List(bytes) = result {
        assert_eq!(bytes.len(), 32);
        assert_eq!(bytes[0], IfaValue::int(44));
    } else {
        panic!("Expected List");
    }
}

#[test]
fn test_irete_base64_encode() {
    let result = run_and_get(r#"ayanmo e = Irete.encode_base64("hello");"#, "e").unwrap();
    if let IfaValue::Str(encoded) = result {
        assert_eq!(encoded, "aGVsbG8=".into());
    } else {
        panic!("Expected Str");
    }
}

#[test]
fn test_irete_base64_decode() {
    let result = run_and_get(r#"ayanmo d = Irete.decode_base64("aGVsbG8=");"#, "d").unwrap();
    if let IfaValue::Str(decoded) = result {
        assert_eq!(decoded, "hello".into());
    } else {
        panic!("Expected Str");
    }
}

#[test]
fn test_owonrin_uuid() {
    let result = run_and_get("ayanmo u = Owonrin.uuid();", "u").unwrap();
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
    assert_eq!(
        run_and_get("ayanmo x = 1 + 2;", "x").unwrap(),
        IfaValue::Int(3)
    );
    assert_eq!(
        run_and_get("ayanmo x = 10 - 4;", "x").unwrap(),
        IfaValue::Int(6)
    );
    assert_eq!(
        run_and_get("ayanmo x = 3 * 4;", "x").unwrap(),
        IfaValue::Int(12)
    );
    assert_eq!(
        run_and_get("ayanmo x = 15 / 3;", "x").unwrap(),
        IfaValue::Int(5)
    );
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
    assert_eq!(
        run_and_get("ayanmo x = 5 > 3;", "x").unwrap(),
        IfaValue::Bool(true)
    );
    assert_eq!(
        run_and_get("ayanmo x = 2 < 1;", "x").unwrap(),
        IfaValue::Bool(false)
    );
    assert_eq!(
        run_and_get("ayanmo x = 4 == 4;", "x").unwrap(),
        IfaValue::Bool(true)
    );
}

#[test]
fn test_string_concat() {
    let result = run_and_get(r#"ayanmo s = "Hello" + " World";"#, "s").unwrap();
    assert_eq!(result, IfaValue::Str("Hello World".into()));
}
