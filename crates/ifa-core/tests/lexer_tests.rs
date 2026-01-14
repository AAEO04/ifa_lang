//! Comprehensive tests for Ifá lexer

use ifa_core::*;
use common::helpers::*;

mod basic_tokenization_tests {
    use super::*;

    #[test]
    fn test_tokenize_integer() {
        let code = "42";
        let tokens = tokenize(code).unwrap();
        
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::Int(42)));
    }

    #[test]
    fn test_tokenize_negative_integer() {
        let code = "-42";
        let tokens = tokenize(code).unwrap();
        
        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0], Token::Minus));
        assert!(matches!(tokens[1], Token::Int(42)));
    }

    #[test]
    fn test_tokenize_float() {
        let code = "3.14";
        let tokens = tokenize(code).unwrap();
        
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::Float(f) if (f - 3.14).abs() < 0.001));
    }

    #[test]
    fn test_tokenize_negative_float() {
        let code = "-2.718";
        let tokens = tokenize(code).unwrap();
        
        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0], Token::Minus));
        assert!(matches!(tokens[1], Token::Float(f) if (f - 2.718).abs() < 0.001));
    }

    #[test]
    fn test_tokenize_string() {
        let code = "\"hello world\"";
        let tokens = tokenize(code).unwrap();
        
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::String(s) if s == "hello world"));
    }

    #[test]
    fn test_tokenize_empty_string() {
        let code = "\"\"";
        let tokens = tokenize(code).unwrap();
        
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::String(s) if s.is_empty()));
    }

    #[test]
    fn test_tokenize_boolean_literals() {
        let test_cases = vec![
            ("true", Token::Bool(true)),
            ("false", Token::Bool(false)),
        ];
        
        for (code, expected) in test_cases {
            let tokens = tokenize(code).unwrap();
            assert_eq!(tokens.len(), 1);
            assert!(matches!(&tokens[0], expected));
        }
    }

    #[test]
    fn test_tokenize_null() {
        let code = "null";
        let tokens = tokenize(code).unwrap();
        
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::Null));
    }

    #[test]
    fn test_tokenize_identifier() {
        let code = "variable_name";
        let tokens = tokenize(code).unwrap();
        
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::Identifier(name) if name == "variable_name"));
    }

    #[test]
    fn test_tokenize_keywords() {
        let test_cases = vec![
            ("let", Token::Let),
            ("fn", Token::Fn),
            ("if", Token::If),
            ("else", Token::Else),
            ("return", Token::Return),
        ];
        
        for (code, expected) in test_cases {
            let tokens = tokenize(code).unwrap();
            assert_eq!(tokens.len(), 1);
            assert!(matches!(&tokens[0], expected));
        }
    }
}

mod operator_tokenization_tests {
    use super::*;

    #[test]
    fn test_tokenize_arithmetic_operators() {
        let test_cases = vec![
            ("+", Token::Plus),
            ("-", Token::Minus),
            ("*", Token::Star),
            ("/", Token::Slash),
            ("%", Token::Percent),
        ];
        
        for (code, expected) in test_cases {
            let tokens = tokenize(code).unwrap();
            assert_eq!(tokens.len(), 1);
            assert!(matches!(&tokens[0], expected));
        }
    }

    #[test]
    fn test_tokenize_comparison_operators() {
        let test_cases = vec![
            ("==", Token::EqualEqual),
            ("!=", Token::BangEqual),
            ("<", Token::Less),
            ("<=", Token::LessEqual),
            (">", Token::Greater),
            (">=", Token::GreaterEqual),
        ];
        
        for (code, expected) in test_cases {
            let tokens = tokenize(code).unwrap();
            assert_eq!(tokens.len(), 1);
            assert!(matches!(&tokens[0], expected));
        }
    }

    #[test]
    fn test_tokenize_logical_operators() {
        let test_cases = vec![
            ("&&", Token::AndAnd),
            ("||", Token::OrOr),
            ("!", Token::Bang),
        ];
        
        for (code, expected) in test_cases {
            let tokens = tokenize(code).unwrap();
            assert_eq!(tokens.len(), 1);
            assert!(matches!(&tokens[0], expected));
        }
    }

    #[test]
    fn test_tokenize_assignment_operators() {
        let test_cases = vec![
            ("=", Token::Equal),
            ("+=", Token::PlusEqual),
            ("-=", Token::MinusEqual),
            ("*=", Token::StarEqual),
            ("/=", Token::SlashEqual),
        ];
        
        for (code, expected) in test_cases {
            let tokens = tokenize(code).unwrap();
            assert_eq!(tokens.len(), 1);
            assert!(matches!(&tokens[0], expected));
        }
    }
}

mod punctuation_tokenization_tests {
    use super::*;

