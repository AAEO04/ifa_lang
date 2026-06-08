//! Target-specific incompatibility checks

use crate::diagnose::Babalawo;
use ifa_types::ast::{Expression, Span, Statement};
use ifa_types::target::Target;

pub fn check_statement_target(stmt: &Statement, target: &Target, baba: &mut Babalawo, file: &str) {
    match stmt {
        Statement::Import { span, .. } => {
            if !target.allows_imports() {
                baba.error(
                    "EMBEDDED_FEATURE",
                    &format!("Imports are not supported on target '{:?}'", target),
                    file,
                    span.line,
                    span.column,
                );
            }
        }
        Statement::OduDef { span, .. } => {
            if !target.allows_odu_domains() {
                baba.error(
                    "EMBEDDED_FEATURE",
                    &format!("Odù domains are not supported on target '{:?}'", target),
                    file,
                    span.line,
                    span.column,
                );
            }
        }
        _ => {}
    }
}

pub fn check_expression_target(
    expr: &Expression,
    target: &Target,
    baba: &mut Babalawo,
    file: &str,
    span: &Span,
) {
    match expr {
        Expression::String(_) | Expression::InterpolatedString { .. } => {
            if !target.allows_strings() {
                baba.error(
                    "EMBEDDED_FEATURE",
                    &format!("Strings are not supported on target '{:?}'", target),
                    file,
                    span.line,
                    span.column,
                );
            }
        }
        Expression::Lambda { .. } => {
            if !target.allows_closures() {
                baba.error(
                    "EMBEDDED_FEATURE",
                    &format!("Closures are not supported on target '{:?}'", target),
                    file,
                    span.line,
                    span.column,
                );
            }
        }
        Expression::List(_) | Expression::Map(_) | Expression::Set(_) => {
            if !target.allows_collections() {
                baba.error(
                    "EMBEDDED_FEATURE",
                    &format!("Collections are not supported on target '{:?}'", target),
                    file,
                    span.line,
                    span.column,
                );
            }
        }
        Expression::MoveExpr(_) => {
            if target.is_embedded() {
                baba.error(
                    "EMBEDDED_FEATURE",
                    &format!(
                        "Move semantics and 'yanda' keywords are not supported on target '{:?}'",
                        target
                    ),
                    file,
                    span.line,
                    span.column,
                );
            }
        }
        Expression::MethodCall { method, .. } => {
            if method == "yanda" && target.is_embedded() {
                baba.error(
                    "EMBEDDED_FEATURE",
                    &format!("The '.yanda()' dynamic ownership escape hatch is not supported on target '{:?}'", target),
                    file,
                    span.line,
                    span.column,
                );
            }
        }
        _ => {}
    }
}
