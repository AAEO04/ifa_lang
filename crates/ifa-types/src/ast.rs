//! # Ifá-Lang AST
//!
//! Abstract Syntax Tree types for Ifá-Lang programs.

use crate::domain::OduDomain;
use serde::{Deserialize, Serialize};

/// Source location for error reporting
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

/// A complete Ifá program
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    pub statements: Vec<Statement>,
}

/// Visibility level for fields, functions, and classes
/// Follows Rust model: private by default
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Visibility {
    /// Private (default) - only accessible within same module/class
    /// Yoruba: ikoko (secret), àdáni (private)
    #[default]
    Private,

    /// Public - accessible from anywhere
    /// Yoruba: gbangba (open/public)
    Public,

    /// Package/crate internal - accessible within same package
    /// Yoruba: gbangba(ile) (public within home)
    Crate,
}

/// All statement types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Statement {
    /// Variable declaration: ayanmo x = 5;
    VarDecl {
        name: String,
        type_hint: Option<TypeHint>,
        value: Expression,
        visibility: Visibility,
        span: Span,
    },

    /// Assignment: x = 5;
    Assignment {
        target: AssignTarget,
        value: Expression,
        span: Span,
    },

    /// Import: iba std.otura;
    Import { path: Vec<String>, span: Span },

    /// Constant declaration: const X = 1;
    Const {
        name: String,
        value: Expression,
        span: Span,
    },

    /// Odù call: Obara.fikun(10);
    Instruction { call: OduCall, span: Span },

    /// Class definition: odu Server { }
    OduDef {
        name: String,
        visibility: Visibility,
        body: Vec<Statement>,
        span: Span,
    },

    /// Function definition: ese start() { }
    EseDef {
        name: String,
        visibility: Visibility,
        params: Vec<Param>,
        body: Vec<Statement>,
        span: Span,
    },

    /// If statement
    If {
        condition: Expression,
        then_body: Vec<Statement>,
        else_body: Option<Vec<Statement>>,
        span: Span,
    },

    /// While loop
    While {
        condition: Expression,
        body: Vec<Statement>,
        span: Span,
    },

    /// For loop: fun i ninu items { }
    For {
        var: String,
        iterable: Expression,
        body: Vec<Statement>,
        span: Span,
    },

    /// Return statement
    Return {
        value: Option<Expression>,
        span: Span,
    },

    /// End statement: ase;
    Ase { span: Span },

    /// Taboo declaration: èèwọ̀: Ose -> Odi;
    Taboo {
        source: String, // Source domain (forbidden caller)
        target: String, // Target domain (forbidden callee)
        span: Span,
    },

    /// Assertion/Constraint: ewo x > 0; or assert balance >= 0, "must be positive";
    Ewo {
        condition: Expression,
        message: Option<String>,
        span: Span,
    },

    /// Opon (memory) directive: #opon kekere;
    Opon {
        size: String, // kekere, arinrin, nla, ailopin (or English aliases)
        span: Span,
    },

    /// Ebo (offering/initiation): ebo "server";
    Ebo { offering: Expression, span: Span },

    /// Match statement: yàn (condition) { arm1, arm2, ... }
    Match {
        condition: Expression,
        arms: Vec<MatchArm>,
        span: Span,
    },

    /// Expression statement (for calls without semicolon handling)
    Expr { expr: Expression, span: Span },

    /// Ailewu (unsafe) block - allows low-level operations
    /// Dual lexicon: ailewu { } or unsafe { }
    /// Yoruba: àìléwu = without danger (ironic - actually means "this is dangerous")
    Ailewu { body: Vec<Statement>, span: Span },

    /// Yield execution: jowo 1000; or yield 1000;
    Yield { duration: Expression, span: Span },

    /// Try/Catch block: gbiyanju { ... } pada (err) { ... }
    Try {
        try_body: Vec<Statement>,
        catch_var: String, // The error variable name (e.g., "e")
        catch_body: Vec<Statement>,
        span: Span,
    },
}

/// Match arm: pattern => body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: MatchPattern,
    pub body: Vec<Statement>,
}

/// Match pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatchPattern {
    /// Literal pattern: 200, "hello"
    Literal(Expression),
    /// Range pattern: 90..99
    Range {
        start: Box<Expression>,
        end: Box<Expression>,
    },
    /// Wildcard pattern: _
    Wildcard,
}

/// Assignment target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssignTarget {
    Variable(String),
    Index {
        name: String,
        index: Box<Expression>,
    },
    Dereference(Box<Expression>),
}

/// Function parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Param {
    pub name: String,
    pub type_hint: Option<TypeHint>,
}

