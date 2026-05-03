//! # Expression Transpilation
//!
//! Transpiles Ifá-Lang expressions to Rust code.

use super::core::RustTranspiler;
use crate::ast::*;

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
                let l = self.transpile_expression(left);
                let r = self.transpile_expression(right);
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
                // Desugar to Rust `?` — transpiler targets Rust, so this is exact.
                let inner = self.transpile_expression(expr);
                format!("{}?", inner)
            }

            Expression::InterpolatedString { parts } => {
                let mut fmt_str = String::new();
                let mut args = Vec::new();
                for part in parts {
                    match part {
                        InterpolatedPart::Literal(s) => {
                            fmt_str.push_str(&s.replace("{", "{{").replace("}", "}}").replace("\"", "\\\""));
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
                    format!("IfaValue::str(format!(\"{}\", {}))", fmt_str, args.join(", "))
                }
            }
        }
    }
}