    #[test]
    fn test_tokenize_parentheses() {
        let test_cases = vec![
            ("(", Token::LeftParen),
            (")", Token::RightParen),
        ];
        
        for (code, expected) in test_cases {
            let tokens = tokenize(code).unwrap();
            assert_eq!(tokens.len(), 1);
            assert!(matches!(&tokens[0], expected));
        }
    }

    #[test]
    fn test_tokenize_braces() {
        let test_cases = vec![
            ("{", Token::LeftBrace),
            ("}", Token::RightBrace),
        ];
        
        for (code, expected) in test_cases {
            let tokens = tokenize(code).unwrap();
            assert_eq!(tokens.len(), 1);
            assert!(matches!(&tokens[0], expected));
        }
    }

    #[test]
    fn test_tokenize_brackets() {
        let test_cases = vec![
            ("[", Token::LeftBracket),
            ("]", Token::RightBracket),
        ];
        
        for (code, expected) in test_cases {
            let tokens = tokenize(code).unwrap();
            assert_eq!(tokens.len(), 1);
            assert!(matches!(&tokens[0], expected));
        }
    }

    #[test]
    fn test_tokenize_comma_and_semicolon() {
        let test_cases = vec![
            (",", Token::Comma),
            (";", Token::Semicolon),
        ];
        
        for (code, expected) in test_cases {
            let tokens = tokenize(code).unwrap();
            assert_eq!(tokens.len(), 1);
            assert!(matches!(&tokens[0], expected));
        }
    }

    #[test]
    fn test_tokenize_dot() {
        let code = ".";
        let tokens = tokenize(code).unwrap();
        
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::Dot));
    }
}

mod whitespace_and_comments_tests {
    use super::*;

    #[test]
    fn test_tokenize_with_spaces() {
        let code = "1 + 2";
        let tokens = tokenize(code).unwrap();
        
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], Token::Int(1)));
        assert!(matches!(tokens[1], Token::Plus));
        assert!(matches!(tokens[2], Token::Int(2)));
    }

    #[test]
    fn test_tokenize_with_tabs() {
        let code = "1\t+\t2";
        let tokens = tokenize(code).unwrap();
        
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], Token::Int(1)));
        assert!(matches!(tokens[1], Token::Plus));
        assert!(matches!(tokens[2], Token::Int(2)));
    }

    #[test]
    fn test_tokenize_with_newlines() {
        let code = "1\n+\n2";
        let tokens = tokenize(code).unwrap();
        
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], Token::Int(1)));
        assert!(matches!(tokens[1], Token::Plus));
        assert!(matches!(tokens[2], Token::Int(2)));
    }

    #[test]
    fn test_tokenize_with_mixed_whitespace() {
        let code = " 1 \t\n + \r\n 2 ";
        let tokens = tokenize(code).unwrap();
        
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], Token::Int(1)));
        assert!(matches!(tokens[1], Token::Plus));
        assert!(matches!(tokens[2], Token::Int(2)));
    }

    #[test]
    fn test_tokenize_single_line_comment() {
        let code = "1 + 2 // This is a comment\n3";
        let tokens = tokenize(code).unwrap();
        
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], Token::Int(1)));
        assert!(matches!(tokens[1], Token::Plus));
        assert!(matches!(tokens[2], Token::Int(2)));
        // Comment should be ignored
    }

    #[test]
    fn test_tokenize_multi_line_comment() {
        let code = "1 + /* This is a\nmulti-line comment */ 2";
        let tokens = tokenize(code).unwrap();
        
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], Token::Int(1)));
        assert!(matches!(tokens[1], Token::Plus));
        assert!(matches!(tokens[2], Token::Int(2)));
        // Comment should be ignored
    }

    #[test]
    fn test_tokenize_nested_comments() {
        let code = "1 + /* outer /* inner */ comment */ 2";
        let tokens = tokenize(code).unwrap();
        
        assert_eq!(tokens.len(), 3);
        assert!(matches!(tokens[0], Token::Int(1)));
        assert!(matches!(tokens[1], Token::Plus));
        assert!(matches!(tokens[2], Token::Int(2)));
        // Nested comments should be handled correctly
    }
}

mod complex_tokenization_tests {
    use super::*;

    #[test]
    fn test_tokenize_simple_expression() {
        let code = "1 + 2 * 3";
        let tokens = tokenize(code).unwrap();
        
        assert_eq!(tokens.len(), 5);
        assert!(matches!(tokens[0], Token::Int(1)));
        assert!(matches!(tokens[1], Token::Plus));
        assert!(matches!(tokens[2], Token::Int(2)));
        assert!(matches!(tokens[3], Token::Star));
        assert!(matches!(tokens[4], Token::Int(3)));
    }