/// Type hints for optional static typing
/// Supports both high-level dynamic types and low-level static types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypeHint {
    // ═══════════════════════════════════════════════════════════════════
    // High-Level (Dynamic) Types
    // ═══════════════════════════════════════════════════════════════════
    /// Dynamic integer (IfaValue::Int)
    Int,
    /// Dynamic float (IfaValue::Float)
    Float,
    /// Dynamic string (IfaValue::Str)
    Str,
    /// Dynamic boolean (IfaValue::Bool)
    Bool,
    /// Dynamic list (IfaValue::List)
    List,
    /// Dynamic map (IfaValue::Map)
    Map,
    /// Any type (fully dynamic)
    Any,
    /// Custom/user-defined type
    Custom(String),

    // ═══════════════════════════════════════════════════════════════════
    // Low-Level (Static) Types - Requires ailewu or static context
    // ═══════════════════════════════════════════════════════════════════
    /// Signed integers with explicit size
    I8,
    I16,
    I32,
    I64,
    /// Unsigned integers with explicit size
    U8,
    U16,
    U32,
    U64,
    /// Floating point with explicit size
    F32,
    F64,
    /// Raw pointer: *T (e.g., *i32, *u8)
    /// Yoruba: àmì (pointer/sign)
    Ptr(Box<TypeHint>),
    /// Reference: &T (borrowed, tracked by IwaEngine)
    /// Yoruba: ìtọ́kasí (reference)
    Ref(Box<TypeHint>),
    /// Mutable reference: &mut T
    RefMut(Box<TypeHint>),
    /// Fixed-size array: [T; N]
    Array {
        element: Box<TypeHint>,
        size: usize,
    },
    /// Void/unit type (for function returns)
    Void,
}

impl TypeHint {
    /// Check if this is a low-level (static) type requiring strict checking
    pub fn is_low_level(&self) -> bool {
        matches!(
            self,
            TypeHint::I8
                | TypeHint::I16
                | TypeHint::I32
                | TypeHint::I64
                | TypeHint::U8
                | TypeHint::U16
                | TypeHint::U32
                | TypeHint::U64
                | TypeHint::F32
                | TypeHint::F64
                | TypeHint::Ptr(_)
                | TypeHint::Ref(_)
                | TypeHint::RefMut(_)
                | TypeHint::Array { .. }
                | TypeHint::Void
        )
    }

    /// Check if this is a pointer or reference type (requires borrow tracking)
    pub fn is_pointer_like(&self) -> bool {
        matches!(
            self,
            TypeHint::Ptr(_) | TypeHint::Ref(_) | TypeHint::RefMut(_)
        )
    }

    /// Check if this type requires ailewu (unsafe) context
    pub fn requires_ailewu(&self) -> bool {
        matches!(self, TypeHint::Ptr(_))
    }

    /// Get the size in bytes for primitive types (None for dynamic/composite)
    pub fn size_bytes(&self) -> Option<usize> {
        match self {
            TypeHint::I8 | TypeHint::U8 => Some(1),
            TypeHint::I16 | TypeHint::U16 => Some(2),
            TypeHint::I32 | TypeHint::U32 | TypeHint::F32 => Some(4),
            TypeHint::I64 | TypeHint::U64 | TypeHint::F64 => Some(8),
            TypeHint::Bool => Some(1),
            TypeHint::Void => Some(0),
            _ => None,
        }
    }
}

/// All expression types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expression {
    /// Integer literal
    Int(i64),

    /// Float literal
    Float(f64),

    /// String literal
    String(String),

    /// Boolean literal
    Bool(bool),

    /// Nil/null
    Nil,

    /// Variable reference
    Identifier(String),

    /// Binary operation
    BinaryOp {
        left: Box<Expression>,
        op: BinaryOperator,
        right: Box<Expression>,
    },

    /// Unary operation
    UnaryOp {
        op: UnaryOperator,
        expr: Box<Expression>,
    },

    /// Odù domain call: Obara.fikun(10)
    OduCall(OduCall),

    /// Method call: obj.method(args)
    MethodCall {
        object: Box<Expression>,
        method: String,
        args: Vec<Expression>,
    },

    /// Function call: func(args)
    Call { name: String, args: Vec<Expression> },

    /// List literal: [1, 2, 3]
    List(Vec<Expression>),

    /// Map literal: { "key": value }
    Map(Vec<(Expression, Expression)>),

    /// Index access: arr\[0\]
    Index {
        object: Box<Expression>,
        index: Box<Expression>,
    },
}

/// Odù domain method call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OduCall {
    pub domain: OduDomain,
    pub method: String,
    pub args: Vec<Expression>,
    pub span: Span,
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BinaryOperator {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,

    // Comparison
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,

    // Logical
    And,
    Or,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UnaryOperator {
    Neg,
    Not,
    AddressOf,   // &x
    Dereference, // *x
}

impl std::fmt::Display for BinaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Add => write!(f, "+"),
            Self::Sub => write!(f, "-"),
            Self::Mul => write!(f, "*"),
            Self::Div => write!(f, "/"),
            Self::Mod => write!(f, "%"),
            Self::Eq => write!(f, "=="),
            Self::NotEq => write!(f, "!="),
            Self::Lt => write!(f, "<"),
            Self::LtEq => write!(f, "<="),
            Self::Gt => write!(f, ">"),
            Self::GtEq => write!(f, ">="),
            Self::And => write!(f, "&&"),
            Self::Or => write!(f, "||"),
        }
    }
}
