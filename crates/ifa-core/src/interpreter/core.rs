//! # Ifá-Lang Interpreter
//!
//! Tree-walking interpreter that executes AST directly.
//! This is the bridge between parsing and execution.

use std::collections::HashMap;
use std::io::{self, Write};

use crate::ast::*;
use crate::error::{IfaError, IfaResult};
use crate::lexer::OduDomain;
use crate::value::IfaValue;
use std::fmt::Debug;

/// Debugger trait for execution tracing
pub trait Debugger: Debug {
    fn on_statement(&mut self, stmt: &Statement, env: &Environment);
}

use super::handlers::HandlerRegistry;
// Conditionally use sandbox for native builds, stub for WASM
#[cfg(feature = "native")]
pub use ifa_sandbox::{CapabilitySet, Ofun};

#[cfg(not(feature = "native"))]
pub use self::sandbox_stub::{CapabilitySet, Ofun};

#[cfg(not(feature = "native"))]
mod sandbox_stub {
    use std::path::PathBuf;

    #[derive(Debug, Clone, PartialEq)]
    pub enum Ofun {
        ReadFiles { root: PathBuf },
        WriteFiles { root: PathBuf },
        Network { domains: Vec<String> },
        Execute { programs: Vec<String> },
        Environment { keys: Vec<String> },
        Time,
        Random,
        Stdio,
        System, // Additional fallback
        Bridge { language: String },
    }

    #[derive(Debug, Clone, Default)]
    pub struct CapabilitySet;

    impl CapabilitySet {
        pub fn new() -> Self {
            Self
        }
        pub fn default() -> Self {
            Self
        }
        pub fn check(&self, _cap: &Ofun) -> bool {
            // WASM environment is already sandboxed by the browser
            true
        }
    }
}

use super::environment::Environment;

/// Ose Canvas for ASCII graphics
#[derive(Clone)]
struct OseCanvas {
    width: usize,
    height: usize,
    buffer: Vec<Vec<char>>,
    cursor_x: usize,
    cursor_y: usize,
}

impl OseCanvas {
    fn new() -> Self {
        Self {
            width: 80,
            height: 24,
            buffer: vec![vec![' '; 80]; 24],
            cursor_x: 0,
            cursor_y: 0,
        }
    }

    fn clear(&mut self, fill: char) {
        for row in &mut self.buffer {
            row.fill(fill);
        }
    }

    fn resize(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.buffer = vec![vec![' '; width]; height];
    }