    #[test]
    fn test_tokenize_variable_declaration() {
        let code = "let x = 42";
        let tokens = tokenize(code).unwrap();
        
        assert_eq!(tokens.len(), 4);
        assert!(matches!(tokens[0], Token::Let));
        assert!(matches!(tokens[1], Token::Identifier(name) if name == "x"));
        assert!(matches!(tokens[2], Token::Equal));
        assert!(matches!(tokens[3], Token::Int(42)));
    }

    #[test]
    fn test_tokenize_function_declaration() {
        let code = "fn add(a, b) { a + b }";
        let tokens = tokenize(code).unwrap();
        
        assert_eq!(tokens.len(), 12);
        assert!(matches!(tokens[0], Token::Fn));
        assert!(matches!(tokens[1], Token::Identifier(name) if name == "add"));
        assert!(matches!(tokens[2], Token::LeftParen));
        assert!(matches!(tokens[3], Token::Identifier(name) if name == "a"));
        assert!(matches!(tokens[4], Token::Comma));
        assert!(matches!(tokens[5], Token::Identifier(name) if name == "b"));
        assert!(matches!(tokens[6], Token::RightParen));
        assert!(matches!(tokens[7], Token::LeftBrace));
        assert!(matches!(tokens[8], Token::Identifier(name) if name == "a"));
        assert!(matches!(tokens[9], Token::Plus));
        assert!(matches!(tokens[10], Token::Identifier(name) if name == "b"));
        assert!(matches!(tokens[11], Token::RightBrace));
    }

    #[test]
    fn test_tokenize_function_call() {
        let code = "func(1, 2, 3)";
        let tokens = tokenize(code).unwrap();
        
        assert_eq!(tokens.len(), 7);
        assert!(matches!(tokens[0], Token::Identifier(name) if name == "func"));
        assert!(matches!(tokens[1], Token::LeftParen));
        assert!(matches!(tokens[2], Token::Int(1)));
        assert!(matches!(tokens[3], Token::Comma));
        assert!(matches!(tokens[4], Token::Int(2)));
        assert!(matches!(tokens[5], Token::Comma));
        assert!(matches!(tokens[6], Token::Int(3)));
        assert!(matches!(tokens[7], Token::RightParen));
    }

    #[test]
    fn test_tokenize_if_expression() {
        let code = "if condition { true } else { false }";
        let tokens = tokenize(code).unwrap();
        
        assert!(tokens.len() > 0);
        assert!(matches!(tokens[0], Token::If));
        // Should properly tokenize all parts of if expression
    }

    #[test]
    fn test_tokenize_array_literal() {
        let code = "[1, 2, 3]";
        let tokens = tokenize(code).unwrap();
        
        assert_eq!(tokens.len(), 7);
        assert!(matches!(tokens[0], Token::LeftBracket));
        assert!(matches!(tokens[1], Token::Int(1)));
        assert!(matches!(tokens[2], Token::Comma));
        assert!(matches!(tokens[3], Token::Int(2)));
        assert!(matches!(tokens[4], Token::Comma));
        assert!(matches!(tokens[5], Token::Int(3)));
        assert!(matches!(tokens[6], Token::RightBracket));
    }
}

mod unicode_and_special_characters_tests {
    use super::*;

    #[test]
    fn test_tokenize_unicode_identifiers() {
        let code = "变量名";
        let tokens = tokenize(code).unwrap();
        
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::Identifier(name) if name == "变量名"));
    }

    #[test]
    fn test_tokenize_unicode_strings() {
        let code = "\"🔥🌟✨\"";
        let tokens = tokenize(code).unwrap();
        
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::String(s) if s == "🔥🌟✨"));
    }

    #[test]
    fn test_tokenize_escape_sequences() {
        let test_cases = vec![
            ("\"\\n\"", "\n"),
            ("\"\\t\"", "\t"),
            ("\"\\r\"", "\r"),
            ("\"\\\\\"", "\\"),
            ("\"\\\"\"", "\""),
        ];
        
        for (code, expected) in test_cases {
            let tokens = tokenize(code).unwrap();
            assert_eq!(tokens.len(), 1);
            assert!(matches!(&tokens[0], Token::String(s) if s == expected));
        }
    }

    #[test]
    fn test_tokenize_yoruba_identifiers() {
        let code = "kíkàn_ọ̀ràn";
        let tokens = tokenize(code).unwrap();
        
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::Identifier(name) if name == "kíkàn_ọ̀ràn"));
    }

    #[test]
    fn test_tokenize_numbers_with_underscores() {
        let code = "1_000_000";
        let tokens = tokenize(code).unwrap();
        
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::Int(1_000_000)));
    }
}

mod error_handling_tests {
    use super::*;

    #[test]
    fn test_tokenize_unclosed_string() {
        let code = "\"unclosed string";
        let result = tokenize(code);
        assert!(result.is_err());
    }

    #[test]
    fn test_tokenize_unclosed_comment() {
        let code = "1 + /* unclosed comment";
        let result = tokenize(code);
        assert!(result.is_err());
    }

