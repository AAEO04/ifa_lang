use crate::checks::LintContext;
use ifa_types::ast::ExprKind;
use ifa_types::ast::{Expression, InterpolatedPart, TypeHint};
use ifa_types::binary_ops::binary_op_result_type;

pub fn infer_expression_type(expr: &Expression, ctx: &LintContext) -> Option<TypeHint> {
    match &expr.kind {
        ExprKind::Int(_) => Some(TypeHint::Int),
        ExprKind::Float(_) => Some(TypeHint::Float),
        ExprKind::String(_) => Some(TypeHint::Str),
        ExprKind::Bool(_) => Some(TypeHint::Bool),
        ExprKind::Nil => None,

        ExprKind::List(items) => {
            if items.is_empty() {
                Some(TypeHint::List)
            } else {
                let inferred: Vec<Option<TypeHint>> = items
                    .iter()
                    .map(|i| infer_expression_type(i, ctx))
                    .collect();
                if inferred.iter().all(|t: &Option<TypeHint>| t.is_some()) {
                    let all_same = inferred.windows(2).all(|w| w[0] == w[1]);
                    if all_same {
                        inferred.into_iter().next().flatten()
                    } else {
                        Some(TypeHint::List)
                    }
                } else {
                    Some(TypeHint::List)
                }
            }
        }

        ExprKind::Map(_) => Some(TypeHint::Map),

        ExprKind::Identifier(name) => ctx.get_var_type(name).cloned(),

        ExprKind::BinaryOp {
            left, right, op, ..
        } => {
            let lhs = infer_expression_type(&left, ctx)?;
            let rhs = infer_expression_type(&right, ctx)?;
            binary_op_result_type(op, &lhs, &rhs)
        }

        ExprKind::UnaryOp { op: _, expr } => match infer_expression_type(&expr, ctx)? {
            TypeHint::Int => Some(TypeHint::Int),
            TypeHint::Float => Some(TypeHint::Float),
            _ => None,
        },

        ExprKind::OduCall(call) => {
            if call.method == "iru" || call.method == "typeof" {
                Some(TypeHint::Str)
            } else if call.method == "wa"
                || call.method == "exists"
                || call.method == "ni"
                || call.method == "has"
                || call.method == "contains"
            {
                Some(TypeHint::Bool)
            } else if call.method == "gigun"
                || call.method == "len"
                || call.method == "iwọn"
                || call.method == "size"
            {
                Some(TypeHint::Int)
            } else if call.method == "gba"
                || call.method == "get"
                || call.method == "ka"
                || call.method == "read"
            {
                Some(TypeHint::Any)
            } else if call.method == "parse_int" {
                Some(TypeHint::Int)
            } else if call.method == "parse_float" || call.method == "ida" || call.method == "float"
            {
                Some(TypeHint::Float)
            } else if call.method == "boolean" || call.method == "bool" {
                Some(TypeHint::Bool)
            } else if call.method == "bayi"
                || call.method == "now"
                || call.method == "timestamp"
                || call.method == "bayi_ms"
                || call.method == "now_ms"
                || call.method == "nọmba"
                || call.method == "random"
                || call.method == "rand"
            {
                Some(TypeHint::Int)
            } else if call.method.ends_with("_json") || call.method.ends_with("Json") {
                Some(TypeHint::Map)
            } else if call.method == "uuid" || call.method == "id_alailẹgbẹ" {
                Some(TypeHint::Str)
            } else {
                None
            }
        }

        ExprKind::MethodCall {
            object,
            method: _,
            args: _,
            is_optional: _,
        } => {
            let _obj_type = infer_expression_type(object, ctx);
            Some(TypeHint::Any)
        }

        ExprKind::Get {
            object,
            name: _,
            is_optional: _,
        } => {
            let obj_type = infer_expression_type(object, ctx);
            if obj_type == Some(TypeHint::Map) {
                Some(TypeHint::Any)
            } else {
                None
            }
        }

        ExprKind::Call { name, args: _ } => {
            if let Some(ret_type) = ctx.var_types.get(name)
                && let ifa_types::ast::TypeHint::Function { ret, .. } = ret_type
            {
                return Some(*ret.clone());
            }

            if let Some(var_type) = ctx.get_var_type(name) {
                if *var_type == TypeHint::Any || matches!(var_type, TypeHint::Custom(_)) {
                    Some(TypeHint::Any)
                } else {
                    None
                }
            } else {
                None
            }
        }

        ExprKind::Await(inner) => infer_expression_type(inner, ctx),

        ExprKind::Index {
            object,
            index: _,
            is_optional: _,
        } => {
            let obj_type = infer_expression_type(object, ctx);
            match obj_type {
                Some(TypeHint::List) => Some(TypeHint::Any),
                Some(TypeHint::Map) => Some(TypeHint::Any),
                Some(TypeHint::Str) => Some(TypeHint::Str),
                _ => None,
            }
        }

        ExprKind::Try(inner) => infer_expression_type(inner, ctx),

        ExprKind::InterpolatedString { parts } => {
            let has_exprs = parts
                .iter()
                .any(|p| matches!(p, InterpolatedPart::Expression(_)));
            if has_exprs {
                for part in parts {
                    if let InterpolatedPart::Expression(expr) = part {
                        infer_expression_type(&expr, ctx);
                    }
                }
            }
            Some(TypeHint::Str)
        }

        ExprKind::Lambda { params: _, body: _ } => Some(TypeHint::Any),

        ExprKind::MoveExpr(inner) => infer_expression_type(inner, ctx),

        ExprKind::Iso(inner) => infer_expression_type(inner, ctx),

        ExprKind::Set(_) => Some(TypeHint::Any),
    }
}