    fn set_pixel(&mut self, x: i64, y: i64, ch: char) {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            self.buffer[y as usize][x as usize] = ch;
        }
    }

    fn write_text(&mut self, x: i64, y: i64, text: &str) {
        for (i, ch) in text.chars().enumerate() {
            self.set_pixel(x + i as i64, y, ch);
        }
    }

    fn draw_line(&mut self, x1: i64, y1: i64, x2: i64, y2: i64, ch: char) {
        // Bresenham's line algorithm
        let dx = (x2 - x1).abs();
        let dy = (y2 - y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx - dy;
        let mut x = x1;
        let mut y = y1;

        loop {
            self.set_pixel(x, y, ch);
            if x == x2 && y == y2 {
                break;
            }
            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }

    fn draw_rect(&mut self, x: i64, y: i64, w: i64, h: i64, ch: char) {
        for i in 0..w {
            self.set_pixel(x + i, y, ch);
            self.set_pixel(x + i, y + h - 1, ch);
        }
        for i in 0..h {
            self.set_pixel(x, y + i, ch);
            self.set_pixel(x + w - 1, y + i, ch);
        }
    }

    fn fill_rect(&mut self, x: i64, y: i64, w: i64, h: i64, ch: char) {
        for dy in 0..h {
            for dx in 0..w {
                self.set_pixel(x + dx, y + dy, ch);
            }
        }
    }

    fn draw_circle(&mut self, xc: i64, yc: i64, r: i64, ch: char) {
        // Midpoint circle algorithm
        let mut x = 0;
        let mut y = r;
        let mut d = 1 - r;

        while x <= y {
            self.set_pixel(xc + x, yc + y, ch);
            self.set_pixel(xc - x, yc + y, ch);
            self.set_pixel(xc + x, yc - y, ch);
            self.set_pixel(xc - x, yc - y, ch);
            self.set_pixel(xc + y, yc + x, ch);
            self.set_pixel(xc - y, yc + x, ch);
            self.set_pixel(xc + y, yc - x, ch);
            self.set_pixel(xc - y, yc - x, ch);

            x += 1;
            if d < 0 {
                d += 2 * x + 1;
            } else {
                y -= 1;
                d += 2 * (x - y) + 1;
            }
        }
    }
}

/// The Ifá Interpreter
pub struct Interpreter {
    pub env: Environment,
    output: Vec<String>,
    /// Already imported modules (to prevent circular imports)
    imported: std::collections::HashSet<String>,
    /// Module search paths
    module_paths: Vec<std::path::PathBuf>,
    /// Current file being executed (for relative imports)
    current_file: Option<std::path::PathBuf>,
    /// Security capabilities
    pub capabilities: CapabilitySet,
    /// Ose canvas for graphics
    canvas: OseCanvas,
    /// Modular domain handlers
    handlers: HandlerRegistry,
    /// Optional debugger hook
    pub debugger: Option<Box<dyn Debugger>>,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut module_paths = Vec::new();
        // Add current directory
        if let Ok(cwd) = std::env::current_dir() {
            module_paths.push(cwd);
        }
        // Add standard library path (if exists)
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                module_paths.push(dir.join("lib"));
            }
        }

        Interpreter {
            env: Environment::new(),
            output: Vec::new(),
            imported: std::collections::HashSet::new(),
            module_paths,
            current_file: None,
            capabilities: CapabilitySet::default(),
            canvas: OseCanvas::new(),
            handlers: HandlerRegistry::new(),
            debugger: None,
        }
    }

    /// Create interpreter with custom file path
    pub fn with_file(file: impl AsRef<std::path::Path>) -> Self {
        let mut interp = Self::new();
        let path = file.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            interp.module_paths.insert(0, parent.to_path_buf());
        }
        interp.current_file = Some(path);
        interp
    }

    /// Set security capabilities
    pub fn set_capabilities(&mut self, capabilities: CapabilitySet) {
        self.capabilities = capabilities;
    }

    /// Attach a debugger
    pub fn set_debugger(&mut self, debugger: Box<dyn Debugger>) {
        self.debugger = Some(debugger);
    }

    /// Check capability and return error if denied
    fn check_capability(&self, cap: &Ofun) -> IfaResult<()> {
        if self.capabilities.check(cap) {
            Ok(())
        } else {
            Err(IfaError::PermissionDenied(format!(
                "Capability denied: {:?}",
                cap
            )))
        }
    }

    /// Execute a program
    pub fn execute(&mut self, program: &Program) -> IfaResult<IfaValue> {
        let mut result = IfaValue::Null;

        for stmt in &program.statements {
            result = self.execute_statement(stmt)?;
        }

        Ok(result)
    }

    /// Get captured output
    pub fn get_output(&self) -> &[String] {
        &self.output
    }

    /// Get canvas output (rendered as string)
    pub fn get_canvas(&self) -> String {
        self.canvas
            .buffer
            .iter()
            .map(|row| row.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Import a module by path (e.g., ["std", "otura"])
    fn import_module(&mut self, path: &[String]) -> IfaResult<()> {
        let module_key = path.join(".");

        // Check for circular imports
        if self.imported.contains(&module_key) {
            return Ok(()); // Already imported
        }

        // Mark as imported to prevent circular imports
        self.imported.insert(module_key.clone());

        // Try to find the module file
        let file_path = self.resolve_module_path(path)?;

        // Read and parse the module
        let source = std::fs::read_to_string(&file_path).map_err(|e| {
            IfaError::Runtime(format!("Cannot read module '{}': {}", module_key, e))
        })?;

        let program = crate::parser::parse(&source).map_err(|e| {
            IfaError::Runtime(format!("Parse error in module '{}': {}", module_key, e))
        })?;

        // Save current file and execute the module
        let prev_file = self.current_file.take();
        self.current_file = Some(file_path);

        // Execute the module's code (will define functions, classes, etc.)
        for stmt in &program.statements {
            self.execute_statement(stmt)?;
        }

        // Restore previous file
        self.current_file = prev_file;

        Ok(())
    }

    /// Resolve module path to file path
    fn resolve_module_path(&self, path: &[String]) -> IfaResult<std::path::PathBuf> {
        if path.is_empty() {
            return Err(IfaError::Runtime("Empty module path".to_string()));
        }

        // Convert module path to file path
        // std.otura -> std/otura.ifa
        let relative_path = format!("{}.ifa", path.join(std::path::MAIN_SEPARATOR_STR));

        // Try each module search path
        for base in &self.module_paths {
            let full_path = base.join(&relative_path);
            if full_path.exists() {
                return Ok(full_path);
            }
        }

        // Also try as a directory with mod.ifa
        let dir_path = format!("{}/mod.ifa", path.join(std::path::MAIN_SEPARATOR_STR));
        for base in &self.module_paths {
            let full_path = base.join(&dir_path);
            if full_path.exists() {
                return Ok(full_path);
            }
        }

        Err(IfaError::Runtime(format!(
            "Module '{}' not found. Searched in: {:?}",
            path.join("."),
            self.module_paths
        )))
    }

    fn execute_statement(&mut self, stmt: &Statement) -> IfaResult<IfaValue> {
        if let Some(debugger) = &mut self.debugger {
            debugger.on_statement(stmt, &self.env);
        }
        match stmt {
            Statement::VarDecl { name, value, .. } => {
                let val = self.evaluate(value)?;
                self.env.define(name, val);
                Ok(IfaValue::Null)
            }

            Statement::Assignment { target, value, .. } => {
                let val = self.evaluate(value)?;
                match target {
                    AssignTarget::Variable(name) => {
                        if !self.env.set(name, val.clone()) {
                            self.env.define(name, val);
                        }
                    }
                    AssignTarget::Index { name, index } => {
                        let idx = self.evaluate(index)?;
                        let container = self.env.get(name).ok_or_else(|| {
                            IfaError::Runtime(format!("Undefined variable: {}", name))
                        })?;

                        match (container, idx) {
                            (IfaValue::List(mut list), IfaValue::Int(i)) => {
                                let len = list.len() as i64;
                                let idx = if i < 0 { len + i } else { i } as usize;
                                if idx < list.len() {
                                    list[idx] = val;
                                    self.env.set(name, IfaValue::List(list));
                                } else {
                                    return Err(IfaError::Runtime(format!(
                                        "Index {} out of bounds",
                                        i
                                    )));
                                }
                            }
                            (IfaValue::Map(mut map), IfaValue::Str(key)) => {
                                map.insert(key, val);
                                self.env.set(name, IfaValue::Map(map));
                            }
                            _ => return Err(IfaError::Runtime("Invalid index assignment".into())),
                        }
                    }
                }
                Ok(IfaValue::Null)
            }

            Statement::Instruction { call, .. } => self.execute_odu_call(call),

            Statement::If {
                condition,
                then_body,
                else_body,
                ..
            } => {
                let cond = self.evaluate(condition)?;
                if cond.is_truthy() {
                    for s in then_body {
                        self.execute_statement(s)?;
                    }
                } else if let Some(else_stmts) = else_body {
                    for s in else_stmts {
                        self.execute_statement(s)?;
                    }
                }
                Ok(IfaValue::Null)
            }

            Statement::While {
                condition, body, ..
            } => {
                while self.evaluate(condition)?.is_truthy() {
                    for s in body {
                        self.execute_statement(s)?;
                    }
                }
                Ok(IfaValue::Null)
            }

            Statement::For {
                var,
                iterable,
                body,
                ..
            } => {
                let iter_val = self.evaluate(iterable)?;
                if let IfaValue::List(items) = iter_val {
                    for item in items {
                        self.env.define(var, item);
                        for s in body {
                            self.execute_statement(s)?;
                        }
                    }
                }
                Ok(IfaValue::Null)
            }

            Statement::Return { value, .. } => {
                let val = if let Some(expr) = value {
                    self.evaluate(expr)?
                } else {
                    IfaValue::Null
                };
                Ok(IfaValue::Return(Box::new(val)))
            }

            Statement::Match {
                condition, arms, ..
            } => {
                let cond_val = self.evaluate(condition)?;
                for arm in arms {
                    let matched = match &arm.pattern {
                        MatchPattern::Literal(expr) => {
                            let pat_val = self.evaluate(expr)?;
                            cond_val == pat_val
                        }
                        MatchPattern::Range { start, end } => {
                            let start_val = self.evaluate(start)?;
                            let end_val = self.evaluate(end)?;
                            match (&cond_val, start_val, end_val) {
                                (IfaValue::Int(v), IfaValue::Int(s), IfaValue::Int(e)) => {
                                    *v >= s && *v <= e
                                }
                                (IfaValue::Float(v), IfaValue::Float(s), IfaValue::Float(e)) => {
                                    *v >= s && *v <= e
                                }
                                _ => false,
                            }
                        }
                        MatchPattern::Wildcard => true,
                    };

                    if matched {
                        for stmt in &arm.body {
                            let res = self.execute_statement(stmt)?;
                            if let IfaValue::Return(_) = res {
                                return Ok(res);
                            }
                        }
                        return Ok(IfaValue::Null);
                    }
                }
                Ok(IfaValue::Null)
            }

            Statement::Ase { .. } => {
                // End marker - no-op
                Ok(IfaValue::Null)
            }

            Statement::EseDef {
                name, params, body, ..
            } => {
                // Store function definition
                let func = IfaValue::AstFn {
                    name: name.clone(),
                    params: params.iter().map(|p| p.name.clone()).collect(),
                    body: body.clone(),
                };
                self.env.define(name, func);
                Ok(IfaValue::Null)
            }

            Statement::Import { path, .. } => {
                self.import_module(path)?;
                Ok(IfaValue::Null)
            }

            Statement::OduDef { name, body, .. } => {
                let mut methods = HashMap::new();
                let mut fields = Vec::new();

                for stmt in body {
                    match stmt {
                        Statement::EseDef {
                            name, params, body, ..
                        } => {
                            let method = IfaValue::AstFn {
                                name: name.clone(),
                                params: params.iter().map(|p| p.name.clone()).collect(),
                                body: body.clone(),
                            };
                            methods.insert(name.clone(), method);
                        }
                        Statement::VarDecl { name, .. } => {
                            fields.push(name.clone());
                        }
                        _ => {}
                    }
                }

                let class = IfaValue::Class {
                    name: name.clone(),
                    fields,
                    methods,
                };

                self.env.define(name, class);
                Ok(IfaValue::Null)
            }

            Statement::Expr { expr, .. } => self.evaluate(expr),

            Statement::Taboo { source, target, .. } => {
                // Taboo declarations are compile-time only, no runtime effect
                // Could log for debugging
                println!("[taboo] {} -> {} forbidden", source, target);
                Ok(IfaValue::Null)
            }

            Statement::Ewo {
                condition,
                message,
                span: _,
            } => {
                // Evaluate the assertion condition
                let result = self.evaluate(condition)?;

                match result {
                    IfaValue::Bool(true) => {
                        // Assertion passed - continue
                        Ok(IfaValue::Null)
                    }
                    IfaValue::Bool(false) => {
                        // Assertion failed - throw error
                        let msg = message
                            .clone()
                            .unwrap_or_else(|| "Assertion failed".to_string());
                        Err(IfaError::Runtime(format!(
                            "[ẹ̀wọ̀/verify] Taboo violated: {}",
                            msg
                        )))
                    }
                    other => {
                        // Non-boolean - throw error
                        Err(IfaError::Runtime(format!(
                            "[ẹ̀wọ̀/verify] Assertion expects boolean, got: {:?}",
                            other
                        )))
                    }
                }
            }

            Statement::Opon { size, .. } => {
                // Opon directive - parse and validate the size
                use crate::opon::OponSize;

                if let Some(opon_size) = OponSize::from_str(size) {
                    println!(
                        "[opon/mem] Memory configured: {} ({} slots, {})",
                        opon_size.display_name(),
                        opon_size.slot_count(),
                        opon_size.approx_memory()
                    );
                } else {
                    println!(
                        "[opon] Warning: Unknown size '{}', using default (arinrin)",
                        size
                    );
                }
                Ok(IfaValue::Null)
            }

            Statement::Ebo { offering, .. } => {
                let val = self.evaluate(offering)?;
                println!("[ẹbọ/sacrifice] Aspect initiated: {}", val);
                Ok(IfaValue::Null)
            }
        }
    }

    fn evaluate(&mut self, expr: &Expression) -> IfaResult<IfaValue> {
        match expr {
            Expression::Int(n) => Ok(IfaValue::Int(*n)),
            Expression::Float(f) => Ok(IfaValue::Float(*f)),
            Expression::String(s) => Ok(IfaValue::Str(s.clone())),
            Expression::Bool(b) => Ok(IfaValue::Bool(*b)),
            Expression::Nil => Ok(IfaValue::Null),

            Expression::Identifier(name) => self
                .env
                .get(name)
                .ok_or_else(|| IfaError::Runtime(format!("Undefined variable: {}", name))),

            Expression::BinaryOp { left, op, right } => {
                let l = self.evaluate(left)?;
                let r = self.evaluate(right)?;
                self.apply_binary_op(&l, op, &r)
            }

            Expression::UnaryOp { op, expr } => {
                let val = self.evaluate(expr)?;
                match op {
                    UnaryOperator::Neg => match val {
                        IfaValue::Int(n) => Ok(IfaValue::Int(-n)),
                        IfaValue::Float(f) => Ok(IfaValue::Float(-f)),
                        _ => Err(IfaError::Runtime("Cannot negate non-number".into())),
                    },
                    UnaryOperator::Not => Ok(IfaValue::Bool(!val.is_truthy())),
                }
            }

            Expression::OduCall(call) => self.execute_odu_call(call),

            Expression::MethodCall {
                object,
                method,
                args,
            } => {
                let obj = self.evaluate(object)?;
                self.call_method(&obj, method, args)
            }

            Expression::Call { name, args } => {
                let func = self
                    .env
                    .get(name)
                    .ok_or_else(|| IfaError::Runtime(format!("Undefined function: {}", name)))?;
                self.call_function(&func, args)
            }

            Expression::List(items) => {
                let mut list = Vec::new();
                for item in items {
                    list.push(self.evaluate(item)?);
                }
                Ok(IfaValue::List(list))
            }

            Expression::Map(entries) => {
                let mut map = HashMap::new();
                for (k, v) in entries {
                    let key = match self.evaluate(k)? {
                        IfaValue::Str(s) => s,
                        _ => return Err(IfaError::Runtime("Map keys must be strings".into())),
                    };
                    map.insert(key, self.evaluate(v)?);
                }
                Ok(IfaValue::Map(map))
            }

            Expression::Index { object, index } => {
                let obj = self.evaluate(object)?;
                let idx = self.evaluate(index)?;

                match (&obj, &idx) {
                    (IfaValue::List(list), IfaValue::Int(i)) => {
                        let i = *i as usize;
                        list.get(i)
                            .cloned()
                            .ok_or_else(|| IfaError::Runtime(format!("Index {} out of bounds", i)))
                    }
                    (IfaValue::Map(map), IfaValue::Str(key)) => map
                        .get(key)
                        .cloned()
                        .ok_or_else(|| IfaError::Runtime(format!("Key '{}' not found", key))),
                    _ => Err(IfaError::Runtime("Invalid index operation".into())),
                }
            }
        }
    }

    fn execute_odu_call(&mut self, call: &OduCall) -> IfaResult<IfaValue> {
        let mut args: Vec<IfaValue> = Vec::new();
        for arg in &call.args {
            args.push(self.evaluate(arg)?);
        }

        // Try modular handlers first (for registered domains)
        if let Some(handler) = self.handlers.get(&call.domain) {
            return handler.call(&call.method, args, &mut self.env, &mut self.output);
        }

        // Fall back to legacy code for domains not yet migrated to handlers
        // (Compound Odù, FFI, Infrastructure stacks, etc.)
        match call.domain {
            // ═══════════════════════════════════════════════════════════════
            // Ìrosù (Console I/O)
            // ═══════════════════════════════════════════════════════════════
            OduDomain::Irosu => match call.method.as_str() {
                "fo" | "sọ" | "print" | "println" => {
                    self.check_capability(&Ofun::Stdio)?;
                    let output: Vec<String> = args.iter().map(|a| a.to_string()).collect();
                    let line = output.join(" ");
                    println!("{}", line);
                    self.output.push(line);
                    Ok(IfaValue::Null)
                }
                "ka" | "input" | "listen" | "gbo" => {
                    self.check_capability(&Ofun::Stdio)?;
                    print!("> ");
                    io::stdout().flush().ok();
                    let mut input = String::new();
                    io::stdin()
                        .read_line(&mut input)
                        .map_err(IfaError::IoError)?;
                    Ok(IfaValue::Str(input.trim().to_string()))
                }
                "kigbe" | "error" => {
                    let msg = args.first().map(|a| a.to_string()).unwrap_or_default();
                    eprintln!("[ERROR] {}", msg);
                    Ok(IfaValue::Null)
                }
                _ => Err(IfaError::Runtime(format!(
                    "Unknown Ìrosù method: {}",
                    call.method
                ))),
            },

            // ═══════════════════════════════════════════════════════════════
            // Ọ̀bàrà (Math Add/Mul)
            // ═══════════════════════════════════════════════════════════════
            OduDomain::Obara => match call.method.as_str() {
                "fikun" | "add" => {
                    let mut sum = 0i64;
                    for arg in &args {
                        match arg {
                            IfaValue::Int(n) => sum += n,
                            IfaValue::Float(f) => sum += *f as i64,
                            _ => {}
                        }
                    }
                    Ok(IfaValue::Int(sum))
                }
                "isodipupo" | "mul" => {
                    let mut product = 1i64;
                    for arg in &args {
                        match arg {
                            IfaValue::Int(n) => product *= n,
                            IfaValue::Float(f) => product *= *f as i64,
                            _ => {}
                        }
                    }
                    Ok(IfaValue::Int(product))
                }
                _ => Err(IfaError::Runtime(format!(
                    "Unknown Ọ̀bàrà method: {}",
                    call.method
                ))),
            },

            // ═══════════════════════════════════════════════════════════════
            // Òtúúrúpọ̀n (Math Sub/Div)
            // ═══════════════════════════════════════════════════════════════
            OduDomain::Oturupon => match call.method.as_str() {
                "din" | "sub" => {
                    if args.len() >= 2 {
                        match (&args[0], &args[1]) {
                            (IfaValue::Int(a), IfaValue::Int(b)) => Ok(IfaValue::Int(a - b)),
                            (IfaValue::Float(a), IfaValue::Float(b)) => Ok(IfaValue::Float(a - b)),
                            _ => Err(IfaError::Runtime("Invalid subtraction operands".into())),
                        }
                    } else {
                        Err(IfaError::Runtime("din requires 2 arguments".into()))
                    }
                }
                "pin" | "div" => {
                    if args.len() >= 2 {
                        match (&args[0], &args[1]) {
                            (IfaValue::Int(a), IfaValue::Int(b)) if *b != 0 => {
                                Ok(IfaValue::Int(a / b))
                            }
                            (IfaValue::Float(a), IfaValue::Float(b)) if *b != 0.0 => {
                                Ok(IfaValue::Float(a / b))
                            }
                            _ => Err(IfaError::Runtime(
                                "Division by zero or invalid operands".into(),
                            )),
                        }
                    } else {
                        Err(IfaError::Runtime("pin requires 2 arguments".into()))
                    }
                }
                _ => Err(IfaError::Runtime(format!(
                    "Unknown Òtúúrúpọ̀n method: {}",
                    call.method
                ))),
            },

            // ═══════════════════════════════════════════════════════════════
            // Ìká (Strings)
            // ═══════════════════════════════════════════════════════════════
            OduDomain::Ika => match call.method.as_str() {
                "so" | "concat" => {
                    let result: String = args.iter().map(|a| a.to_string()).collect();
                    Ok(IfaValue::Str(result))
                }
                "dapo" | "join" => {
                    if args.len() >= 2 {
                        if let (IfaValue::List(parts), IfaValue::Str(delim)) = (&args[0], &args[1])
                        {
                            let result: String = parts
                                .iter()
                                .map(|a| a.to_string())
                                .collect::<Vec<String>>()
                                .join(delim.as_str());
                            return Ok(IfaValue::Str(result));
                        }
                    }
                    Err(IfaError::Runtime("join requires list and delimiter".into()))
                }
                "gigun" | "len" => {
                    if let Some(IfaValue::Str(s)) = args.first() {
                        // Using chars().count() for proper Unicode character count
                        // (Yoruba diacritics like ẹ, ọ, ṣ are single characters)
                        Ok(IfaValue::Int(s.chars().count() as i64))
                    } else {
                        Err(IfaError::Runtime("len requires a string argument".into()))
                    }
                }
                "pin" | "split" => {
                    if args.len() >= 2 {
                        if let (IfaValue::Str(s), IfaValue::Str(delim)) = (&args[0], &args[1]) {
                            let parts: Vec<IfaValue> = s
                                .split(delim.as_str())
                                .map(|p| IfaValue::Str(p.to_string()))
                                .collect();
                            return Ok(IfaValue::List(parts));
                        }
                    }
                    Err(IfaError::Runtime(
                        "split requires string and delimiter".into(),
                    ))
                }
                "ga" | "upper" | "nla" => {
                    if let Some(IfaValue::Str(s)) = args.first() {
                        return Ok(IfaValue::Str(s.to_uppercase()));
                    }
                    Err(IfaError::Runtime("nla requires a string".into()))
                }
                "isale" | "lower" | "kekere" => {
                    if let Some(IfaValue::Str(s)) = args.first() {
                        return Ok(IfaValue::Str(s.to_lowercase()));
                    }
                    Err(IfaError::Runtime("kekere requires a string".into()))
                }
                "ge" | "trim" => {
                    if let Some(IfaValue::Str(s)) = args.first() {
                        return Ok(IfaValue::Str(s.trim().to_string()));
                    }
                    Err(IfaError::Runtime("trim requires a string".into()))
                }
                "ni" | "contains" | "has" => {
                    if args.len() >= 2 {
                        if let (IfaValue::Str(s), IfaValue::Str(sub)) = (&args[0], &args[1]) {
                            return Ok(IfaValue::Bool(s.contains(sub.as_str())));
                        }
                    }
                    Err(IfaError::Runtime(
                        "has requires string and substring".into(),
                    ))
                }
                "rọpo" | "replace" => {
                    if args.len() >= 3 {
                        if let (IfaValue::Str(s), IfaValue::Str(from), IfaValue::Str(to)) =
                            (&args[0], &args[1], &args[2])
                        {
                            return Ok(IfaValue::Str(s.replace(from.as_str(), to.as_str())));
                        }
                    }
                    Err(IfaError::Runtime(
                        "replace requires string, from, and to".into(),
                    ))
                }
                "bẹrẹ_pẹlu" | "starts_with" => {
                    if args.len() >= 2 {
                        if let (IfaValue::Str(s), IfaValue::Str(prefix)) = (&args[0], &args[1]) {
                            return Ok(IfaValue::Bool(s.starts_with(prefix.as_str())));
                        }
                    }
                    Err(IfaError::Runtime(
                        "starts_with requires string and prefix".into(),
                    ))
                }
                "pari_pẹlu" | "ends_with" => {
                    if args.len() >= 2 {
                        if let (IfaValue::Str(s), IfaValue::Str(suffix)) = (&args[0], &args[1]) {
                            return Ok(IfaValue::Bool(s.ends_with(suffix.as_str())));
                        }
                    }
                    Err(IfaError::Runtime(
                        "ends_with requires string and suffix".into(),
                    ))
                }
                "wa" | "find" => {
                    if args.len() >= 2 {
                        if let (IfaValue::Str(s), IfaValue::Str(sub)) = (&args[0], &args[1]) {
                            return Ok(s
                                .find(sub.as_str())
                                .map(|i| IfaValue::Int(i as i64))
                                .unwrap_or(IfaValue::Null));
                        }
                    }
                    Err(IfaError::Runtime(
                        "find requires string and substring".into(),
                    ))
                }
                "bo_asiri" | "encode" => {
                    if let Some(val) = args.first() {
                        return Ok(IfaValue::Str(
                            serde_json::to_string(val).unwrap_or_default(),
                        ));
                    }
                    Err(IfaError::Runtime("encode requires value".into()))
                }
                "titu_asiri" | "decode" => {
                    if let Some(IfaValue::Str(s)) = args.first() {
                        return Ok(serde_json::from_str(s).unwrap_or(IfaValue::Null));
                    }
                    Err(IfaError::Runtime("decode requires string".into()))
                }
                _ => Err(IfaError::Runtime(format!(
                    "Unknown Ìká method: {}",
                    call.method
                ))),
            },

            // ═══════════════════════════════════════════════════════════════
            // Ọ̀yẹ̀kú (Exit/Sleep)
            // ═══════════════════════════════════════════════════════════════
            OduDomain::Oyeku => match call.method.as_str() {
                "jade" | "exit" => {
                    let code = args
                        .first()
                        .and_then(|v| {
                            if let IfaValue::Int(n) = v {
                                Some(*n as i32)
                            } else {
                                None
                            }
                        })
                        .unwrap_or(0);
                    std::process::exit(code);
                }
                "sun" | "sleep" => {
                    self.check_capability(&Ofun::Time)?;
                    if let Some(IfaValue::Int(ms)) = args.first() {
                        std::thread::sleep(std::time::Duration::from_millis(*ms as u64));
                    }
                    Ok(IfaValue::Null)
                }
                _ => Err(IfaError::Runtime(format!(
                    "Unknown Ọ̀yẹ̀kú method: {}",
                    call.method
                ))),
            },

            // ═══════════════════════════════════════════════════════════════
            // Ọ̀wọ́nrín (Random)
            // ═══════════════════════════════════════════════════════════════
            OduDomain::Owonrin => match call.method.as_str() {
                "nọmba" | "random" => {
                    self.check_capability(&Ofun::Random)?;
                    use std::time::{SystemTime, UNIX_EPOCH};
                    let seed = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_nanos() as u64;
                    // Simple LCG random
                    let random = (seed.wrapping_mul(1103515245).wrapping_add(12345) >> 16) & 0x7fff;
                    Ok(IfaValue::Int(random as i64))
                }
                "laarin" | "range" => {
                    self.check_capability(&Ofun::Random)?;
                    // Random between min and max
                    if args.len() >= 2 {
                        if let (IfaValue::Int(min), IfaValue::Int(max)) = (&args[0], &args[1]) {
                            use std::time::{SystemTime, UNIX_EPOCH};
                            let seed = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_nanos() as u64;
                            let random =
                                (seed.wrapping_mul(1103515245).wrapping_add(12345) >> 16) & 0x7fff;
                            let val = min + (random as i64 % (max - min + 1));
                            return Ok(IfaValue::Int(val));
                        }
                    }
                    Err(IfaError::Runtime("range requires two int arguments".into()))
                }
                _ => Err(IfaError::Runtime(format!(
                    "Unknown Ọ̀wọ́nrín method: {}",
                    call.method
                ))),
            },

            // ═══════════════════════════════════════════════════════════════
            // Ògúndá (Arrays)
            // ═══════════════════════════════════════════════════════════════
            OduDomain::Ogunda => match call.method.as_str() {
                "da" | "create" => Ok(IfaValue::List(args)),
                "gigun" | "len" => {
                    if let Some(IfaValue::List(list)) = args.first() {
                        Ok(IfaValue::Int(list.len() as i64))
                    } else {
                        Err(IfaError::Runtime("len requires a list argument".into()))
                    }
                }
                "fikun" | "push" => {
                    if args.len() >= 2 {
                        if let IfaValue::List(mut list) = args[0].clone() {
                            list.push(args[1].clone());
                            return Ok(IfaValue::List(list));
                        }
                    }
                    Err(IfaError::Runtime("push requires list and element".into()))
                }
                "yọ" | "pop" => {
                    if let Some(IfaValue::List(mut list)) = args.first().cloned() {
                        let val = list.pop().unwrap_or(IfaValue::Null);
                        return Ok(val);
                    }
                    Err(IfaError::Runtime("pop requires a list".into()))
                }
                "yipada" | "reverse" => {
                    if let Some(IfaValue::List(mut list)) = args.first().cloned() {
                        list.reverse();
                        return Ok(IfaValue::List(list));
                    }
                    Err(IfaError::Runtime("reverse requires a list".into()))
                }
                "akọkọ" | "first" => {
                    if let Some(IfaValue::List(list)) = args.first() {
                        return Ok(list.first().cloned().unwrap_or(IfaValue::Null));
                    }
                    Err(IfaError::Runtime("first requires a list".into()))
                }
                "ikẹhin" | "last" => {
                    if let Some(IfaValue::List(list)) = args.first() {
                        return Ok(list.last().cloned().unwrap_or(IfaValue::Null));
                    }
                    Err(IfaError::Runtime("last requires a list".into()))
                }
                _ => Err(IfaError::Runtime(format!(
                    "Unknown Ògúndá method: {}",
                    call.method
                ))),
            },

            // ═══════════════════════════════════════════════════════════════
            // Ọ̀gbè (System)
            // ═══════════════════════════════════════════════════════════════
            OduDomain::Ogbe => match call.method.as_str() {
                "awọn_asẹ" | "args" => {
                    let args_list: Vec<IfaValue> =
                        std::env::args().skip(1).map(IfaValue::Str).collect();
                    Ok(IfaValue::List(args_list))
                }
                "ayika" | "env" => {
                    if let Some(IfaValue::Str(key)) = args.first() {
                        self.check_capability(&Ofun::Environment {
                            keys: vec![key.clone()],
                        })?;
                        let val = std::env::var(key).unwrap_or_default();
                        return Ok(IfaValue::Str(val));
                    }
                    Err(IfaError::Runtime("env requires a string key".into()))
                }
                "ẹya" | "platform" => Ok(IfaValue::Str(std::env::consts::OS.to_string())),
                _ => Err(IfaError::Runtime(format!(
                    "Unknown Ọ̀gbè method: {}",
                    call.method
                ))),
            },

            // ═══════════════════════════════════════════════════════════════
            // Ìwòrì (Time)
            // ═══════════════════════════════════════════════════════════════
            OduDomain::Iwori => match call.method.as_str() {
                "akoko" | "now" => {
                    self.check_capability(&Ofun::Time)?;
                    use std::time::{SystemTime, UNIX_EPOCH};
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    Ok(IfaValue::Int(now as i64))
                }
                "millis" => {
                    self.check_capability(&Ofun::Time)?;
                    use std::time::{SystemTime, UNIX_EPOCH};
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis();
                    Ok(IfaValue::Int(now as i64))
                }
                _ => Err(IfaError::Runtime(format!(
                    "Unknown Ìwòrì method: {}",
                    call.method
                ))),
            },

            // ═══════════════════════════════════════════════════════════════
            // Òtúrá (Networking / Backend)
            // ═══════════════════════════════════════════════════════════════
            OduDomain::Otura => match call.method.as_str() {
                "http_get" | "gba" => {
                    if let Some(IfaValue::Str(url)) = args.first() {
                        let domain = url
                            .split("://")
                            .nth(1)
                            .and_then(|s| s.split('/').next())
                            .unwrap_or("")
                            .to_string();
                        self.check_capability(&Ofun::Network {
                            domains: vec![domain],
                        })?;
                        println!("[HTTP] GET {}", url);
                        // Placeholder - would use reqwest
                        Ok(IfaValue::Str(format!(
                            "{{\"url\": \"{}\", \"status\": 200}}",
                            url
                        )))
                    } else {
                        Err(IfaError::Runtime("http_get requires URL".into()))
                    }
                }
                "http_post" | "fi" => {
                    if args.len() >= 2 {
                        if let (IfaValue::Str(url), body) = (&args[0], &args[1]) {
                            let domain = url
                                .split("://")
                                .nth(1)
                                .and_then(|s| s.split('/').next())
                                .unwrap_or("")
                                .to_string();
                            self.check_capability(&Ofun::Network {
                                domains: vec![domain.clone()],
                            })?;
                            println!("[HTTP] POST {} {:?}", url, body);
                            return Ok(IfaValue::Str("{\"status\": 201}".to_string()));
                        }
                    }
                    Err(IfaError::Runtime("http_post requires URL and body".into()))
                }
                "serve" | "sin" => {
                    if let Some(IfaValue::Int(port)) = args.first() {
                        self.check_capability(&Ofun::Network {
                            domains: vec!["*".to_string()],
                        })?;
                        println!("🚀 HTTP Server starting on port {}", port);
                        return Ok(IfaValue::Str(format!("Server running on :{}", port)));
                    }
                    Err(IfaError::Runtime("serve requires port number".into()))
                }
                "ws_connect" => {
                    if let Some(IfaValue::Str(url)) = args.first() {
                        let domain = url
                            .split("://")
                            .nth(1)
                            .and_then(|s| s.split('/').next())
                            .unwrap_or("")
                            .to_string();
                        self.check_capability(&Ofun::Network {
                            domains: vec![domain],
                        })?;
                        println!("[WS] Connecting to {}", url);
                        return Ok(IfaValue::Str("websocket connected".to_string()));
                    }
                    Err(IfaError::Runtime("ws_connect requires URL".into()))
                }
                _ => Err(IfaError::Runtime(format!(
                    "Unknown Òtúrá method: {}",
                    call.method
                ))),
            },

            // ═══════════════════════════════════════════════════════════════
            // Òdí (Files / Database)
            // ═══════════════════════════════════════════════════════════════
            OduDomain::Odi => match call.method.as_str() {
                "ka" | "read" => {
                    if let Some(IfaValue::Str(path)) = args.first() {
                        self.check_capability(&Ofun::ReadFiles {
                            root: std::path::PathBuf::from(path),
                        })?;
                        match std::fs::read_to_string(path) {
                            Ok(content) => Ok(IfaValue::Str(content)),
                            Err(e) => Err(IfaError::Runtime(format!("Cannot read file: {}", e))),
                        }
                    } else {
                        Err(IfaError::Runtime("read requires file path".into()))
                    }
                }
                "ko" | "write" => {
                    if args.len() >= 2 {
                        if let (IfaValue::Str(path), content) = (&args[0], &args[1]) {
                            self.check_capability(&Ofun::WriteFiles {
                                root: std::path::PathBuf::from(path),
                            })?;
                            let text = content.to_string();
                            match std::fs::write(path, &text) {
                                Ok(_) => return Ok(IfaValue::Bool(true)),
                                Err(e) => {
                                    return Err(IfaError::Runtime(format!(
                                        "Cannot write file: {}",
                                        e
                                    )));
                                }
                            }
                        }
                    }
                    Err(IfaError::Runtime("write requires path and content".into()))
                }
                "wa" | "exists" => {
                    if let Some(IfaValue::Str(path)) = args.first() {
                        self.check_capability(&Ofun::ReadFiles {
                            root: std::path::PathBuf::from(path),
                        })?;
                        return Ok(IfaValue::Bool(std::path::Path::new(path).exists()));
                    }
                    Err(IfaError::Runtime("exists requires path".into()))
                }
                "ṣẹda_fọọ́dà" | "mkdir" => {
                    if let Some(IfaValue::Str(path)) = args.first() {
                        self.check_capability(&Ofun::WriteFiles {
                            root: std::path::PathBuf::from(path),
                        })?;
                        match std::fs::create_dir_all(path) {
                            Ok(_) => return Ok(IfaValue::Bool(true)),
                            Err(e) => {
                                return Err(IfaError::Runtime(format!("Cannot mkdir: {}", e)));
                            }
                        }
                    }
                    Err(IfaError::Runtime("mkdir requires path".into()))
                }
                "àwọn_faili" | "list" => {
                    if let Some(IfaValue::Str(path)) = args.first() {
                        match std::fs::read_dir(path) {
                            Ok(entries) => {
                                let files: Vec<IfaValue> = entries
                                    .filter_map(|e| e.ok())
                                    .filter_map(|e| e.file_name().into_string().ok())
                                    .map(IfaValue::Str)
                                    .collect();
                                return Ok(IfaValue::List(files));
                            }
                            Err(e) => return Err(IfaError::Runtime(format!("Cannot list: {}", e))),
                        }
                    }
                    Err(IfaError::Runtime("list requires path".into()))
                }
                "sql" => {
                    // SQL requires a database connection - return error in interpreter
                    // Real SQL execution happens via Odi.connect() + query methods
                    Err(IfaError::Runtime(
                        "sql() requires database connection. Use Odi.connect(path) first, then db.query(sql)".into()
                    ))
                }
                _ => Err(IfaError::Runtime(format!(
                    "Unknown Òdí method: {}",
                    call.method
                ))),
            },

            // ═══════════════════════════════════════════════════════════════
            // Ìrẹtẹ̀ (Crypto / Security)
            // ═══════════════════════════════════════════════════════════════
            OduDomain::Irete => match call.method.as_str() {
                "sha256" => {
                    if let Some(IfaValue::Str(data)) = args.first() {
                        // Real SHA-256 implementation using simple software hash
                        // This matches crypto.rs but doesn't require ring dependency
                        let hash = sha256_simple(data.as_bytes());
                        let hex: String = hash.iter().map(|b| format!("{:02x}", b)).collect();
                        return Ok(IfaValue::Str(hex));
                    }
                    Err(IfaError::Runtime("sha256 requires string".into()))
                }
                "base64_encode" | "si_base64" => {
                    if let Some(IfaValue::Str(data)) = args.first() {
                        // Real base64 encoding
                        let encoded = base64_encode(data.as_bytes());
                        return Ok(IfaValue::Str(encoded));
                    }
                    Err(IfaError::Runtime("base64_encode requires string".into()))
                }
                "uuid" => {
                    use std::time::{SystemTime, UNIX_EPOCH};
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_nanos();
                    let uuid = format!(
                        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
                        (now >> 96) as u32,
                        (now >> 80) as u16,
                        (now >> 64) as u16,
                        (now >> 48) as u16,
                        now as u64 & 0xFFFFFFFFFFFF
                    );
                    Ok(IfaValue::Str(uuid))
                }
                "random_bytes" => {
                    if let Some(IfaValue::Int(n)) = args.first() {
                        let bytes: Vec<IfaValue> = (0..*n as usize)
                            .map(|i| IfaValue::Int((i * 17 % 256) as i64))
                            .collect();
                        return Ok(IfaValue::List(bytes));
                    }
                    Err(IfaError::Runtime("random_bytes requires count".into()))
                }
                _ => Err(IfaError::Runtime(format!(
                    "Unknown Ìrẹtẹ̀ method: {}",
                    call.method
                ))),
            },

            // ═══════════════════════════════════════════════════════════════
            // Ọ̀sá (Concurrency)
            // ═══════════════════════════════════════════════════════════════
            OduDomain::Osa => match call.method.as_str() {
                "spawn" | "pilẹ" => {
                    println!("[ASYNC] Spawning task...");
                    Ok(IfaValue::Str("task_handle".to_string()))
                }
                "sleep" | "sun" => {
                    if let Some(IfaValue::Int(ms)) = args.first() {
                        std::thread::sleep(std::time::Duration::from_millis(*ms as u64));
                        return Ok(IfaValue::Null);
                    }
                    Err(IfaError::Runtime("sleep requires milliseconds".into()))
                }
                "await" | "duro" => {
                    println!("[ASYNC] Awaiting task...");
                    Ok(IfaValue::Null)
                }
                "channel" => {
                    println!("[ASYNC] Creating channel...");
                    Ok(IfaValue::Str("channel_handle".to_string()))
                }
                _ => Err(IfaError::Runtime(format!(
                    "Unknown Ọ̀sá method: {}",
                    call.method
                ))),
            },

            // ═══════════════════════════════════════════════════════════════
            // Ọ̀ṣẹ́ (Graphics / UI - via ratatui)
            // ═══════════════════════════════════════════════════════════════
            OduDomain::Ose => match call.method.as_str() {
                // Canvas operations
                "nu" => {
                    let fill = args
                        .first()
                        .and_then(|v| {
                            if let IfaValue::Str(s) = v {
                                s.chars().next()
                            } else {
                                None
                            }
                        })
                        .unwrap_or(' ');
                    self.canvas.clear(fill);
                    Ok(IfaValue::Null)
                }
                "ya" => {
                    if args.len() >= 3 {
                        if let (IfaValue::Int(x), IfaValue::Int(y), IfaValue::Str(ch_str)) =
                            (&args[0], &args[1], &args[2])
                        {
                            let ch = ch_str.chars().next().unwrap_or('#');
                            self.canvas.set_pixel(*x, *y, ch);
                            return Ok(IfaValue::Null);
                        }
                    }
                    Err(IfaError::Runtime(
                        "ya requires (x: int, y: int, char: str)".into(),
                    ))
                }
                "han" | "kunle" | "fihan" => {
                    // Render canvas - just return null, canvas will be captured by get_canvas()
                    Ok(IfaValue::Null)
                }
                "tobi" => {
                    if args.len() >= 2 {
                        if let (IfaValue::Int(w), IfaValue::Int(h)) = (&args[0], &args[1]) {
                            self.canvas.resize(*w as usize, *h as usize);
                            return Ok(IfaValue::Null);
                        }
                    }
                    Err(IfaError::Runtime(
                        "tobi requires (width: int, height: int)".into(),
                    ))
                }
                "ila" => {
                    if args.len() >= 5 {
                        if let (
                            IfaValue::Int(x1),
                            IfaValue::Int(y1),
                            IfaValue::Int(x2),
                            IfaValue::Int(y2),
                            IfaValue::Str(ch_str),
                        ) = (&args[0], &args[1], &args[2], &args[3], &args[4])
                        {
                            let ch = ch_str.chars().next().unwrap_or('#');
                            self.canvas.draw_line(*x1, *y1, *x2, *y2, ch);
                            return Ok(IfaValue::Null);
                        }
                    }
                    Err(IfaError::Runtime(
                        "ila requires (x1, y1, x2, y2: int, char: str)".into(),
                    ))
                }
                "onigun" => {
                    if args.len() >= 5 {
                        if let (
                            IfaValue::Int(x),
                            IfaValue::Int(y),
                            IfaValue::Int(w),
                            IfaValue::Int(h),
                            IfaValue::Str(ch_str),
                        ) = (&args[0], &args[1], &args[2], &args[3], &args[4])
                        {
                            let ch = ch_str.chars().next().unwrap_or('#');
                            self.canvas.draw_rect(*x, *y, *w, *h, ch);
                            return Ok(IfaValue::Null);
                        }
                    }
                    Err(IfaError::Runtime(
                        "onigun requires (x, y, w, h: int, char: str)".into(),
                    ))
                }
                "onigun_kun" => {
                    if args.len() >= 5 {
                        if let (
                            IfaValue::Int(x),
                            IfaValue::Int(y),
                            IfaValue::Int(w),
                            IfaValue::Int(h),
                            IfaValue::Str(ch_str),
                        ) = (&args[0], &args[1], &args[2], &args[3], &args[4])
                        {
                            let ch = ch_str.chars().next().unwrap_or('#');
                            self.canvas.fill_rect(*x, *y, *w, *h, ch);
                            return Ok(IfaValue::Null);
                        }
                    }
                    Err(IfaError::Runtime(
                        "onigun_kun requires (x, y, w, h: int, char: str)".into(),
                    ))
                }
                "iyokoto" => {
                    if args.len() >= 4 {
                        if let (
                            IfaValue::Int(xc),
                            IfaValue::Int(yc),
                            IfaValue::Int(r),
                            IfaValue::Str(ch_str),
                        ) = (&args[0], &args[1], &args[2], &args[3])
                        {
                            let ch = ch_str.chars().next().unwrap_or('O');
                            self.canvas.draw_circle(*xc, *yc, *r, ch);
                            return Ok(IfaValue::Null);
                        }
                    }
                    Err(IfaError::Runtime(
                        "iyokoto requires (xc, yc, r: int, char: str)".into(),
                    ))
                }
                "ko" => {
                    if args.len() >= 3 {
                        if let (IfaValue::Int(x), IfaValue::Int(y), IfaValue::Str(text)) =
                            (&args[0], &args[1], &args[2])
                        {
                            self.canvas.write_text(*x, *y, text);
                            return Ok(IfaValue::Null);
                        }
                    }
                    Err(IfaError::Runtime(
                        "ko requires (x, y: int, text: str)".into(),
                    ))
                }
                "fi_x" => {
                    if let Some(IfaValue::Int(x)) = args.first() {
                        self.canvas.cursor_x = *x as usize;
                        return Ok(IfaValue::Null);
                    }
                    Err(IfaError::Runtime("fi_x requires (x: int)".into()))
                }
                "fi_y" => {
                    if let Some(IfaValue::Int(y)) = args.first() {
                        self.canvas.cursor_y = *y as usize;
                        return Ok(IfaValue::Null);
                    }
                    Err(IfaError::Runtime("fi_y requires (y: int)".into()))
                }
                "ya_nibi" => {
                    let ch = args
                        .first()
                        .and_then(|v| {
                            if let IfaValue::Str(s) = v {
                                s.chars().next()
                            } else {
                                None
                            }
                        })
                        .unwrap_or('#');
                    self.canvas.set_pixel(
                        self.canvas.cursor_x as i64,
                        self.canvas.cursor_y as i64,
                        ch,
                    );
                    Ok(IfaValue::Null)
                }
                // Legacy terminal methods (kept for compatibility)
                "clear" | "mọ́" => {
                    print!("\x1B[2J\x1B[H");
                    Ok(IfaValue::Null)
                }
                "color" | "àwọ̀" | "awo" => {
                    if let Some(IfaValue::Str(color)) = args.first() {
                        let code = match color.as_str() {
                            "red" | "pupa" => "\x1B[31m",
                            "green" | "ewé" => "\x1B[32m",
                            "blue" | "búlù" => "\x1B[34m",
                            "yellow" | "iyẹlé" => "\x1B[33m",
                            "reset" | "pada" => "\x1B[0m",
                            _ => "\x1B[0m",
                        };
                        print!("{}", code);
                        return Ok(IfaValue::Null);
                    }
                    Ok(IfaValue::Null) // Stub for canvas mode
                }
                "cursor" | "atọka" => {
                    if args.len() >= 2 {
                        if let (IfaValue::Int(x), IfaValue::Int(y)) = (&args[0], &args[1]) {
                            print!("\x1B[{};{}H", y, x);
                            return Ok(IfaValue::Null);
                        }
                    }
                    Err(IfaError::Runtime("cursor requires x and y".into()))
                }
                "box" | "apoti" | "botini" => {
                    if args.len() >= 3 {
                        if let (IfaValue::Int(x), IfaValue::Int(y)) = (&args[0], &args[1]) {
                            // For botini (button), third arg is label
                            if let Some(IfaValue::Str(label)) = args.get(2) {
                                let w = label.len() as i64 + 2;
                                self.canvas.draw_rect(*x, *y, w, 3, '#');
                                self.canvas.write_text(*x + 1, *y + 1, label);
                                return Ok(IfaValue::Null);
                            }
                            // For box, need width and height
                            if args.len() >= 4 {
                                if let (IfaValue::Int(w), IfaValue::Int(h)) = (&args[2], &args[3]) {
                                    self.canvas.draw_rect(*x, *y, *w, *h, '#');
                                    return Ok(IfaValue::Null);
                                }
                            }
                        }
                    }
                    Err(IfaError::Runtime(
                        "box requires x, y, width, height OR botini requires x, y, label".into(),
                    ))
                }
                _ => Err(IfaError::Runtime(format!(
                    "Unknown Ọ̀ṣẹ́ method: {}",
                    call.method
                ))),
            },

            // ═══════════════════════════════════════════════════════════════
            // Òfún (Permissions / Sandbox)
            // ═══════════════════════════════════════════════════════════════
            OduDomain::Ofun => match call.method.as_str() {
                "allow" | "gba_laaye" => {
                    if let Some(IfaValue::Str(perm)) = args.first() {
                        println!("[PERM] Requesting permission: {}", perm);
                        return Ok(IfaValue::Bool(true));
                    }
                    Err(IfaError::Runtime("allow requires permission name".into()))
                }
                "deny" | "kọ" => {
                    if let Some(IfaValue::Str(perm)) = args.first() {
                        println!("[PERM] Denying permission: {}", perm);
                        return Ok(IfaValue::Bool(true));
                    }
                    Err(IfaError::Runtime("deny requires permission name".into()))
                }
                "check" | "ṣayẹwo" => {
                    if let Some(IfaValue::Str(perm)) = args.first() {
                        println!("[PERM] Checking permission: {}", perm);
                        return Ok(IfaValue::Bool(true)); // Default allow
                    }
                    Err(IfaError::Runtime("check requires permission name".into()))
                }
                _ => Err(IfaError::Runtime(format!(
                    "Unknown Òfún method: {}",
                    call.method
                ))),
            },

            // ═══════════════════════════════════════════════════════════════
            // Coop / Àjọṣe (FFI Bridge)
            // ═══════════════════════════════════════════════════════════════
            OduDomain::Coop => match call.method.as_str() {
                "py" => {
                    // Call Python function via subprocess: Coop.py("math", "sqrt", 16)
                    if args.len() >= 2 {
                        let module = args[0].to_string();
                        let func = args[1].to_string();
                        let func_args: Vec<String> = args[2..]
                            .iter()
                            .map(|a| {
                                let s = a.to_string();
                                // Quote strings for Python
                                if matches!(a, IfaValue::Str(_)) {
                                    format!("\"{}\"", s)
                                } else {
                                    s
                                }
                            })
                            .collect();

                        // Build Python script: import module; print(module.func(args))
                        let py_args = func_args.join(", ");
                        let script =
                            format!("import {}; print({}.{}({}))", module, module, func, py_args);

                        // Security: Clear environment to prevent side-loading
                        // and add a timeout mechanism
                        let mut command = std::process::Command::new("python3");
                        command
                            .args(["-c", &script])
                            .env_clear() // 🛡️ Prevent env-based injection
                            .env("PATH", std::env::var("PATH").unwrap_or_default()); // Keep basic path

                        let output = command.output().or_else(|_| {
                            std::process::Command::new("python")
                                .args(["-c", &script])
                                .env_clear()
                                .env("PATH", std::env::var("PATH").unwrap_or_default())
                                .output()
                        });

                        match output {
                            Ok(o) if o.status.success() => {
                                let result = String::from_utf8_lossy(&o.stdout).trim().to_string();
                                return Ok(IfaValue::Str(result));
                            }
                            Ok(o) => {
                                let err = String::from_utf8_lossy(&o.stderr).trim().to_string();
                                return Err(IfaError::Runtime(format!("Python error: {}", err)));
                            }
                            Err(e) => {
                                return Err(IfaError::Runtime(format!(
                                    "Failed to run Python: {}. Is Python installed?",
                                    e
                                )));
                            }
                        }
                    }
                    Err(IfaError::Runtime(
                        "py requires module and function name".into(),
                    ))
                }
                "sh" | "shell" => {
                    // Execute shell command: Coop.sh("echo hello")
                    if let Some(IfaValue::Str(cmd)) = args.first() {
                        // Block dangerous commands
                        let blocked = ["rm", "del", "format", "mkfs", "dd", "sudo"];
                        for b in blocked {
                            if cmd.contains(b) {
                                return Err(IfaError::Runtime(format!("Blocked command: {}", b)));
                            }
                        }

                        #[cfg(windows)]
                        let output = std::process::Command::new("cmd").args(["/C", cmd]).output();
                        #[cfg(not(windows))]
                        let output = std::process::Command::new("sh").args(["-c", cmd]).output();

                        match output {
                            Ok(o) => Ok(IfaValue::Str(
                                String::from_utf8_lossy(&o.stdout).to_string(),
                            )),
                            Err(e) => Err(IfaError::Runtime(format!("Shell error: {}", e))),
                        }
                    } else {
                        Err(IfaError::Runtime("sh requires command string".into()))
                    }
                }
                "eval" => {
                    // SECURITY: Dynamic code execution is blocked
                    // This prevents code injection attacks
                    Err(IfaError::Runtime(
                        "eval() is disabled for security. Use pattern matching or closures instead.".into()
                    ))
                }
                "ffi" => {
                    // Generic FFI call: Coop.ffi("lib", "func", arg1, arg2)
                    if args.len() >= 2 {
                        let lib = args[0].to_string();
                        let func = args[1].to_string();
                        println!("[FFI] {}.{}({:?})", lib, func, &args[2..]);
                        return Ok(IfaValue::Null);
                    }
                    Err(IfaError::Runtime("ffi requires lib and func".into()))
                }
                "js" | "node" => {
                    // Call JavaScript via Node.js: Coop.js("console.log(Math.sqrt(16))")
                    if let Some(IfaValue::Str(code)) = args.first() {
                        let output = std::process::Command::new("node")
                            .args(["-e", code])
                            .output();

                        match output {
                            Ok(o) if o.status.success() => {
                                let result = String::from_utf8_lossy(&o.stdout).trim().to_string();
                                return Ok(IfaValue::Str(result));
                            }
                            Ok(o) => {
                                let err = String::from_utf8_lossy(&o.stderr).trim().to_string();
                                return Err(IfaError::Runtime(format!(
                                    "JavaScript error: {}",
                                    err
                                )));
                            }
                            Err(e) => {
                                return Err(IfaError::Runtime(format!(
                                    "Failed to run Node.js: {}. Is Node installed?",
                                    e
                                )));
                            }
                        }
                    }
                    Err(IfaError::Runtime("js requires code string".into()))
                }
                "c" | "gcc" => {
                    // Compile and run C code: Coop.c("#include <stdio.h>\nint main(){printf(\"hi\");return 0;}")
                    if let Some(IfaValue::Str(code)) = args.first() {
                        use std::io::Write;

                        // Create temp files
                        let temp_dir = std::env::temp_dir();
                        let src_file = temp_dir.join("ifa_temp.c");
                        let exe_file = temp_dir.join(if cfg!(windows) {
                            "ifa_temp.exe"
                        } else {
                            "ifa_temp"
                        });

                        // Write C source
                        if let Ok(mut f) = std::fs::File::create(&src_file) {
                            let _ = f.write_all(code.as_bytes());
                        } else {
                            return Err(IfaError::Runtime("Failed to create temp C file".into()));
                        }

                        // Compile with gcc/clang
                        let compiler = if cfg!(windows) { "gcc" } else { "cc" };
                        let compile = std::process::Command::new(compiler)
                            .args(["-o", exe_file.to_str().unwrap(), src_file.to_str().unwrap()])
                            .output();

                        match compile {
                            Ok(o) if o.status.success() => {
                                // Run the compiled executable
                                let run = std::process::Command::new(&exe_file).output();
                                let _ = std::fs::remove_file(&src_file);
                                let _ = std::fs::remove_file(&exe_file);

                                match run {
                                    Ok(o) => {
                                        let result = String::from_utf8_lossy(&o.stdout).to_string();
                                        return Ok(IfaValue::Str(result));
                                    }
                                    Err(e) => {
                                        return Err(IfaError::Runtime(format!(
                                            "Failed to run C program: {}",
                                            e
                                        )));
                                    }
                                }
                            }
                            Ok(o) => {
                                let _ = std::fs::remove_file(&src_file);
                                let err = String::from_utf8_lossy(&o.stderr).to_string();
                                return Err(IfaError::Runtime(format!("C compile error: {}", err)));
                            }
                            Err(e) => {
                                let _ = std::fs::remove_file(&src_file);
                                return Err(IfaError::Runtime(format!(
                                    "Failed to run compiler: {}. Is GCC installed?",
                                    e
                                )));
                            }
                        }
                    }
                    Err(IfaError::Runtime("c requires code string".into()))
                }
                "itumo" | "summon" | "bridge" => {
                    if let Some(IfaValue::Str(lang)) = args.first() {
                        // Check capability
                        self.check_capability(&Ofun::Bridge {
                            language: lang.clone(),
                        })?;

                        // In a real implementation, we'd initialize the bridge here
                        // For now, we'll log it and return success
                        println!("[ffi] Summoned {} bridge successfully", lang);
                        return Ok(IfaValue::Bool(true));
                    }
                    Err(IfaError::Runtime(
                        "itumo requires language name (e.g. 'js', 'python')".into(),
                    ))
                }
                "version" => Ok(IfaValue::Str("AjoseBridge v1.2".to_string())),
                _ => Err(IfaError::Runtime(format!(
                    "Unknown Coop method: {}",
                    call.method
                ))),
            },

            // ═══════════════════════════════════════════════════════════════
            // Ọpẹlẹ (Divination / Compound Odù)
            // ═══════════════════════════════════════════════════════════════
            OduDomain::Opele => match call.method.as_str() {
                "cast" | "ju" => {
                    // Cast a simple 2-level Odu
                    use std::time::{SystemTime, UNIX_EPOCH};
                    let seed = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map(|d| d.as_nanos() as u64)
                        .unwrap_or(0);
                    let random = seed
                        .wrapping_mul(6364136223846793005)
                        .wrapping_add(1442695040888963407);
                    let byte = (random >> 32) as u8;
                    let right = byte & 0x0F;
                    let left = (byte >> 4) & 0x0F;

                    let names = [
                        "Ogbe", "Oyeku", "Iwori", "Odi", "Irosu", "Owonrin", "Obara", "Okanran",
                        "Ogunda", "Osa", "Ika", "Oturupon", "Otura", "Irete", "Ose", "Ofun",
                    ];

                    if right == left {
                        Ok(IfaValue::Str(format!("{} Meji", names[right as usize])))
                    } else {
                        Ok(IfaValue::Str(format!(
                            "{}_{}",
                            names[right as usize], names[left as usize]
                        )))
                    }
                }
                "cast_compound" | "ju_apapo" => {
                    // Cast compound with specified depth: Opele.cast_compound(3)
                    let depth = match args.first() {
                        Some(IfaValue::Int(d)) if *d >= 1 => *d as usize,
                        _ => 2, // Default to 2-level
                    };

                    use std::time::{SystemTime, UNIX_EPOCH};
                    let mut seed = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map(|d| d.as_nanos() as u64)
                        .unwrap_or(0);

                    let names = [
                        "Ogbe", "Oyeku", "Iwori", "Odi", "Irosu", "Owonrin", "Obara", "Okanran",
                        "Ogunda", "Osa", "Ika", "Oturupon", "Otura", "Irete", "Ose", "Ofun",
                    ];

                    let mut ancestors = Vec::with_capacity(depth);
                    for _ in 0..depth {
                        let random = seed
                            .wrapping_mul(6364136223846793005)
                            .wrapping_add(1442695040888963407);
                        let odu_idx = ((random >> 32) as u8) % 16;
                        ancestors.push(names[odu_idx as usize]);
                        seed = random;
                    }

                    Ok(IfaValue::Str(ancestors.join("_")))
                }
                "lineage" | "iran" => {
                    // Get lineage description: Opele.lineage("Ogbe_Otura_Ika")
                    if let Some(IfaValue::Str(compound)) = args.first() {
                        let parts: Vec<&str> = compound.split('_').collect();
                        let depth = parts.len();

                        let roles = match depth {
                            1 => vec!["Self"],
                            2 => vec!["Parent", "Child"],
                            3 => vec!["Grandparent", "Parent", "Child"],
                            4 => vec!["Great-Grandparent", "Grandparent", "Parent", "Child"],
                            5 => vec![
                                "Great²-Grandparent",
                                "Great-Grandparent",
                                "Grandparent",
                                "Parent",
                                "Child",
                            ],
                            _ => (0..depth)
                                .map(|i| {
                                    if i == depth - 1 {
                                        "Current"
                                    } else {
                                        "Ancestor"
                                    }
                                })
                                .collect(),
                        };

                        let lineage: Vec<String> = parts
                            .iter()
                            .zip(roles.iter())
                            .map(|(odu, role)| format!("  {}: {}", role, odu))
                            .collect();

                        return Ok(IfaValue::Str(lineage.join("\n")));
                    }
                    Err(IfaError::Runtime(
                        "lineage requires compound name string".into(),
                    ))
                }
                "depth" | "ijinle" => {
                    // Get depth of compound: Opele.depth("Ogbe_Otura_Ika") -> 3
                    if let Some(IfaValue::Str(compound)) = args.first() {
                        let parts: Vec<&str> = compound.split('_').collect();
                        return Ok(IfaValue::Int(parts.len() as i64));
                    }
                    Err(IfaError::Runtime(
                        "depth requires compound name string".into(),
                    ))
                }
                "divine" | "dawole" => {
                    // Divination with question: Opele.divine("Should I proceed?")
                    use std::time::{SystemTime, UNIX_EPOCH};
                    let seed = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map(|d| d.as_nanos() as u64)
                        .unwrap_or(0);
                    let random = seed
                        .wrapping_mul(6364136223846793005)
                        .wrapping_add(1442695040888963407);
                    let byte = (random >> 32) as u8;
                    let right_idx = (byte & 0x0F) as usize;

                    let proverbs = [
                        "The path is clear, move forward with confidence.",
                        "In darkness, wisdom prepares for dawn.",
                        "Look within before seeking without.",
                        "Close one door, and another opens.",
                        "Speak truth, for lies have no legs to stand.",
                        "Change is the only constant; embrace it.",
                        "What you give, returns to you multiplied.",
                        "The tongue is sharper than the sword.",
                        "Clear the path with patience, not force.",
                        "Let go of what no longer serves you.",
                        "Words bind; choose them carefully.",
                        "Balance is the key to health.",
                        "The journey teaches more than the destination.",
                        "Secrets revealed bring freedom.",
                        "Beauty flows from inner peace.",
                        "The ancestors watch; honor their wisdom.",
                    ];

                    let question = args
                        .first()
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "...".to_string());

                    let result =
                        format!("Question: {}\nGuidance: {}", question, proverbs[right_idx]);

                    Ok(IfaValue::Str(result))
                }
                _ => Err(IfaError::Runtime(format!(
                    "Unknown Ọpẹlẹ method: {}",
                    call.method
                ))),
            },

            // ═══════════════════════════════════════════════════════════════
            // INFRASTRUCTURE LAYER
            // ═══════════════════════════════════════════════════════════════

            // CPU (Parallel Computing)
            OduDomain::Cpu => match call.method.as_str() {
                "num_threads" | "iye_threads" => Ok(IfaValue::Int(num_cpus::get() as i64)),
                "par_sum" | "apapọ" => {
                    if let Some(IfaValue::List(items)) = args.first() {
                        let sum: i64 = items
                            .iter()
                            .filter_map(|v| match v {
                                IfaValue::Int(n) => Some(*n),
                                IfaValue::Float(f) => Some(*f as i64),
                                _ => None,
                            })
                            .sum();
                        Ok(IfaValue::Int(sum))
                    } else {
                        Err(IfaError::Runtime("par_sum requires a list".into()))
                    }
                }
                "configure" | "ṣètò" => {
                    if let Some(IfaValue::Int(n)) = args.first() {
                        println!("[Cpu] Configured {} threads", n);
                        Ok(IfaValue::Bool(true))
                    } else {
                        Err(IfaError::Runtime("configure requires thread count".into()))
                    }
                }
                _ => Err(IfaError::Runtime(format!(
                    "Unknown Cpu method: {}",
                    call.method
                ))),
            },

            // GPU (Compute Shaders)
            OduDomain::Gpu => match call.method.as_str() {
                "available" | "wà" => {
                    // Check if GPU is available (placeholder)
                    Ok(IfaValue::Bool(false)) // Default: no GPU
                }
                "info" | "àlàyé" => Ok(IfaValue::Str(
                    "GPU: Not available (use native build)".into(),
                )),
                _ => Err(IfaError::Runtime(format!(
                    "Unknown Gpu method: {}",
                    call.method
                ))),
            },

            // Storage (Key-Value Store)
            OduDomain::Storage => match call.method.as_str() {
                "get" | "gba" => {
                    if let Some(IfaValue::Str(key)) = args.first() {
                        // Placeholder - would use OduStore
                        println!("[Storage] GET {}", key);
                        Ok(IfaValue::Null)
                    } else {
                        Err(IfaError::Runtime("get requires key".into()))
                    }
                }
                "set" | "fi" => {
                    if args.len() >= 2 {
                        if let (IfaValue::Str(key), value) = (&args[0], &args[1]) {
                            println!("[Storage] SET {} = {:?}", key, value);
                            Ok(IfaValue::Bool(true))
                        } else {
                            Err(IfaError::Runtime("set requires string key".into()))
                        }
                    } else {
                        Err(IfaError::Runtime("set requires key and value".into()))
                    }
                }
                "delete" | "pa" => {
                    if let Some(IfaValue::Str(key)) = args.first() {
                        println!("[Storage] DELETE {}", key);
                        Ok(IfaValue::Bool(true))
                    } else {
                        Err(IfaError::Runtime("delete requires key".into()))
                    }
                }
                _ => Err(IfaError::Runtime(format!(
                    "Unknown Storage method: {}",
                    call.method
                ))),
            },

            // ═══════════════════════════════════════════════════════════════
            // APPLICATION STACKS
            // ═══════════════════════════════════════════════════════════════

            // Backend (HTTP Server, ORM)
            OduDomain::Backend => match call.method.as_str() {
                "serve" | "sìn" => {
                    let port = args
                        .first()
                        .and_then(|v| {
                            if let IfaValue::Int(n) = v {
                                Some(*n)
                            } else {
                                None
                            }
                        })
                        .unwrap_or(8080);
                    println!("[Backend] Server would start on port {}", port);
                    Ok(IfaValue::Bool(true))
                }
                "route" | "ọ̀nà" => {
                    if args.len() >= 2 {
                        let method = args.first().map(|v| v.to_string()).unwrap_or_default();
                        let path = args.get(1).map(|v| v.to_string()).unwrap_or_default();
                        println!("[Backend] Route: {} {}", method, path);
                        Ok(IfaValue::Bool(true))
                    } else {
                        Err(IfaError::Runtime("route requires method and path".into()))
                    }
                }
                _ => Err(IfaError::Runtime(format!(
                    "Unknown Backend method: {}",
                    call.method
                ))),
            },

            // Frontend (HTML/CSS Generation)
            OduDomain::Frontend => match call.method.as_str() {
                "html" => {
                    if let Some(IfaValue::Str(content)) = args.first() {
                        // Escape HTML to prevent XSS
                        let escaped = content
                            .replace('&', "&amp;")
                            .replace('<', "&lt;")
                            .replace('>', "&gt;")
                            .replace('"', "&quot;");
                        Ok(IfaValue::Str(escaped))
                    } else {
                        Err(IfaError::Runtime("html requires content".into()))
                    }
                }
                "element" | "ẹya" => {
                    if args.len() >= 2 {
                        let tag = args.first().map(|v| v.to_string()).unwrap_or_default();
                        let content = args.get(1).map(|v| v.to_string()).unwrap_or_default();
                        Ok(IfaValue::Str(format!("<{}>{}</{}>", tag, content, tag)))
                    } else {
                        Err(IfaError::Runtime("element requires tag and content".into()))
                    }
                }
                _ => Err(IfaError::Runtime(format!(
                    "Unknown Frontend method: {}",
                    call.method
                ))),
            },

            // Crypto (extends Irete with more functions)
            OduDomain::Crypto => match call.method.as_str() {
                "sha256" => {
                    if let Some(IfaValue::Str(input)) = args.first() {
                        // Simple hash placeholder
                        use std::collections::hash_map::DefaultHasher;
                        use std::hash::{Hash, Hasher};
                        let mut hasher = DefaultHasher::new();
                        input.hash(&mut hasher);
                        let hash = format!("{:016x}", hasher.finish());
                        Ok(IfaValue::Str(hash))
                    } else {
                        Err(IfaError::Runtime("sha256 requires string input".into()))
                    }
                }
                "random_bytes" | "nọmba_aṣírí" => {
                    let count = args
                        .first()
                        .and_then(|v| {
                            if let IfaValue::Int(n) = v {
                                Some(*n as usize)
                            } else {
                                None
                            }
                        })
                        .unwrap_or(16);
                    // Simple random bytes (not cryptographically secure - placeholder)
                    use std::time::{SystemTime, UNIX_EPOCH};
                    let seed = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_nanos() as u64;
                    let bytes: Vec<IfaValue> = (0..count)
                        .map(|i| {
                            IfaValue::Int(
                                ((seed.wrapping_mul(1103515245 + i as u64)) >> 24) as i64 & 0xFF,
                            )
                        })
                        .collect();
                    Ok(IfaValue::List(bytes))
                }
                _ => Err(IfaError::Runtime(format!(
                    "Unknown Crypto method: {}",
                    call.method
                ))),
            },

            // ML (Machine Learning)
            OduDomain::Ml => match call.method.as_str() {
                "tensor" => {
                    // Create tensor from list
                    if let Some(IfaValue::List(data)) = args.first() {
                        Ok(IfaValue::List(data.clone()))
                    } else {
                        Err(IfaError::Runtime("tensor requires list data".into()))
                    }
                }
                "relu" => {
                    // ReLU activation
                    if let Some(IfaValue::List(data)) = args.first() {
                        let result: Vec<IfaValue> = data
                            .iter()
                            .map(|v| match v {
                                IfaValue::Float(f) => IfaValue::Float(f.max(0.0)),
                                IfaValue::Int(n) => IfaValue::Int((*n).max(0)),
                                other => other.clone(),
                            })
                            .collect();
                        Ok(IfaValue::List(result))
                    } else {
                        Err(IfaError::Runtime("relu requires tensor".into()))
                    }
                }
                "dot" => {
                    // Dot product
                    if args.len() >= 2 {
                        if let (IfaValue::List(a), IfaValue::List(b)) = (&args[0], &args[1]) {
                            let sum: f64 = a
                                .iter()
                                .zip(b.iter())
                                .filter_map(|(x, y)| match (x, y) {
                                    (IfaValue::Float(a), IfaValue::Float(b)) => Some(a * b),
                                    (IfaValue::Int(a), IfaValue::Int(b)) => {
                                        Some((*a as f64) * (*b as f64))
                                    }
                                    _ => None,
                                })
                                .sum();
                            Ok(IfaValue::Float(sum))
                        } else {
                            Err(IfaError::Runtime("dot requires two tensors".into()))
                        }
                    } else {
                        Err(IfaError::Runtime("dot requires two tensors".into()))
                    }
                }
                _ => Err(IfaError::Runtime(format!(
                    "Unknown Ml method: {}",
                    call.method
                ))),
            },

            // GameDev (Game Engine)
            OduDomain::GameDev => match call.method.as_str() {
                "vec2" => {
                    let x = args
                        .first()
                        .and_then(|v| match v {
                            IfaValue::Float(f) => Some(*f),
                            IfaValue::Int(n) => Some(*n as f64),
                            _ => None,
                        })
                        .unwrap_or(0.0);
                    let y = args
                        .get(1)
                        .and_then(|v| match v {
                            IfaValue::Float(f) => Some(*f),
                            IfaValue::Int(n) => Some(*n as f64),
                            _ => None,
                        })
                        .unwrap_or(0.0);
                    let mut map = HashMap::new();
                    map.insert("x".to_string(), IfaValue::Float(x));
                    map.insert("y".to_string(), IfaValue::Float(y));
                    Ok(IfaValue::Map(map))
                }
                "distance" | "jìnnà" => {
                    // Distance between two Vec2
                    if args.len() >= 2 {
                        if let (IfaValue::Map(a), IfaValue::Map(b)) = (&args[0], &args[1]) {
                            let ax = a
                                .get("x")
                                .and_then(|v| {
                                    if let IfaValue::Float(f) = v {
                                        Some(*f)
                                    } else {
                                        None
                                    }
                                })
                                .unwrap_or(0.0);
                            let ay = a
                                .get("y")
                                .and_then(|v| {
                                    if let IfaValue::Float(f) = v {
                                        Some(*f)
                                    } else {
                                        None
                                    }
                                })
                                .unwrap_or(0.0);
                            let bx = b
                                .get("x")
                                .and_then(|v| {
                                    if let IfaValue::Float(f) = v {
                                        Some(*f)
                                    } else {
                                        None
                                    }
                                })
                                .unwrap_or(0.0);
                            let by = b
                                .get("y")
                                .and_then(|v| {
                                    if let IfaValue::Float(f) = v {
                                        Some(*f)
                                    } else {
                                        None
                                    }
                                })
                                .unwrap_or(0.0);
                            let dist = ((bx - ax).powi(2) + (by - ay).powi(2)).sqrt();
                            Ok(IfaValue::Float(dist))
                        } else {
                            Err(IfaError::Runtime("distance requires two vec2".into()))
                        }
                    } else {
                        Err(IfaError::Runtime("distance requires two vec2".into()))
                    }
                }
                _ => Err(IfaError::Runtime(format!(
                    "Unknown GameDev method: {}",
                    call.method
                ))),
            },

            // IoT (Embedded/GPIO)
            OduDomain::Iot => match call.method.as_str() {
                "pin_mode" | "ipo_pin" => {
                    if args.len() >= 2 {
                        let pin = args.first().map(|v| v.to_string()).unwrap_or_default();
                        let mode = args.get(1).map(|v| v.to_string()).unwrap_or_default();
                        println!("[IoT] Pin {} set to {}", pin, mode);
                        Ok(IfaValue::Bool(true))
                    } else {
                        Err(IfaError::Runtime("pin_mode requires pin and mode".into()))
                    }
                }
                "digital_write" | "kọ_pin" => {
                    if args.len() >= 2 {
                        let pin = args.first().map(|v| v.to_string()).unwrap_or_default();
                        let value = args
                            .get(1)
                            .and_then(|v| match v {
                                IfaValue::Bool(b) => Some(*b),
                                IfaValue::Int(n) => Some(*n != 0),
                                _ => None,
                            })
                            .unwrap_or(false);
                        println!("[IoT] Digital write pin {} = {}", pin, value);
                        Ok(IfaValue::Bool(true))
                    } else {
                        Err(IfaError::Runtime(
                            "digital_write requires pin and value".into(),
                        ))
                    }
                }
                "digital_read" | "ka_pin" => {
                    if let Some(IfaValue::Int(pin)) = args.first() {
                        println!("[IoT] Digital read pin {}", pin);
                        Ok(IfaValue::Bool(false)) // Placeholder
                    } else {
                        Err(IfaError::Runtime("digital_read requires pin".into()))
                    }
                }
                _ => Err(IfaError::Runtime(format!(
                    "Unknown Iot method: {}",
                    call.method
                ))),
            },

            // Default for other domains
            _ => {
                println!(
                    "Warning: {:?}.{}() not yet implemented",
                    call.domain, call.method
                );
                Ok(IfaValue::Null)
            }
        }
    }

    fn apply_binary_op(
        &self,
        left: &IfaValue,
        op: &BinaryOperator,
        right: &IfaValue,
    ) -> IfaResult<IfaValue> {
        match op {
            BinaryOperator::Add => match (left, right) {
                (IfaValue::Int(a), IfaValue::Int(b)) => Ok(IfaValue::Int(a + b)),
                (IfaValue::Float(a), IfaValue::Float(b)) => Ok(IfaValue::Float(a + b)),
                (IfaValue::Int(a), IfaValue::Float(b)) => Ok(IfaValue::Float(*a as f64 + b)),
                (IfaValue::Float(a), IfaValue::Int(b)) => Ok(IfaValue::Float(a + *b as f64)),
                (IfaValue::Str(a), IfaValue::Str(b)) => Ok(IfaValue::Str(format!("{}{}", a, b))),
                // String coercion: convert other types to string for concatenation
                (IfaValue::Str(a), other) => Ok(IfaValue::Str(format!("{}{}", a, other))),
                (other, IfaValue::Str(b)) => Ok(IfaValue::Str(format!("{}{}", other, b))),
                _ => Err(IfaError::Runtime("Invalid operands for +".into())),
            },
            BinaryOperator::Sub => match (left, right) {
                (IfaValue::Int(a), IfaValue::Int(b)) => Ok(IfaValue::Int(a - b)),
                (IfaValue::Float(a), IfaValue::Float(b)) => Ok(IfaValue::Float(a - b)),
                _ => Err(IfaError::Runtime("Invalid operands for -".into())),
            },
            BinaryOperator::Mul => match (left, right) {
                (IfaValue::Int(a), IfaValue::Int(b)) => Ok(IfaValue::Int(a * b)),
                (IfaValue::Float(a), IfaValue::Float(b)) => Ok(IfaValue::Float(a * b)),
                _ => Err(IfaError::Runtime("Invalid operands for *".into())),
            },
            BinaryOperator::Div => match (left, right) {
                (IfaValue::Int(a), IfaValue::Int(b)) if *b != 0 => Ok(IfaValue::Int(a / b)),
                (IfaValue::Float(a), IfaValue::Float(b)) if *b != 0.0 => Ok(IfaValue::Float(a / b)),
                _ => Err(IfaError::Runtime("Division by zero".into())),
            },
            BinaryOperator::Mod => match (left, right) {
                (IfaValue::Int(a), IfaValue::Int(b)) if *b != 0 => Ok(IfaValue::Int(a % b)),
                _ => Err(IfaError::Runtime("Invalid operands for %".into())),
            },
            BinaryOperator::Eq => Ok(IfaValue::Bool(left == right)),
            BinaryOperator::NotEq => Ok(IfaValue::Bool(left != right)),
            BinaryOperator::Lt => match (left, right) {
                (IfaValue::Int(a), IfaValue::Int(b)) => Ok(IfaValue::Bool(a < b)),
                (IfaValue::Float(a), IfaValue::Float(b)) => Ok(IfaValue::Bool(a < b)),
                _ => Err(IfaError::Runtime("Invalid operands for <".into())),
            },
            BinaryOperator::LtEq => match (left, right) {
                (IfaValue::Int(a), IfaValue::Int(b)) => Ok(IfaValue::Bool(a <= b)),
                (IfaValue::Float(a), IfaValue::Float(b)) => Ok(IfaValue::Bool(a <= b)),
                _ => Err(IfaError::Runtime("Invalid operands for <=".into())),
            },
            BinaryOperator::Gt => match (left, right) {
                (IfaValue::Int(a), IfaValue::Int(b)) => Ok(IfaValue::Bool(a > b)),
                (IfaValue::Float(a), IfaValue::Float(b)) => Ok(IfaValue::Bool(a > b)),
                _ => Err(IfaError::Runtime("Invalid operands for >".into())),
            },
            BinaryOperator::GtEq => match (left, right) {
                (IfaValue::Int(a), IfaValue::Int(b)) => Ok(IfaValue::Bool(a >= b)),
                (IfaValue::Float(a), IfaValue::Float(b)) => Ok(IfaValue::Bool(a >= b)),
                _ => Err(IfaError::Runtime("Invalid operands for >=".into())),
            },
            BinaryOperator::And => Ok(IfaValue::Bool(left.is_truthy() && right.is_truthy())),
            BinaryOperator::Or => Ok(IfaValue::Bool(left.is_truthy() || right.is_truthy())),
        }
    }

    fn call_method(
        &mut self,
        _obj: &IfaValue,
        method: &str,
        _args: &[Expression],
    ) -> IfaResult<IfaValue> {
        Err(IfaError::Runtime(format!(
            "Method '{}' not implemented",
            method
        )))
    }

    fn call_function(&mut self, func: &IfaValue, args: &[Expression]) -> IfaResult<IfaValue> {
        if let IfaValue::AstFn { params, body, .. } = func {
            // Create new scope
            let parent_env = std::mem::take(&mut self.env);
            self.env = Environment::with_parent(parent_env);

            // Bind arguments to parameters
            for (param, arg) in params.iter().zip(args.iter()) {
                let val = self.evaluate(arg)?;
                self.env.define(param, val);
            }

            // Execute body
            let mut result = IfaValue::Null;
            for stmt in body {
                result = self.execute_statement(stmt)?;
            }

            // Restore parent scope
            if let Some(parent) = self.env.parent.take() {
                self.env = *parent;
            }

            Ok(result)
        } else {
            Err(IfaError::Runtime("Not a callable".into()))
        }
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse;

    #[test]
    fn test_var_decl_and_use() {
        let program = parse("ayanmo x = 42;").unwrap();
        let mut interp = Interpreter::new();
        interp.execute(&program).unwrap();

        assert_eq!(interp.env.get("x"), Some(IfaValue::Int(42)));
    }

    #[test]
    fn test_arithmetic_precedence() {
        // Test that * has higher precedence than +
        let program = parse("ayanmo x = 2 + 3 * 4;").unwrap();
        let mut interp = Interpreter::new();
        interp.execute(&program).unwrap();

        // Should be 2 + (3 * 4) = 14
        assert_eq!(interp.env.get("x"), Some(IfaValue::Int(14)));
    }

    #[test]
    fn test_print() {
        // Note: IrosuHandler prints directly to stdout
        // The handler registry doesn't populate the interpreter's output buffer
        let program = parse(r#"Irosu.fo("Hello");"#).unwrap();
        let mut interp = Interpreter::new();
        // Grant Stdio capability for print tests
        interp.capabilities.grant(ifa_sandbox::Ofun::Stdio);
        let result = interp.execute(&program);

        // Verify execution succeeds (print goes to stdout, visible in test output)
        assert!(result.is_ok());
    }

    #[test]
    fn test_string_concat() {
        let program = parse(r#"ayanmo s = "Hello" + " World";"#).unwrap();
        let mut interp = Interpreter::new();
        interp.execute(&program).unwrap();

        assert_eq!(
            interp.env.get("s"),
            Some(IfaValue::Str("Hello World".to_string()))
        );
    }

    #[test]
    fn test_if_statement() {
        let program = parse(
            r#"
            ayanmo x = 0;
            ti 5 > 3 {
                x = 1;
            }
        "#,
        )
        .unwrap();
        let mut interp = Interpreter::new();
        interp.execute(&program).unwrap();

        assert_eq!(interp.env.get("x"), Some(IfaValue::Int(1)));
    }

    #[test]
    fn test_while_loop() {
        let program = parse(
            r#"
            ayanmo x = 0;
            nigba x < 5 {
                x = x + 1;
            }
        "#,
        )
        .unwrap();
        let mut interp = Interpreter::new();
        interp.execute(&program).unwrap();

        assert_eq!(interp.env.get("x"), Some(IfaValue::Int(5)));
    }

    #[test]
    fn test_list_operations() {
        let program = parse(
            r#"
            ayanmo list = [1, 2, 3];
            ayanmo len = Ogunda.gigun(list);
        "#,
        )
        .unwrap();
        let mut interp = Interpreter::new();
        interp.execute(&program).unwrap();

        assert_eq!(interp.env.get("len"), Some(IfaValue::Int(3)));
    }

    #[test]
    fn test_string_upper() {
        let program = parse(r#"ayanmo s = Ika.upper("hello");"#).unwrap();
        let mut interp = Interpreter::new();
        interp.execute(&program).unwrap();

        assert_eq!(
            interp.env.get("s"),
            Some(IfaValue::Str("HELLO".to_string()))
        );
    }

    #[test]
    fn test_math_add() {
        let program = parse(r#"ayanmo x = Obara.add(1, 2, 3, 4);"#).unwrap();
        let mut interp = Interpreter::new();
        interp.execute(&program).unwrap();

        assert_eq!(interp.env.get("x"), Some(IfaValue::Int(10)));
    }

    #[test]
    fn test_comparison_ops() {
        let program = parse(
            r#"
            ayanmo a = 5 == 5;
            ayanmo b = 5 != 3;
            ayanmo c = 5 > 3;
            ayanmo d = 3 < 5;
        "#,
        )
        .unwrap();
        let mut interp = Interpreter::new();
        interp.execute(&program).unwrap();

        assert_eq!(interp.env.get("a"), Some(IfaValue::Bool(true)));
        assert_eq!(interp.env.get("b"), Some(IfaValue::Bool(true)));
        assert_eq!(interp.env.get("c"), Some(IfaValue::Bool(true)));
        assert_eq!(interp.env.get("d"), Some(IfaValue::Bool(true)));
    }
}

// =============================================================================
// CRYPTO HELPERS (Pure Rust, no external dependencies)
// =============================================================================

/// SHA-256 implementation (FIPS 180-4 compliant)
fn sha256_simple(data: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];

    // Pad message
    let bit_len = (data.len() as u64) * 8;
    let mut padded = data.to_vec();
    padded.push(0x80);
    while (padded.len() % 64) != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    // Process 512-bit chunks
    for chunk in padded.chunks(64) {
        let mut w = [0u32; 64];
        for (i, word) in chunk.chunks(4).enumerate() {
            w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);

            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    let mut result = [0u8; 32];
    for (i, val) in h.iter().enumerate() {
        result[i * 4..i * 4 + 4].copy_from_slice(&val.to_be_bytes());
    }
    result
}

/// Base64 encoding (RFC 4648)
fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity(data.len().div_ceil(3) * 4);

    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        result.push(ALPHABET[b0 >> 2] as char);

        if chunk.len() > 1 {
            let b1 = chunk[1] as usize;
            result.push(ALPHABET[((b0 & 0x03) << 4) | (b1 >> 4)] as char);

            if chunk.len() > 2 {
                let b2 = chunk[2] as usize;
                result.push(ALPHABET[((b1 & 0x0f) << 2) | (b2 >> 6)] as char);
                result.push(ALPHABET[b2 & 0x3f] as char);
            } else {
                result.push(ALPHABET[(b1 & 0x0f) << 2] as char);
                result.push('=');
            }
        } else {
            result.push(ALPHABET[(b0 & 0x03) << 4] as char);
            result.push_str("==");
        }
    }
    result
}

/// Base64 decoding (RFC 4648)
#[allow(dead_code)]
fn base64_decode(encoded: &str) -> Result<Vec<u8>, String> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let chars: Vec<char> = encoded
        .chars()
        .filter(|&c| c != '=' && !c.is_whitespace())
        .collect();
    let mut result = Vec::with_capacity(chars.len() * 3 / 4);

    for chunk in chars.chunks(4) {
        if chunk.is_empty() {
            break;
        }

        let indices: Result<Vec<usize>, _> = chunk
            .iter()
            .map(|&c| {
                ALPHABET
                    .iter()
                    .position(|&b| b as char == c)
                    .ok_or_else(|| format!("Invalid base64 char: {}", c))
            })
            .collect();

        let indices = indices?;

        if indices.len() >= 2 {
            result.push(((indices[0] << 2) | (indices[1] >> 4)) as u8);
        }
        if indices.len() >= 3 {
            result.push((((indices[1] & 0x0f) << 4) | (indices[2] >> 2)) as u8);
        }
        if indices.len() >= 4 {
            result.push((((indices[2] & 0x03) << 6) | indices[3]) as u8);
        }
    }
    Ok(result)
}
