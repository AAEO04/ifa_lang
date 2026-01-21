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
        match method {
            // Clear terminal
            "nu" | "clear" => {
                print!("\x1B[2J\x1B[H");
                Ok(IfaValue::Null)
            }

            // Set cursor position
            "lọ_si" | "goto" | "move_to" => {
                if args.len() >= 2 {
                    if let (IfaValue::Int(x), IfaValue::Int(y)) = (&args[0], &args[1]) {
                        print!("\x1B[{};{}H", y + 1, x + 1);
                        return Ok(IfaValue::Null);
                    }
                }
                Err(IfaError::Runtime(
                    "goto requires x and y coordinates".into(),
                ))
            }

            // Set text color
            "awọ" | "color" => {
                if let Some(IfaValue::Str(color)) = args.first() {
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
                    return Ok(IfaValue::Null);
                }
                Err(IfaError::Runtime("color requires color name".into()))
            }

            // Draw box
            "apoti" | "box" => {
                if args.len() >= 4 {
                    if let (
                        IfaValue::Int(x),
                        IfaValue::Int(y),
                        IfaValue::Int(w),
                        IfaValue::Int(h),
                    ) = (&args[0], &args[1], &args[2], &args[3])
                    {
                        // Draw top
                        print!("\x1B[{};{}H┌", y + 1, x + 1);
                        for _ in 0..(*w - 2) {
                            print!("─");
                        }
                        print!("┐");

                        // Draw sides
                        for i in 1..(*h - 1) {
                            print!("\x1B[{};{}H│", y + 1 + i, x + 1);
                            print!("\x1B[{};{}H│", y + 1 + i, x + *w);
                        }

                        // Draw bottom
                        print!("\x1B[{};{}H└", y + *h, x + 1);
                        for _ in 0..(*w - 2) {
                            print!("─");
                        }
                        print!("┘");

                        return Ok(IfaValue::Null);
                    }
                }
                Err(IfaError::Runtime("box requires x, y, width, height".into()))
            }

            // Print at position
            "kọ_si" | "print_at" => {
                if args.len() >= 3 {
                    if let (IfaValue::Int(x), IfaValue::Int(y), text) =
                        (&args[0], &args[1], &args[2])
                    {
                        print!("\x1B[{};{}H{}", y + 1, x + 1, text);
                        return Ok(IfaValue::Null);
                    }
                }
                Err(IfaError::Runtime("print_at requires x, y, text".into()))
            }

            // Hide cursor
            "fia_kasọta" | "hide_cursor" => {
                print!("\x1B[?25l");
                Ok(IfaValue::Null)
            }

            // Show cursor
            "ṣafihan_kasọta" | "show_cursor" => {
                print!("\x1B[?25h");
                Ok(IfaValue::Null)
            }

            // Get terminal size (placeholder)
            "iwọn" | "size" => {
                Ok(IfaValue::List(vec![
                    IfaValue::Int(80), // width
                    IfaValue::Int(24), // height
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
