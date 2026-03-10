//! # Ifá Error Types (Ọ̀kànràn)
//!
//! Structured error handling with Yoruba proverbs for educational context.

use thiserror::Error;

/// Result type alias for Ifá operations
pub type IfaResult<T> = Result<T, IfaError>;

/// Core error type for Ifá-Lang runtime
#[derive(Error, Debug, Clone)]
pub enum IfaError {
    // =========================================================================
    // MATH ERRORS (Ọ̀bàrà / Òtúúrúpọ̀n)
    // =========================================================================
    #[error("Division by zero - Ọ̀bàrà rejects: {0}")]
    DivisionByZero(String),

    #[error("Numeric overflow: {0}")]
    Overflow(String),

    #[error("Numeric underflow: {0}")]
    Underflow(String),

    // =========================================================================
    // TYPE ERRORS
    // =========================================================================
    #[error("Arity mismatch: expected {expected} arguments, got {got}")]
    ArityMismatch { expected: usize, got: usize },

    #[error("Argument error: {0}")]
    ArgumentError(String),

    #[error("Type mismatch: expected {expected}, got {got}")]
    TypeError { expected: String, got: String },

    #[error("Cannot convert {from} to {to}")]
    ConversionError { from: String, to: String },

    // =========================================================================
    // INDEX/KEY ERRORS (Ògúndá / Ìká)
    // =========================================================================
    #[error("Index out of bounds: {index} (length: {length})")]
    IndexOutOfBounds { index: i64, length: usize },

    #[error("Key not found: {0}")]
    KeyNotFound(String),

    // =========================================================================
    // I/O ERRORS (Òdí / Ìrosù)
    // =========================================================================
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("I/O error: {0}")]
    IoError(String), // Changed from std::io::Error to String for Clone support

    // =========================================================================
    // NETWORK ERRORS (Òtúrá)
    // =========================================================================
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Timeout after {0}ms")]
    Timeout(u64),

    #[error("SSRF blocked: {0}")]
    SsrfBlocked(String),

    // =========================================================================
    // VM ERRORS
    // =========================================================================
    #[error("Unknown opcode: {0}")]
    UnknownOpcode(u8),

    #[error("Stack underflow")]
    StackUnderflow,

    #[error("Stack overflow (limit: {0})")]
    StackOverflow(usize),

    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),

    #[error("Undefined function: {0}")]
    UndefinedFunction(String),

    // =========================================================================
    // MEMORY ERRORS (Opon)
    // =========================================================================
    #[error("Opon (memory) exhausted: requested {requested}, available {available}")]
    OponExhausted { requested: usize, available: usize },

    // =========================================================================
    // GENERIC
    // =========================================================================
    #[error("Assertion failed: {0}")]
    AssertionFailed(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("{0}")]
    Custom(String),

    #[error("User error: {0}")]
    UserError(String),

    // =========================================================================
    // PARSER/INTERPRETER ERRORS
    // =========================================================================
    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Compile error: {0}")]
    Compile(String),

    #[error("Runtime error: {0}")]
    Runtime(String),
}

impl IfaError {
    /// Get the standard numeric error code for this error
    pub fn error_code(&self) -> ifa_types::ErrorCode {
        use ifa_types::ErrorCode;

        match self {
            IfaError::DivisionByZero(_) => ErrorCode::DivByZero,
            IfaError::Overflow(_) | IfaError::Underflow(_) => ErrorCode::Overflow,

            IfaError::TypeError { .. } => ErrorCode::TypeMismatch,
            IfaError::ConversionError { .. } => ErrorCode::InvalidCast,

            IfaError::IndexOutOfBounds { .. } => ErrorCode::OutOfBounds,
            IfaError::KeyNotFound(_) => ErrorCode::OutOfBounds,

            IfaError::FileNotFound(_) => ErrorCode::FileNotFound,
            IfaError::PermissionDenied(_) | IfaError::SsrfBlocked(_) => ErrorCode::PermissionDenied,
            IfaError::IoError(_) | IfaError::ConnectionFailed(_) => ErrorCode::IoError,
            IfaError::Timeout(_) => ErrorCode::Timeout,

            IfaError::UnknownOpcode(_) => ErrorCode::InvalidOpCode,
            IfaError::StackUnderflow => ErrorCode::StackUnderflow,
            IfaError::StackOverflow(_) => ErrorCode::StackOverflow,

            IfaError::UndefinedVariable(_) | IfaError::UndefinedFunction(_) => {
                ErrorCode::UndefinedVar
            }
            IfaError::OponExhausted { .. } => ErrorCode::OutOfMemory,

            IfaError::Parse(_) | IfaError::Compile(_) => ErrorCode::InvalidBytecode,

            // Generic mappings
            IfaError::ArityMismatch { .. } | IfaError::ArgumentError(_) => ErrorCode::VmError,
            IfaError::AssertionFailed(_) => ErrorCode::VmError,
            IfaError::NotImplemented(_) => ErrorCode::VmError,
            IfaError::Custom(_) => ErrorCode::VmError,
            IfaError::UserError(_) => ErrorCode::VmError,
            IfaError::Runtime(_) => ErrorCode::VmError,
        }
    }

    /// Get a Yoruba proverb related to this error (for educational context)
    pub fn proverb(&self) -> &'static str {
        match self {
            IfaError::DivisionByZero(_) => {
                "Bí a bá pín ohun tí kò sí, a ò lè rí nǹkan. (If we divide nothing, we find nothing.)"
            }
            IfaError::TypeError { .. } => "Kì í ṣe gbogbo ẹyẹ ló ń fò. (Not all birds can fly.)",
            IfaError::IndexOutOfBounds { .. } => {
                "Ẹni tó gbéra lé òpin, yóò ṣubú. (One who goes beyond the edge will fall.)"
            }
            IfaError::KeyNotFound(_) => {
                "A kì í wá ohun tí kò sí. (One cannot find what does not exist.)"
            }
            IfaError::StackUnderflow => {
                "A kì í gbé ohun tí kò sí nínú àpótí. (One cannot lift what is not in the basket.)"
            }
            IfaError::OponExhausted { .. } => {
                "Ìgbà méjì kì í wọ inú àwo kan. (Two times cannot fit in one calabash.)"
            }
            _ => "Gbogbo ìṣòro ní ojúùtù. (Every problem has a solution.)",
        }
    }
}

