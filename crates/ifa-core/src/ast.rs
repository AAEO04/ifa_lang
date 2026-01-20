//! # Ifá-Lang AST
//!
//! Abstract Syntax Tree types for Ifá-Lang programs.

use crate::lexer::OduDomain;

/// Source location for error reporting
#[derive(Debug, Clone, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

/// A complete Ifá program
#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}

/// Visibility level for fields, functions, and classes
/// Follows Rust model: private by default
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
#[derive(Debug, Clone)]
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
    Ebo {
        offering: Expression,
        span: Span,
    },

    /// Match statement: yàn (condition) { arm1, arm2, ... }
    Match {
        condition: Expression,
        arms: Vec<MatchArm>,
        span: Span,
    },

    /// Expression statement (for calls without semicolon handling)
    Expr { expr: Expression, span: Span },
}

/// Match arm: pattern => body
#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: MatchPattern,
    pub body: Vec<Statement>,
}

/// Match pattern
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub enum AssignTarget {
    Variable(String),
    Index {
        name: String,
        index: Box<Expression>,
    },
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub type_hint: Option<TypeHint>,
}

/// Type hints for optional static typing
#[derive(Debug, Clone)]
pub enum TypeHint {
    Int,
    Float,
    Str,
    Bool,
    List,
    Map,
    Any,
    Custom(String),
}

/// All expression types
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct OduCall {
    pub domain: OduDomain,
    pub method: String,
    pub args: Vec<Expression>,
    pub span: Span,
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Neg,
    Not,
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
