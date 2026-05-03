//! # Ọ̀ṣẹ́ Domain (1010)
//!
//! The Painter - Graphics and UI
//!
//! Terminal UI using ratatui, converted to a declarative interface for Ifá-Lang scripts.

use crate::impl_odu_domain;
#[cfg(feature = "full")]
use crossterm::event::{self, Event, KeyCode};
#[cfg(feature = "full")]
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
#[cfg(feature = "full")]
use ratatui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io;
use std::sync::{Arc, Mutex};
use ifa_core::IfaValue;
use ifa_core::error::{IfaError, IfaResult};
use crate::handlers::registry::REGISTRY;
use ifa_types::ResourceToken;

/// Ọ̀ṣẹ́ - The Painter (Graphics/UI)
pub struct Ose;

impl_odu_domain!(Ose, "Ọ̀ṣẹ́", "1010", "The Painter - Graphics/UI");

#[cfg(feature = "full")]
impl Ose {
    pub fn dispatch(method: &str, args: Vec<IfaValue>) -> IfaResult<IfaValue> {
        match method {
            "bere" | "init" => Self::handle_bere(),
            "pari" | "end" => Self::handle_pari(args),
            "ya" | "draw" => Self::handle_ya(args),
            "gboran" | "listen" => Self::handle_gboran(),
            "gbile" | "read_key" => Self::handle_gbile(),
            _ => Err(IfaError::Custom(format!("Ose: unknown method '{}'", method))),
        }
    }

    fn handle_bere() -> IfaResult<IfaValue> {
        enable_raw_mode().map_err(|e| IfaError::Runtime(e.to_string()))?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen).map_err(|e| IfaError::Runtime(e.to_string()))?;
        
        // Register panic hook to restore terminal in case of VM crash
        let default_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            let _ = disable_raw_mode();
            let _ = execute!(io::stdout(), LeaveAlternateScreen);
            default_hook(panic_info);
        }));

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).map_err(|e| IfaError::Runtime(e.to_string()))?;
        let token = REGISTRY.register(Mutex::new(terminal));
        
        Ok(IfaValue::Resource(Arc::new(token)))
    }

    fn handle_pari(args: Vec<IfaValue>) -> IfaResult<IfaValue> {
        if let Some(IfaValue::Resource(token_arc)) = args.first() {
            if let Some(token) = token_arc.downcast_ref::<ResourceToken>() {
                if let Some(terminal_mutex) = REGISTRY.get::<Mutex<Terminal<CrosstermBackend<io::Stdout>>>>(token.clone()) {
                    let mut terminal = terminal_mutex.lock().unwrap();
                    disable_raw_mode().ok();
                    execute!(terminal.backend_mut(), LeaveAlternateScreen).ok();
                    terminal.show_cursor().ok();
                }
                REGISTRY.close(token.clone());
            }
        }
        Ok(IfaValue::null())
    }

    fn handle_ya(args: Vec<IfaValue>) -> IfaResult<IfaValue> {
        if args.len() < 2 {
            return Err(IfaError::ArgumentError("Ose.ya requires (terminal, ui_map)".into()));
        }
        
        let token_arc = match &args[0] {
            IfaValue::Resource(arc) => arc,
            _ => return Err(IfaError::TypeError { expected: "Resource".into(), got: args[0].type_name().into() }),
        };
        
        let token = token_arc.downcast_ref::<ResourceToken>()
            .ok_or_else(|| IfaError::Runtime("Invalid resource token type".into()))?;
            
        let ui_map = match &args[1] {
            IfaValue::Map(map) => map,
            _ => return Err(IfaError::TypeError { expected: "Map".into(), got: args[1].type_name().into() }),
        };

        let terminal_mutex = REGISTRY.get::<Mutex<Terminal<CrosstermBackend<io::Stdout>>>>(token.clone())
            .ok_or_else(|| IfaError::Runtime("Terminal resource not found".into()))?;
            
        let mut terminal = terminal_mutex.lock().unwrap();
        
        terminal.draw(|f| {
            let area = f.area();
            if let Some(IfaValue::Str(widget_type)) = ui_map.get("type") {
                match widget_type.as_str() {
                    "apoti" => {
                        let title = ui_map.get("title").map(|v| v.to_string()).unwrap_or_default();
                        let text = ui_map.get("text").map(|v| v.to_string()).unwrap_or_default();
                        let block = Block::default().title(title).borders(Borders::ALL);
                        let paragraph = Paragraph::new(text).block(block);
                        f.render_widget(paragraph, area);
                    }
                    "ipinro" => {
                        let text = ui_map.get("text").map(|v| v.to_string()).unwrap_or_default();
                        let paragraph = Paragraph::new(text);
                        f.render_widget(paragraph, area);
                    }
                    _ => {}
                }
            }
        }).map_err(|e| IfaError::Runtime(e.to_string()))?;

        Ok(IfaValue::null())
    }

    fn handle_gboran() -> IfaResult<IfaValue> {
        if event::poll(std::time::Duration::from_millis(10)).unwrap_or(false) {
            if let Ok(Event::Key(key)) = event::read() {
                let s = match key.code {
                    KeyCode::Char(c) => c.to_string(),
                    KeyCode::Enter => "Enter".to_string(),
                    KeyCode::Esc => "Esc".to_string(),
                    KeyCode::Up => "Up".to_string(),
                    KeyCode::Down => "Down".to_string(),
                    KeyCode::Left => "Left".to_string(),
                    KeyCode::Right => "Right".to_string(),
                    KeyCode::Backspace => "Backspace".to_string(),
                    _ => return Ok(IfaValue::null()),
                };
                return Ok(IfaValue::str(s));
            }
        }
        Ok(IfaValue::null())
    }

    fn handle_gbile() -> IfaResult<IfaValue> {
        loop {
            if let Ok(Event::Key(key)) = event::read() {
                let s = match key.code {
                    KeyCode::Char(c) => c.to_string(),
                    KeyCode::Enter => "Enter".to_string(),
                    KeyCode::Esc => "Esc".to_string(),
                    _ => "?".to_string(),
                };
                return Ok(IfaValue::str(s));
            }
        }
    }
}

#[cfg(not(feature = "full"))]
impl Ose {
    pub fn dispatch(method: &str, _args: Vec<IfaValue>) -> IfaResult<IfaValue> {
        Err(IfaError::Runtime("TUI not compiled in minimal mode".into()))
    }
}

