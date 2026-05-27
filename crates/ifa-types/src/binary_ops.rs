use crate::ast::{BinaryOperator, TypeHint};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpCompat {
    Valid(TypeHint),
    Invalid,
}

impl OpCompat {
    pub fn result_type(&self) -> Option<TypeHint> {
        match self {
            OpCompat::Valid(t) => Some(t.clone()),
            OpCompat::Invalid => None,
        }
    }

    pub fn is_valid(&self) -> bool {
        matches!(self, OpCompat::Valid(_))
    }
}

pub fn binary_op_result_type(
    op: &BinaryOperator,
    lhs: &TypeHint,
    rhs: &TypeHint,
) -> Option<TypeHint> {
    use BinaryOperator::*;
    use TypeHint::*;

    match op {
        Add => check_numeric(lhs, rhs).or_else(|| check_string(lhs, rhs)),

        Sub | Mul => check_numeric(lhs, rhs),

        Div | Mod => check_numeric_nonzero(lhs, rhs, op),

        Power => check_numeric(lhs, rhs),

        Eq | NotEq => {
            if types_compat_for_eq(lhs, rhs) {
                Some(Bool)
            } else {
                None
            }
        }

        Lt | LtEq | Gt | GtEq => check_numeric(lhs, rhs).map(|_| Bool),

        And | Or | NullCoalesce => {
            if *lhs == Any || *rhs == Any {
                Some(Any)
            } else {
                Some(rhs.clone())
            }
        }
    }
}

pub fn is_valid_binary_op(op: &BinaryOperator, lhs: &TypeHint, rhs: &TypeHint) -> bool {
    binary_op_result_type(op, lhs, rhs).is_some()
}

fn check_numeric(lhs: &TypeHint, rhs: &TypeHint) -> Option<TypeHint> {
    use TypeHint::*;

    match (lhs, rhs) {
        (Int, Int) => Some(Int),
        (Float, Float) => Some(Float),
        (Int, Float) => Some(Float),
        (Float, Int) => Some(Float),

        (I8, I8) => Some(I8),
        (I16, I16) => Some(I16),
        (I32, I32) => Some(I32),
        (I64, I64) => Some(I64),
        (I8, I16) => Some(I16),
        (I16, I8) => Some(I16),
        (I8, I32) => Some(I32),
        (I32, I8) => Some(I32),
        (I16, I32) => Some(I32),
        (I32, I16) => Some(I32),
        (I8, I64) => Some(I64),
        (I64, I8) => Some(I64),
        (I16, I64) => Some(I64),
        (I64, I16) => Some(I64),
        (I32, I64) => Some(I64),
        (I64, I32) => Some(I64),
        (F32, F32) => Some(F32),
        (F64, F64) => Some(F64),
        (F32, F64) => Some(F64),
        (F64, F32) => Some(F64),
        (Int, I8) => Some(Int),
        (I8, Int) => Some(Int),
        (Int, I16) => Some(Int),
        (I16, Int) => Some(Int),
        (Int, I32) => Some(Int),
        (I32, Int) => Some(Int),
        (Int, I64) => Some(Int),
        (I64, Int) => Some(Int),
        (Float, F32) => Some(Float),
        (F32, Float) => Some(Float),
        (Float, F64) => Some(Float),
        (F64, Float) => Some(Float),

        _ => None,
    }
}

fn check_numeric_nonzero(lhs: &TypeHint, rhs: &TypeHint, _op: &BinaryOperator) -> Option<TypeHint> {
    check_numeric(lhs, rhs)
}

fn check_string(lhs: &TypeHint, rhs: &TypeHint) -> Option<TypeHint> {
    use TypeHint::*;
    match (lhs, rhs) {
        (Str, Str) => Some(Str),
        _ => None,
    }
}

fn types_compat_for_eq(lhs: &TypeHint, rhs: &TypeHint) -> bool {
    use TypeHint::*;
    match (lhs, rhs) {
        (Int, Int) | (Float, Float) | (Str, Str) | (Bool, Bool) => true,
        (Int, Float) | (Float, Int) => true,
        (Any, _) | (_, Any) => true,
        _ => lhs == rhs,
    }
}
