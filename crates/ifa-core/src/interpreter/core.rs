//! # Ifá-Lang Interpreter
//!
//! Tree-walking interpreter that executes AST directly.
//! This is the bridge between parsing and execution.




use std::collections::HashMap;
use crate::ast::*;
use crate::error::{IfaError, IfaResult};

use crate::opon::Opon;
// use crate::value::IfaValue; // Legacy
use ifa_types::value_union::IfaValue;
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

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AstFnData {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AstClassData {
    pub name: String,
    pub fields: Vec<String>,
    pub methods: HashMap<String, IfaValue>,
}

use super::environment::Environment;

/// Ose Canvas for ASCII graphics
#[derive(Clone)]
#[allow(dead_code)]
struct OseCanvas {
    width: usize,
    height: usize,
    buffer: Vec<Vec<char>>,
    cursor_x: usize,
    cursor_y: usize,
}

#[allow(dead_code)]
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
    /// Memory (The Calabash)
    pub opon: Opon,
    /// Unsafe block nesting depth (0 = safe mode)
    unsafe_depth: usize,
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
            opon: Opon::default(),
            unsafe_depth: 0,
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

    /// Register a new domain handler
    pub fn register_handler(&mut self, handler: Box<dyn super::handlers::OduHandler>) {
        self.handlers.register(handler);
    }

    /// Check capability and return error if denied
    #[allow(dead_code)]
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
        let mut result = IfaValue::null();

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

    /// Check if currently in an unsafe block
    pub fn is_unsafe(&self) -> bool {
        self.unsafe_depth > 0
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

    /// Execute a block of statements in a new scope
    fn execute_block(&mut self, statements: &[Statement]) -> IfaResult<IfaValue> {
        // Push Scope - Manually (GPC Pattern)
        // We take the current env and wrap it as the parent of a new empty env
        let old_env = std::mem::take(&mut self.env);
        self.env = crate::interpreter::environment::Environment::with_parent(old_env);

        let mut result = Ok(IfaValue::null());
        for stmt in statements {
            result = self.execute_statement(stmt);
            
            if result.is_err() {
                break;
            }
            
            //Check for return values to propagate break/return
            if let Ok(val) = &result {
                if val.is_return() {
                    break;
                }
            }
        }

        // Pop Scope
        // We take the parent back and restore it as current env
        if let Some(parent) = self.env.parent.take() {
            self.env = *parent;
        }

        result
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
                Ok(IfaValue::null())
            }

            Statement::Const { name, value, .. } => {
                // Runtime interpretation: identical to VarDecl but conceptually constant
                let val = self.evaluate(value)?;
                self.env.define(name, val);
                Ok(IfaValue::null())
            }

            Statement::Try {
                try_body,
                catch_var,
                catch_body,
                ..
            } => {
                // Execute try block with new scope
                match self.execute_block(try_body) {
                    Ok(val) => Ok(val),
                    Err(e) => {
                        // Execute catch block with new scope
                        // We must manually enter scope for catch to bind the error variable
                        let old_env = std::mem::take(&mut self.env);
                        self.env = Environment::with_parent(old_env);
                        
                        if !catch_var.is_empty() {
                            self.env.define(catch_var, IfaValue::str(e.to_string()));
                        }
                        
                        // Execute catch body statements manually inside this scope
                        // (We reusing execute_block logic but inline to avoid double-scoping 
                        // or we could use execute_block if we didn't already push scope... 
                        // Actually execute_block pushes scope. So we can't use it if we want to bind var *in* that scope first.
                        // So we do it manually here for catch.)
                        
                        let mut result = Ok(IfaValue::null());
                        for s in catch_body {
                            result = self.execute_statement(s);
                            if result.is_err() { break; }
                        }
                        
                        // Pop scope
                        if let Some(parent) = self.env.parent.take() {
                            self.env = *parent;
                        }
                        
                        result
                    }
                }
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
                        let mut container = self.env.get(name).ok_or_else(|| {
                            IfaError::Runtime(format!("Undefined variable: {}", name))
                        })?;

                        // Index Assignment: xs[0] = 10
                         match container {
                             IfaValue::List(ref mut vec_arc) => {
                                 let i = match idx {
                                     IfaValue::Int(n) => n as usize,
                                     _ => return Err(IfaError::Runtime("List index must be Int".into()))
                                 };
                                 
                                 // HIGH PERFORMANCE: CoW using make_mut
                                 // O(1) if unique, O(N) if shared.
                                 let vec = std::sync::Arc::make_mut(vec_arc);
                                 if i >= vec.len() { return Err(IfaError::Runtime("Index out of bounds".into())); }
                                 vec[i] = val;
                             }
                             
                             IfaValue::Map(ref mut map_arc) => {
                                 let k = match idx {
                                     IfaValue::Str(s) => s.clone(),
                                     _ => return Err(IfaError::Runtime("Map key must be Str".into()))
                                 };
                                 // HIGH PERFORMANCE: CoW using make_mut
                                 let map = std::sync::Arc::make_mut(map_arc);
                                 map.insert(k, val);
                             }
                             _ => return Err(IfaError::Runtime("Invalid index assignment target".into()))
                        }
                        self.env.set(name, container);
                    }
                    AssignTarget::Dereference(expr) => {
                        // *ptr = val
                        let ptr = self.evaluate(expr)?;
                        match ptr {
        
                            IfaValue::Int(addr) => {
                                if !self.is_unsafe() {
                                    return Err(IfaError::Runtime(format!(
                                        "Safety violation: Writing to raw pointer *0x{:X} requires 'àìléwu' (unsafe) block",
                                        addr
                                    )));
                                }
                                self.opon
                                    .try_set(addr as usize, val)
                                    .map_err(|e| IfaError::Runtime(e.to_string()))?;
                            }
        
                            IfaValue::Str(name) => {
                                if !self.env.set(&name, val.clone()) {
                                    return Err(IfaError::Runtime(format!(
                                        "Reference to undefined variable: {}",
                                        name
                                    )));
                                }
                            }
                            _ => {
                                return Err(IfaError::Runtime(format!(
                                    "Cannot dereference type: {}",
                                    ptr.type_name()
                                )));
                            }
                        }
                    }
                }
                Ok(IfaValue::null())
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
                Ok(IfaValue::null())
            }

            Statement::While {
                condition, body, ..
            } => {
                while self.evaluate(condition)?.is_truthy() {
                    for s in body {
                        self.execute_statement(s)?;
                    }
                }
                Ok(IfaValue::null())
            }

            Statement::For {
                var,
                iterable,
                body,
                ..
            } => {
                let iter_val = self.evaluate(iterable)?;
                // Pattern match using kind
                if let IfaValue::List(items) = iter_val {
                     // Note: items is &[IfaValue]. We need to iterate.
                     // But we can't iterate 'items' directly if we need to execute statements 
                     // because statements might mutate state or invalidate borrows?
                     // Safer to clone the items first.
                     let items_vec = items.to_vec(); // Clone items (IfaValue clone)
                     for item in items_vec {
                        self.env.define(var, item);
                        for s in body {
                            self.execute_statement(s)?;
                        }
                     }
                }
                Ok(IfaValue::null())
            }

            Statement::Return { value, .. } => {
                let val = if let Some(expr) = value {
                    self.evaluate(expr)?
                } else {
                    IfaValue::null()
                };
                Ok(IfaValue::return_value(val))
            }

            Statement::Match {
                condition, arms, ..
            } => {
                let cond_val = self.evaluate(condition)?;
                for arm in arms {
                    // MatchPattern is NOT an Expression, so we handle it directly
                    let matches = match &arm.pattern {
                        MatchPattern::Literal(expr) => {
                            let pat_val = self.evaluate(expr)?;
                            cond_val == pat_val
                        }
                        MatchPattern::Range { start, end } => {
                             let start_val = self.evaluate(start)?;
                             let end_val = self.evaluate(end)?;
                             match (&cond_val, start_val, end_val) {
                                 (IfaValue::Int(v), IfaValue::Int(s), IfaValue::Int(e)) => *v >= s && *v <= e,
                                 (IfaValue::Float(v), IfaValue::Float(s), IfaValue::Float(e)) => *v >= s && *v <= e,
                                 _ => false
                             }
                        }
                        MatchPattern::Wildcard => true,
                    };

                    if matches {
                        for stmt in &arm.body {
                            let res = self.execute_statement(stmt)?;
                            // Check for Return signal
                            if matches!(res, IfaValue::Return(_)) {
                                return Ok(res);
                            }
                        }
                        return Ok(IfaValue::null());
                    }
                }
                Ok(IfaValue::null())
            }

            Statement::Ase { .. } => {
                Ok(IfaValue::null())
            }



