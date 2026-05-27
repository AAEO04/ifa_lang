//! # Ọ̀ṣẹ́ Domain (1010)
//!
//! The Painter - Graphics and UI
//!
//! Terminal UI using ratatui, converted to a declarative interface for Ifá-Lang scripts.

use crate::impl_odu_domain;
#[cfg(feature = "tui")]
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEventKind,
};
#[cfg(feature = "tui")]
use crossterm::{
    execute,
    terminal::{
        EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode, size,
    },
};
use ifa_types::ResourceToken;
use ifa_vm::IfaValue;
use ifa_vm::error::{IfaError, IfaResult};
use ifa_vm::native::VmContext;
#[cfg(feature = "tui")]
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{
        Block, Borders, Chart, Dataset, Gauge, GraphType, List, ListItem, Paragraph, Row,
        Scrollbar, ScrollbarOrientation, Sparkline, Table, Tabs,
    },
};
use std::collections::HashMap;
use std::io;
use std::sync::{Arc, Mutex};

#[cfg(feature = "tui")]
use ratatui::widgets::RenderDirection;

/// Ọ̀ṣẹ́ - The Painter (Graphics/UI)
pub struct Ose;

impl_odu_domain!(Ose, "Ọ̀ṣẹ́", "1010", "The Painter - Graphics/UI");

#[cfg(feature = "tui")]
impl Ose {
    pub fn dispatch(method: &str, args: Vec<IfaValue>, ctx: &mut VmContext) -> IfaResult<IfaValue> {
        match method {
            "bere" | "init" => Self::handle_bere(ctx),
            "pari" | "end" => Self::handle_pari(args, ctx),
            "ya" | "draw" => Self::handle_ya(args, ctx),
            "gboran" | "listen" => Self::handle_gboran(),
            "gbile" | "read_key" => Self::handle_gbile(),
            "ipile" | "layout" => Self::handle_ipile(args, ctx),
            "ẹmí" | "mouse_on" => Self::handle_mouse_on(args, ctx),
            "pari_ẹmí" | "mouse_off" => Self::handle_mouse_off(args, ctx),
            "iwọn" | "size" => Self::handle_iwọn(),
            "duro" | "wait" => Self::handle_duro(args),
            _ => Err(IfaError::Custom(format!(
                "Ose: unknown method '{}'",
                method
            ))),
        }
    }

    fn extract_token(
        args: &[IfaValue],
    ) -> IfaResult<(ResourceToken, &HashMap<ifa_types::CompactString, IfaValue>)> {
        let token_arc = match args.first() {
            Some(IfaValue::Resource(arc)) => arc,
            Some(other) => {
                return Err(IfaError::TypeError {
                    expected: "Resource".into(),
                    got: other.type_name().into(),
                });
            }
            None => {
                return Err(IfaError::ArgumentError("Missing terminal resource".into()));
            }
        };
        let token = **token_arc;
        let map = match args.get(1) {
            Some(IfaValue::Map(m)) => m,
            Some(other) => {
                return Err(IfaError::TypeError {
                    expected: "Map".into(),
                    got: other.type_name().into(),
                });
            }
            None => {
                return Err(IfaError::ArgumentError("Missing map argument".into()));
            }
        };
        Ok((token, map))
    }

    // ── Init / Close ───────────────────────────────────────────────────────

    fn handle_bere(ctx: &mut VmContext) -> IfaResult<IfaValue> {
        enable_raw_mode().map_err(|e| IfaError::Runtime(e.to_string()))?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen).map_err(|e| IfaError::Runtime(e.to_string()))?;

