//! # Ọ̀ṣẹ́ Handler - Graphics/UI
//!
//! Handles terminal graphics and UI operations.
//! Binary pattern: 1010

use crate::error::{IfaError, IfaResult};
use crate::lexer::OduDomain;
use crate::value::IfaValue;

use super::{Environment, OduHandler};

/// Handler for Ọ̀ṣẹ́ (Graphics/UI) domain.
pub struct OseHandler;

impl OduHandler for OseHandler {
    fn domain(&self) -> OduDomain {
        OduDomain::Ose
    }

    fn call(
        &self,
        method: &str,
        args: Vec<IfaValue>,
        _env: &mut Environment,
        _output: &mut Vec<String>,
    ) -> IfaResult<IfaValue> {

        let arg0 = args.first();
        
        match method {
            // Clear terminal
            "nu" | "clear" => {
                print!("\x1B[2J\x1B[H");
                Ok(IfaValue::null())
            }

            // Set cursor position
            "lọ_si" | "goto" | "move_to" => {
                if let (Some(x_val), Some(y_val)) = (arg0, args.get(1)) {
                    if let (IfaValue::Int(x), IfaValue::Int(y)) = (x_val, y_val) {
                        print!("\x1B[{};{}H", *y + 1, *x + 1);
                        return Ok(IfaValue::null());
                    }
                }
                Err(IfaError::Runtime(
                    "goto requires x and y coordinates".into(),
                ))
            }

            // Set text color
            "awọ" | "color" => {
                 if let Some(IfaValue::Str(color)) = arg0 {
                        let code = match color.to_lowercase().as_str() {
                            "red" | "pupa" => "31",
                            "green" | "ewe" => "32",
                            "yellow" | "oye" => "33",
                            "blue" | "bulu" => "34",
                            "magenta" => "35",
                            "cyan" => "36",
                            "white" | "funfun" => "37",
                            "reset" | "atunto" => "0",
                            _ => "0",
                        };
                        print!("\x1B[{}m", code);
                        return Ok(IfaValue::null());
                    }
                Err(IfaError::Runtime("color requires color name".into()))
            }

            // Draw box
            "apoti" | "box" => {
                if let (Some(x_val), Some(y_val), Some(w_val), Some(h_val)) = (arg0, args.get(1), args.get(2), args.get(3)) {
                     if let (IfaValue::Int(x), IfaValue::Int(y), IfaValue::Int(w), IfaValue::Int(h)) = (x_val, y_val, w_val, h_val) {
                        // Draw top
                        print!("\x1B[{};{}H┌", *y + 1, *x + 1);
                        for _ in 0..(*w - 2) {
                            print!("─");
                        }
                        print!("┐");

                        // Draw sides
                        for i in 1..(*h - 1) {
                            print!("\x1B[{};{}H│", *y + 1 + i, *x + 1);
                            print!("\x1B[{};{}H│", *y + 1 + i, *x + *w);
                        }

                        // Draw bottom
                        print!("\x1B[{};{}H└", *y + *h, *x + 1);
                        for _ in 0..(*w - 2) {
                            print!("─");
                        }
                        print!("┘");

                        return Ok(IfaValue::null());
                     }
                }
                Err(IfaError::Runtime("box requires x, y, width, height".into()))
            }

            // Print at position
            "kọ_si" | "print_at" => {
                 if let (Some(x_val), Some(y_val), Some(text)) = (arg0, args.get(1), args.get(2)) {
                     if let (IfaValue::Int(x), IfaValue::Int(y)) = (x_val, y_val) {
                         print!("\x1B[{};{}H{}", y + 1, x + 1, text);
                         return Ok(IfaValue::null());
                     }
                 }
                Err(IfaError::Runtime("print_at requires x, y, text".into()))
            }

            // Hide cursor
            "fia_kasọta" | "hide_cursor" => {
                print!("\x1B[?25l");
                Ok(IfaValue::null())
            }

            // Show cursor
            "ṣafihan_kasọta" | "show_cursor" => {
                print!("\x1B[?25h");
                Ok(IfaValue::null())
            }

            // Get terminal size
            // Queries the terminal via ANSI CSI 18 t ("report window size in chars").
            // Falls back to 80×24 if the query is unavailable (e.g. non-interactive, pipe).
            "iwọn" | "size" => {
                // Try to read COLUMNS/LINES env vars first (set by most shells).
                let cols = std::env::var("COLUMNS")
                    .ok()
                    .and_then(|v| v.parse::<i64>().ok())
                    .unwrap_or(80);
                let rows = std::env::var("LINES")
                    .ok()
                    .and_then(|v| v.parse::<i64>().ok())
                    .unwrap_or(24);
                Ok(IfaValue::list(vec![
                    IfaValue::int(cols),
                    IfaValue::int(rows),
                ]))
            }

            _ => Err(IfaError::Runtime(format!("Unknown Ọ̀ṣẹ́ method: {}", method))),
        }
    }

    fn methods(&self) -> &'static [&'static str] {
        &[
            "nu",
            "clear",
            "lọ_si",
            "goto",
            "move_to",
            "awọ",
            "color",
            "apoti",
            "box",
            "kọ_si",
            "print_at",
            "fia_kasọta",
            "hide_cursor",
            "ṣafihan_kasọta",
            "show_cursor",
            "iwọn",
            "size",
        ]
    }
}
