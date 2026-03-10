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
                        format!("IfaValue::Bool(({}).is_truthy() && ({}).is_truthy())", l, r)
                    }
                    BinaryOperator::Or => {
                        format!("IfaValue::Bool(({}).is_truthy() || ({}).is_truthy())", l, r)
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
                let args_str: Vec<String> =
                    args.iter().map(|a| self.transpile_expression(a)).collect();
                format!("{}({})", name, args_str.join(", "))
            }

            Expression::Index { object, index } => {
                let obj = self.transpile_expression(object);
                let idx = self.transpile_expression(index);
                format!("{}[{}]", obj, idx)
            }

            Expression::MethodCall {
                object,
                method,
                args,
            } => {
                let obj = self.transpile_expression(object);
                let args_str: Vec<String> =
                    args.iter().map(|a| self.transpile_expression(a)).collect();
                format!("{}.{}({})", obj, method, args_str.join(", "))
            }
        }
    }
}
