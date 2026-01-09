//! # Ìwà Pẹ̀lẹ́ - Gentle Character / Graceful Degradation (v2)
//!
//! Error handling philosophy based on Yoruba concept of "gentle character".
//! Includes both runtime graceful degradation AND compile-time guidance.
//!
//! Use `#[iwa_pele]` macro from `ifa-macros` for compile-time balance checking.

use std::error::Error;
use std::fmt;

/// Result type with Ìwà Pẹ̀lẹ́ error handling
pub type IwaPeleResult<T> = Result<T, IwaPeleError>;

/// Error with graceful degradation and Yoruba proverbs
#[derive(Debug, Clone)]
pub struct IwaPeleError {
    pub kind: IwaPeleErrorKind,
    pub message: String,
    pub proverb: &'static str,
    pub suggestion: Option<String>,
}

impl IwaPeleError {
    pub fn new(kind: IwaPeleErrorKind, message: impl Into<String>) -> Self {
        IwaPeleError {
            kind,
            message: message.into(),
            proverb: kind.proverb(),
            suggestion: None,
        }
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Create a "missing" error with fallback
    pub fn missing(what: &str) -> Self {
        Self::new(IwaPeleErrorKind::Missing, format!("{} not found", what))
    }

    /// Create a "timeout" error
    pub fn timeout(operation: &str) -> Self {
        Self::new(
            IwaPeleErrorKind::Timeout,
            format!("{} timed out", operation),
        )
    }
}

impl fmt::Display for IwaPeleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[Iwa Pele] {}: {}\n  Proverb: {}",
            self.kind, self.message, self.proverb
        )?;
        if let Some(ref suggestion) = self.suggestion {
            write!(f, "\n  Hint: {}", suggestion)?;
        }
        Ok(())
    }
}

impl Error for IwaPeleError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IwaPeleErrorKind {
    Missing,
    TypeMismatch,
    OperationFailed,
    Unavailable,
    ValidationFailed,
    Timeout,
    PermissionDenied,
    Imbalanced, // For compile-time balance violations
}

impl IwaPeleErrorKind {
    pub const fn proverb(&self) -> &'static str {
        match self {
            Self::Missing => "Ohun tí a wá kò sí, ṣùgbọ́n ohun mìíràn wà.",
            Self::TypeMismatch => "A kì í fi ẹja wẹ́ ọmọ.",
            Self::OperationFailed => "Bí a bá ṣubú, a tún dìde.",
            Self::Unavailable => "Ọ̀nà kan kò wọ ọjà.",
            Self::ValidationFailed => "Ẹni tó mọ̀wé máà kó ẹ̀sìn.",
            Self::Timeout => "Sùúrù ni baba ìwà.",
            Self::PermissionDenied => "Àṣẹ ológun kò ju ti Ọlọ́run lọ.",
            Self::Imbalanced => "Ohun tí a ṣí, a gbọdọ̀ pa.",
        }
    }
}

impl fmt::Display for IwaPeleErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::Missing => "Missing",
            Self::TypeMismatch => "Type Mismatch",
            Self::OperationFailed => "Operation Failed",
            Self::Unavailable => "Unavailable",
            Self::ValidationFailed => "Validation Failed",
            Self::Timeout => "Timeout",
            Self::PermissionDenied => "Permission Denied",
            Self::Imbalanced => "Imbalanced",
        };
        write!(f, "{}", name)
    }
}

/// Trait for graceful degradation
pub trait IwaPele<T> {
    /// Get value or use fallback with logging
    fn or_gentle(self, fallback: T) -> T;

    /// Get value or recover via function
    fn or_recover<F: FnOnce() -> T>(self, f: F) -> T;

    /// Convert to IwaPeleResult
    fn gentle(self) -> IwaPeleResult<T>;
}

impl<T> IwaPele<T> for Option<T> {
    #[inline]
    fn or_gentle(self, fallback: T) -> T {
        self.unwrap_or(fallback)
    }

    #[inline]
    fn or_recover<F: FnOnce() -> T>(self, f: F) -> T {
        self.unwrap_or_else(f)
    }

    fn gentle(self) -> IwaPeleResult<T> {
        self.ok_or_else(|| IwaPeleError::missing("value"))
    }
}

