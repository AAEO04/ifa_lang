//! # Ifá Error Types (Ọ̀kànràn)
//!
//! Structured error handling with Yoruba proverbs for educational context.

use thiserror::Error;

/// Result type alias for Ifá operations
pub type IfaResult<T> = Result<T, IfaError>;

/// Core error type for Ifá-Lang runtime
#[derive(Error, Debug)]
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
    IoError(#[from] std::io::Error),

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

    // =========================================================================
    // PARSER/INTERPRETER ERRORS
    // =========================================================================
    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Runtime error: {0}")]
    Runtime(String),
}

impl IfaError {
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
