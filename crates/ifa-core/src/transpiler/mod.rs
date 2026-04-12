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
mod core;
mod domains;
mod expressions;
mod statements;

pub use self::core::{RustTranspiler, transpile_to_rust};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

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
}
