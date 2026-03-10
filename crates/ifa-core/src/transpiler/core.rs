//! # Core Transpiler
//!
//! Main transpiler struct and entry point.

use crate::ast::*;

/// Transpile an Ifá program to Rust source code
pub fn transpile_to_rust(program: &Program) -> String {
    let mut transpiler = RustTranspiler::new();
    transpiler.transpile_program(program)
}

/// Rust code transpiler state
pub struct RustTranspiler {
    pub(crate) indent: usize,
    pub has_async: bool,
    pub needs_tokio: bool,
    pub needs_reqwest: bool,
    pub needs_rand: bool,
}

impl Default for RustTranspiler {
    fn default() -> Self {
        Self::new()
    }
}

impl RustTranspiler {
    pub fn new() -> Self {
        Self {
            indent: 0,
            has_async: false,
            needs_tokio: false,
            needs_reqwest: false,
            needs_rand: false,
        }
    }

    /// Mangle identifiers that conflict with Rust keywords
    pub(crate) fn mangle_identifier(&self, name: &str) -> String {
        const RUST_KEYWORDS: &[&str] = &[
            "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn",
            "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
            "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
            "unsafe", "use", "where", "while", "async", "await", "dyn", "abstract", "become",
            "box", "do", "final", "macro", "override", "priv", "typeof", "unsized", "virtual",
            "yield", "try",
        ];

        if RUST_KEYWORDS.contains(&name) {
            format!("{}_ifa", name)
        } else {
            name.to_string()
        }
    }

    /// Get current indentation string
    pub(crate) fn indent_str(&self) -> String {
        "    ".repeat(self.indent)
    }
}
