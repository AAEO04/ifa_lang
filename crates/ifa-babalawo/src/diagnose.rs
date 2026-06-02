//! # Babalawo Diagnosis
//!
//! The Babalawo (Priest) - Diagnoses errors with proverb-based messages.
//! Uses minimal output format by default.

use crate::wisdom::{ERROR_TO_ODU, ODU_WISDOM};
use std::fmt;

/// Severity level for diagnostics
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,   // Aṣiṣe - must fix
    Warning, // Ìkìlọ̀ - should fix
    Info,    // Ìmọ̀ràn - suggestion
    Style,   // Style recommendation
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Info => write!(f, "info"),
            Severity::Style => write!(f, "style"),
        }
    }
}

/// An Ifá error with Babalawo-style messaging
#[derive(Debug, Clone)]
pub struct LintError {
    pub code: String,
    pub message: String,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub span: Option<ifa_types::ast::Span>,
    pub context: Option<String>,
    pub notes: Vec<String>,
}

impl LintError {
    pub fn new(code: &str, message: &str, file: &str, line: usize, column: usize) -> Self {
        Self {
            code: code.to_string(),
            message: message.to_string(),
            file: file.to_string(),
            line,
            column,
            span: None,
            context: None,
            notes: Vec::new(),
        }
    }

    pub fn with_span(mut self, span: ifa_types::ast::Span) -> Self {
        self.span = Some(span);
        self
    }

    pub fn with_context(mut self, ctx: &str) -> Self {
        self.context = Some(ctx.to_string());
        self
    }

    pub fn with_note(mut self, note: &str) -> Self {
        self.notes.push(note.to_string());
        self
    }
}

/// A diagnostic message with severity and wisdom
#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub error: LintError,
    pub odu: String,
    pub wisdom: Option<String>,
}

/// The Babalawo - Error diagnosis system
pub struct Babalawo {
    pub diagnostics: Vec<Diagnostic>,
    pub verbose: bool,
    pub include_wisdom: bool,
}

impl Default for Babalawo {
    fn default() -> Self {
        Self::new()
    }
}

impl Babalawo {
    pub fn new() -> Self {
        Self {
            diagnostics: Vec::new(),
            verbose: false,
            include_wisdom: true,
        }
    }

    /// Add a diagnostic with a full span
    pub fn add_full(
        &mut self,
        severity: Severity,
        code: &str,
        msg: &str,
        file: &str,
        span: ifa_types::ast::Span,
    ) {
        let odu_key = ERROR_TO_ODU.get(code).copied().unwrap_or("OKANRAN");

        let wisdom = if self.include_wisdom {
            ODU_WISDOM.get(odu_key).map(|w| w.advice.to_string())
        } else {
            None
        };

        let diagnostic = Diagnostic {
            severity,
            error: LintError::new(code, msg, file, span.line, span.column).with_span(span),
            odu: odu_key.to_string(),
            wisdom,
        };

        self.diagnostics.push(diagnostic);
    }

    pub fn verbose(mut self) -> Self {
        self.verbose = true;
        self
    }

    /// Add a diagnostic with a full span and additional context data
    pub fn add_with_context(
        &mut self,
        severity: Severity,
        code: &str,
        msg: &str,
        file: &str,
        span: ifa_types::ast::Span,
        context: &str,
    ) {
        let odu_key = ERROR_TO_ODU.get(code).copied().unwrap_or("OKANRAN");

        let wisdom = if self.include_wisdom {
            ODU_WISDOM.get(odu_key).map(|w| w.advice.to_string())
        } else {
            None
        };

        let diagnostic = Diagnostic {
            severity,
            error: LintError::new(code, msg, file, span.line, span.column)
                .with_span(span)
                .with_context(context),
            odu: odu_key.to_string(),
            wisdom,
        };

        self.diagnostics.push(diagnostic);
    }

    /// fast mode disables wisdom generation
    pub fn fast(mut self) -> Self {
        self.include_wisdom = false;
        self
    }

    /// Add an error diagnostic
    pub fn error(&mut self, code: &str, msg: &str, file: &str, line: usize, col: usize) {
        self.add_diagnostic(Severity::Error, code, msg, file, line, col);
    }

    /// Add a warning diagnostic
    pub fn warning(&mut self, code: &str, msg: &str, file: &str, line: usize, col: usize) {
        self.add_diagnostic(Severity::Warning, code, msg, file, line, col);
    }

