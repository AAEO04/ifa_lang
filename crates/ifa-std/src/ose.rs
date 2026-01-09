//! # Ọ̀ṣẹ́ Domain (1010)
//!
//! The Painter - Graphics and UI
//!
//! Terminal UI using ratatui.

use crate::impl_odu_domain;
#[cfg(feature = "full")]
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
#[cfg(feature = "full")]
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    style::Color,
    widgets::{Block, Borders, Paragraph},
};
use std::io;

/// Ọ̀ṣẹ́ - The Painter (Graphics/UI)
pub struct Ose;

impl_odu_domain!(Ose, "Ọ̀ṣẹ́", "1010", "The Painter - Graphics/UI");

#[cfg(feature = "full")]
impl Ose {
    /// Initialize terminal UI (bẹ̀rẹ̀)
    pub fn bere(&self) -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        Terminal::new(backend)
    }

    /// Cleanup terminal (parí)
    pub fn pari(&self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()
    }

    /// Create bordered box widget (àpótí)
    pub fn apoti(&self, title: &str) -> Block<'static> {
        Block::default()
            .title(title.to_string())
            .borders(Borders::ALL)
    }

    /// Create paragraph widget (ìpínrọ̀)
    pub fn ipinro<'a>(&self, text: &'a str) -> Paragraph<'a> {
        Paragraph::new(text)
    }

    /// Get Odù-themed color
    pub fn awo_odu(&self, binary: &str) -> Color {
        match binary {
            "1111" => Color::White,   // Ọ̀gbè - Full light
            "0000" => Color::Black,   // Ọ̀yẹ̀kú - Full dark
            "1100" => Color::Yellow,  // Ìrosù - Voice (bright)
            "0011" => Color::Magenta, // Ọ̀wọ́nrín - Random (chaotic)
            "1010" => Color::Green,   // Ọ̀ṣẹ́ - Growth
            "0101" => Color::Blue,    // Òfún - Reflection
            "1001" => Color::Cyan,    // Òdí - Container
            "0110" => Color::Gray,    // Ìwòrì - Time
            _ => Color::Reset,
        }
    }
}

#[cfg(not(feature = "full"))]
impl Ose {
    /// Placeholder for minimal builds
    pub fn placeholder(&self) {
        // TUI not available in minimal mode
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_odu_colors() {
        let ose = Ose;
        // Just test that it doesn't panic
        let _ = ose.awo_odu("1111");
        let _ = ose.awo_odu("0000");
    }
}
