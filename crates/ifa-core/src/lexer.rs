//! # Ifá-Lang Lexer
//!
//! Tokenizer for Ifá-Lang source code using logos.
//! Handles Yoruba diacritics and ASCII aliases.

use logos::{Lexer, Logos};
use std::fmt;

/// Normalize Yoruba text to ASCII for matching
fn normalize_yoruba(text: &str) -> String {
    text.to_lowercase()
        .replace('ẹ', "e")
        .replace('ọ', "o")
        .replace('ṣ', "s")
        .replace('à', "a")
        .replace('á', "a")
        .replace('è', "e")
        .replace('é', "e")
        .replace('ì', "i")
        .replace('í', "i")
        .replace('ò', "o")
        .replace('ó', "o")
        .replace('ù', "u")
        .replace('ú', "u")
        .replace('̀', "")
        .replace('́', "")
        .replace('̣', "")
}

/// Check if identifier is an Odù domain (supports both Yoruba and English)
fn check_domain(lex: &mut Lexer<Token>) -> Option<OduDomain> {
    let slice = lex.slice();
    let normalized = normalize_yoruba(slice);
    let lower = slice.to_lowercase();

    match normalized.as_str() {
        // Yoruba names
        "ogbe" => Some(OduDomain::Ogbe),
        "oyeku" => Some(OduDomain::Oyeku),
        "iwori" => Some(OduDomain::Iwori),
        "odi" => Some(OduDomain::Odi),
        "irosu" => Some(OduDomain::Irosu),
        "owonrin" => Some(OduDomain::Owonrin),
        "obara" => Some(OduDomain::Obara),
        "okanran" => Some(OduDomain::Okanran),
        "ogunda" => Some(OduDomain::Ogunda),
        "osa" => Some(OduDomain::Osa),
        "ika" => Some(OduDomain::Ika),
        "oturupon" => Some(OduDomain::Oturupon),
        "otura" => Some(OduDomain::Otura),
        "irete" => Some(OduDomain::Irete),
        "ose" => Some(OduDomain::Ose),
        "ofun" => Some(OduDomain::Ofun),
        "opele" => Some(OduDomain::Opele),
        _ => {
            // Standard programming aliases (max 2 per domain - keep it simple!)
            match lower.as_str() {
                // Ogbe (1111) - System/Lifecycle
                "sys" | "os" => Some(OduDomain::Ogbe),

                // Oyeku (0000) - Exit/Cleanup
                "exit" => Some(OduDomain::Oyeku),

                // Iwori (0110) - Time/DateTime
                "time" | "datetime" => Some(OduDomain::Iwori),

                // Odi (1001) - File I/O
                "fs" | "io" => Some(OduDomain::Odi),

                // Irosu (1100) - Console/Logging
                "fmt" | "log" => Some(OduDomain::Irosu),

                // Owonrin (0011) - Random
                "rand" => Some(OduDomain::Owonrin),

                // Obara (1000) - Math+
                "math" => Some(OduDomain::Obara),

                // Okanran (0001) - Errors
                "err" | "panic" => Some(OduDomain::Okanran),

                // Ogunda (1110) - Collections
                "vec" | "list" => Some(OduDomain::Ogunda),

                // Osa (0111) - Concurrency
                "async" | "thread" => Some(OduDomain::Osa),

                // Ika (0100) - Strings
                "str" | "string" => Some(OduDomain::Ika),

                // Oturupon (0010) - Division
                "div" => Some(OduDomain::Oturupon),

                // Otura (1011) - Networking
                "net" | "http" => Some(OduDomain::Otura),

                // Irete (1101) - Crypto
                "crypto" | "hash" => Some(OduDomain::Irete),

                // Ose (1010) - Terminal UI
                "tui" | "term" => Some(OduDomain::Ose),

                // Ofun (0101) - Permissions
                "perm" | "auth" => Some(OduDomain::Ofun),

                // Opele - Divination/Compound Odù
                "opele" | "oracle" => Some(OduDomain::Opele),

                // Coop - FFI Bridge
                "ffi" | "bridge" => Some(OduDomain::Coop),

                // Infrastructure Layer
                "cpu" | "parallel" => Some(OduDomain::Cpu),
                "gpu" | "compute" => Some(OduDomain::Gpu),
                "storage" | "kv" | "db" => Some(OduDomain::Storage),
                "ohun" | "audio" | "sound" => Some(OduDomain::Ohun),
                "fidio" | "video" | "media" => Some(OduDomain::Fidio),

                // Application Stacks
                "backend" | "server" => Some(OduDomain::Backend),
                "frontend" | "html" | "web" => Some(OduDomain::Frontend),
                "ml" | "tensor" | "ai" => Some(OduDomain::Ml),
                "gamedev" | "game" | "engine" => Some(OduDomain::GameDev),
                "iot" | "gpio" | "embedded" => Some(OduDomain::Iot),

                _ => None,
            }
        }
    }
}