        let default_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            let _ = disable_raw_mode();
            let _ = execute!(io::stdout(), LeaveAlternateScreen);
            default_hook(panic_info);
        }));

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).map_err(|e| IfaError::Runtime(e.to_string()))?;
        let token = ctx.resource_registry().register(Mutex::new(terminal));
        Ok(IfaValue::Resource(Arc::new(token)))
    }

    fn handle_pari(args: Vec<IfaValue>, ctx: &mut VmContext) -> IfaResult<IfaValue> {
        if let Some(IfaValue::Resource(token_arc)) = args.first() {
            let token: ResourceToken = **token_arc;
            if let Some(terminal_mutex) = ctx
                .resource_registry()
                .get::<Mutex<Terminal<CrosstermBackend<io::Stdout>>>>(token)
            {
                let mut terminal = terminal_mutex.lock().unwrap();
                let _ = execute!(terminal.backend_mut(), DisableMouseCapture);
                disable_raw_mode().ok();
                let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen);
                terminal.show_cursor().ok();
            }
            ctx.resource_registry().close(token);
        }
        Ok(IfaValue::null())
    }

    // ── Draw / Widgets ─────────────────────────────────────────────────────

    fn handle_ya(args: Vec<IfaValue>, ctx: &mut VmContext) -> IfaResult<IfaValue> {
        if args.len() < 2 {
            return Err(IfaError::ArgumentError(
                "Ose.ya requires (terminal, ui_map[, area])".into(),
            ));
        }

        let (token, ui_map) = Self::extract_token(&args)?;

        let area_override: Option<Rect> = args.get(2).and_then(|v| match v {
            IfaValue::Map(m) => Some(Self::parse_area(m)),
            _ => None,
        });

        let terminal_arc = ctx
            .resource_registry()
            .get::<Mutex<Terminal<CrosstermBackend<io::Stdout>>>>(token)
            .ok_or_else(|| IfaError::Runtime("Terminal resource not found".into()))?;
        let mut terminal = terminal_arc.lock().unwrap();
        terminal
            .draw(|f| {
                let area = area_override.unwrap_or_else(|| f.area());
                Self::render_widget(ui_map, area, f);
            })
            .map_err(|e| IfaError::Runtime(e.to_string()))?;

        Ok(IfaValue::null())
    }

    // ── Input ──────────────────────────────────────────────────────────────

    fn handle_gboran() -> IfaResult<IfaValue> {
        if event::poll(std::time::Duration::from_millis(10)).unwrap_or(false) {
            match event::read() {
                Ok(Event::Key(key)) => {
                    let s = match key.code {
                        KeyCode::Char(c) => c.to_string(),
                        KeyCode::Enter => "Enter".to_string(),
                        KeyCode::Esc => "Esc".to_string(),
                        KeyCode::Up => "Up".to_string(),
                        KeyCode::Down => "Down".to_string(),
                        KeyCode::Left => "Left".to_string(),
                        KeyCode::Right => "Right".to_string(),
                        KeyCode::Backspace => "Backspace".to_string(),
                        KeyCode::Tab => "Tab".to_string(),
                        KeyCode::Home => "Home".to_string(),
                        KeyCode::End => "End".to_string(),
                        KeyCode::PageUp => "PageUp".to_string(),
                        KeyCode::PageDown => "PageDown".to_string(),
                        KeyCode::Delete => "Delete".to_string(),
                        KeyCode::Insert => "Insert".to_string(),
                        KeyCode::F(n) => return Ok(IfaValue::str(format!("F{}", n))),
                        _ => return Ok(IfaValue::null()),
                    };
                    Ok(IfaValue::str(s))
                }
                Ok(Event::Mouse(m)) => {
                    let kind = match m.kind {
                        MouseEventKind::Down(_) => "down",
                        MouseEventKind::Up(_) => "up",
                        MouseEventKind::Drag(_) => "drag",
                        MouseEventKind::Moved => "move",
                        MouseEventKind::ScrollDown => "scrolldown",
                        MouseEventKind::ScrollUp => "scrollup",
                        MouseEventKind::ScrollLeft => "scrollleft",
                        MouseEventKind::ScrollRight => "scrollright",
                    };
                    let mut map = HashMap::new();
                    map.insert("type".into(), IfaValue::str("mouse"));
                    map.insert("kind".into(), IfaValue::str(kind));
                    map.insert("col".into(), IfaValue::int(m.column as i64));
                    map.insert("row".into(), IfaValue::int(m.row as i64));
                    Ok(IfaValue::map(map))
                }
                Ok(Event::Resize(cols, rows)) => {
                    let mut map = HashMap::new();
                    map.insert("type".into(), IfaValue::str("resize"));
                    map.insert("cols".into(), IfaValue::int(cols as i64));
                    map.insert("rows".into(), IfaValue::int(rows as i64));
                    Ok(IfaValue::map(map))
                }
                _ => Ok(IfaValue::null()),
            }
        } else {
            Ok(IfaValue::null())
        }
    }

    fn handle_gbile() -> IfaResult<IfaValue> {
        loop {
            match event::read() {
                Ok(Event::Key(key)) => {
                    let s = match key.code {
                        KeyCode::Char(c) => c.to_string(),
                        KeyCode::Enter => "Enter".to_string(),
                        KeyCode::Esc => "Esc".to_string(),
                        KeyCode::Up => "Up".to_string(),
                        KeyCode::Down => "Down".to_string(),
                        KeyCode::Left => "Left".to_string(),
                        KeyCode::Right => "Right".to_string(),
                        KeyCode::Backspace => "Backspace".to_string(),
                        KeyCode::Tab => "Tab".to_string(),
                        KeyCode::Home => "Home".to_string(),
                        KeyCode::End => "End".to_string(),
                        KeyCode::PageUp => "PageUp".to_string(),
                        KeyCode::PageDown => "PageDown".to_string(),
                        KeyCode::Delete => "Delete".to_string(),
                        KeyCode::Insert => "Insert".to_string(),
                        KeyCode::F(n) => return Ok(IfaValue::str(format!("F{}", n))),
                        _ => return Ok(IfaValue::str("?")),
                    };
                    return Ok(IfaValue::str(s));
                }
                _ => {}
            }
        }
    }

    /// Blocking wait for next event with optional timeout.
    /// `args[0]` = optional timeout_ms (int). Returns a structured map.
    fn handle_duro(args: Vec<IfaValue>) -> IfaResult<IfaValue> {
        let timeout_ms = args.first().and_then(|v| match v {
            IfaValue::Int(n) => Some(*n),
            _ => None,
        });

        let timeout = timeout_ms
            .map(|ms| std::time::Duration::from_millis(ms.max(0) as u64))
            .unwrap_or(std::time::Duration::from_millis(100));

        if event::poll(timeout).unwrap_or(false) {
            match event::read() {
                Ok(Event::Key(key)) => {
                    let mut map = HashMap::new();
                    map.insert("type".into(), IfaValue::str("key"));
                    let key_str = match key.code {
                        KeyCode::Char(c) => c.to_string(),
                        KeyCode::Enter => "Enter".to_string(),
                        KeyCode::Esc => "Esc".to_string(),
                        KeyCode::Up => "Up".to_string(),
                        KeyCode::Down => "Down".to_string(),
                        KeyCode::Left => "Left".to_string(),
                        KeyCode::Right => "Right".to_string(),
                        KeyCode::Backspace => "Backspace".to_string(),
                        KeyCode::Tab => "Tab".to_string(),
                        KeyCode::Home => "Home".to_string(),
                        KeyCode::End => "End".to_string(),
                        KeyCode::PageUp => "PageUp".to_string(),
                        KeyCode::PageDown => "PageDown".to_string(),
                        KeyCode::Delete => "Delete".to_string(),
                        KeyCode::Insert => "Insert".to_string(),
                        KeyCode::F(n) => format!("F{}", n),
                        _ => "?".to_string(),
                    };
                    map.insert("key".into(), IfaValue::str(key_str));
                    Ok(IfaValue::map(map))
                }
                Ok(Event::Mouse(m)) => {
                    let kind = match m.kind {
                        MouseEventKind::Down(_) => "down",
                        MouseEventKind::Up(_) => "up",
                        MouseEventKind::Drag(_) => "drag",
                        MouseEventKind::Moved => "move",
                        MouseEventKind::ScrollDown => "scrolldown",
                        MouseEventKind::ScrollUp => "scrollup",
                        MouseEventKind::ScrollLeft => "scrollleft",
                        MouseEventKind::ScrollRight => "scrollright",
                    };
                    let mut map = HashMap::new();
                    map.insert("type".into(), IfaValue::str("mouse"));
                    map.insert("kind".into(), IfaValue::str(kind));
                    map.insert("col".into(), IfaValue::int(m.column as i64));
                    map.insert("row".into(), IfaValue::int(m.row as i64));
                    Ok(IfaValue::map(map))
                }
                Ok(Event::Resize(cols, rows)) => {
                    let mut map = HashMap::new();
                    map.insert("type".into(), IfaValue::str("resize"));
                    map.insert("cols".into(), IfaValue::int(cols as i64));
                    map.insert("rows".into(), IfaValue::int(rows as i64));
                    Ok(IfaValue::map(map))
                }
                _ => {
                    let mut map = HashMap::new();
                    map.insert("type".into(), IfaValue::str("unknown"));
                    Ok(IfaValue::map(map))
                }
            }
        } else {
            let mut map = HashMap::new();
            map.insert("type".into(), IfaValue::str("timeout"));
            Ok(IfaValue::map(map))
        }
    }

    // ── Layout ─────────────────────────────────────────────────────────────

    fn handle_ipile(args: Vec<IfaValue>, ctx: &mut VmContext) -> IfaResult<IfaValue> {
        if args.len() < 2 {
            return Err(IfaError::ArgumentError(
                "Ose.ipile requires (terminal, layout_map)".into(),
            ));
        }

        let (token, layout_map) = Self::extract_token(&args)?;
        let terminal_arc = ctx
            .resource_registry()
            .get::<Mutex<Terminal<CrosstermBackend<io::Stdout>>>>(token)
            .ok_or_else(|| IfaError::Runtime("Terminal resource not found".into()))?;
        let terminal = terminal_arc.lock().unwrap();
        let terminal_size = terminal
            .size()
            .map_err(|e| IfaError::Runtime(e.to_string()))?;
        drop(terminal);

        let full_area = Rect::new(0, 0, terminal_size.width, terminal_size.height);

        let direction = match layout_map.get("direction").and_then(|v| match v {
            IfaValue::Str(s) => Some(s.as_str()),
            _ => None,
        }) {
            Some("inaro" | "vertical") => Direction::Vertical,
            _ => Direction::Horizontal,
        };

        let gap = layout_map
            .get("gap")
            .and_then(|v| match v {
                IfaValue::Int(n) => Some(*n as u16),
                _ => None,
            })
            .unwrap_or(0);

        let mut constraints: Vec<Constraint> = Vec::new();
        if let Some(IfaValue::List(items)) = layout_map.get("constraints") {
            for item in items.iter() {
                if let IfaValue::Map(item_map) = item {
                    let c = Self::parse_constraint(item_map);
                    constraints.push(c);
                }
            }
        }
        if constraints.is_empty() {
            constraints.push(Constraint::Percentage(100));
        }

        let layout = Layout::default()
            .direction(direction)
            .constraints(constraints);

        let layout = if gap > 0 {
            match direction {
                Direction::Horizontal => layout.horizontal_margin(gap / 2),
                Direction::Vertical => layout.vertical_margin(gap / 2),
            }
        } else {
            layout
        };

        let rects = layout.split(full_area);
        let areas: Vec<Rect> = rects.iter().copied().collect();

        let result: Vec<IfaValue> = areas
            .iter()
            .map(|r| {
                let mut m = HashMap::new();
                m.insert("x".into(), IfaValue::int(r.x as i64));
                m.insert("y".into(), IfaValue::int(r.y as i64));
                m.insert("width".into(), IfaValue::int(r.width as i64));
                m.insert("height".into(), IfaValue::int(r.height as i64));
                IfaValue::map(m)
            })
            .collect();

        Ok(IfaValue::list(result))
    }

    // ── Mouse capture ──────────────────────────────────────────────────────

    fn handle_mouse_on(args: Vec<IfaValue>, ctx: &mut VmContext) -> IfaResult<IfaValue> {
        let token_arc = match args.first() {
            Some(IfaValue::Resource(arc)) => arc,
            _ => {
                return Err(IfaError::ArgumentError(
                    "Ose.mouse_on requires (terminal)".into(),
                ));
            }
        };
        let token: ResourceToken = **token_arc;
        let terminal_arc = ctx
            .resource_registry()
            .get::<Mutex<Terminal<CrosstermBackend<io::Stdout>>>>(token)
            .ok_or_else(|| IfaError::Runtime("Terminal resource not found".into()))?;
        let mut terminal = terminal_arc.lock().unwrap();
        execute!(terminal.backend_mut(), EnableMouseCapture)
            .map_err(|e| IfaError::Runtime(e.to_string()))?;
        Ok(IfaValue::null())
    }

    fn handle_mouse_off(args: Vec<IfaValue>, ctx: &mut VmContext) -> IfaResult<IfaValue> {
        let token_arc = match args.first() {
            Some(IfaValue::Resource(arc)) => arc,
            _ => {
                return Err(IfaError::ArgumentError(
                    "Ose.mouse_off requires (terminal)".into(),
                ));
            }
        };
        let token: ResourceToken = **token_arc;
        let terminal_arc = ctx
            .resource_registry()
            .get::<Mutex<Terminal<CrosstermBackend<io::Stdout>>>>(token)
            .ok_or_else(|| IfaError::Runtime("Terminal resource not found".into()))?;
        let mut terminal = terminal_arc.lock().unwrap();
        execute!(terminal.backend_mut(), DisableMouseCapture)
            .map_err(|e| IfaError::Runtime(e.to_string()))?;
        Ok(IfaValue::null())
    }

    // ── Terminal size ──────────────────────────────────────────────────────

    fn handle_iwọn() -> IfaResult<IfaValue> {
        let (cols, rows) = size().map_err(|e| IfaError::Runtime(e.to_string()))?;
        let mut map = HashMap::new();
        map.insert("cols".into(), IfaValue::int(cols as i64));
        map.insert("rows".into(), IfaValue::int(rows as i64));
        Ok(IfaValue::map(map))
    }

    // ── Widget Renderers ───────────────────────────────────────────────────

    fn render_widget(
        ui_map: &HashMap<ifa_types::CompactString, IfaValue>,
        area: Rect,
        f: &mut ratatui::Frame,
    ) {
        let widget_type = match ui_map.get("type").and_then(|v| match v {
            IfaValue::Str(s) => Some(s.as_str()),
            _ => None,
        }) {
            Some(t) => t,
            None => return,
        };

        let style = Self::parse_style(ui_map);

        match widget_type {
            "apoti" | "box" => Self::render_apoti(ui_map, area, f, style),
            "ipinro" | "section" => Self::render_ipinro(ui_map, area, f, style),
            "akojọ" | "list" => Self::render_akojọ(ui_map, area, f, style),
            "tabili" | "table" => Self::render_tabili(ui_map, area, f, style),
            "iwọn" | "gauge" => Self::render_iwọn_widget(ui_map, area, f, style),
            "apẹrẹ" | "chart" => Self::render_apẹrẹ(ui_map, area, f, style),
            "tabs" => Self::render_tabs_widget(ui_map, area, f, style),
            "sipaki" | "sparkline" => Self::render_sipaki(ui_map, area, f, style),
            "yiyipo" | "scrollbar" => Self::render_yiyipo(ui_map, area, f, style),
            _ => {}
        }
    }

    fn render_apoti(
        ui_map: &HashMap<ifa_types::CompactString, IfaValue>,
        area: Rect,
        f: &mut ratatui::Frame,
        style: Style,
    ) {
        let title = ui_map
            .get("title")
            .map(|v| v.to_string())
            .unwrap_or_default();
        let text = ui_map
            .get("text")
            .map(|v| v.to_string())
            .unwrap_or_default();
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(style);
        let paragraph = Paragraph::new(text).style(style).block(block);
        f.render_widget(paragraph, area);
    }

    fn render_ipinro(
        ui_map: &HashMap<ifa_types::CompactString, IfaValue>,
        area: Rect,
        f: &mut ratatui::Frame,
        style: Style,
    ) {
        let text = ui_map
            .get("text")
            .map(|v| v.to_string())
            .unwrap_or_default();
        let align = match ui_map.get("align").and_then(|v| match v {
            IfaValue::Str(s) => Some(s.as_str()),
            _ => None,
        }) {
            Some("center") => Alignment::Center,
            Some("right") => Alignment::Right,
            _ => Alignment::Left,
        };
        let paragraph = Paragraph::new(text).style(style).alignment(align);
        f.render_widget(paragraph, area);
    }

    fn render_akojọ(
        ui_map: &HashMap<ifa_types::CompactString, IfaValue>,
        area: Rect,
        f: &mut ratatui::Frame,
        style: Style,
    ) {
        let items: Vec<ListItem> = match ui_map.get("items").and_then(|v| match v {
            IfaValue::List(l) => Some(l),
            _ => None,
        }) {
            Some(list) => list
                .iter()
                .map(|v| ListItem::new(v.to_string()).style(style))
                .collect(),
            None => return,
        };

        let highlight_index = ui_map.get("selected").and_then(|v| match v {
            IfaValue::Int(n) => Some(*n as usize),
            _ => None,
        });

        let highlight_style = ui_map
            .get("highlight_style")
            .and_then(|v| match v {
                IfaValue::Map(m) => Some(Self::parse_style(m)),
                _ => None,
            })
            .unwrap_or_else(|| {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD)
            });

        let widget = List::new(items)
            .style(style)
            .highlight_style(highlight_style)
            .highlight_symbol("> ");

        let mut state = ratatui::widgets::ListState::default();
        if let Some(idx) = highlight_index {
            state.select(Some(idx));
        }
        f.render_stateful_widget(widget, area, &mut state);
    }

    fn render_tabili(
        ui_map: &HashMap<ifa_types::CompactString, IfaValue>,
        area: Rect,
        f: &mut ratatui::Frame,
        style: Style,
    ) {
        let widths: Vec<Constraint> = match ui_map.get("widths").and_then(|v| match v {
            IfaValue::List(l) => Some(l),
            _ => None,
        }) {
            Some(list) => list
                .iter()
                .map(|v| match v {
                    IfaValue::Int(n) => Constraint::Length(*n as u16),
                    IfaValue::Float(n) => Constraint::Percentage(*n as u16),
                    _ => Constraint::Fill(1),
                })
                .collect(),
            None => vec![Constraint::Fill(1)],
        };

        let header_style = ui_map
            .get("header_style")
            .and_then(|v| match v {
                IfaValue::Map(m) => Some(Self::parse_style(m)),
                _ => None,
            })
            .unwrap_or_else(|| style.add_modifier(Modifier::BOLD));

        let header = match ui_map.get("header").and_then(|v| match v {
            IfaValue::List(l) => Some(l),
            _ => None,
        }) {
            Some(list) => {
                Row::new(list.iter().map(|v| v.to_string()).collect::<Vec<_>>()).style(header_style)
            }
            None => Row::new(Vec::<String>::new()),
        };

        let rows: Vec<Row> = match ui_map.get("rows").and_then(|v| match v {
            IfaValue::List(l) => Some(l),
            _ => None,
        }) {
            Some(list) => list
                .iter()
                .map(|row_v| {
                    let cells: Vec<String> = match row_v {
                        IfaValue::List(cells) => cells.iter().map(|c| c.to_string()).collect(),
                        _ => vec![row_v.to_string()],
                    };
                    Row::new(cells).style(style)
                })
                .collect(),
            None => vec![],
        };

        let mut table = Table::new(rows, widths).header(header).style(style);

        if let Some(title) = ui_map.get("title").map(|v| v.to_string()) {
            if !title.is_empty() {
                table = table.block(Block::default().title(title).borders(Borders::ALL));
            }
        }

        f.render_widget(table, area);
    }

    fn render_iwọn_widget(
        ui_map: &HashMap<ifa_types::CompactString, IfaValue>,
        area: Rect,
        f: &mut ratatui::Frame,
        style: Style,
    ) {
        let percent = ui_map
            .get("percent")
            .and_then(|v| match v {
                IfaValue::Int(n) => Some(*n as u16),
                IfaValue::Float(n) => Some(*n as u16),
                _ => None,
            })
            .unwrap_or(0)
            .min(100);

        let label = ui_map
            .get("label")
            .map(|v| v.to_string())
            .unwrap_or_default();

        let mut gauge = Gauge::default().percent(percent).gauge_style(style);

        if !label.is_empty() {
            gauge = gauge.label(label);
        }

        f.render_widget(gauge, area);
    }

    fn render_apẹrẹ(
        ui_map: &HashMap<ifa_types::CompactString, IfaValue>,
        area: Rect,
        f: &mut ratatui::Frame,
        style: Style,
    ) {
        let datasets_list = match ui_map.get("datasets").and_then(|v| match v {
            IfaValue::List(l) => Some(l),
            _ => None,
        }) {
            Some(l) => l,
            None => return,
        };

        struct DatasetSpec {
            name: String,
            data: Vec<(f64, f64)>,
            style: Style,
            graph_type: GraphType,
        }

        let specs: Vec<DatasetSpec> = datasets_list
            .iter()
            .filter_map(|ds_v| {
                let ds_map = match ds_v {
                    IfaValue::Map(m) => m,
                    _ => return None,
                };
                let name = ds_map
                    .get("name")
                    .map(|v| v.to_string())
                    .unwrap_or_default();
                let data: Vec<(f64, f64)> = match ds_map.get("data").and_then(|v| match v {
                    IfaValue::List(l) => Some(l),
                    _ => None,
                }) {
                    Some(points) => points
                        .iter()
                        .filter_map(|pt_v| match pt_v {
                            IfaValue::Map(pt) => {
                                let x = pt.get("x").and_then(|v| match v {
                                    IfaValue::Int(n) => Some(*n as f64),
                                    IfaValue::Float(f) => Some(*f),
                                    _ => None,
                                })?;
                                let y = pt.get("y").and_then(|v| match v {
                                    IfaValue::Int(n) => Some(*n as f64),
                                    IfaValue::Float(f) => Some(*f),
                                    _ => None,
                                })?;
                                Some((x, y))
                            }
                            _ => None,
                        })
                        .collect(),
                    None => vec![],
                };
                let ds_style = ds_map
                    .get("style")
                    .and_then(|v| match v {
                        IfaValue::Map(m) => Some(Self::parse_style(m)),
                        _ => None,
                    })
                    .unwrap_or(style);
                let graph_type = match ds_map.get("graph_type").and_then(|v| match v {
                    IfaValue::Str(s) => Some(s.as_str()),
                    _ => None,
                }) {
                    Some("bar" | "ọpọ") => GraphType::Bar,
                    _ => GraphType::Line,
                };
                Some(DatasetSpec {
                    name,
                    data,
                    style: ds_style,
                    graph_type,
                })
            })
            .collect();

        if specs.is_empty() {
            return;
        }

        let datasets: Vec<Dataset> = specs
            .iter()
            .map(|spec| {
                Dataset::default()
                    .name(spec.name.as_str())
                    .data(&spec.data)
                    .graph_type(spec.graph_type)
                    .style(spec.style)
            })
            .collect();

        if datasets.is_empty() {
            return;
        }

        let x_bounds = ui_map
            .get("x_bounds")
            .and_then(|v| match v {
                IfaValue::List(l) => {
                    let items: Vec<f64> = l
                        .iter()
                        .filter_map(|v| match v {
                            IfaValue::Int(n) => Some(*n as f64),
                            IfaValue::Float(f) => Some(*f),
                            _ => None,
                        })
                        .collect();
                    if items.len() == 2 {
                        Some([items[0], items[1]])
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .unwrap_or([0.0, 100.0]);

        let y_bounds = ui_map
            .get("y_bounds")
            .and_then(|v| match v {
                IfaValue::List(l) => {
                    let items: Vec<f64> = l
                        .iter()
                        .filter_map(|v| match v {
                            IfaValue::Int(n) => Some(*n as f64),
                            IfaValue::Float(f) => Some(*f),
                            _ => None,
                        })
                        .collect();
                    if items.len() == 2 {
                        Some([items[0], items[1]])
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .unwrap_or([0.0, 100.0]);

        let x_title = ui_map
            .get("x_title")
            .map(|v| v.to_string())
            .unwrap_or_default();
        let y_title = ui_map
            .get("y_title")
            .map(|v| v.to_string())
            .unwrap_or_default();

        let x_axis = ratatui::widgets::Axis::default()
            .title(x_title)
            .bounds(x_bounds);
        let y_axis = ratatui::widgets::Axis::default()
            .title(y_title)
            .bounds(y_bounds);

        let chart = Chart::new(datasets)
            .x_axis(x_axis)
            .y_axis(y_axis)
            .style(style);

        f.render_widget(chart, area);
    }

    fn render_tabs_widget(
        ui_map: &HashMap<ifa_types::CompactString, IfaValue>,
        area: Rect,
        f: &mut ratatui::Frame,
        style: Style,
    ) {
        let titles: Vec<Line> = match ui_map.get("titles").and_then(|v| match v {
            IfaValue::List(l) => Some(l),
            _ => None,
        }) {
            Some(list) => list.iter().map(|v| Line::from(v.to_string())).collect(),
            None => return,
        };

        let selected = ui_map
            .get("selected")
            .and_then(|v| match v {
                IfaValue::Int(n) => Some(*n as usize),
                _ => None,
            })
            .unwrap_or(0);

        let highlight_style = ui_map
            .get("highlight_style")
            .and_then(|v| match v {
                IfaValue::Map(m) => Some(Self::parse_style(m)),
                _ => None,
            })
            .unwrap_or_else(|| {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::White)
                    .add_modifier(Modifier::BOLD)
            });

        let tabs = Tabs::new(titles)
            .select(selected)
            .style(style)
            .highlight_style(highlight_style);

        f.render_widget(tabs, area);
    }

    fn render_sipaki(
        ui_map: &HashMap<ifa_types::CompactString, IfaValue>,
        area: Rect,
        f: &mut ratatui::Frame,
        style: Style,
    ) {
        let data: Vec<u64> = match ui_map.get("data").and_then(|v| match v {
            IfaValue::List(l) => Some(l),
            _ => None,
        }) {
            Some(list) => list
                .iter()
                .filter_map(|v| match v {
                    IfaValue::Int(n) => Some(*n as u64),
                    _ => None,
                })
                .collect(),
            None => return,
        };

        let mut sparkline = Sparkline::default().data(&data).style(style);

        match ui_map.get("direction").and_then(|v| match v {
            IfaValue::Str(s) => Some(s.as_str()),
            _ => None,
        }) {
            Some("rtl" | "right_to_left") => {
                sparkline = sparkline.direction(RenderDirection::RightToLeft);
            }
            _ => {
                sparkline = sparkline.direction(RenderDirection::LeftToRight);
            }
        }

        f.render_widget(sparkline, area);
    }

    fn render_yiyipo(
        ui_map: &HashMap<ifa_types::CompactString, IfaValue>,
        area: Rect,
        f: &mut ratatui::Frame,
        style: Style,
    ) {
        let position = ui_map
            .get("position")
            .and_then(|v| match v {
                IfaValue::Int(n) => Some(*n as usize),
                _ => None,
            })
            .unwrap_or(0);

        let content_length = ui_map
            .get("content_length")
            .and_then(|v| match v {
                IfaValue::Int(n) => Some(*n as usize),
                _ => None,
            })
            .unwrap_or(1)
            .max(1);

        let orientation = match ui_map.get("orientation").and_then(|v| match v {
            IfaValue::Str(s) => Some(s.as_str()),
            _ => None,
        }) {
            Some("horiz" | "horizontal" | "petẹsì") => ScrollbarOrientation::HorizontalBottom,
            _ => ScrollbarOrientation::VerticalRight,
        };

        let mut state = ratatui::widgets::ScrollbarState::default()
            .position(position)
            .content_length(content_length);

        let scrollbar = Scrollbar::new(orientation)
            .style(style)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"))
            .track_symbol(Some("│"))
            .thumb_symbol("█");

        f.render_stateful_widget(scrollbar, area, &mut state);
    }

    // ── Helpers ────────────────────────────────────────────────────────────

    fn parse_style(map: &HashMap<ifa_types::CompactString, IfaValue>) -> Style {
        let style_map = match map.get("style").and_then(|v| match v {
            IfaValue::Map(m) => Some(m),
            _ => None,
        }) {
            Some(m) => m,
            None => return Style::default(),
        };

        let fg = parse_color_value(style_map.get("fg"))
            .or_else(|| parse_color_value(style_map.get("foreground")));
        let bg = parse_color_value(style_map.get("bg"))
            .or_else(|| parse_color_value(style_map.get("background")));

        let mut style = Style::default();
        if let Some(c) = fg {
            style = style.fg(c);
        }
        if let Some(c) = bg {
            style = style.bg(c);
        }

        let mut modifier = Modifier::empty();
        if is_truthy(style_map.get("bold")) {
            modifier |= Modifier::BOLD;
        }
        if is_truthy(style_map.get("italic")) {
            modifier |= Modifier::ITALIC;
        }
        if is_truthy(style_map.get("underline")) {
            modifier |= Modifier::UNDERLINED;
        }
        if is_truthy(style_map.get("blink")) {
            modifier |= Modifier::SLOW_BLINK;
        }
        if is_truthy(style_map.get("strikethrough")) {
            modifier |= Modifier::CROSSED_OUT;
        }
        if is_truthy(style_map.get("dim")) {
            modifier |= Modifier::DIM;
        }
        if is_truthy(style_map.get("reverse")) {
            modifier |= Modifier::REVERSED;
        }
        if is_truthy(style_map.get("hidden")) {
            modifier |= Modifier::HIDDEN;
        }

        style.add_modifier(modifier)
    }

    fn parse_area(map: &HashMap<ifa_types::CompactString, IfaValue>) -> Rect {
        let x = map
            .get("x")
            .and_then(|v| match v {
                IfaValue::Int(n) => Some(*n as u16),
                _ => None,
            })
            .unwrap_or(0);
        let y = map
            .get("y")
            .and_then(|v| match v {
                IfaValue::Int(n) => Some(*n as u16),
                _ => None,
            })
            .unwrap_or(0);
        let width = map
            .get("width")
            .and_then(|v| match v {
                IfaValue::Int(n) => Some((*n as u16).max(1)),
                _ => None,
            })
            .unwrap_or(1);
        let height = map
            .get("height")
            .and_then(|v| match v {
                IfaValue::Int(n) => Some((*n as u16).max(1)),
                _ => None,
            })
            .unwrap_or(1);
        Rect::new(x, y, width, height)
    }

    fn parse_constraint(map: &HashMap<ifa_types::CompactString, IfaValue>) -> Constraint {
        let ctype = match map.get("type").and_then(|v| match v {
            IfaValue::Str(s) => Some(s.as_str()),
            _ => None,
        }) {
            Some(t) => t,
            None => return Constraint::Percentage(100),
        };
        let value = map
            .get("value")
            .and_then(|v| match v {
                IfaValue::Int(n) => Some(*n as u16),
                IfaValue::Float(n) => Some(*n as u16),
                _ => None,
            })
            .unwrap_or(100);

        match ctype {
            "ipin" | "percentage" => Constraint::Percentage(value.min(100)),
            "ipin_on" | "ratio" => {
                let of = map
                    .get("of")
                    .and_then(|v| match v {
                        IfaValue::Int(n) => Some(*n as u32),
                        IfaValue::Float(n) => Some(*n as u32),
                        _ => None,
                    })
                    .unwrap_or(100);
                Constraint::Ratio(value as u32, of.max(1))
            }
            "gigun" | "length" => Constraint::Length(value),
            "kere" | "min" => Constraint::Min(value),
            "pọ" | "max" => Constraint::Max(value),
            "kun" | "fill" => Constraint::Fill(value.max(1) as u16),
            _ => Constraint::Percentage(value.min(100)),
        }
    }
}

#[cfg(not(feature = "tui"))]
impl Ose {
    pub fn dispatch(
        method: &str,
        _args: Vec<IfaValue>,
        _ctx: &mut VmContext,
    ) -> IfaResult<IfaValue> {
        Err(IfaError::Runtime("TUI not compiled in minimal mode".into()))
    }
}

// ── Free helper functions ──────────────────────────────────────────────────

#[cfg(feature = "tui")]
fn parse_color_value(value: Option<&IfaValue>) -> Option<Color> {
    match value {
        Some(IfaValue::Str(s)) => {
            let c = s.as_str();
            match c.to_lowercase().as_str() {
                "reset" | "atunto" => None,
                "black" | "dudu" => Some(Color::Black),
                "red" | "pupa" => Some(Color::Red),
                "green" | "ewe" => Some(Color::Green),
                "yellow" | "oye" => Some(Color::Yellow),
                "blue" | "bulu" => Some(Color::Blue),
                "magenta" | "arosayin" => Some(Color::Magenta),
                "cyan" | "omi" => Some(Color::Cyan),
                "white" | "funfun" => Some(Color::White),
                "gray" | "grey" | "eeru" => Some(Color::Gray),
                "dark_gray" | "dark_grey" => Some(Color::DarkGray),
                "light_red" => Some(Color::LightRed),
                "light_green" => Some(Color::LightGreen),
                "light_yellow" => Some(Color::LightYellow),
                "light_blue" => Some(Color::LightBlue),
                "light_magenta" => Some(Color::LightMagenta),
                "light_cyan" => Some(Color::LightCyan),
                c if c.starts_with("color_") => c[6..].parse::<u8>().ok().map(Color::Indexed),
                c if c.starts_with('#') && (c.len() == 4 || c.len() == 7) => {
                    let hex = &c[1..];
                    if hex.len() == 6 {
                        u32::from_str_radix(hex, 16)
                            .ok()
                            .map(|v| Color::Rgb((v >> 16) as u8, (v >> 8) as u8, v as u8))
                    } else {
                        let r = u8::from_str_radix(&hex[0..1], 16).ok()? * 17;
                        let g = u8::from_str_radix(&hex[1..2], 16).ok()? * 17;
                        let b = u8::from_str_radix(&hex[2..3], 16).ok()? * 17;
                        Some(Color::Rgb(r, g, b))
                    }
                }
                _ => None,
            }
        }
        Some(IfaValue::Int(n)) => {
            let i = *n;
            if i >= 0 && i <= 255 {
                Some(Color::Indexed(i as u8))
            } else {
                None
            }
        }
        _ => None,
    }
}

#[cfg(feature = "tui")]
fn is_truthy(value: Option<&IfaValue>) -> bool {
    match value {
        Some(IfaValue::Bool(b)) => *b,
        Some(IfaValue::Int(n)) => *n != 0,
        _ => false,
    }
}
