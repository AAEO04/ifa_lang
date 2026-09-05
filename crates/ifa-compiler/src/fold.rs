use super::*;
use ifa_types::ast::*;
use ifa_types::OpCode;
use ifa_types::IfaError;
use ifa_types::IfaResult;

pub(crate) fn fold_expression(expr: &Expression) -> Expression {
    match &expr.kind { ExprKind::BinaryOp { left, op, right } => {
            let left_folded = fold_expression(left);
            let right_folded = fold_expression(right);
            match (left_folded, op, right_folded) {
                // Arithmetic
                (ExprKind::Int(l), BinaryOperator::Add, ExprKind::Int(r)) => {
                    ExprKind::Int(l + r)
                }
                (ExprKind::Int(l), BinaryOperator::Sub, ExprKind::Int(r)) => {
                    ExprKind::Int(l - r)
                }
                (ExprKind::Int(l), BinaryOperator::Mul, ExprKind::Int(r)) => {
                    ExprKind::Int(l * r)
                }
                (ExprKind::Int(l), BinaryOperator::Div, ExprKind::Int(r)) if r != 0 => {
                    ExprKind::Int(l / r)
                }
                (ExprKind::Int(l), BinaryOperator::Mod, ExprKind::Int(r)) if r != 0 => {
                    ExprKind::Int(l % r)
                }
                (ExprKind::Int(l), BinaryOperator::Power, ExprKind::Int(r)) => {
                    if (0..=30).contains(&r) {
                        ExprKind::Int(l.pow(r as u32))
                    } else {
                        ExprKind::BinaryOp {
                            left: Box::new(ExprKind::Int(l)),
                            op: BinaryOperator::Power,
                            right: Box::new(ExprKind::Int(r)),
                        }
                    }
                }

                (ExprKind::Float(l), BinaryOperator::Add, ExprKind::Float(r)) => {
                    ExprKind::Float(l + r)
                }
                (ExprKind::Float(l), BinaryOperator::Sub, ExprKind::Float(r)) => {
                    ExprKind::Float(l - r)
                }
                (ExprKind::Float(l), BinaryOperator::Mul, ExprKind::Float(r)) => {
                    ExprKind::Float(l * r)
                }
                (ExprKind::Float(l), BinaryOperator::Div, ExprKind::Float(r)) => {
                    ExprKind::Float(l / r)
                }

                // Mixing Int and Float (coerce to Float)
                (ExprKind::Int(l), BinaryOperator::Add, ExprKind::Float(r)) => {
                    ExprKind::Float((l as f64) + r)
                }
                (ExprKind::Float(l), BinaryOperator::Add, ExprKind::Int(r)) => {
                    ExprKind::Float(l + (r as f64))
                }
                (ExprKind::Int(l), BinaryOperator::Sub, ExprKind::Float(r)) => {
                    ExprKind::Float((l as f64) - r)
                }
                (ExprKind::Float(l), BinaryOperator::Sub, ExprKind::Int(r)) => {
                    ExprKind::Float(l - (r as f64))
                }
                (ExprKind::Int(l), BinaryOperator::Mul, ExprKind::Float(r)) => {
                    ExprKind::Float((l as f64) * r)
                }
                (ExprKind::Float(l), BinaryOperator::Mul, ExprKind::Int(r)) => {
                    ExprKind::Float(l * (r as f64))
                }
                (ExprKind::Int(l), BinaryOperator::Div, ExprKind::Float(r)) => {
                    ExprKind::Float((l as f64) / r)
                }
                (ExprKind::Float(l), BinaryOperator::Div, ExprKind::Int(r)) => {
                    ExprKind::Float(l / (r as f64))
                }

                // String concatenation
                (ExprKind::String(l), BinaryOperator::Add, ExprKind::String(r)) => {
                    ExprKind::String(format!("{}{}", l, r))
                }

                // Comparison
                (ExprKind::Int(l), BinaryOperator::Eq, ExprKind::Int(r)) => {
                    ExprKind::Bool(l == r)
                }
                (ExprKind::Int(l), BinaryOperator::NotEq, ExprKind::Int(r)) => {
                    ExprKind::Bool(l != r)
                }
                (ExprKind::Int(l), BinaryOperator::Lt, ExprKind::Int(r)) => {
                    ExprKind::Bool(l < r)
                }
                (ExprKind::Int(l), BinaryOperator::LtEq, ExprKind::Int(r)) => {
                    ExprKind::Bool(l <= r)
                }
                (ExprKind::Int(l), BinaryOperator::Gt, ExprKind::Int(r)) => {
                    ExprKind::Bool(l > r)
                }
                (ExprKind::Int(l), BinaryOperator::GtEq, ExprKind::Int(r)) => {
                    ExprKind::Bool(l >= r)
                }

                (ExprKind::Float(l), BinaryOperator::Eq, ExprKind::Float(r)) => {
                    ExprKind::Bool(l == r)
                }
                (ExprKind::Float(l), BinaryOperator::NotEq, ExprKind::Float(r)) => {
                    ExprKind::Bool(l != r)
                }
                (ExprKind::Float(l), BinaryOperator::Lt, ExprKind::Float(r)) => {
                    ExprKind::Bool(l < r)
                }
                (ExprKind::Float(l), BinaryOperator::LtEq, ExprKind::Float(r)) => {
                    ExprKind::Bool(l <= r)
                }
                (ExprKind::Float(l), BinaryOperator::Gt, ExprKind::Float(r)) => {
                    ExprKind::Bool(l > r)
                }
                (ExprKind::Float(l), BinaryOperator::GtEq, ExprKind::Float(r)) => {
                    ExprKind::Bool(l >= r)
                }

                // Logical
                (ExprKind::Bool(l), BinaryOperator::And, ExprKind::Bool(r)) => {
                    ExprKind::Bool(l && r)
                }
                (ExprKind::Bool(l), BinaryOperator::Or, ExprKind::Bool(r)) => {
                    ExprKind::Bool(l || r)
                }

                (l_f, op, r_f) => ExprKind::BinaryOp {
                    left: Box::new(l_f),
                    op: *op,
                    right: Box::new(r_f),
                },
            }
        }
        ExprKind::UnaryOp { op, expr } => {
            let expr_folded = fold_expression(expr);
            match (op, expr_folded) {
                (UnaryOperator::Neg, ExprKind::Int(n)) => ExprKind::Int(-n),
                (UnaryOperator::Neg, ExprKind::Float(f)) => ExprKind::Float(-f),
                (UnaryOperator::Not, ExprKind::Bool(b)) => ExprKind::Bool(!b),
                (op, e_f) => ExprKind::UnaryOp {
                    op: *op,
                    expr: Box::new(e_f),
                },
            }
        }
        ExprKind::MoveExpr(inner) => {
            let inner_folded = fold_expression(inner);
            ExprKind::MoveExpr(Box::new(inner_folded))
        }
        ExprKind::OduCall(call) => {
            let folded_args: Vec<Expression> = call.args.iter().map(fold_expression).collect();
            let all_consts = folded_args.iter().all(|a| {
                matches!(
                    a.kind, ExprKind::Int(_)
                        | ExprKind::Float(_)
                        | ExprKind::String(_)
                        | ExprKind::Bool(_)
                )
            });

            if all_consts {
                // E6: Constant Divination
                match (call.domain, call.method.as_str()) {
                    (ifa_types::OduDomain::Obara, "fikun")
                    | (ifa_types::OduDomain::Obara, "add") => {
                        let mut sum_int = 0;
                        let mut sum_float = 0.0;
                        let mut is_float = false;
                        for arg in &folded_args {
                            match &arg.kind { ExprKind::Int(n) => {
                                    if is_float {
                                        sum_float += *n as f64;
                                    } else {
                                        sum_int += n;
                                    }
                                }
                                ExprKind::Float(f) => {
                                    if !is_float {
                                        is_float = true;
                                        sum_float = sum_int as f64 + *f;
                                    } else {
                                        sum_float += f;
                                    }
                                }
                                _ => {
                                    return ExprKind::OduCall(ifa_types::ast::OduCall {
                                        domain: call.domain,
                                        method: call.method.clone(),
                                        args: folded_args,
                                        is_optional: call.is_optional,
                                        resolved_domain: call.resolved_domain,
                                        resolved_method_id: call.resolved_method_id,
                                        span: call.span.clone(),
                                    });
                                }
                            }
                        }
                        return if is_float {
                            ExprKind::Float(sum_float)
                        } else {
                            ExprKind::Int(sum_int)
                        };
                    }
                    (ifa_types::OduDomain::Obara, "isodipupo")
                    | (ifa_types::OduDomain::Obara, "mul") => {
                        let mut prod_int = 1;
                        let mut prod_float = 1.0;
                        let mut is_float = false;
                        for arg in &folded_args {
                            match &arg.kind { ExprKind::Int(n) => {
                                    if is_float {
                                        prod_float *= *n as f64;
                                    } else {
                                        prod_int *= n;
                                    }
                                }
                                ExprKind::Float(f) => {
                                    if !is_float {
                                        is_float = true;
                                        prod_float = prod_int as f64 * *f;
                                    } else {
                                        prod_float *= f;
                                    }
                                }
                                _ => {
                                    return ExprKind::OduCall(ifa_types::ast::OduCall {
                                        domain: call.domain,
                                        method: call.method.clone(),
                                        args: folded_args,
                                        is_optional: call.is_optional,
                                        resolved_domain: call.resolved_domain,
                                        resolved_method_id: call.resolved_method_id,
                                        span: call.span.clone(),
                                    });
                                }
                            }
                        }
                        return if is_float {
                            ExprKind::Float(prod_float)
                        } else {
                            ExprKind::Int(prod_int)
                        };
                    }
                    (ifa_types::OduDomain::Ika, "gigun") | (ifa_types::OduDomain::Ika, "len")
                        if folded_args.len() == 1 =>
                    {
                        if let ExprKind::String(s) = &folded_args[0] {
                            return ExprKind::Int(s.chars().count() as i64);
                        }
                    }
                    (ifa_types::OduDomain::Ika, "upper") if folded_args.len() == 1 => {
                        if let ExprKind::String(s) = &folded_args[0] {
                            return ExprKind::String(s.to_uppercase());
                        }
                    }
                    (ifa_types::OduDomain::Ika, "lower") if folded_args.len() == 1 => {
                        if let ExprKind::String(s) = &folded_args[0] {
                            return ExprKind::String(s.to_lowercase());
                        }
                    }
                    _ => {}
                }
            }

            ExprKind::OduCall(ifa_types::ast::OduCall {
                domain: call.domain,
                method: call.method.clone(),
                args: folded_args,
                is_optional: call.is_optional,
                resolved_domain: call.resolved_domain,
                resolved_method_id: call.resolved_method_id,
                span: call.span.clone(),
            })
        }
        ExprKind::InterpolatedString { parts } => {
            let folded_parts: Vec<InterpolatedPart> = parts
                .iter()
                .map(|part| match part {
                    InterpolatedPart::Literal(s) => InterpolatedPart::Literal(s.clone()),
                    InterpolatedPart::Expression(expr) => {
                        InterpolatedPart::Expression(Box::new(fold_expression(expr)))
                    }
                })
                .collect();
            let mut combined_parts = Vec::new();
            for part in folded_parts {
                match part {
                    InterpolatedPart::Literal(s) => {
                        if let Some(InterpolatedPart::Literal(last)) = combined_parts.last_mut() {
                            last.push_str(&s);
                        } else {
                            combined_parts.push(InterpolatedPart::Literal(s));
                        }
                    }
                    InterpolatedPart::Expression(expr) => match &expr.kind { ExprKind::String(s) => {
                            if let Some(InterpolatedPart::Literal(last)) = combined_parts.last_mut()
                            {
                                last.push_str(&s);
                            } else {
                                combined_parts.push(InterpolatedPart::Literal(s));
                            }
                        }
                        ExprKind::Int(n) => {
                            let s = n.to_string();
                            if let Some(InterpolatedPart::Literal(last)) = combined_parts.last_mut()
                            {
                                last.push_str(&s);
                            } else {
                                combined_parts.push(InterpolatedPart::Literal(s));
                            }
                        }
                        ExprKind::Float(f) => {
                            let s = f.to_string();
                            if let Some(InterpolatedPart::Literal(last)) = combined_parts.last_mut()
                            {
                                last.push_str(&s);
                            } else {
                                combined_parts.push(InterpolatedPart::Literal(s));
                            }
                        }
                        ExprKind::Bool(b) => {
                            let s = b.to_string();
                            if let Some(InterpolatedPart::Literal(last)) = combined_parts.last_mut()
                            {
                                last.push_str(&s);
                            } else {
                                combined_parts.push(InterpolatedPart::Literal(s));
                            }
                        }
                        _ => combined_parts.push(InterpolatedPart::Expression(expr)),
                    },
                }
            }
            #[allow(clippy::collapsible_if)]
            if combined_parts.len() == 1 {
                if let InterpolatedPart::Literal(s) = &combined_parts[0] {
                    return ExprKind::String(s.clone());
                }
            }
            ExprKind::InterpolatedString {
                parts: combined_parts,
            }
        }
        _ => expr.clone(),
    }
}