/// The 16 Odù domains + Infrastructure + Stacks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OduDomain {
    // Core 16 Odù
    Ogbe,     // 1111 - Lifecycle
    Oyeku,    // 0000 - Exit/Sleep
    Iwori,    // 0110 - Time
    Odi,      // 1001 - Files
    Irosu,    // 1100 - Console
    Owonrin,  // 0011 - Random
    Obara,    // 1000 - Math+
    Okanran,  // 0001 - Errors
    Ogunda,   // 1110 - Arrays
    Osa,      // 0111 - Concurrency
    Ika,      // 0100 - Strings
    Oturupon, // 0010 - Math-
    Otura,    // 1011 - Network
    Irete,    // 1101 - Crypto
    Ose,      // 1010 - UI
    Ofun,     // 0101 - Permissions

    // Pseudo-domains
    Coop,  // Co-op / Àjọṣe - FFI Bridge
    Opele, // Ọpẹlẹ - Divination/Compound Odù

    // Infrastructure Layer
    Cpu,     // Parallel computing (rayon)
    Gpu,     // GPU compute (wgpu)
    Storage, // Key-value store (OduStore)
    Ohun,    // Audio I/O (rodio)
    Fidio,   // Video I/O (ffmpeg)

    // Application Stacks
    Backend,  // HTTP server, ORM
    Frontend, // HTML, CSS generation
    Crypto,   // Hashing, encryption (extends Irete)
    Ml,       // Machine learning, tensors
    GameDev,  // Game engine, ECS
    Iot,      // Embedded, GPIO
}

/// Token types for Ifá-Lang
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r]+")] // Skip whitespace
pub enum Token {
    // ═══════════════════════════════════════════════════════════════════════
    // KEYWORDS (Reserved)
    // ═══════════════════════════════════════════════════════════════════════

    // Variable declaration
    #[token("ayanmo")]
    #[token("àyànmọ́")]
    #[token("let")]
    Let,

    #[token("const")]
    Const,

    // Control flow
    #[token("ti")]
    #[token("bí")]
    #[token("if")]
    If,

    #[token("bibẹkọ")]
    #[token("bíbẹ́kọ́")]
    #[token("else")]
    Else,

    #[token("fun")]
    #[token("for")]
    For,

    #[token("nigba")]
    #[token("while")]
    While,

    #[token("pada")]
    #[token("return")]
    Return,

    #[token("da")]
    #[token("break")]
    Break,

    #[token("continue")]
    Continue,

    #[token("yàn")]
    #[token("yán")]
    #[token("match")]
    Match,

    // Function/class
    #[token("ese")]
    #[token("ẹsẹ")]
    #[token("fn")]
    #[token("function")]
    #[token("def")]
    Function,

    #[token("odu")]
    #[token("ọdù")]
    #[token("class")]
    Class,

    #[token("iba")]
    #[token("ìbà")]
    #[token("import")]
    Import,

    // Boolean
    #[token("otito")]
    #[token("true")]
    True,

    #[token("iro")]
    #[token("false")]
    False,

    #[token("ohunkohun")]
    #[token("nil")]
    #[token("null")]
    Nil,

    // CEN Model
    #[token("ebo")]
    #[token("ẹbọ")]
    #[token("sacrifice")]
    Ebo,

    #[token("ewo")]
    #[token("ẹ̀wọ̀")]
    #[token("assert")]
    #[token("verify")]
    Ewo,

    #[token("ajose")]
    #[token("àjọṣe")]
    #[token("co-op")]
    Ajose,

    #[token("ase")]
    #[token("àṣẹ")]
    #[token("end")]
    Ase,

    #[token("ewọ")]
    #[token("èèwọ̀")]
    #[token("taboo")]
    Taboo,

    // Visibility modifiers
    #[token("gbangba")] // Public (Yoruba: "open/public")
    #[token("pub")] // Public (English)
    #[token("public")] // Public (English)
    Pub,

    #[token("ikoko")] // Private (Yoruba: "secret")
    #[token("àdáni")] // Private (Yoruba: "private")
    #[token("private")] // Private (English)
    Private,

