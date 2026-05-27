//! # Rust Transpiler Module
//!
//! Transpiles Ifá-Lang AST to Rust source code for native compilation.
//!
//! ## Structure
//! - `constants.rs` - Odù domain and method name constants
//! - `core.rs` - Main transpiler struct and entry point
//! - `statements.rs` - Statement transpilation
//! - `expressions.rs` - Expression transpilation
//! - `domains.rs` - Odù domain call transpilation

pub mod constants;
pub mod core;
mod domains;
mod expressions;
mod statements;

pub use self::core::{RustTranspiler, transpile_to_rust};

#[cfg(test)]
mod tests {
    use super::*;
    use ifa_parser::parse;

    #[test]
    fn test_simple_transpile() {
        let source = r#"
        ayanmo x = 10;
        Irosu.fo(x);
        "#;

        let program = parse(source).unwrap();
        let rust_code = transpile_to_rust(&program);

        assert!(rust_code.contains("let mut x"));
        assert!(rust_code.contains("fn main()"));
    }

    #[test]
    fn test_file_io_transpile_exposes_errors() {
        let source = r#"
        ayanmo contents = Odi.ka("missing.txt");
        Odi.ko("out.txt", "hello");
        "#;

        let program = parse(source).unwrap();
        let rust_code = transpile_to_rust(&program);

        assert!(rust_code.contains("\"IoError\""));
        assert!(!rust_code.contains("std::fs::write(&p, &c).ok()"));
    }

    #[test]
    fn test_exponentiation_transpile() {
        let source = r#"
        ayanmo a = 2;
        ayanmo b = 3;
        ayanmo x = a ** b;
        "#;
        let program = parse(source).unwrap();
        let rust_code = transpile_to_rust(&program);
        println!("RUST CODE EXP:\n{}", rust_code);
        assert!(rust_code.contains(".pow(&"));
    }

    #[test]
    fn test_literal_folding_transpile() {
        let source = r#"
        ayanmo x = 1 + 2;
        ayanmo y = 3.5 * 2.0;
        "#;
        let program = parse(source).unwrap();
        let rust_code = transpile_to_rust(&program);
        // Literal folding wraps raw arithmetic in IfaValue at the boundary
        assert!(rust_code.contains("IfaValue::Int(1i64 + 2i64)"));
        assert!(rust_code.contains("IfaValue::Float(3.5f64 * 2f64)"));
    }
}
