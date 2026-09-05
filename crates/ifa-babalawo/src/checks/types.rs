use super::*;
use ifa_types::ast::*;

pub(crate) fn check_iwa_compliance(
    th: &TypeHint,
    value: &Expression,
    ctx: &LintContext,
    baba: &mut Babalawo,
    file: &str,
    span: &ifa_types::ast::Span,
) {
    if let TypeHint::Iwa(iwa_name) = th {
        if let Some(iwa_def) = ctx.iwa_defs.get(iwa_name).cloned() {
            let mut inner_val = value;

            match &inner_val.kind {
                ExprKind::Map(entries) => {
                    let mut map_keys = HashSet::new();
                    for (k, v) in entries {
                        let mut inner_k = k;

                        let mut inner_v = v;

                        if let ExprKind::String(k_str) = &inner_k.kind {
                            if matches!(
                                inner_v.kind,
                                ExprKind::Lambda { .. } | ExprKind::Identifier(_)
                            ) {
                                map_keys.insert(k_str.clone());
                            }
                        } else if let ExprKind::Identifier(k_id) = &inner_k.kind
                            && matches!(
                                inner_v.kind,
                                ExprKind::Lambda { .. } | ExprKind::Identifier(_)
                            )
                        {
                            map_keys.insert(k_id.clone());
                        }
                    }

                    for method in &iwa_def.methods {
                        if !map_keys.contains(&method.name) {
                            baba.error(
                                "IWA_SHAPE_MISMATCH",
                                &format!(
                                    "Map does not satisfy Iwa '{}': missing method '{}'",
                                    iwa_name, method.name
                                ),
                                file,
                                span.line,
                                span.column,
                            );
                        }
                    }
                }
                ExprKind::Identifier(var_name) => {
                    if let Some(TypeHint::Iwa(other_iwa)) = ctx.get_var_type(var_name) {
                        if other_iwa != iwa_name {
                            baba.error(
                                "IWA_TYPE_MISMATCH",
                                &format!("Cannot assign Iwa '{}' to Iwa '{}'", other_iwa, iwa_name),
                                file,
                                span.line,
                                span.column,
                            );
                        }
                    } else {
                        baba.error(
                            "IWA_SHAPE_MISMATCH",
                            &format!(
                                "Variable '{}' must have Iwa '{}' type to be assigned",
                                var_name, iwa_name
                            ),
                            file,
                            span.line,
                            span.column,
                        );
                    }
                }
                ExprKind::MethodCall { .. } | ExprKind::OduCall(_) => {
                    // Assume it returns the right Iwa for now, dynamic boundary
                }
                _ => {
                    baba.error(
                        "IWA_SHAPE_MISMATCH",
                        &format!("Iwa type '{}' must be assigned a Map literal, another Iwa, or method call", iwa_name),
                        file,
                        span.line,
                        span.column,
                    );
                }
            }
        } else {
            baba.error(
                "UNKNOWN_IWA",
                &format!("Unknown Iwa protocol: {}", iwa_name),
                file,
                span.line,
                span.column,
            );
        }
    }
}

/// Infer the type of an expression (returns None for dynamic/unknown types)
pub(crate) fn infer_expression_type(expr: &Expression, ctx: &LintContext) -> Option<TypeHint> {
    match &expr.kind {
        ExprKind::Int(_) => Some(TypeHint::Int),
        ExprKind::Float(_) => Some(TypeHint::Float),
        ExprKind::String(_) => Some(TypeHint::Str),
        ExprKind::Bool(_) => Some(TypeHint::Bool),
        ExprKind::Nil => None, // Nil is compatible with any type
        ExprKind::List(_) => Some(TypeHint::List),
        ExprKind::Map(_) => Some(TypeHint::Map),

        ExprKind::Identifier(name) => {
            // Check aliases first
            if let Some(target) = ctx.aliases.get(name) {
                return infer_expression_type(&target, ctx);
            }
            // Look up variable type in context
            ctx.get_var_type(name).cloned()
        }

        ExprKind::BinaryOp {
            left, right, op: _, ..
        } => {
            let left_type = infer_expression_type(&left, ctx)?;
            let right_type = infer_expression_type(&right, ctx)?;

            // Basic inference rules
            if types_compatible(&left_type, &right_type)
                || types_compatible(&right_type, &left_type)
            {
                // If one is float and other is int, result is float (usually)
                // If both same, result is same.
                // Simplified:
                if matches!(left_type, TypeHint::Float | TypeHint::F32 | TypeHint::F64)
                    || matches!(right_type, TypeHint::Float | TypeHint::F32 | TypeHint::F64)
                {
                    // Return the float one
                    if matches!(left_type, TypeHint::Float | TypeHint::F32 | TypeHint::F64) {
                        Some(left_type)
                    } else {
                        Some(right_type)
                    }
                } else {
                    // Assume left type dominates (e.g. i32 + i32 -> i32)
                    Some(left_type)
                }
            } else {
                None // Incompatible types in binary op
            }
        }

        _ => None, // Cannot infer type for complex expressions
    }
}

/// Check if two types are compatible for assignment
pub(crate) fn types_compatible(declared: &TypeHint, inferred: &TypeHint) -> bool {
    // Dynamic types are compatible with each other
    if matches!(declared, TypeHint::Any) {
        return true;
    }

    // Iwa types matching
    if let (TypeHint::Iwa(name1), TypeHint::Iwa(name2)) = (declared, inferred) {
        return name1 == name2;
    }

    if let (TypeHint::Iwa(_), TypeHint::Map) = (declared, inferred) {
        return true; // We do deep structural check elsewhere
    }

    // Exact match
    if declared == inferred {
        return true;
    }

    // Int/Float compatibility with sized versions
    match (declared, inferred) {
        // Dynamic Int is compatible with any integer literal
        (TypeHint::Int, TypeHint::Int) => true,
        (TypeHint::I64, TypeHint::Int) => true,
        (TypeHint::I32, TypeHint::Int) => true,
        (TypeHint::I16, TypeHint::Int) => true,
        (TypeHint::I8, TypeHint::Int) => true,

        // Dynamic Float is compatible with float literal
        (TypeHint::Float, TypeHint::Float) => true,
        (TypeHint::F64, TypeHint::Float) => true,
        (TypeHint::F32, TypeHint::Float) => true,

        // Allow Int -> Float promotion
        (TypeHint::Float, TypeHint::Int) => true,
        (TypeHint::F64, TypeHint::Int) => true,
        (TypeHint::F32, TypeHint::Int) => true,

        _ => false,
    }
}