impl<T, E: fmt::Debug> IwaPele<T> for Result<T, E> {
    #[inline]
    fn or_gentle(self, fallback: T) -> T {
        self.unwrap_or(fallback)
    }

    #[inline]
    fn or_recover<F: FnOnce() -> T>(self, f: F) -> T {
        self.unwrap_or_else(|_| f())
    }

    fn gentle(self) -> IwaPeleResult<T> {
        self.map_err(|e| IwaPeleError::new(IwaPeleErrorKind::OperationFailed, format!("{:?}", e)))
    }
}

/// Retry operation with exponential backoff
pub fn with_patience<T, E, F>(max_attempts: usize, mut f: F) -> IwaPeleResult<T>
where
    F: FnMut() -> Result<T, E>,
    E: fmt::Debug,
{
    let mut delay_ms = 50;
    let mut attempts = 0;

    loop {
        attempts += 1;
        match f() {
            Ok(v) => return Ok(v),
            Err(e) if attempts < max_attempts => {
                std::thread::sleep(std::time::Duration::from_millis(delay_ms));
                delay_ms = (delay_ms * 2).min(5000);
            }
            Err(e) => {
                return Err(IwaPeleError::new(
                    IwaPeleErrorKind::OperationFailed,
                    format!("After {} attempts: {:?}", attempts, e),
                )
                .with_suggestion("Consider alternative approach"));
            }
        }
    }
}

/// Circuit breaker for preventing cascade failures
pub struct CircuitBreaker {
    name: &'static str,
    failure_threshold: usize,
    failures: std::sync::atomic::AtomicUsize,
    open: std::sync::atomic::AtomicBool,
}

impl CircuitBreaker {
    pub const fn new(name: &'static str, threshold: usize) -> Self {
        CircuitBreaker {
            name,
            failure_threshold: threshold,
            failures: std::sync::atomic::AtomicUsize::new(0),
            open: std::sync::atomic::AtomicBool::new(false),
        }
    }

    pub fn call<T, E: fmt::Debug, F: FnOnce() -> Result<T, E>>(&self, f: F) -> IwaPeleResult<T> {
        use std::sync::atomic::Ordering::SeqCst;

        if self.open.load(SeqCst) {
            return Err(IwaPeleError::new(
                IwaPeleErrorKind::Unavailable,
                format!("Circuit '{}' is open", self.name),
            ));
        }

        match f() {
            Ok(v) => {
                self.failures.store(0, SeqCst);
                Ok(v)
            }
            Err(e) => {
                let failures = self.failures.fetch_add(1, SeqCst) + 1;
                if failures >= self.failure_threshold {
                    self.open.store(true, SeqCst);
                }
                Err(IwaPeleError::new(
                    IwaPeleErrorKind::OperationFailed,
                    format!("{:?}", e),
                ))
            }
        }
    }

    pub fn reset(&self) {
        use std::sync::atomic::Ordering::SeqCst;
        self.failures.store(0, SeqCst);
        self.open.store(false, SeqCst);
    }

    pub fn is_open(&self) -> bool {
        self.open.load(std::sync::atomic::Ordering::SeqCst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_option_or_gentle() {
        let x: Option<i32> = None;
        assert_eq!(x.or_gentle(42), 42);

        let y: Option<i32> = Some(10);
        assert_eq!(y.or_gentle(42), 10);
    }

    #[test]
    fn test_result_gentle() {
        let ok: Result<i32, &str> = Ok(42);
        assert!(ok.gentle().is_ok());

        let err: Result<i32, &str> = Err("fail");
        assert!(err.gentle().is_err());
    }

    #[test]
    fn test_circuit_breaker() {
        static CB: CircuitBreaker = CircuitBreaker::new("test", 2);
        CB.reset();

        let _: IwaPeleResult<()> = CB.call(|| Err::<(), _>("fail"));
        assert!(!CB.is_open());

        let _: IwaPeleResult<()> = CB.call(|| Err::<(), _>("fail"));
        assert!(CB.is_open());

        let result: IwaPeleResult<i32> = CB.call(|| Ok(42));
        assert!(result.is_err());
    }
}