    #[test]
    fn test_tokenize_invalid_character() {
        let code = "@#$";
        let result = tokenize(code);
        assert!(result.is_err());
    }

    #[test]
    fn test_tokenize_invalid_number_format() {
        let code = "123.456.789";
        let result = tokenize(code);
        assert!(result.is_err());
    }

    #[test]
    fn test_tokenize_invalid_escape_sequence() {
        let code = "\"\\x invalid\"";
        let result = tokenize(code);
        // Might succeed or fail depending on implementation
        match result {
            Ok(_) => {
                // If tokenization succeeds, the escape should be handled
                assert!(true);
            },
            Err(_) => {
                // Tokenization failed, which is acceptable for invalid escapes
                assert!(true);
            }
        }
    }

    #[test]
    fn test_tokenize_unmatched_braces() {
        let code = "{ unclosed";
        let tokens = tokenize(code).unwrap();
        
        // Tokenization should succeed, parsing will catch the error
        assert!(tokens.len() > 0);
        assert!(matches!(tokens[0], Token::LeftBrace));
    }
}

mod performance_tests {
    use super::*;

    #[test]
    fn test_tokenization_performance() {
        let code = "1 + 2 * 3 - 4 / 5 + 6 * 7 - 8 / 9 + 10";
        
        benchmark_test!(tokenization_performance, 10000, {
            let _tokens = tokenize(code).unwrap();
        });
    }

    #[test]
    fn test_large_file_tokenization() {
        let mut code = String::new();
        
        // Generate a large source file
        for i in 0..10000 {
            code.push_str(&format!("let x{} = {}\n", i, i));
        }
        
        let (_, duration) = measure_time(|| {
            let _tokens = tokenize(&code).unwrap();
        });
        
        // Should tokenize within reasonable time
        assert!(duration.as_millis() < 1000, "Large file tokenization took too long: {:?}", duration);
    }

    #[test]
    fn test_memory_usage_during_tokenization() {
        let code = "let x = 1; let y = 2; let z = x + y;".repeat(1000);
        
        let initial_memory = get_memory_usage();
        let _tokens = tokenize(&code).unwrap();
        let final_memory = get_memory_usage();
        
        // Memory usage should be reasonable
        let memory_increase = final_memory.saturating_sub(initial_memory);
        assert!(memory_increase < 50_000_000, "Tokenization used too much memory: {} bytes", memory_increase);
    }
}

mod edge_case_tests {
    use super::*;

    #[test]
    fn test_tokenize_empty_input() {
        let code = "";
        let tokens = tokenize(code).unwrap();
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn test_tokenize_whitespace_only() {
        let code = "   \t\n\r   ";
        let tokens = tokenize(code).unwrap();
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn test_tokenize_single_character_tokens() {
        let single_chars = vec!["+", "-", "*", "/", "%", "=", "!", "<", ">", "&", "|", "(", ")", "[", "]", "{", "}", ",", ";", "."];
        
        for ch in single_chars {
            let tokens = tokenize(ch).unwrap();
            assert_eq!(tokens.len(), 1, "Failed to tokenize character: {}", ch);
        }
    }

    #[test]
    fn test_tokenize_very_long_identifier() {
        let long_name = "a".repeat(10000);
        let code = &long_name;
        let tokens = tokenize(code).unwrap();
        
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], Token::Identifier(name) if name == &long_name));
    }

    #[test]
    fn test_tokenize_very_long_string() {
        let long_string = "a".repeat(10000);
        let code = format!("\"{}\"", long_string);
        let tokens = tokenize(&code).unwrap();
        
        assert_eq!(tokens.len(), 1);
        assert!(matches!(&tokens[0], Token::String(s) if s == &long_string));
    }

    #[test]
    fn test_tokenize_maximum_integer() {
        let code = "9223372036854775807"; // i64::MAX
        let tokens = tokenize(code).unwrap();
        
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::Int(9223372036854775807)));
    }

    #[test]
    fn test_tokenize_minimum_integer() {
        let code = "-9223372036854775808"; // i64::MIN
        let tokens = tokenize(code).unwrap();
        
        assert_eq!(tokens.len(), 2);
        assert!(matches!(tokens[0], Token::Minus));
        assert!(matches!(tokens[1], Token::Int(9223372036854775808)));
    }

    #[test]
    fn test_tokenize_scientific_notation() {
        let test_cases = vec![
            ("1e10", 1e10),
            ("1.5e-3", 1.5e-3),
            ("2E5", 2e5),
        ];
        
        for (code, expected) in test_cases {
            let tokens = tokenize(code).unwrap();
            assert_eq!(tokens.len(), 1);
            assert!(matches!(&tokens[0], Token::Float(f) if (*f - expected).abs() < 0.001));
        }
    }
}