    // ═══════════════════════════════════════════════════════════════════════
    // LITERALS
    // ═══════════════════════════════════════════════════════════════════════
    #[regex(r"[0-9]+(\.[0-9]+)?", |lex| lex.slice().to_string())]
    Number(String),

    #[regex(r#""[^"]*""#, |lex| {
        let s = lex.slice();
        s[1..s.len()-1].to_string()
    })]
    #[regex(r#"'[^']*'"#, |lex| {
        let s = lex.slice();
        s[1..s.len()-1].to_string()
    })]
    String(String),

    // ═══════════════════════════════════════════════════════════════════════
    // IDENTIFIERS & DOMAINS
    // ═══════════════════════════════════════════════════════════════════════

    // Odù domain names (checked via callback)
    #[regex(r"[A-Z][a-zA-Z_\u0080-\uFFFF]*", |lex| {
        check_domain(lex)
    })]
    Domain(OduDomain),

    // Regular identifiers
    #[regex(r"[a-z_\u0080-\uFFFF][a-zA-Z0-9_\u0080-\uFFFF]*", |lex| lex.slice().to_string())]
    Identifier(String),

    // ═══════════════════════════════════════════════════════════════════════
    // OPERATORS
    // ═══════════════════════════════════════════════════════════════════════
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,

    #[token("==")]
    EqEq,
    #[token("!=")]
    NotEq,
    #[token("<")]
    Lt,
    #[token("<=")]
    LtEq,
    #[token(">")]
    Gt,
    #[token(">=")]
    GtEq,

    #[token("&&")]
    And,
    #[token("||")]
    Or,
    #[token("!")]
    Not,

    #[token("=>")]
    FatArrow,

    #[token("..")]
    DoubleDot,

    // ═══════════════════════════════════════════════════════════════════════
    // PUNCTUATION
    // ═══════════════════════════════════════════════════════════════════════
    #[token("=")]
    Assign,
    #[token(".")]
    Dot,
    #[token(",")]
    Comma,
    #[token(";")]
    Semicolon,
    #[token(":")]
    Colon,

    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,

    // ═══════════════════════════════════════════════════════════════════════
    // SPECIAL
    // ═══════════════════════════════════════════════════════════════════════
    #[regex(r"#[^\n]*", |lex| lex.slice().to_string())]
    Comment(String),

    #[token("\n")]
    Newline,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Number(n) => write!(f, "Number({})", n),
            Token::String(s) => write!(f, "String(\"{}\")", s),
            Token::Identifier(i) => write!(f, "Ident({})", i),
            Token::Domain(d) => write!(f, "Domain({:?})", d),
            Token::Comment(_c) => write!(f, "Comment"),
            _ => write!(f, "{:?}", self),
        }
    }
}

/// Span information for tokens
#[derive(Debug, Clone)]
pub struct Spanned<T> {
    pub value: T,
    pub span: std::ops::Range<usize>,
}

/// Tokenize source code
pub fn tokenize(source: &str) -> Vec<Spanned<Token>> {
    let mut lexer = Token::lexer(source);
    let mut tokens = Vec::new();

    while let Some(result) = lexer.next() {
        match result {
            Ok(token) => {
                tokens.push(Spanned {
                    value: token,
                    span: lexer.span(),
                });
            }
            Err(_) => {
                // Skip invalid tokens for now
                eprintln!("Lex error at {:?}: {:?}", lexer.span(), lexer.slice());
            }
        }
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let tokens = tokenize("ayanmo x = 42");
        assert!(matches!(tokens[0].value, Token::Let));
        assert!(matches!(&tokens[1].value, Token::Identifier(s) if s == "x"));
        assert!(matches!(tokens[2].value, Token::Assign));
        assert!(matches!(&tokens[3].value, Token::Number(n) if n == "42"));
    }

    #[test]
    fn test_domain() {
        let tokens = tokenize("Obara.fikun(10)");
        assert!(matches!(tokens[0].value, Token::Domain(OduDomain::Obara)));
        assert!(matches!(tokens[1].value, Token::Dot));
    }

    #[test]
    fn test_string() {
        let tokens = tokenize(r#""Hello Ifá!""#);
        assert!(matches!(&tokens[0].value, Token::String(s) if s == "Hello Ifá!"));
    }

    #[test]
    fn test_yoruba_keywords() {
        let tokens = tokenize("àyànmọ́ x = otito");
        assert!(matches!(tokens[0].value, Token::Let));
        assert!(matches!(tokens[3].value, Token::True));
    }
}
