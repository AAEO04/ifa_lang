//! # Core Transpiler
//!
//! Main transpiler struct and entry point.

use crate::ast::*;
use ifa_types::domain::OduDomain;

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
    pub(crate) module_defs: Vec<String>,
    pub(crate) module_aliases: std::collections::HashSet<String>,
    pub(crate) std_modules: std::collections::HashMap<String, OduDomain>,
    pub(crate) std_named: std::collections::HashMap<String, OduDomain>,
    pub(crate) uses: Vec<String>,
    pub(crate) in_module: bool,
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
            module_defs: Vec::new(),
            module_aliases: std::collections::HashSet::new(),
            std_modules: std::collections::HashMap::new(),
            std_named: std::collections::HashMap::new(),
            uses: Vec::new(),
            in_module: false,
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

    pub(crate) fn visibility_prefix(&self, visibility: Visibility) -> &'static str {
        if self.in_module && visibility == Visibility::Public {
            "pub "
        } else {
            ""
        }
    }
}

pub(crate) fn std_domain_from_name(name: &str) -> Option<OduDomain> {
    match name.to_lowercase().as_str() {
        "ogbe" => Some(OduDomain::Ogbe),
        "oyeku" => Some(OduDomain::Oyeku),
        "iwori" => Some(OduDomain::Iwori),
        "odi" => Some(OduDomain::Odi),
        "irosu" => Some(OduDomain::Irosu),
        "owonrin" => Some(OduDomain::Owonrin),
        "obara" => Some(OduDomain::Obara),
        "okanran" => Some(OduDomain::Okanran),
        "ogunda" => Some(OduDomain::Ogunda),
        "osa" => Some(OduDomain::Osa),
        "ika" => Some(OduDomain::Ika),
        "oturupon" => Some(OduDomain::Oturupon),
        "otura" => Some(OduDomain::Otura),
        "irete" => Some(OduDomain::Irete),
        "ose" => Some(OduDomain::Ose),
        "ofun" => Some(OduDomain::Ofun),
        "coop" => Some(OduDomain::Coop),
        "opele" => Some(OduDomain::Opele),
        "cpu" => Some(OduDomain::Cpu),
        "gpu" => Some(OduDomain::Gpu),
        "storage" => Some(OduDomain::Storage),
        "backend" => Some(OduDomain::Backend),
        "frontend" => Some(OduDomain::Frontend),
        "crypto" => Some(OduDomain::Crypto),
        "ml" => Some(OduDomain::Ml),
        "gamedev" => Some(OduDomain::GameDev),
        "iot" => Some(OduDomain::Iot),
        "ohun" => Some(OduDomain::Ohun),
        "fidio" => Some(OduDomain::Fidio),
        "sys" => Some(OduDomain::Sys),
        _ => None,
    }
}
