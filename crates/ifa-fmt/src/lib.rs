//! # Ifá-Lang Formatter
//!
//! An opinionated code formatter for Ifá-Lang.
//! Uses a Token Stream approach to preserve comments.

use ifa_core::lexer::{Token, tokenize};

/// Formatting configuration
#[derive(Debug, Clone)]
pub struct FormatterConfig {
    pub indent_size: usize,
    pub max_width: usize,
}

impl Default for FormatterConfig {
    fn default() -> Self {
        Self {
            indent_size: 4,
            max_width: 100,
        }
    }
}

/// Format Ifá-Lang source code
pub fn format(source: &str, config: FormatterConfig) -> String {
    let tokens = tokenize(source);
    let mut formatted = String::with_capacity(source.len() * 2);
    let mut indent_level = 0;

    // Track if we just started a new line (to know if we need to indentation)
    let mut is_start_of_line = true;

    // Iterate through tokens
    let mut iter = tokens.into_iter().peekable();

    while let Some(spanned) = iter.next() {
        let token = spanned.value;
        let original_text = &source[spanned.span];

        // Handle indentation before printing token if at start of line
        if is_start_of_line {
            // Check if this token DECREASES indent (closing brace)
            if matches!(token, Token::RBrace) && indent_level > 0 {
                indent_level -= 1;
            }

            // Only print indentation if it's not a newline
            if !matches!(token, Token::Newline) {
                formatted.push_str(&" ".repeat(indent_level * config.indent_size));
            }
            is_start_of_line = false;
        }

        match token {
            Token::Newline => {
                formatted.push('\n');
                is_start_of_line = true;
            }
            Token::Comment(_c) => {
                formatted.push_str(original_text);
            }
            Token::LBrace => {
                formatted.push_str(" {");
                indent_level += 1;
                if let Some(next) = iter.peek() {
                    if !matches!(next.value, Token::Newline) {
                        formatted.push('\n');
                        is_start_of_line = true;
                    }
                }
            }
            Token::RBrace => {
                formatted.push('}');
                // Check what comes after the closing brace
                if let Some(next) = iter.peek() {
                    let next_text = source[next.span.clone()].trim();
                    if matches!(
                        next.value,
                        Token::Else | Token::Semicolon | Token::Comma | Token::RParen
                    ) || next_text == "else"
                        || next_text == "bibẹkọ"
                    {
                        // Don't add newline, it will continue on same line (e.g. `} else {`)
                        formatted.push(' ');
                    } else if !matches!(next.value, Token::Newline) {
                        formatted.push('\n');
                        is_start_of_line = true;
                    }
                }
            }
            Token::Semicolon => {
                formatted.push(';');
                if let Some(next) = iter.peek() {
                    if !matches!(next.value, Token::Newline) {
                        formatted.push('\n');
                        is_start_of_line = true;
                    }
                }
            }
            Token::Comma => {
                formatted.push(',');
                formatted.push(' ');
            }
            Token::Colon => {
                formatted.push_str(": ");
            }
            Token::Assign => {
                formatted.push_str(" = ");
            }
            Token::EqEq => formatted.push_str(" == "),
            Token::NotEq => formatted.push_str(" != "),
            Token::Lt => formatted.push_str(" < "),
            Token::Gt => formatted.push_str(" > "),
            Token::LtEq => formatted.push_str(" <= "),
            Token::GtEq => formatted.push_str(" >= "),
            Token::And => formatted.push_str(" && "),
            Token::Or => formatted.push_str(" || "),
            Token::Plus | Token::Minus | Token::Star | Token::Slash | Token::Percent => {
                // Spacing around math operators? "x+y" vs "x + y".
                // Linus prefers space.
                formatted.push(' ');
                formatted.push_str(original_text);
                formatted.push(' ');
            }
            Token::FatArrow => formatted.push_str(" => "),

            // Keywords
            Token::Let
            | Token::Const
            | Token::If
            | Token::Else
            | Token::For
            | Token::While
            | Token::Return
            | Token::Function
            | Token::Class
            | Token::Import
            | Token::Ebo
            | Token::Ewo
            | Token::Ase
            | Token::Match
            | Token::Pub
            | Token::Private => {
                formatted.push_str(original_text);
                formatted.push(' '); // Keywords usually followed by space
            }

            // Literals / Identifiers
            _ => {
                formatted.push_str(original_text);
                // Check next token to see if we need a space (e.g. ident followed by ident? shouldn't happen)
                // ident followed by brace?
                if let Some(next) = iter.peek() {
                    // Space before opening brace if not already handled
                    if matches!(next.value, Token::LBrace) && !matches!(token, Token::LParen) {
                        // e.g. "if x {" -> "if x {"
                    }
                }
            }
        }
    }

    // Post-processing: trim trailing spaces from each line
    let mut final_out = String::with_capacity(formatted.len());
    for line in formatted.lines() {
        final_out.push_str(line.trim_end());
        final_out.push('\n');
    }

    // Remove the very last newline if it wasn't in original?
    // Let's just return it as cleanly formatted.
    final_out.trim_end().to_string() + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_basic_if_else() {
        let input = "ti x>5{Irosu.fo(1);}else{Irosu.fo(2);}";
        let expected = "ti x > 5 {\n    Irosu.fo(1);\n} else {\n    Irosu.fo(2);\n}\n";
        assert_eq!(format(input, FormatterConfig::default()), expected);
    }

    #[test]
    fn test_format_nested_blocks() {
        let input = "ese main(){ti otito{Irosu.fo(\"yes\");}}";
        let expected = "ese main() {\n    ti otito {\n        Irosu.fo(\"yes\");\n    }\n}\n";
        assert_eq!(format(input, FormatterConfig::default()), expected);
    }

    #[test]
    fn test_format_preserves_comments() {
        let input = "ayanmo x = 1; # initial value\nx += 1;";
        let expected = "ayanmo x = 1; # initial value\nx += 1;\n";
        assert_eq!(format(input, FormatterConfig::default()), expected);
    }
}
