//! # Expression Transpilation
//!
//! Transpiles Ifá-Lang expressions to Rust code.

use super::core::RustTranspiler;
use ifa_types::ast::*;

impl RustTranspiler {
    /// Transpile an expression to Rust
    pub fn transpile_expression(&mut self, expr: &Expression) -> String {
        match expr {
            Expression::Int(n) => format!("IfaValue::Int({})", n),
            Expression::Float(f) => format!("IfaValue::Float({})", f),
            Expression::String(s) => format!("IfaValue::Str(\"{}\".to_string())", s),
            Expression::Bool(b) => format!("IfaValue::Bool({})", b),
            Expression::Nil => "IfaValue::Nil".to_string(),
            Expression::Identifier(name) => self.mangle_identifier(name),

            Expression::BinaryOp { left, op, right } => {
                if let Some(opt) = self.try_transpile_literal_binop(left, op, right) {
                    return opt;
                }

                let left_type = self.get_expr_type(left);
                let right_type = self.get_expr_type(right);

                let l = self.transpile_expression(left);
                let r = self.transpile_expression(right);

                if left_type == Some(TypeHint::Int) && right_type == Some(TypeHint::Int) {
                    match op {
                        BinaryOperator::Add => {
                            return format!("IfaValue::Int({}.as_int() + {}.as_int())", l, r);
                        }
                        BinaryOperator::Sub => {
                            return format!("IfaValue::Int({}.as_int() - {}.as_int())", l, r);
                        }
                        BinaryOperator::Mul => {
                            return format!("IfaValue::Int({}.as_int() * {}.as_int())", l, r);
                        }
                        BinaryOperator::Eq => {
                            return format!("IfaValue::Bool({}.as_int() == {}.as_int())", l, r);
                        }
                        BinaryOperator::NotEq => {
                            return format!("IfaValue::Bool({}.as_int() != {}.as_int())", l, r);
                        }
                        BinaryOperator::Lt => {
                            return format!("IfaValue::Bool({}.as_int() < {}.as_int())", l, r);
                        }
                        BinaryOperator::LtEq => {
                            return format!("IfaValue::Bool({}.as_int() <= {}.as_int())", l, r);
                        }
                        BinaryOperator::Gt => {
                            return format!("IfaValue::Bool({}.as_int() > {}.as_int())", l, r);
                        }
                        BinaryOperator::GtEq => {
                            return format!("IfaValue::Bool({}.as_int() >= {}.as_int())", l, r);
                        }
                        _ => {}
                    }
                } else if left_type == Some(TypeHint::Float) && right_type == Some(TypeHint::Float)
                {
                    match op {
                        BinaryOperator::Add => {
                            return format!("IfaValue::Float({}.as_float() + {}.as_float())", l, r);
                        }
                        BinaryOperator::Sub => {
                            return format!("IfaValue::Float({}.as_float() - {}.as_float())", l, r);
                        }
                        BinaryOperator::Mul => {
                            return format!("IfaValue::Float({}.as_float() * {}.as_float())", l, r);
                        }
                        BinaryOperator::Eq => {
                            return format!("IfaValue::Bool({}.as_float() == {}.as_float())", l, r);
                        }
                        BinaryOperator::NotEq => {
                            return format!("IfaValue::Bool({}.as_float() != {}.as_float())", l, r);
                        }
                        BinaryOperator::Lt => {
                            return format!("IfaValue::Bool({}.as_float() < {}.as_float())", l, r);
                        }
                        BinaryOperator::LtEq => {
                            return format!("IfaValue::Bool({}.as_float() <= {}.as_float())", l, r);
                        }
                        BinaryOperator::Gt => {
                            return format!("IfaValue::Bool({}.as_float() > {}.as_float())", l, r);
                        }
                        BinaryOperator::GtEq => {
                            return format!("IfaValue::Bool({}.as_float() >= {}.as_float())", l, r);
                        }
                        _ => {}
                    }
                }

                match op {
                    BinaryOperator::Eq => format!("IfaValue::Bool({} == {})", l, r),
                    BinaryOperator::NotEq => format!("IfaValue::Bool({} != {})", l, r),
                    BinaryOperator::Lt => format!("IfaValue::Bool({} < {})", l, r),
                    BinaryOperator::LtEq => format!("IfaValue::Bool({} <= {})", l, r),
                    BinaryOperator::Gt => format!("IfaValue::Bool({} > {})", l, r),
                    BinaryOperator::GtEq => format!("IfaValue::Bool({} >= {})", l, r),
                    BinaryOperator::And => {
                        format!(
                            "({{ let __ifa_l = {}; if !__ifa_l.is_truthy() {{ __ifa_l }} else {{ {} }} }})",
                            l, r
                        )
                    }
                    BinaryOperator::Or => {
                        format!(
                            "({{ let __ifa_l = {}; if __ifa_l.is_truthy() {{ __ifa_l }} else {{ {} }} }})",
                            l, r
                        )
                    }
                    BinaryOperator::Power => {
                        format!("{}.pow(&{})", l, r)
                    }
                    BinaryOperator::NullCoalesce => {
                        format!(
                            "({{ let __ifa_l = {}; if matches!(__ifa_l, IfaValue::Nil) {{ {} }} else {{ __ifa_l }} }})",
                            l, r
                        )
                    }
                    _ => format!("({} {} {})", l, op, r),
                }
            }

            Expression::UnaryOp { op, expr } => {
                let o = self.transpile_expression(expr);
                match op {
                    UnaryOperator::Neg => format!("(-{})", o),
                    UnaryOperator::Not => format!("(!{})", o),
                    UnaryOperator::AddressOf | UnaryOperator::Dereference => {
                        // Not supported in transpiler yet
                        format!("/* Pointers Unimplemented */ IfaValue::Nil")
                    }
                }
            }

            Expression::List(items) => {
                let items_str: Vec<String> =
                    items.iter().map(|i| self.transpile_expression(i)).collect();
                format!("IfaValue::List(vec![{}])", items_str.join(", "))
            }

            Expression::Map(pairs) => {
                let pairs_str: Vec<String> = pairs
                    .iter()
                    .map(|(k, v)| {
                        format!(
                            "({}, {})",
                            self.transpile_expression(k),
                            self.transpile_expression(v)
                        )
                    })
                    .collect();
                format!("IfaValue::Map(HashMap::from([{}]))", pairs_str.join(", "))
            }

            Expression::OduCall(call) => self.transpile_odu_call(call),

            Expression::Call { name, args } => {
                if let Some(domain) = self.std_named.get(name) {
                    let call = OduCall {
                        domain: *domain,
                        method: name.clone(),
                        args: args.clone(),
                        is_optional: false,
                        resolved_domain: None,
                        resolved_method_id: None,
                        span: Span::default(),
                    };
                    return self.transpile_odu_call(&call);
                }
                let args_str: Vec<String> =
                    args.iter().map(|a| self.transpile_expression(a)).collect();
                format!("{}({})", name, args_str.join(", "))
            }

            Expression::Await(expr) => {
                self.has_async = true;
                let inner = self.transpile_expression(expr);
                format!("({}).await", inner)
            }

            Expression::Index {
                object,
                index,
                is_optional,
            } => {
                let obj = self.transpile_expression(object);
                let idx = self.transpile_expression(index);
                if *is_optional {
                    format!("(({}).get_optional({}))", obj, idx)
                } else {
                    format!("{}[{}]", obj, idx)
                }
            }

            Expression::Get {
                object,
                name,
                is_optional,
            } => {
                let obj = self.transpile_expression(object);
                if *is_optional {
                    format!("(({}).get_attr_optional(\"{}\"))", obj, name)
                } else {
                    format!("{}.{}", obj, name)
                }
            }

            Expression::MethodCall {
                object,
                method,
                args,
                is_optional,
            } => {
                if let Expression::Identifier(obj_name) = &**object {
                    if let Some(domain) = self.std_modules.get(obj_name) {
                        let call = OduCall {
                            domain: *domain,
                            method: method.clone(),
                            args: args.clone(),
                            is_optional: *is_optional,
                            resolved_domain: None,
                            resolved_method_id: None,
                            span: Span::default(),
                        };
                        return self.transpile_odu_call(&call);
                    }
                    if self.module_aliases.contains(obj_name) {
                        let args_str: Vec<String> =
                            args.iter().map(|a| self.transpile_expression(a)).collect();
                        return format!("{}::{}({})", obj_name, method, args_str.join(", "));
                    }
                }
                let obj = self.transpile_expression(object);
                let args_str: Vec<String> =
                    args.iter().map(|a| self.transpile_expression(a)).collect();
                format!("{}.{}({})", obj, method, args_str.join(", "))
            }
            Expression::Try(expr) => {
                let inner = self.transpile_expression(expr);
                format!("{}?", inner)
            }

            Expression::InterpolatedString { parts } => {
                let mut fmt_str = String::new();
                let mut args = Vec::new();
                for part in parts {
                    match part {
                        InterpolatedPart::Literal(s) => {
                            fmt_str.push_str(
                                &s.replace("{", "{{")
                                    .replace("}", "}}")
                                    .replace("\"", "\\\""),
                            );
                        }
                        InterpolatedPart::Expression(expr) => {
                            fmt_str.push_str("{}");
                            args.push(self.transpile_expression(expr));
                        }
                    }
                }
                if args.is_empty() {
                    format!("IfaValue::str(\"{}\")", fmt_str)
                } else {
                    format!(
                        "IfaValue::str(format!(\"{}\", {}))",
                        fmt_str,
                        args.join(", ")
                    )
                }
            }

            Expression::Lambda { params, body } => {
                let params_str: Vec<String> =
                    params.iter().map(|p| format!("{}: IfaValue", p)).collect();
                let mut inner = String::new();
                for s in body {
                    inner.push_str(&self.transpile_statement(s));
                    inner.push('\n');
                }
                format!(
                    "(|{}| -> IfaValue {{ {}; IfaValue::Nil }})",
                    params_str.join(", "),
                    inner.trim()
                )
            }
        }
    }

