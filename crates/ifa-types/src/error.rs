//! # Ifá Error Types (Ọ̀kànràn)
//!
//! THE canonical error type for all Ifá-Lang runtimes.
//! Every other crate re-exports this — do NOT define parallel error enums.

use crate::ErrorCode;
use thiserror::Error;

/// Result type alias for Ifá operations
pub type IfaResult<T> = Result<T, IfaError>;

/// Core error type for Ifá-Lang runtime
///
/// This is the ONE error type. ifa-core, ifa-std, ifa-cli all re-export it.
/// If you need a different error shape (lint diagnostics, installer errors),
/// use a different type name — never another `IfaError`.
#[derive(Error, Debug, Clone)]
pub enum IfaError {
    // =========================================================================
    // MATH ERRORS (Ọ̀bàrà / Òtúúrúpọ̀n)  — ErrorCode 0x03XX
    // =========================================================================
    #[error("Division by zero - Ọ̀bàrà rejects: {0}")]
    DivisionByZero(String),

    #[error("Numeric overflow: {0}")]
    Overflow(String),

    #[error("Numeric underflow: {0}")]
    Underflow(String),

    // =========================================================================
    // TYPE ERRORS  — ErrorCode 0x02XX
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
    // INDEX/KEY ERRORS (Ògúndá / Ìká)  — ErrorCode 0x01XX
    // =========================================================================
    #[error("Index out of bounds: {index} (length: {length})")]
    IndexOutOfBounds { index: i64, length: usize },

    #[error("Key not found: {0}")]
    KeyNotFound(String),

    // =========================================================================
    // I/O ERRORS (Òdí / Ìrosù)  — ErrorCode 0x04XX
    // =========================================================================
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("I/O error: {0}")]
    IoError(String), // String, not std::io::Error — Clone requires it

    // =========================================================================
    // NETWORK ERRORS (Òtúrá)  — ErrorCode 0x04XX
    // =========================================================================
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Timeout after {0}ms")]
    Timeout(u64),

    #[error("SSRF blocked: {0}")]
    SsrfBlocked(String),

    // =========================================================================
    // VM ERRORS  — ErrorCode 0x00XX
    // =========================================================================
    #[error("Native registry is not attached. Cannot invoke FFI: {0}")]
    RegistryNotAttached(String),

    #[error("Execution yielded")]
    Yielded,

    #[error("Unknown opcode: {0}")]
    UnknownOpcode(u8),

    #[error("Stack underflow")]
    StackUnderflow,

    #[error("Async not available: {0}")]
    AsyncNotAvailable(String),

    #[error("stack overflow — program declared #opon {directive:?} (limit: {limit} slots)")]
    StackOverflow {
        limit: usize,
        directive: crate::bytecode::OponSize,
    },

    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),

    #[error("Undefined function: {0}")]
    UndefinedFunction(String),

    // =========================================================================
    // MEMORY ERRORS (Opon)  — ErrorCode 0x01XX
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

    /// User-raised error via `ta` (throw). Carries the original thrown value
    /// unchanged — may be a String, Map, List, or any other IfaValue.
    /// §12.2: `ta` accepts any expression, not just strings.
    #[error("User error: {0}")]
    UserError(Box<crate::IfaValue>),

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

// Preserve `?` from std::io::Error → IfaError (manual impl since we use String)
impl From<std::io::Error> for IfaError {
    fn from(e: std::io::Error) -> Self {
        IfaError::IoError(e.to_string())
    }
}

impl IfaError {
    /// Get the standard numeric error code for this error.
    ///
    /// These codes are stable across all runtimes (VM, embedded, FFI).
    /// See `ifa_bytecode::ErrorCode` for the full taxonomy.
    pub fn error_code(&self) -> ErrorCode {
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
            IfaError::StackOverflow { .. } => ErrorCode::StackOverflow,
            IfaError::AsyncNotAvailable(_) => ErrorCode::AsyncNotAvailable,

            IfaError::UndefinedVariable(_) | IfaError::UndefinedFunction(_) => {
                ErrorCode::UndefinedVar
            }
            IfaError::OponExhausted { .. } => ErrorCode::OutOfMemory,

            IfaError::Parse(_) | IfaError::Compile(_) => ErrorCode::InvalidBytecode,

            // Generic mappings — these need finer codes in future
            IfaError::ArityMismatch { .. }
            | IfaError::ArgumentError(_)
            | IfaError::AssertionFailed(_)
            | IfaError::NotImplemented(_)
            | IfaError::Custom(_)
            | IfaError::UserError(_)
            | IfaError::Yielded
            | IfaError::RegistryNotAttached(_)
            | IfaError::Runtime(_) => ErrorCode::VmError,
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
            IfaError::UserError(_) => {
                "Ọwọ́ ara ẹni la fí tún ìwà ara ẹni ṣe. (One shapes their own character with their own hands.)"
            }
            _ => "Gbogbo ìṣòro ní ojúùtù. (Every problem has a solution.)",
        }
    }

    /// If this is a `UserError`, extract the inner `IfaValue`.
    /// Used by the VM's catch handler to bind the error variable.
    pub fn user_value(&self) -> Option<&crate::IfaValue> {
        if let IfaError::UserError(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

/// Error with location information for clear debugging
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
        let location = match &self.file {
            Some(file) => format!("{}:{}:{}", file, self.line, self.column),
            None => format!("line {}:{}", self.line, self.column),
        };

        writeln!(f, "ERROR at {}: {}", location, self.error)?;

        if let Some(ref source) = self.source_line {
            writeln!(f, "  {} | {}", self.line, source)?;
            writeln!(
                f,
                "  {} | {}^",
                " ".repeat(self.line.to_string().len()),
                " ".repeat(self.column.saturating_sub(1))
            )?;
        }

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