impl From<ifa_types::IfaError> for IfaError {
    fn from(err: ifa_types::IfaError) -> Self {
        match err {
            ifa_types::IfaError::DivisionByZero(msg) => IfaError::DivisionByZero(msg),
            ifa_types::IfaError::Overflow(msg) => IfaError::Overflow(msg),
            ifa_types::IfaError::Underflow(msg) => IfaError::Underflow(msg),
            ifa_types::IfaError::ArityMismatch { expected, got } => {
                IfaError::ArityMismatch { expected, got }
            }
            ifa_types::IfaError::ArgumentError(msg) => IfaError::ArgumentError(msg),
            ifa_types::IfaError::TypeError { expected, got } => {
                IfaError::TypeError { expected, got }
            }
            ifa_types::IfaError::ConversionError { from, to } => {
                IfaError::ConversionError { from, to }
            }
            ifa_types::IfaError::IndexOutOfBounds { index, length } => {
                IfaError::IndexOutOfBounds { index, length }
            }
            ifa_types::IfaError::KeyNotFound(k) => IfaError::KeyNotFound(k),
            ifa_types::IfaError::FileNotFound(f) => IfaError::FileNotFound(f),
            ifa_types::IfaError::PermissionDenied(p) => IfaError::PermissionDenied(p),
            // Map IO error manually or through string if needed, io::Error isn't cloneable easily but here we construct new
            ifa_types::IfaError::IoError(e) => {
                IfaError::IoError(e.to_string())
            }
            ifa_types::IfaError::ConnectionFailed(c) => IfaError::ConnectionFailed(c),
            ifa_types::IfaError::Timeout(t) => IfaError::Timeout(t),
            ifa_types::IfaError::SsrfBlocked(s) => IfaError::SsrfBlocked(s),
            ifa_types::IfaError::UnknownOpcode(o) => IfaError::UnknownOpcode(o),
            ifa_types::IfaError::StackUnderflow => IfaError::StackUnderflow,
            ifa_types::IfaError::StackOverflow(s) => IfaError::StackOverflow(s),
            ifa_types::IfaError::UndefinedVariable(v) => IfaError::UndefinedVariable(v),
            ifa_types::IfaError::UndefinedFunction(f) => IfaError::UndefinedFunction(f),
            ifa_types::IfaError::OponExhausted {
                requested,
                available,
            } => IfaError::OponExhausted {
                requested,
                available,
            },
            ifa_types::IfaError::AssertionFailed(a) => IfaError::AssertionFailed(a),
            ifa_types::IfaError::NotImplemented(n) => IfaError::NotImplemented(n),
            ifa_types::IfaError::Custom(c) => IfaError::Custom(c),
            ifa_types::IfaError::UserError(u) => IfaError::UserError(u),
            ifa_types::IfaError::Parse(p) => IfaError::Parse(p),
            ifa_types::IfaError::Compile(c) => IfaError::Compile(c),
            ifa_types::IfaError::Runtime(r) => IfaError::Runtime(r),
        }
    }
}

/// Error with location information for clear debugging
/// (Line number first - as suggested by code review)
#[derive(Debug)]
pub struct SpannedError {
    pub error: IfaError,
    pub line: usize,
    pub column: usize,
    pub file: Option<String>,
    pub source_line: Option<String>,
}

impl SpannedError {
    pub fn new(error: IfaError, line: usize, column: usize) -> Self {
        SpannedError {
            error,
            line,
            column,
            file: None,
            source_line: None,
        }
    }

    pub fn with_file(mut self, file: &str) -> Self {
        self.file = Some(file.to_string());
        self
    }

    pub fn with_source(mut self, source: &str) -> Self {
        self.source_line = Some(source.to_string());
        self
    }
}

impl std::fmt::Display for SpannedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // LINE NUMBER FIRST (Linus-approved format)
        let location = match &self.file {
            Some(file) => format!("{}:{}:{}", file, self.line, self.column),
            None => format!("line {}:{}", self.line, self.column),
        };

        writeln!(f, "ERROR at {}: {}", location, self.error)?;

        // Show source line with pointer
        if let Some(ref source) = self.source_line {
            writeln!(f, "  {} | {}", self.line, source)?;
            writeln!(
                f,
                "  {} | {}^",
                " ".repeat(self.line.to_string().len()),
                " ".repeat(self.column.saturating_sub(1))
            )?;
        }

        // Wisdom comes AFTER the useful stuff
        writeln!(f, "  Hint: {}", self.error.proverb())?;

        Ok(())
    }
}

impl std::error::Error for SpannedError {}

/// Helper to format errors the right way
pub fn format_error(
    error: &IfaError,
    file: &str,
    line: usize,
    col: usize,
    source: Option<&str>,
) -> String {
    let mut result = format!("ERROR at {}:{}:{}: {}\n", file, line, col, error);

    if let Some(src) = source {
        result.push_str(&format!("  {} | {}\n", line, src));
        result.push_str(&format!(
            "  {} | {}^\n",
            " ".repeat(line.to_string().len()),
            " ".repeat(col.saturating_sub(1))
        ));
    }

    result.push_str(&format!("  Hint: {}\n", error.proverb()));
    result
}