    /// Add an info diagnostic
    pub fn info(&mut self, code: &str, msg: &str, file: &str, line: usize, col: usize) {
        self.add_diagnostic(Severity::Info, code, msg, file, line, col);
    }

    fn add_diagnostic(
        &mut self,
        severity: Severity,
        code: &str,
        msg: &str,
        file: &str,
        line: usize,
        col: usize,
    ) {
        let odu_key = ERROR_TO_ODU.get(code).copied().unwrap_or("OKANRAN");

        let wisdom = if self.include_wisdom {
            ODU_WISDOM.get(odu_key).map(|w| w.advice.to_string())
        } else {
            None
        };

        let diagnostic = Diagnostic {
            severity,
            error: LintError::new(code, msg, file, line, col),
            odu: odu_key.to_string(),
            wisdom,
        };

        self.diagnostics.push(diagnostic);
    }

    /// Get error count
    pub fn error_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count()
    }

    /// Get warning count
    pub fn warning_count(&self) -> usize {
        self.diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .count()
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        self.error_count() > 0
    }

    /// Format output (minimal by default)
    pub fn format(&self) -> String {
        let mut output = String::new();

        if self.diagnostics.is_empty() {
            return "No issues found. Àṣẹ!\n".to_string();
        }

        output.push_str("babalawo:\n\n");

        for diag in &self.diagnostics {
            let odu_name = ODU_WISDOM
                .get(diag.odu.as_str())
                .map(|w| w.name)
                .unwrap_or("Ọ̀kànràn");

            // Minimal format: error[Odu] file:line:col
            output.push_str(&format!(
                "{}[{}] {}:{}:{}\n",
                diag.severity, odu_name, diag.error.file, diag.error.line, diag.error.column
            ));

            // Message
            output.push_str(&format!("  {}\n", diag.error.message));

            // Wisdom (only if verbose or it's an error)
            if (self.verbose || diag.severity == Severity::Error)
                && let Some(wisdom) = &diag.wisdom
            {
                output.push_str(&format!("  Wisdom: {}\n", wisdom));
            }

            output.push('\n');
        }

        // Summary
        let errors = self.error_count();
        let warnings = self.warning_count();
        output.push_str(&format!(
            "{} error{}, {} warning{}. Àṣẹ!\n",
            errors,
            if errors == 1 { "" } else { "s" },
            warnings,
            if warnings == 1 { "" } else { "s" }
        ));

        output
    }

    /// Format as JSON for IDE integration
    pub fn format_json(&self) -> String {
        let items: Vec<serde_json::Value> = self
            .diagnostics
            .iter()
            .map(|d| {
                serde_json::json!({
                    "severity": d.severity.to_string(),
                    "code": d.error.code,
                    "message": d.error.message,
                    "file": d.error.file,
                    "line": d.error.line,
                    "column": d.error.column,
                    "odu": d.odu,
                })
            })
            .collect();

        serde_json::to_string_pretty(&items).unwrap_or_default()
    }

    /// Format compact (one line per error)
    pub fn format_compact(&self) -> String {
        self.diagnostics
            .iter()
            .map(|d| {
                format!(
                    "{}:{}:{}: {}: {}\n",
                    d.error.file, d.error.line, d.error.column, d.severity, d.error.message
                )
            })
            .collect()
    }

    /// Format with source-code snippets, carets, and ANSI colors.
    /// Pass the original source text to render the offending lines.
    pub fn format_with_source(&self, source: &str) -> String {
        let source_lines: Vec<&str> = source.lines().collect();
        let mut output = String::new();

        if self.diagnostics.is_empty() {
            return "\x1b[1mNo issues found. Àṣẹ!\x1b[0m\n".to_string();
        }

        output.push_str("\x1b[1mbabalawo:\x1b[0m\n\n");
        let reset = "\x1b[0m";

        for diag in &self.diagnostics {
            let odu_name = ODU_WISDOM
                .get(diag.odu.as_str())
                .map(|w| w.name)
                .unwrap_or("Ọ̀kànràn");

            let color = match diag.severity {
                Severity::Error => "\x1b[31m",
                Severity::Warning => "\x1b[33m",
                _ => "\x1b[36m",
            };

            // Severity + code
            output.push_str(&format!(
                "{}{}[{}]{} {}:{}:{}\n",
                color,
                diag.severity,
                odu_name,
                reset,
                diag.error.file,
                diag.error.line,
                diag.error.column,
            ));

            // Source annotation
            let line_i = diag.error.line.max(1) - 1;
            if line_i < source_lines.len() {
                // Location arrow
                output.push_str(&format!(
                    " {} {}:{}:{}\n",
                    "\x1b[36m-->\x1b[0m", diag.error.file, diag.error.line, diag.error.column
                ));
                output.push_str("   \x1b[36m|\x1b[0m\n");

                // Context lines: show line before, error line, line after
                let context_start = if line_i > 0 { line_i - 1 } else { line_i };
                let context_end = (line_i + 2).min(source_lines.len());

                // Calculate line number width for alignment
                let line_width = (context_end + 1).to_string().len();

                for i in context_start..context_end {
                    let is_error_line = i == line_i;
                    let lineno = i + 1;
                    let marker = if is_error_line {
                        format!(
                            "\x1b[{}m >\x1b[0m",
                            if diag.severity == Severity::Error {
                                "31"
                            } else {
                                "33"
                            }
                        )
                    } else {
                        "   ".to_string()
                    };
                    let gutter = format!("{:>width$}", lineno, width = line_width);
                    output.push_str(&format!(
                        "{}{} {} {}\n",
                        marker, "\x1b[36m|\x1b[0m", gutter, source_lines[i]
                    ));

                    // Caret line
                    if is_error_line {
                        let col = diag
                            .error
                            .column
                            .max(1)
                            .min(source_lines[i].len().saturating_add(1));
                        let padding =
                            " ".repeat(gutter.len().saturating_add(col).saturating_add(1));
                        let caret = format!(
                            "{}{}{}^{}{} {}",
                            "  ", "\x1b[36m|\x1b[0m", padding, color, reset, diag.error.message
                        );
                        output.push_str(&format!("{}\n", caret));
                    }
                }
                output.push_str(&format!("   \x1b[36m|\x1b[0m\n"));
            }

            // Notes
            for note in &diag.error.notes {
                output.push_str(&format!(
                    "    {} {}: {}\n",
                    "\x1b[36m=\x1b[0m", "\x1b[1mnote\x1b[0m", note
                ));
            }

            // Wisdom
            if (self.verbose || diag.severity == Severity::Error)
                && let Some(wisdom) = &diag.wisdom
            {
                output.push_str(&format!(
                    "    {} {}: {}\n",
                    "\x1b[36m=\x1b[0m", "\x1b[1mọ̀rọ̀\x1b[0m", wisdom
                ));
            }

            output.push('\n');
        }

        // Summary
        let errors = self.error_count();
        let warnings = self.warning_count();
        let summary_color = if errors > 0 { "\x1b[31m" } else { "\x1b[32m" };
        output.push_str(&format!(
            "{}{} error{}, {} warning{}.{} Àṣẹ!\n",
            summary_color,
            errors,
            if errors == 1 { "" } else { "s" },
            warnings,
            if warnings == 1 { "" } else { "s" },
            reset,
        ));

        output
    }
}

impl fmt::Display for Babalawo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_babalawo_error() {
        let mut baba = Babalawo::new();
        baba.error(
            "UNDEFINED_VARIABLE",
            "Variable 'x' not defined",
            "test.ifa",
            10,
            5,
        );

        assert_eq!(baba.error_count(), 1);
        assert!(baba.has_errors());

        let output = baba.format();
        assert!(output.contains("error[Ogbè]"));
        assert!(output.contains("Variable 'x' not defined"));
    }

    #[test]
    fn test_babalawo_warning() {
        let mut baba = Babalawo::new();
        baba.warning(
            "UNUSED_VARIABLE",
            "Variable 'temp' is unused",
            "test.ifa",
            5,
            10,
        );

        assert_eq!(baba.warning_count(), 1);
        assert!(!baba.has_errors());
    }

    #[test]
    fn test_compact_format() {
        let mut baba = Babalawo::new();
        baba.error(
            "DIVISION_BY_ZERO",
            "Cannot divide by zero",
            "math.ifa",
            25,
            15,
        );

        let compact = baba.format_compact();
        assert!(compact.contains("math.ifa:25:15: error: Cannot divide by zero"));
    }
}