// Inside execute_statement match
            Statement::EseDef {
                name, params, body, ..
            } => {
                // AST-mode function dispatch is disabled in the interpreter
                // (call_function returns Err for all non-bytecode callables).
                // Store Null as the function value so the name is defined in scope.
                // When the compiler path is active, EseDef is compiled to bytecode
                // and called via OpCode::Call — not through the interpreter env.
                let _data = AstFnData {
                    name: name.clone(),
                    params: params.iter().map(|p| p.name.clone()).collect(),
                    body: body.clone(),
                };
                // `_data` is dropped here — no raw pointer, no leak.
                self.env.define(name, IfaValue::null());
                Ok(IfaValue::null())
            }

            Statement::Import { path, .. } => {
                self.import_module(path)?;
                Ok(IfaValue::null())
            }

            Statement::OduDef { name, body, .. } => {
                let mut fields = Vec::new();

                for stmt in body {
                    match stmt {
                        Statement::EseDef {
                            name, params, body, ..
                        } => {
                            // AST-mode function dispatch is disabled; drop cleanly.
                            let _data = AstFnData {
                                name: name.clone(),
                                params: params.iter().map(|p| p.name.clone()).collect(),
                                body: body.clone(),
                            };
                            // `_data` dropped here — no pointer, no leak.
                        }
                        Statement::VarDecl { name, .. } => {
                            fields.push(name.clone());
                        }
                        _ => {}
                    }
                }

                // Class dispatch is handled by the VM's DefineClass opcode.
                // The interpreter registers the class name as Null in the current scope
                // so subsequent references don't produce "undefined variable" errors.
                let _ = fields;
                self.env.define(name, IfaValue::null());
                Ok(IfaValue::null())
            }


            Statement::Expr { expr, .. } => self.evaluate(expr),

            Statement::Taboo { source, target, .. } => {
                println!("[taboo] {} -> {} forbidden", source, target);
                Ok(IfaValue::null())
            }

            Statement::Ewo {
                condition,
                message,
                span: _,
            } => {
                let condition_val = self.evaluate(condition)?;
                match condition_val {
                    IfaValue::Bool(true) => Ok(IfaValue::null()),
                    IfaValue::Bool(false) => {
                        let msg = message
                            .clone()
                            .unwrap_or_else(|| "Assertion failed".to_string());
                        Err(IfaError::Runtime(format!(
                            "[ẹ̀wọ̀/verify] Taboo violated: {}",
                            msg
                        )))
                    }
                    _ => {
                        Err(IfaError::Runtime(format!(
                            "[ẹ̀wọ̀/verify] Assertion expects boolean, got: {:?}",
                            condition_val.type_name()
                        )))
                    }
                }
            }

            Statement::Opon { size, .. } => {
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
                Ok(IfaValue::null())
            }

            Statement::Ebo { offering, .. } => {
                let val = self.evaluate(offering)?;
                println!("[ẹbọ/sacrifice] Aspect initiated: {}", val);
                Ok(IfaValue::null())
            }

            Statement::Ailewu { body, .. } => {
                self.unsafe_depth += 1;
                let result = (|| {
                    for s in body {
                        let res = self.execute_statement(s)?;
                        if matches!(res, IfaValue::Return(_)) {
                            return Ok(res);
                        }
                    }
                    Ok(IfaValue::null())
                })();
                self.unsafe_depth -= 1;
                result
            }

            Statement::Yield { duration, .. } => {
                let val = self.evaluate(duration)?;
                if let IfaValue::Int(micros) = val {
                    std::thread::sleep(std::time::Duration::from_micros(micros as u64));
                } else {
                    return Err(IfaError::Runtime(
                        "Yield duration must be an integer (microseconds)".to_string(),
                    ));
                }
                Ok(IfaValue::null())
            }
        }
    }

    fn evaluate(&mut self, expr: &Expression) -> IfaResult<IfaValue> {
        match expr {
            Expression::Int(n) => Ok(IfaValue::Int(*n)),
            Expression::Float(f) => Ok(IfaValue::Float(*f)),
            Expression::String(s) => Ok(IfaValue::Str(s.clone().into())),
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
                // Special handling for AddressOf to avoid evaluating the expression fully if it's an Identifier
                if matches!(op, UnaryOperator::AddressOf) {
                    // &x -> Ref("x") - Unsupported in current type system
                    if let Expression::Identifier(_name) = &**expr {
                         return Err(IfaError::Runtime("AddressOf (&) not supported in new type system".into()));
                    }
                }
                
                let r = self.evaluate(expr)?;
                match op {
                    UnaryOperator::Not => Ok(IfaValue::bool(!r.is_truthy())),
                    UnaryOperator::Neg => match r {
                        IfaValue::Int(n) => Ok(IfaValue::int(-n)),
                        IfaValue::Float(f) => Ok(IfaValue::float(-f)),
                        _ => Err(IfaError::Runtime("Operand must be a number".into())),
                    },
                    UnaryOperator::AddressOf => {
                        // If we are here, it means it wasn't a simple identifier. 
                        // We can't really take address of a literal or expression result in this simple VM yet.
                        Err(IfaError::Runtime("Cannot take address of this expression".into()))
                    }
                    UnaryOperator::Dereference => {
                         // *r
                         // Deref not supported currently as AddressOf is disabled
                         Err(IfaError::Runtime("Deref (*) not supported in new type system".into()))
                    }
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
                Ok(IfaValue::list(list))
            }

            Expression::Map(entries) => {
                let mut map = HashMap::new();
                for (k, v) in entries {
                    let key = match self.evaluate(k)? {
                        IfaValue::Str(s) => s.to_string(),
                         _ => return Err(IfaError::Runtime("Map keys must be strings".into())),
                    };
                    map.insert(key.into(), self.evaluate(v)?);
                }
                Ok(IfaValue::map(map))
            }

            Expression::Index { object, index } => {
                let obj = self.evaluate(object)?;
                let idx = self.evaluate(index)?;

                match (obj, idx) {
                    (IfaValue::List(list), IfaValue::Int(i)) => {
                        let i = i as usize;
                        if i < list.len() {
                             Ok(list[i].clone())
                        } else {
                            Err(IfaError::Runtime(format!("Index {} out of bounds", i)))
                        }
                    }
                    (IfaValue::Map(map), IfaValue::Str(key)) => map
                        .get(&key)
                        .cloned()
                        .ok_or_else(|| IfaError::Runtime(format!("Key '{}' not found", key))),
                    _ => Err(IfaError::Runtime("Invalid index operation".into())),
                }
            }
        }
    }


    fn execute_odu_call(&mut self, call: &OduCall) -> IfaResult<IfaValue> {
        let args: Vec<IfaValue> = call
            .args
            .iter()
            .map(|arg| self.evaluate(arg))
            .collect::<Result<_, _>>()?;

        self.handlers.dispatch(
            call.domain.clone(),
            &call.method,
            args,
            &mut self.env,
            &mut self.output,
        )
    }


    fn apply_binary_op(
        &self,
        left: &IfaValue,
        op: &BinaryOperator,
        right: &IfaValue,
    ) -> IfaResult<IfaValue> {
        match op {
            BinaryOperator::Add => match (left, right) {
                (IfaValue::Int(a), IfaValue::Int(b)) => Ok(IfaValue::int(a + b)),
                (IfaValue::Float(a), IfaValue::Float(b)) => Ok(IfaValue::float(a + b)),
                (IfaValue::Int(a), IfaValue::Float(b)) => Ok(IfaValue::float(*a as f64 + b)),
                (IfaValue::Float(a), IfaValue::Int(b)) => Ok(IfaValue::float(a + *b as f64)),
                (IfaValue::Str(a), IfaValue::Str(b)) => {
                    Ok(IfaValue::str(format!("{}{}", a, b)))
                }
                (IfaValue::Str(a), _) => Ok(IfaValue::str(format!("{}{}", a, right))),
                (_, IfaValue::Str(b)) => Ok(IfaValue::str(format!("{}{}", left, b))),
                _ => Err(IfaError::Runtime("Invalid operands for +".into())),
            },
            BinaryOperator::Sub => match (left, right) {
                (IfaValue::Int(a), IfaValue::Int(b)) => Ok(IfaValue::int(a - b)),
                (IfaValue::Float(a), IfaValue::Float(b)) => Ok(IfaValue::float(a - b)),
                (IfaValue::Int(a), IfaValue::Float(b)) => Ok(IfaValue::float(*a as f64 - b)),
                (IfaValue::Float(a), IfaValue::Int(b)) => Ok(IfaValue::float(a - *b as f64)),
                 _ => Err(IfaError::Runtime("Invalid operands for -".into())),
            },
            BinaryOperator::Mul => match (left, right) {
                (IfaValue::Int(a), IfaValue::Int(b)) => Ok(IfaValue::int(a * b)),
                (IfaValue::Float(a), IfaValue::Float(b)) => Ok(IfaValue::float(a * b)),
                (IfaValue::Int(a), IfaValue::Float(b)) => Ok(IfaValue::float(*a as f64 * b)),
                (IfaValue::Float(a), IfaValue::Int(b)) => Ok(IfaValue::float(a * *b as f64)),
                 _ => Err(IfaError::Runtime("Invalid operands for *".into())),
            },
            BinaryOperator::Div => match (left, right) {
                (IfaValue::Int(a), IfaValue::Int(b)) if *b != 0 => Ok(IfaValue::int(a / b)),
                (IfaValue::Float(a), IfaValue::Float(b)) if *b != 0.0 => Ok(IfaValue::float(a / b)),
                 // Mixed
                (IfaValue::Int(a), IfaValue::Float(b)) if *b != 0.0 => Ok(IfaValue::float(*a as f64 / b)),
                (IfaValue::Float(a), IfaValue::Int(b)) if *b != 0 => Ok(IfaValue::float(a / *b as f64)),
                
                _ => Err(IfaError::Runtime("Division by zero or invalid operands".into())),
            },
            BinaryOperator::Mod => match (left, right) {
                (IfaValue::Int(a), IfaValue::Int(b)) if *b != 0 => Ok(IfaValue::int(a % b)),
                _ => Err(IfaError::Runtime("Invalid operands for %".into())),
            },
            BinaryOperator::Eq => {
                let eq = match (left, right) {
                    (IfaValue::Int(a), IfaValue::Int(b)) => a == b,
                    (IfaValue::Float(a), IfaValue::Float(b)) => (a - b).abs() < f64::EPSILON,
                    (IfaValue::Str(a), IfaValue::Str(b)) => a == b,
                    (IfaValue::Bool(a), IfaValue::Bool(b)) => a == b,
                    (IfaValue::Null, IfaValue::Null) => true,
                    _ => false // Default to false for mismatched types
                };
                Ok(IfaValue::bool(eq))
            },
            BinaryOperator::NotEq => {
                 let eq = match (left, right) {
                    (IfaValue::Int(a), IfaValue::Int(b)) => a == b,
                    (IfaValue::Float(a), IfaValue::Float(b)) => (a - b).abs() < f64::EPSILON,
                    (IfaValue::Str(a), IfaValue::Str(b)) => a == b,
                    (IfaValue::Bool(a), IfaValue::Bool(b)) => a == b,
                    (IfaValue::Null, IfaValue::Null) => true,
                    _ => false
                };
                Ok(IfaValue::bool(!eq))
            },
            BinaryOperator::Lt => match (left, right) {
                (IfaValue::Int(a), IfaValue::Int(b)) => Ok(IfaValue::bool(a < b)),
                (IfaValue::Float(a), IfaValue::Float(b)) => Ok(IfaValue::bool(a < b)),
                (IfaValue::Int(a), IfaValue::Float(b)) => Ok(IfaValue::bool((*a as f64) < *b)),
                (IfaValue::Float(a), IfaValue::Int(b)) => Ok(IfaValue::bool(*a < (*b as f64))),
                 _ => Ok(IfaValue::bool(false)),
            },
            BinaryOperator::LtEq => match (left, right) {
                (IfaValue::Int(a), IfaValue::Int(b)) => Ok(IfaValue::bool(a <= b)),
                (IfaValue::Float(a), IfaValue::Float(b)) => Ok(IfaValue::bool(a <= b)),
                 (IfaValue::Int(a), IfaValue::Float(b)) => Ok(IfaValue::bool((*a as f64) <= *b)),
                 (IfaValue::Float(a), IfaValue::Int(b)) => Ok(IfaValue::bool(*a <= (*b as f64))),
                 _ => Ok(IfaValue::bool(false)),
            },
            BinaryOperator::Gt => match (left, right) {
                (IfaValue::Int(a), IfaValue::Int(b)) => Ok(IfaValue::bool(a > b)),
                (IfaValue::Float(a), IfaValue::Float(b)) => Ok(IfaValue::bool(a > b)),
                 (IfaValue::Int(a), IfaValue::Float(b)) => Ok(IfaValue::bool((*a as f64) > *b)),
                 (IfaValue::Float(a), IfaValue::Int(b)) => Ok(IfaValue::bool(*a > (*b as f64))),
                 _ => Ok(IfaValue::bool(false)),
            },
            BinaryOperator::GtEq => match (left, right) {
                (IfaValue::Int(a), IfaValue::Int(b)) => Ok(IfaValue::bool(a >= b)),
                (IfaValue::Float(a), IfaValue::Float(b)) => Ok(IfaValue::bool(a >= b)),
                 (IfaValue::Int(a), IfaValue::Float(b)) => Ok(IfaValue::bool((*a as f64) >= *b)),
                 (IfaValue::Float(a), IfaValue::Int(b)) => Ok(IfaValue::bool(*a >= (*b as f64))),
                 _ => Ok(IfaValue::bool(false)),
            },
            BinaryOperator::And => Ok(IfaValue::bool(left.is_truthy() && right.is_truthy())),
            BinaryOperator::Or => Ok(IfaValue::bool(left.is_truthy() || right.is_truthy())),
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

    fn call_function(&mut self, func: &IfaValue, _args: &[Expression]) -> IfaResult<IfaValue> {
        match func {

             IfaValue::Fn(_) => {
                 Err(IfaError::Runtime("Bytecode function cannot be called in Interpreter (AST mode)".into()))
             }
             // AstFn removed/disabled

             IfaValue::Int(_) => { return Err(IfaError::Runtime("AstFn disabled/removed".into())); }
                 // Unsafe cast to access AstFnData
                 // We know we created it as Box<AstFnData>


             _ => Err(IfaError::Runtime("Not a callable function".into()))
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
        let program = parse("ayanmo x = 42;").expect("Parse failed");
        let mut interp = Interpreter::new();
        interp.execute(&program).unwrap();

        assert_eq!(interp.env.get("x"), Some(IfaValue::int(42)));
    }

    #[test]
    fn test_arithmetic_precedence() {
        // Test that * has higher precedence than +
        let program = parse("ayanmo x = 2 + 3 * 4;").unwrap();
        let mut interp = Interpreter::new();
        interp.execute(&program).unwrap();

        // Should be 2 + (3 * 4) = 14
        assert_eq!(interp.env.get("x"), Some(IfaValue::int(14)));
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
            Some(IfaValue::str("Hello World"))
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

        assert_eq!(interp.env.get("x"), Some(IfaValue::int(1)));
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

        assert_eq!(interp.env.get("x"), Some(IfaValue::int(5)));
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

        assert_eq!(interp.env.get("len"), Some(IfaValue::int(3)));
    }

    #[test]
    fn test_string_upper() {
        let program = parse(r#"ayanmo s = Ika.upper("hello");"#).unwrap();
        let mut interp = Interpreter::new();
        interp.execute(&program).unwrap();

        assert_eq!(
            interp.env.get("s"),
            Some(IfaValue::str("HELLO"))
        );
    }

    #[test]
    fn test_math_add() {
        let program = parse(r#"ayanmo x = Obara.add(1, 2, 3, 4);"#).unwrap();
        let mut interp = Interpreter::new();
        interp.execute(&program).unwrap();

        assert_eq!(interp.env.get("x"), Some(IfaValue::int(10)));
    }

    #[test]
    fn test_comparison_ops() {
        let program = parse(
            r#"
            ayanmo a = 5 == 5;
            ayanmo b = 5 != 3;
            ayanmo c = 5 > 3;
            ayanmo d = 3 < 5;
            ayanmo d2 = 3 <= 5;
            ayanmo d3 = 5 >= 3;
        "#,
        )
        .unwrap();
        let mut interp = Interpreter::new();
        interp.execute(&program).unwrap();

        assert_eq!(interp.env.get("a"), Some(IfaValue::bool(true)));
        assert_eq!(interp.env.get("b"), Some(IfaValue::bool(true)));
        assert_eq!(interp.env.get("c"), Some(IfaValue::bool(true)));
        assert_eq!(interp.env.get("d"), Some(IfaValue::bool(true)));
    }
}

// =============================================================================
// CRYPTO HELPERS (Pure Rust, no external dependencies)
// =============================================================================

/// SHA-256 implementation (FIPS 180-4 compliant)
#[allow(dead_code)]
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
#[allow(dead_code)]
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
