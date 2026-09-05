use super::*;
use ifa_types::ast::*;

/// Check for unused variables
pub(crate) fn check_unused_vars(ctx: &LintContext, baba: &mut Babalawo, file: &str) {
    for (var, span) in &ctx.defined_vars {
        if !ctx.used_vars.contains(var) && !var.starts_with('_') {
            baba.add_with_context(
                Severity::Warning,
                "UNUSED_VARIABLE",
                &format!("Variable '{}' is defined but never used", var),
                file,
                span.clone(),
                var,
            );
        }
    }
}

/// Check for unclosed resources
pub(crate) fn check_unclosed_resources(ctx: &LintContext, baba: &mut Babalawo, file: &str) {
    for (resource, (line, col)) in &ctx.open_resources {
        baba.warning(
            "UNCLOSED_RESOURCE",
            &format!("Resource '{}' opened but never closed", resource),
            file,
            *line,
            *col,
        );
    }
}
