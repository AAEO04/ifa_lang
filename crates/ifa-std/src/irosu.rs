//! # Ìrosù Domain (1100)
//!
//! The Voice - Console Input/Output
//!
//! ## Methods
//! - `fo` - Print with newline
//! - `so` - Print without newline  
//! - `gbo` - Read input
//! - `awo` - Colored output
//! - `mo` - Clear screen

use crate::impl_odu_domain;
use crossterm::{
    cursor, execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal,
};
use ifa_core::IfaValue;
use std::io::{self, BufRead, Write};

use ifa_sandbox::{CapabilitySet, Ofun};

/// Ìrosù - The Voice (Console I/O)
#[derive(Default)]
pub struct Irosu {
    capabilities: CapabilitySet,
}

impl_odu_domain!(Irosu, "Ìrosù", "1100", "The Voice - Console I/O");

impl Irosu {
    pub fn new(capabilities: CapabilitySet) -> Self {
        Irosu { capabilities }
    }

    fn check(&self) -> bool {
        self.capabilities.check(&Ofun::Stdio)
    }

    /// Print with newline (fọ̀)
    pub fn fo(&self, value: &IfaValue) {
        if self.check() {
            println!("{}", value);
        }
    }

    /// Print without newline (sọ)
    pub fn so(&self, value: &IfaValue) {
        if self.check() {
            print!("{}", value);
            io::stdout().flush().ok();
        }
    }

    /// Read input from user (gbọ́)
    pub fn gbo(&self, prompt: &str) -> String {
        if !self.check() {
            return String::new();
        }

        if !prompt.is_empty() {
            print!("{}", prompt);
            io::stdout().flush().ok();
        }
        let mut input = String::new();
        io::stdin().lock().read_line(&mut input).ok();
        input.trim().to_string()
    }

    /// Read integer input
    pub fn gbo_nomba(&self, prompt: &str) -> i64 {
        self.gbo(prompt).parse().unwrap_or(0)
    }

    /// Read float input
    pub fn gbo_odidi(&self, prompt: &str) -> f64 {
        self.gbo(prompt).parse().unwrap_or(0.0)
    }

    /// Print with color (àwọ̀)
    pub fn awo(&self, text: &str, color: &str) {
        if !self.check() {
            return;
        }

        let color_code = match color.to_lowercase().as_str() {
            "red" | "pupa" => Color::Red,
            "green" | "ewe" => Color::Green,
            "blue" | "bulu" => Color::Blue,
            "yellow" | "ofeefe" => Color::Yellow,
            "magenta" => Color::Magenta,
            "cyan" => Color::Cyan,
            "white" | "funfun" => Color::White,
            _ => Color::Reset,
        };

        let mut stdout = io::stdout();
        execute!(
            stdout,
            SetForegroundColor(color_code),
            Print(text),
            Print("\n"),
            ResetColor
        )
        .ok();
    }

    /// Clear screen (mọ́)
    pub fn mo(&self) {
        if !self.check() {
            return;
        }

        let mut stdout = io::stdout();
        execute!(
            stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0)
        )
        .ok();
    }

    /// Flush stdout (sàn)
    pub fn san(&self) {
        if self.check() {
            io::stdout().flush().ok();
            io::stderr().flush().ok();
        }
    }

    /// Print to stderr (kígbe)
    pub fn kigbe(&self, text: &str) {
        if self.check() {
            eprintln!("[ERROR] {}", text);
        }
    }

    /// Print progress bar
    pub fn ilosiwaju(&self, current: usize, total: usize, width: usize) {
        if !self.check() {
            return;
        }

        let percent = if total > 0 { current * 100 / total } else { 0 };
        let filled = width * current / total.max(1);
        let bar: String = "█".repeat(filled) + &"░".repeat(width - filled);
        print!("\r[{}] {}%", bar, percent);
        io::stdout().flush().ok();
        if current >= total {
            println!();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::OduDomain;

    #[test]
    fn test_irosu_creation() {
        let irosu = Irosu::default();
        assert_eq!(irosu.name(), "Ìrosù");
        assert_eq!(irosu.binary(), "1100");
    }
}