    fn get_expr_type(&self, expr: &Expression) -> Option<TypeHint> {
        match expr {
            Expression::Int(_) => Some(TypeHint::Int),
            Expression::Float(_) => Some(TypeHint::Float),
            Expression::String(_) => Some(TypeHint::Str),
            Expression::Bool(_) => Some(TypeHint::Bool),
            Expression::Identifier(name) => self.type_env.get(name).cloned(),
            // A more complete implementation would recursively call into binary_op_result_type,
            // but for Phase B variable specialization, Identifier is the main target.
            _ => None,
        }
    }

    fn try_transpile_literal_binop(
        &self,
        left: &Expression,
        op: &BinaryOperator,
        right: &Expression,
    ) -> Option<String> {
        use BinaryOperator::*;
        use Expression::*;

        match (left, right) {
            (Int(a), Int(b)) => match op {
                Add => Some(format!("IfaValue::Int({}i64 + {}i64)", a, b)),
                Sub => Some(format!("IfaValue::Int({}i64 - {}i64)", a, b)),
                Mul => Some(format!("IfaValue::Int({}i64 * {}i64)", a, b)),
                Div => {
                    if *b == 0 {
                        Some("IfaValue::Nil".to_string())
                    } else {
                        Some(format!(
                            "IfaValue::Float({}f64 / {}f64)",
                            *a as f64, *b as f64
                        ))
                    }
                }
                Mod => {
                    if *b == 0 {
                        Some("IfaValue::Nil".to_string())
                    } else {
                        Some(format!("IfaValue::Int({}i64 % {}i64)", a, b))
                    }
                }
                Power => {
                    if *b >= 0 {
                        Some(format!("IfaValue::Int({}i64.pow({}u32))", a, b))
                    } else {
                        Some(format!(
                            "IfaValue::Float(({}f64).powi({}i32))",
                            *a as f64, *b as i32
                        ))
                    }
                }
                Eq => Some(format!("IfaValue::Bool({}i64 == {}i64)", a, b)),
                NotEq => Some(format!("IfaValue::Bool({}i64 != {}i64)", a, b)),
                Lt => Some(format!("IfaValue::Bool({}i64 < {}i64)", a, b)),
                LtEq => Some(format!("IfaValue::Bool({}i64 <= {}i64)", a, b)),
                Gt => Some(format!("IfaValue::Bool({}i64 > {}i64)", a, b)),
                GtEq => Some(format!("IfaValue::Bool({}i64 >= {}i64)", a, b)),
                _ => None,
            },
            (Float(a), Float(b)) => match op {
                Add => Some(format!("IfaValue::Float({}f64 + {}f64)", a, b)),
                Sub => Some(format!("IfaValue::Float({}f64 - {}f64)", a, b)),
                Mul => Some(format!("IfaValue::Float({}f64 * {}f64)", a, b)),
                Div => {
                    if *b == 0.0 {
                        Some("IfaValue::Nil".to_string())
                    } else {
                        Some(format!("IfaValue::Float({}f64 / {}f64)", a, b))
                    }
                }
                Mod => Some(format!("IfaValue::Float({}f64 % {}f64)", a, b)),
                Power => Some(format!("IfaValue::Float(({}f64).powf({}f64))", a, b)),
                Eq => Some(format!("IfaValue::Bool({}f64 == {}f64)", a, b)),
                NotEq => Some(format!("IfaValue::Bool({}f64 != {}f64)", a, b)),
                Lt => Some(format!("IfaValue::Bool({}f64 < {}f64)", a, b)),
                LtEq => Some(format!("IfaValue::Bool({}f64 <= {}f64)", a, b)),
                Gt => Some(format!("IfaValue::Bool({}f64 > {}f64)", a, b)),
                GtEq => Some(format!("IfaValue::Bool({}f64 >= {}f64)", a, b)),
                _ => None,
            },
            (Int(a), Float(b)) => {
                let af = *a as f64;
                match op {
                    Add => Some(format!("IfaValue::Float({}f64 + {}f64)", af, b)),
                    Sub => Some(format!("IfaValue::Float({}f64 - {}f64)", af, b)),
                    Mul => Some(format!("IfaValue::Float({}f64 * {}f64)", af, b)),
                    Div => {
                        if *b == 0.0 {
                            Some("IfaValue::Nil".to_string())
                        } else {
                            Some(format!("IfaValue::Float({}f64 / {}f64)", af, b))
                        }
                    }
                    Mod => Some(format!("IfaValue::Float({}f64 % {}f64)", af, b)),
                    Power => Some(format!("IfaValue::Float(({}f64).powf({}f64))", af, b)),
                    Eq => Some(format!("IfaValue::Bool({}f64 == {}f64)", af, b)),
                    NotEq => Some(format!("IfaValue::Bool({}f64 != {}f64)", af, b)),
                    Lt => Some(format!("IfaValue::Bool({}f64 < {}f64)", af, b)),
                    LtEq => Some(format!("IfaValue::Bool({}f64 <= {}f64)", af, b)),
                    Gt => Some(format!("IfaValue::Bool({}f64 > {}f64)", af, b)),
                    GtEq => Some(format!("IfaValue::Bool({}f64 >= {}f64)", af, b)),
                    _ => None,
                }
            }
            (Float(a), Int(b)) => {
                let bf = *b as f64;
                match op {
                    Add => Some(format!("IfaValue::Float({}f64 + {}f64)", a, bf)),
                    Sub => Some(format!("IfaValue::Float({}f64 - {}f64)", a, bf)),
                    Mul => Some(format!("IfaValue::Float({}f64 * {}f64)", a, bf)),
                    Div => {
                        if *b == 0 {
                            Some("IfaValue::Nil".to_string())
                        } else {
                            Some(format!("IfaValue::Float({}f64 / {}f64)", a, bf))
                        }
                    }
                    Mod => Some(format!("IfaValue::Float({}f64 % {}f64)", a, bf)),
                    Power => Some(format!(
                        "IfaValue::Float(({}f64).powi({}i32))",
                        a, *b as i32
                    )),
                    Eq => Some(format!("IfaValue::Bool({}f64 == {}f64)", a, bf)),
                    NotEq => Some(format!("IfaValue::Bool({}f64 != {}f64)", a, bf)),
                    Lt => Some(format!("IfaValue::Bool({}f64 < {}f64)", a, bf)),
                    LtEq => Some(format!("IfaValue::Bool({}f64 <= {}f64)", a, bf)),
                    Gt => Some(format!("IfaValue::Bool({}f64 > {}f64)", a, bf)),
                    GtEq => Some(format!("IfaValue::Bool({}f64 >= {}f64)", a, bf)),
                    _ => None,
                }
            }
            (Bool(a), Bool(b)) => match op {
                Eq => Some(format!("IfaValue::Bool({} == {})", a, b)),
                NotEq => Some(format!("IfaValue::Bool({} != {})", a, b)),
                And => Some(format!("IfaValue::Bool({} && {})", a, b)),
                Or => Some(format!("IfaValue::Bool({} || {})", a, b)),
                _ => None,
            },
            _ => None,
        }
    }
}
