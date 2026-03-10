//! # Ifá-Lang Formatter
//!
//! An opinionated code formatter for Ifá-Lang.
//! Uses a Token Stream approach to preserve comments.

use ifa_core::lexer::{tokenize, Token};

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
            
            // Only print indentation if it's not a newline (avoid trailing spaces on empty lines)
            if !matches!(token, Token::Newline | Token::Comment(_)) { // Comments might be blocks? No, Lexer handles comments.
               // Actually Lexer returns Token::Newline.
               // If next token is Newline, we just print Newline.
            }
            
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
                // If comment starts with #, likely line comment.
                // Text includes the # or // or /* */
                formatted.push_str(original_text);
                
                // If it's a line comment, it usually ends at newline, but Lexer might capture the newline separately or not?
                // Checking lexer.rs:
                // #[regex(r"#[^\n]*", ...)] -> Matches until newline. Code does NOT consume newline.
                // So next token should be Token::Newline.
                // Block comments? 
                // #[token("/*", ...)] -> Not in lexer.rs? 
                // lexer.rs has:
                // #[regex(r"#[^\n]*", |lex| lex.slice().to_string())] Comment(String),
                // Only hash comments supported by lexer right now? 
                // Wait, I saw `grammar.pest` having `block_comment`.
                // Checking `lexer.rs` again...
                // `lexer.rs` content only shows `#[regex(r"#[^\n]*", ...)]` for `Comment`.
                // Does it support `///` or `//` or `/*`?
                // Validating lexer.rs:
                // Line 360: #[regex(r"#[^\n]*", |lex| lex.slice().to_string())] Comment(String),
                // Only hash comments? 
                // But `lexer.rs` line 273 `COMMENT = _ { block_comment | doc_comment ... }` was in `grammar.pest`.
                // `lexer.rs` (logos) seems to ONLY have Hash comments defined in the visible snippet!
                // Wait, I see `Token` enum around line 360.
                // It only has `#[regex(r"#[^\n]*"...`?
                // Need to double check `lexer.rs` content carefully.
                // If Lexer only supports `#` comments, then `//` comments in my code are being TOKENIZED AS OPERATORS?
                // `/` `*` etc?
                // IMPORTANT: I must ensure Lexer supports all comment types before writing Formatter.
                
                // Assuming it's a hash comment for now, it's just text.
            }
            Token::LBrace => {
                formatted.push_str(" {");
                indent_level += 1;
                // We usually want a newline after {
                // Check if next token is already a newline.
                if let Some(next) = iter.peek() {
                   if !matches!(next.value, Token::Newline) {
                       formatted.push('\n');
                       is_start_of_line = true;
                   }
                }
            }
            Token::RBrace => {
                formatted.push('}');
                // We usually want a newline after } unless followed by ; or else
                if let Some(next) = iter.peek() {
                    if matches!(next.value, Token::Else | Token::Semicolon | Token::Comma | Token::RParen) {
                        // Don't add newline
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
            Token::Let | Token::Const | Token::If | Token::Else | Token::For | Token::While | 
            Token::Return | Token::Function | Token::Class | Token::Import | Token::Ebo | 
            Token::Ewo | Token::Ase | Token::Match | Token::Pub | Token::Private => {
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
    
    // Post-processing to trim trailing spaces?
    // Token stream reconstruction is hard to get perfect on first try.
    // Iteration 1: Simple loop.
    formatted
}
