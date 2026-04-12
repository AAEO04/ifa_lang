//! # Ifá-Lang Interpreter
//!
//! Tree-walking interpreter that executes AST directly.
//! This is the bridge between parsing and execution.

use crate::ast::*;
use crate::error::{IfaError, IfaResult};
use ifa_types::domain::OduDomain;
use std::collections::{HashMap, VecDeque};

use crate::opon::Opon;
// use crate::value::IfaValue; // Legacy
use ifa_types::value_union::IfaValue;
use std::fmt::Debug;

/// Debugger trait for execution tracing
pub trait Debugger: Debug {
    fn on_statement(&mut self, stmt: &Statement, env: &EnvRef);
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

use super::environment::{EnvRef, Environment};
use std::sync::Arc;


/// The Ifá Interpreter

pub struct Interpreter {
    pub env: EnvRef,
    output: Vec<String>,
    /// Captured closure environments by id.
    closures: HashMap<u64, EnvRef>,
    next_closure_id: u64,
    /// Already imported modules (prevents re-execution)
    imported: std::collections::HashSet<String>,
    /// Circular import guard
    import_guard: crate::module_resolver::ImportGuard,
    /// Cached module exports (key -> exports map)
    module_cache: HashMap<String, IfaValue>,
    /// Canonical resolver initialized once, used for every import
    resolver: crate::module_resolver::ModuleResolver,
    /// Current file being executed (for relative imports)
    current_file: Option<std::path::PathBuf>,
    /// Security capabilities
    pub capabilities: CapabilitySet,


    /// Modular domain handlers
    handlers: HandlerRegistry,
    /// Memory (The Calabash)
    pub opon: Opon,
    /// Unsafe block nesting depth (0 = safe mode)
    unsafe_depth: usize,
    /// Optional debugger hook
    pub debugger: Option<Box<dyn Debugger>>,
    /// Current function call depth
    call_depth: usize,
    /// Max allowed call frames (from #opon)
    call_depth_limit: Option<usize>,
    /// Async task queue
    task_queue: VecDeque<AstTask>,
}

#[derive(Clone)]
struct AstTask {
    func: IfaValue,
    args: Vec<IfaValue>,
    future: ifa_types::value_union::FutureCell,
}

impl Interpreter {
    pub fn new() -> Self {
        let mut module_paths = Vec::new();
        if let Ok(cwd) = std::env::current_dir() {
            module_paths.push(cwd);
        }
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                module_paths.push(dir.join("lib"));
            }
        }
        let resolver = crate::module_resolver::ModuleResolver::new(module_paths);

        Interpreter {
            env: Environment::new(),
            output: Vec::new(),
            closures: HashMap::new(),
            next_closure_id: 1,
            imported: std::collections::HashSet::new(),
            import_guard: crate::module_resolver::ImportGuard::new(),
            module_cache: HashMap::new(),
            resolver,
            current_file: None,
            capabilities: CapabilitySet::default(),

            handlers: HandlerRegistry::new(),
            opon: Opon::default(),
            unsafe_depth: 0,
            debugger: None,
            call_depth: 0,
            call_depth_limit: None,
            task_queue: VecDeque::new(),
        }
    }

    /// Create interpreter with custom file path
    pub fn with_file(file: impl AsRef<std::path::Path>) -> Self {
        let mut interp = Self::new();
        let path = file.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            interp.resolver.search_paths.insert(0, parent.to_path_buf());
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

        match result {
            IfaValue::Return(v) => Ok((*v).clone()),
            other => Ok(other),
        }
    }

    fn spawn_task(&mut self, func: IfaValue, args: Vec<IfaValue>) -> IfaResult<IfaValue> {
        let cell = match IfaValue::future_pending() {
            IfaValue::Future(cell) => cell,
            _ => unreachable!(),
        };
        let task = AstTask {
            func,
            args,
            future: cell.clone(),
        };
        self.task_queue.push_back(task);
        Ok(IfaValue::Future(cell))
    }

    fn poll_one_task(&mut self) -> IfaResult<bool> {
        let Some(task) = self.task_queue.pop_front() else {
            return Ok(false);
        };
        let result = match task.func {
            IfaValue::AstFn(data) => {
                let env = self
                    .closures
                    .get(&data.closure_id)
                    .cloned()
                    .ok_or_else(|| IfaError::Runtime("Closure environment missing".into()))?;
                self.call_ast_function_values(&data.params, &data.body, env, task.args)?
            }
            other => {
                return Err(IfaError::TypeError {
                    expected: "Function".into(),
                    got: other.type_name().into(),
                });
            }
        };
        let mut state = task
            .future
            .lock()
            .map_err(|_| IfaError::Runtime("Future lock poisoned".into()))?;
        *state = ifa_types::value_union::FutureState::Ready(result);
        Ok(true)
    }

    fn await_future(&mut self, cell: &ifa_types::value_union::FutureCell) -> IfaResult<IfaValue> {
        loop {
            let ready = {
                let state = cell
                    .lock()
                    .map_err(|_| IfaError::Runtime("Future lock poisoned".into()))?;
                match &*state {
                    ifa_types::value_union::FutureState::Ready(v) => Some(v.clone()),
                    ifa_types::value_union::FutureState::Pending => None,
                }
            };
            if let Some(v) = ready {
                return Ok(v);
            }
            if !self.poll_one_task()? {
                return Err(IfaError::Runtime(
                    "Future pending with no runnable tasks".into(),
                ));
            }
        }
    }

    /// Get captured output
    pub fn get_output(&self) -> &[String] {
        &self.output
    }

    fn record_runtime_message(&mut self, spirit: &str, action: &str, message: impl Into<String>) {
        let message = message.into();
        self.output.push(message.clone());
        self.opon.record_msg(spirit, action, &message);
    }


    /// Check if currently in an unsafe block
    pub fn is_unsafe(&self) -> bool {
        self.unsafe_depth > 0
    }

    /// Import a module by path (e.g., ["std", "otura"])
    fn import_module(&mut self, path: &[String]) -> IfaResult<IfaValue> {
        let module_key = path.join(".");

        if path.first().map(|p| p == "std").unwrap_or(false) {
            let domain = path.last().cloned().unwrap_or_default();
            let marker = format!("__odu_mod__:{domain}");
            return Ok(IfaValue::str(marker));
        }

        if self.imported.contains(&module_key) {
            if let Some(exports) = self.module_cache.get(&module_key) {
                return Ok(exports.clone());
            }
        }

        // Check for circular imports
        self.import_guard.enter(&module_key)?;


        // Try to find the module file
        let file_path = self.resolve_module_path(path)?;

        // Read and parse the module
        let source = std::fs::read_to_string(&file_path).map_err(|e| {
            IfaError::Runtime(format!("Cannot read module '{}': {}", module_key, e))
        })?;

        let program = crate::parser::parse(&source).map_err(|e| {
            IfaError::Runtime(format!("Parse error in module '{}': {}", module_key, e))
        })?;

        let export_names = collect_exports(&program);

        // Save current file and execute the module
        let prev_file = self.current_file.take();
        self.current_file = Some(file_path);

        let old_env = self.env.clone();
        self.env = Environment::new();
        // Execute the module's code in isolated scope
        for stmt in &program.statements {
            self.execute_statement(stmt)?;
        }

        let mut exports = std::collections::HashMap::new();
        for name in export_names {
            if let Some(val) = Environment::get(&self.env, &name) {
                exports.insert(name, val);
            }
        }

        self.env = old_env;
        // Restore previous file
        self.current_file = prev_file;

        let exports_val = IfaValue::map(exports);
        self.import_guard.exit(&module_key);
        self.imported.insert(module_key.clone());
        self.module_cache.insert(module_key, exports_val.clone());

        Ok(exports_val)
    }

    /// Execute a block of statements in a new scope
    fn execute_block(&mut self, statements: &[Statement]) -> IfaResult<IfaValue> {
        // Push Scope - Manually (GPC Pattern)
        // We take the current env and wrap it as the parent of a new empty env
        let old_env = self.env.clone();
        self.env = Environment::with_parent(old_env.clone());

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

        // Pop scope: restore previous env.
        self.env = old_env;
        result
    }

    /// Delegate to the unified ModuleResolver so AST and VM share identical
    /// path resolution logic (mod.ifa fallback, OS separators, etc.)
    fn resolve_module_path(&self, path: &[String]) -> IfaResult<std::path::PathBuf> {
        let raw = path.join(".");
        let resolved = self.resolver.resolve(&raw)?;
        Ok(resolved.path)
    }

    fn execute_statement(&mut self, stmt: &Statement) -> IfaResult<IfaValue> {
        if let Some(debugger) = &mut self.debugger {
            debugger.on_statement(stmt, &self.env);
        }
        match stmt {
            Statement::VarDecl { name, value, .. } => {
                let val = self.evaluate(value)?;
                Environment::define(&self.env, name, val);
                Ok(IfaValue::null())
            }

            Statement::Const { name, value, .. } => {
                // Runtime interpretation: identical to VarDecl but conceptually constant
                let val = self.evaluate(value)?;
                Environment::define_const(&self.env, name, val);
                Ok(IfaValue::null())
            }

            Statement::Try {
                try_body,
                catch_var,
                catch_body,
                finally_body,
                ..
            } => {
                let mut result = match self.execute_block(try_body) {
                    Ok(val) => Ok(val),
                    Err(e) => {
                        // Execute catch block with new scope
                        // We must manually enter scope for catch to bind the error variable
                        let old_env = self.env.clone();
                        self.env = Environment::with_parent(old_env.clone());

                        if !catch_var.is_empty() {
                            // R4: Bind structured error value, not a plain string
                            let kind_name = match &e {
                                IfaError::TypeError { .. } => "TypeError",
                                IfaError::Runtime(_) => "RuntimeError",
                                IfaError::DivisionByZero(_) => "DivisionByZeroError",
                                IfaError::PermissionDenied(_) => "PermissionError",
                                IfaError::UndefinedVariable(_) => "ReferenceError",
                                _ => "Error",
                            };
                            let mut error_map = std::collections::HashMap::new();
                            error_map.insert("message".to_string(), IfaValue::str(e.to_string()));
                            error_map
                                .insert("kind".to_string(), IfaValue::str(kind_name.to_string()));
                            Environment::define(&self.env, catch_var, IfaValue::map(error_map));
                        }

                        // Execute catch body statements manually inside this scope
                        // (We reusing execute_block logic but inline to avoid double-scoping
                        // or we could use execute_block if we didn't already push scope...
                        // Actually execute_block pushes scope. So we can't use it if we want to bind var *in* that scope first.
                        // So we do it manually here for catch.)

                        let mut result = Ok(IfaValue::null());
                        for s in catch_body {
                            result = self.execute_statement(s);
                            if result.is_err() {
                                break;
                            }
                        }

                        // Pop scope
                        self.env = old_env;

                        result
                    }
                };

                if let Some(finally_body) = finally_body {
                    let finally_result = self.execute_block(finally_body);
                    result = match finally_result {
                        Ok(val) if val.is_return() => Ok(val),
                        Ok(_) => result,
                        Err(err) => Err(err),
                    };
                }

                result
            }

            Statement::Assignment { target, value, .. } => {
                let val = self.evaluate(value)?;
                match target {
                    AssignTarget::Variable(name) => {
                        if Environment::is_const(&self.env, name) {
                            return Err(IfaError::TypeError {
                                expected: "Mutable binding".into(),
                                got: format!("const {name}"),
                            });
                        }
                        if !Environment::set(&self.env, name, val.clone()) {
                            Environment::define(&self.env, name, val);
                        }
                    }
                    AssignTarget::Index { name, index } => {
                        let idx = self.evaluate(index)?;
                        let mut container = Environment::get(&self.env, name).ok_or_else(|| {
                            IfaError::Runtime(format!("Undefined variable: {}", name))
                        })?;

                        // Index Assignment: xs[0] = 10
                        match container {
                            IfaValue::List(ref mut vec_arc) => {
                                let i = match idx {
                                    IfaValue::Int(n) => n as usize,
                                    _ => {
                                        return Err(IfaError::Runtime(
                                            "List index must be Int".into(),
                                        ));
                                    }
                                };

                                // HIGH PERFORMANCE: CoW using make_mut
                                // O(1) if unique, O(N) if shared.
                                let vec = std::sync::Arc::make_mut(vec_arc);
                                if i >= vec.len() {
                                    return Err(IfaError::Runtime("Index out of bounds".into()));
                                }
                                vec[i] = val;
                            }

                            IfaValue::Map(ref mut map_arc) => {
                                let k = match idx {
                                    IfaValue::Str(s) => s.clone(),
                                    _ => {
                                        return Err(IfaError::Runtime("Map key must be Str".into()));
                                    }
                                };
                                // HIGH PERFORMANCE: CoW using make_mut
                                let map = std::sync::Arc::make_mut(map_arc);
                                map.insert(k, val);
                            }
                            _ => {
                                return Err(IfaError::Runtime(
                                    "Invalid index assignment target".into(),
                                ));
                            }
                        }
                        Environment::set(&self.env, name, container);
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
                                if Environment::is_const(&self.env, &name) {
                                    return Err(IfaError::TypeError {
                                        expected: "Mutable binding".into(),
                                        got: format!("const {name}"),
                                    });
                                }
                                if !Environment::set(&self.env, &name, val.clone()) {
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
                        let res = self.execute_statement(s)?;
                        if res.is_return() {
                            return Ok(res);
                        }
                    }
                } else if let Some(else_stmts) = else_body {
                    for s in else_stmts {
                        let res = self.execute_statement(s)?;
                        if res.is_return() {
                            return Ok(res);
                        }
                    }
                }
                Ok(IfaValue::null())
            }

            Statement::While {
                condition, body, ..
            } => {
                while self.evaluate(condition)?.is_truthy() {
                    for s in body {
                        let res = self.execute_statement(s)?;
                        if res.is_return() {
                            return Ok(res);
                        }
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
                        Environment::define(&self.env, var, item);
                        for s in body {
                            let res = self.execute_statement(s)?;
                            if res.is_return() {
                                return Ok(res);
                            }
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

            Statement::Ase { .. } => Ok(IfaValue::null()),

            // Inside execute_statement match
            Statement::EseDef {
                name,
                params,
                body,
                is_async,
                ..
            } => {
                let closure_id = self.next_closure_id;
                self.next_closure_id = self.next_closure_id.saturating_add(1);
                self.closures.insert(closure_id, self.env.clone());

                let value = IfaValue::AstFn(Arc::new(ifa_types::value_union::AstFnData {
                    name: name.clone(),
                    params: params.iter().map(|p| p.name.clone()).collect(),
                    body: body.clone(),
                    closure_id,
                    is_async: *is_async,
                }));
                Environment::define(&self.env, name, value);
                Ok(IfaValue::null())
            }

            Statement::Import { path, names, .. } => {
                let exports = self.import_module(path)?;

                if let Some(names) = names {
                    for name in names {
                        let val = match &exports {
                            IfaValue::Map(map) => {
                                let key: std::sync::Arc<str> = std::sync::Arc::from(name.as_str());
                                map.get(&key).cloned().ok_or_else(|| {
                                    IfaError::Runtime(format!("Export '{}' not found", name))
                                })?
                            }
                            _ => {
                                return Err(IfaError::Runtime(
                                    "Import did not return an exports map".into(),
                                ));
                            }
                        };
                        Environment::define(&self.env, name, val);
                    }
                } else {
                    let module_name = path.last().cloned().unwrap_or_else(|| "module".into());
                    Environment::define(&self.env, &module_name, exports);
                }
                Ok(IfaValue::null())
            }

            Statement::OduDef { name, body, .. } => {
                let _ = (name, body);
                Err(IfaError::NotImplemented(
                    "Class (odu) definitions are not supported in the AST interpreter backend"
                        .into(),
                ))
            }

            Statement::Expr { expr, .. } => self.evaluate(expr),

            Statement::Taboo { source, target, .. } => {
                self.record_runtime_message(
                    "Eewo",
                    "declare",
                    format!("[taboo] {} -> {} forbidden", source, target),
                );
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
                    _ => Err(IfaError::Runtime(format!(
                        "[ẹ̀wọ̀/verify] Assertion expects boolean, got: {:?}",
                        condition_val.type_name()
                    ))),
                }
            }

            Statement::Opon { size, .. } => {
                let opon_size = match size.as_str() {
                    "kekere" => crate::bytecode::OponSize::Kekere,
                    "arinrin" => crate::bytecode::OponSize::Arinrin,
                    "nla" => crate::bytecode::OponSize::Nla,
                    "ailopin" => crate::bytecode::OponSize::Ailopin,
                    _ => {
                        self.record_runtime_message(
                            "Opon",
                            "configure",
                            format!("[opon] Unknown size '{}', defaulting to arinrin", size),
                        );
                        crate::bytecode::OponSize::Arinrin
                    }
                };
                // Set call-frame limit for this interpreter session
                let (_, frame_cap) = opon_size.limits();
                self.call_depth_limit = frame_cap;
                Ok(IfaValue::null())
            }

            Statement::Ebo { offering, .. } => {
                let val = self.evaluate(offering)?;
                self.record_runtime_message(
                    "Ebo",
                    "initiate",
                    format!("[ẹbọ/sacrifice] Aspect initiated: {}", val),
                );
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
                match val {
                    IfaValue::Int(micros) if micros >= 0 => {
                        self.record_runtime_message(
                            "Osa",
                            "yield",
                            format!("[yield] requested {} microseconds", micros),
                        );
                    }
                    IfaValue::Int(_) => {
                        return Err(IfaError::Runtime(
                            "Yield duration must be non-negative".to_string(),
                        ));
                    }
                    _ => {
                        return Err(IfaError::Runtime(
                            "Yield duration must be an integer (microseconds)".to_string(),
                        ));
                    }
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

            Expression::Identifier(name) => Environment::get(&self.env, name)
                .ok_or_else(|| IfaError::Runtime(format!("Undefined variable: {}", name))),

            Expression::BinaryOp { left, op, right } => {
                // Short-circuit operators must not evaluate the RHS eagerly.
                match op {
                    BinaryOperator::And => {
                        let l = self.evaluate(left)?;
                        if !l.is_truthy() {
                            return Ok(l);
                        }
                        self.evaluate(right)
                    }
                    BinaryOperator::Or => {
                        let l = self.evaluate(left)?;
                        if l.is_truthy() {
                            return Ok(l);
                        }
                        self.evaluate(right)
                    }
                    _ => {
                        let l = self.evaluate(left)?;
                        let r = self.evaluate(right)?;
                        self.apply_binary_op(&l, op, &r)
                    }
                }
            }

            Expression::UnaryOp { op, expr } => {
                // Special handling for AddressOf to avoid evaluating the expression fully if it's an Identifier
                if matches!(op, UnaryOperator::AddressOf) {
                    // &x -> Ref("x") - Unsupported in current type system
                    if let Expression::Identifier(_name) = &**expr {
                        return Ok(IfaValue::Str(_name.clone().into()));
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
                        {
                            if let IfaValue::Int(addr) = r {
                                Ok(IfaValue::Int(addr))
                            } else {
                                Err(IfaError::Runtime(
                                    "Cannot take address of a non-integer literal".into(),
                                ))
                            }
                        }
                    }
                    UnaryOperator::Dereference => {
                        // *r
                        // Deref not supported currently as AddressOf is disabled
                        {
                            let ptr = self.evaluate(expr)?;
                            match ptr {
                                IfaValue::Str(name) => Environment::get(&self.env, &name)
                                    .ok_or_else(|| {
                                        IfaError::Runtime(format!(
                                            "Reference to undefined variable: {}",
                                            name
                                        ))
                                    }),
                                IfaValue::Int(addr) => {
                                    if !self.is_unsafe() {
                                        return Err(IfaError::Runtime(format!(
                                            "Safety violation: *0x{:X} requires 'ailewu'",
                                            addr
                                        )));
                                    }
                                    self.opon.get(addr as usize).cloned().ok_or_else(|| {
                                        IfaError::Runtime(format!(
                                            "Invalid memory address: {}",
                                            addr
                                        ))
                                    })
                                }
                                _ => Err(IfaError::Runtime(format!(
                                    "Cannot dereference type: {}",
                                    ptr.type_name()
                                ))),
                            }
                        }
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
                let value = Environment::get(&self.env, name)
                    .ok_or_else(|| IfaError::UndefinedFunction(name.clone()))?;
                match value {
                    IfaValue::AstFn(data) => {
                        if data.is_async {
                            let mut arg_values = Vec::with_capacity(args.len());
                            for arg in args {
                                arg_values.push(self.evaluate(arg)?);
                            }
                            self.spawn_task(IfaValue::AstFn(data.clone()), arg_values)
                        } else {
                            let env =
                                self.closures
                                    .get(&data.closure_id)
                                    .cloned()
                                    .ok_or_else(|| {
                                        IfaError::Runtime("Closure environment missing".into())
                                    })?;
                            self.call_ast_function(&data.params, &data.body, env, args)
                        }
                    }
                    IfaValue::Str(s) => {
                        if let Some((domain, method)) = parse_odu_fn_marker(&s) {
                            let mut arg_values = Vec::with_capacity(args.len());
                            for arg in args {
                                arg_values.push(self.evaluate(arg)?);
                            }
                            self.handlers.dispatch(
                                domain,
                                &method,
                                arg_values,
                                &self.env,
                                &mut self.output,
                            )
                        } else {
                            Err(IfaError::TypeError {
                                expected: "Function".into(),
                                got: "Str".into(),
                            })
                        }
                    }
                    other => Err(IfaError::TypeError {
                        expected: "Function".into(),
                        got: other.type_name().into(),
                    }),
                }
            }

            Expression::Await(expr) => {
                let value = self.evaluate(expr)?;
                match value {
                    IfaValue::Future(cell) => self.await_future(&cell),
                    other => Err(IfaError::TypeError {
                        expected: "Future".into(),
                        got: other.type_name().into(),
                    }),
                }
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

            Expression::Try(expr) => {
                // §12.3: Error propagation in the tree-walking interpreter.
                // Delegate to `evaluate`. Since the interpreter operates on
                // IfaResult<IfaValue>, error propagation is already handled
                // by Rust — this just surfaces the inner error to the caller.
                self.evaluate(expr)
            }

            Expression::InterpolatedString { parts } => {
                let mut res = String::new();
                for part in parts {
                    match part {
                        InterpolatedPart::Literal(s) => res.push_str(s),
                        InterpolatedPart::Expression(expr) => {
                            let val = self.evaluate(expr)?;
                            res.push_str(&val.to_string());
                        }
                    }
                }
                Ok(IfaValue::Str(res.into()))
            }
        }
    }

    fn execute_odu_call(&mut self, call: &OduCall) -> IfaResult<IfaValue> {
        let args: Vec<IfaValue> = call
            .args
            .iter()
            .map(|arg| self.evaluate(arg))
            .collect::<Result<_, _>>()?;

        // Minimal async support for Osa domain (spawn/await helpers)
        if call.domain == OduDomain::Osa {
            match call.method.as_str() {
                "ise" | "spawn" | "sa" | "bẹrẹ" => {
                    let task = args
                        .get(0)
                        .cloned()
                        .ok_or_else(|| IfaError::ArgumentError("Osa.ise expects a task".into()))?;
                    let task_args = args
                        .get(1)
                        .and_then(|v| {
                            if let IfaValue::List(list) = v {
                                Some(list.to_vec())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_default();
                    return self.spawn_task(task, task_args);
                }
                "duro" | "sleep" => {
                    let duration = args.get(0).ok_or_else(|| {
                        IfaError::ArgumentError(
                            "Osa.sleep expects a duration in milliseconds".into(),
                        )
                    })?;
                    match duration {
                        IfaValue::Int(ms) if *ms >= 0 => {
                            self.record_runtime_message(
                                "Osa",
                                "sleep",
                                format!("[osa.sleep] requested {} milliseconds", ms),
                            );
                            return Ok(IfaValue::future_ready(IfaValue::null()));
                        }
                        IfaValue::Int(_) => {
                            return Err(IfaError::ArgumentError(
                                "Osa.sleep duration must be non-negative".into(),
                            ));
                        }
                        other => {
                            return Err(IfaError::ArgumentError(format!(
                                "Osa.sleep expects an integer duration, got {}",
                                other.type_name()
                            )));
                        }
                    }
                }
                "gbogbo" | "all" => {
                    if let Some(IfaValue::List(list)) = args.get(0) {
                        return Ok(IfaValue::list(list.to_vec()));
                    }
                }
                _ => {}
            }
        }

        self.handlers.dispatch(
            call.domain.clone(),
            &call.method,
            args,
            &self.env,
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
                (IfaValue::Str(a), IfaValue::Str(b)) => Ok(IfaValue::str(format!("{}{}", a, b))),
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
                // Spec: Float division by zero never errors; it yields IEEE 754 results (Inf/NaN).
                (IfaValue::Float(a), IfaValue::Float(b)) => Ok(IfaValue::float(a / b)),
                // Mixed
                (IfaValue::Int(a), IfaValue::Float(b)) => Ok(IfaValue::float(*a as f64 / b)),
                (IfaValue::Float(a), IfaValue::Int(b)) => Ok(IfaValue::float(a / *b as f64)),

                _ => Err(IfaError::DivisionByZero(
                    "Division by zero or invalid operands".into(),
                )),
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
                    _ => false, // Default to false for mismatched types
                };
                Ok(IfaValue::bool(eq))
            }
            BinaryOperator::NotEq => {
                let eq = match (left, right) {
                    (IfaValue::Int(a), IfaValue::Int(b)) => a == b,
                    (IfaValue::Float(a), IfaValue::Float(b)) => (a - b).abs() < f64::EPSILON,
                    (IfaValue::Str(a), IfaValue::Str(b)) => a == b,
                    (IfaValue::Bool(a), IfaValue::Bool(b)) => a == b,
                    (IfaValue::Null, IfaValue::Null) => true,
                    _ => false,
                };
                Ok(IfaValue::bool(!eq))
            }
            BinaryOperator::Lt => match (left, right) {
                (IfaValue::Int(a), IfaValue::Int(b)) => Ok(IfaValue::bool(a < b)),
                (IfaValue::Float(a), IfaValue::Float(b)) => Ok(IfaValue::bool(a < b)),
                (IfaValue::Int(a), IfaValue::Float(b)) => Ok(IfaValue::bool((*a as f64) < *b)),
                (IfaValue::Float(a), IfaValue::Int(b)) => Ok(IfaValue::bool(*a < (*b as f64))),
                _ => Err(IfaError::TypeError {
                    expected: "Comparable types (Int/Float)".into(),
                    got: format!("{} < {}", left.type_name(), right.type_name()),
                }),
            },
            BinaryOperator::LtEq => match (left, right) {
                (IfaValue::Int(a), IfaValue::Int(b)) => Ok(IfaValue::bool(a <= b)),
                (IfaValue::Float(a), IfaValue::Float(b)) => Ok(IfaValue::bool(a <= b)),
                (IfaValue::Int(a), IfaValue::Float(b)) => Ok(IfaValue::bool((*a as f64) <= *b)),
                (IfaValue::Float(a), IfaValue::Int(b)) => Ok(IfaValue::bool(*a <= (*b as f64))),
                _ => Err(IfaError::TypeError {
                    expected: "Comparable types (Int/Float)".into(),
                    got: format!("{} <= {}", left.type_name(), right.type_name()),
                }),
            },
            BinaryOperator::Gt => match (left, right) {
                (IfaValue::Int(a), IfaValue::Int(b)) => Ok(IfaValue::bool(a > b)),
                (IfaValue::Float(a), IfaValue::Float(b)) => Ok(IfaValue::bool(a > b)),
                (IfaValue::Int(a), IfaValue::Float(b)) => Ok(IfaValue::bool((*a as f64) > *b)),
                (IfaValue::Float(a), IfaValue::Int(b)) => Ok(IfaValue::bool(*a > (*b as f64))),
                _ => Err(IfaError::TypeError {
                    expected: "Comparable types (Int/Float)".into(),
                    got: format!("{} > {}", left.type_name(), right.type_name()),
                }),
            },
            BinaryOperator::GtEq => match (left, right) {
                (IfaValue::Int(a), IfaValue::Int(b)) => Ok(IfaValue::bool(a >= b)),
                (IfaValue::Float(a), IfaValue::Float(b)) => Ok(IfaValue::bool(a >= b)),
                (IfaValue::Int(a), IfaValue::Float(b)) => Ok(IfaValue::bool((*a as f64) >= *b)),
                (IfaValue::Float(a), IfaValue::Int(b)) => Ok(IfaValue::bool(*a >= (*b as f64))),
                _ => Err(IfaError::TypeError {
                    expected: "Comparable types (Int/Float)".into(),
                    got: format!("{} >= {}", left.type_name(), right.type_name()),
                }),
            },
            // Note: short-circuiting is handled in `evaluate()`; this is a value-level fallback.
            BinaryOperator::And => Ok(if !left.is_truthy() {
                left.clone()
            } else {
                right.clone()
            }),
            BinaryOperator::Or => Ok(if left.is_truthy() {
                left.clone()
            } else {
                right.clone()
            }),
        }
    }

    fn call_method(
        &mut self,
        obj: &IfaValue,
        method: &str,
        args: &[Expression],
    ) -> IfaResult<IfaValue> {
        if let IfaValue::Str(s) = obj {
            if let Some(domain) = parse_odu_mod_marker(s) {
                let mut arg_values = Vec::with_capacity(args.len());
                for arg in args {
                    arg_values.push(self.evaluate(arg)?);
                }
                return self.handlers.dispatch(
                    domain,
                    method,
                    arg_values,
                    &self.env,
                    &mut self.output,
                );
            }
        }

        match obj {
            IfaValue::Map(map) => {
                let key: std::sync::Arc<str> = std::sync::Arc::from(method);
                let func = map.get(&key).cloned().ok_or_else(|| {
                    IfaError::Runtime(format!("Method '{}' not found on map", method))
                })?;

                let mut arg_values = Vec::with_capacity(args.len());
                for arg in args {
                    arg_values.push(self.evaluate(arg)?);
                }

                match func {
                    IfaValue::AstFn(data) => {
                        let env =
                            self.closures
                                .get(&data.closure_id)
                                .cloned()
                                .ok_or_else(|| {
                                    IfaError::Runtime("Closure environment missing".into())
                                })?;
                        self.call_ast_function_values(&data.params, &data.body, env, arg_values)
                    }
                    IfaValue::Str(s) => {
                        if let Some((domain, method)) = parse_odu_fn_marker(&s) {
                            self.handlers.dispatch(
                                domain,
                                &method,
                                arg_values,
                                &self.env,
                                &mut self.output,
                            )
                        } else {
                            Err(IfaError::TypeError {
                                expected: "Function".into(),
                                got: "Str".into(),
                            })
                        }
                    }
                    _ => Err(IfaError::TypeError {
                        expected: "Function".into(),
                        got: func.type_name().into(),
                    }),
                }
            }
            _ => Err(IfaError::Runtime(format!(
                "Method '{}' not implemented",
                method
            ))),
        }
    }

    fn call_ast_function(
        &mut self,
        params: &[String],
        body: &[Statement],
        env: EnvRef,
        args: &[Expression],
    ) -> IfaResult<IfaValue> {
        if args.len() != params.len() {
            return Err(IfaError::ArityMismatch {
                expected: params.len(),
                got: args.len(),
            });
        }

        // Enforce call depth limit set by #opon
        self.call_depth += 1;
        if let Some(limit) = self.call_depth_limit {
            if self.call_depth > limit {
                self.call_depth -= 1;
                return Err(IfaError::StackOverflow {
                    limit,
                    directive: match limit {
                        64 => crate::bytecode::OponSize::Kekere,
                        512 => crate::bytecode::OponSize::Arinrin,
                        4096 => crate::bytecode::OponSize::Nla,
                        _ => crate::bytecode::OponSize::Arinrin,
                    },
                });
            }
        }

        let mut arg_values = Vec::with_capacity(args.len());
        for arg in args {
            arg_values.push(self.evaluate(arg)?);
        }

        // Enter function scope (lexical parent = env at definition time).
        let old_env = self.env.clone();
        self.env = Environment::with_parent(env);

        for (param, value) in params.iter().zip(arg_values.into_iter()) {
            Environment::define(&self.env, param, value);
        }

        let mut result = Ok(IfaValue::null());
        for stmt in body {
            result = self.execute_statement(stmt);
            if let Ok(val) = &result {
                if val.is_return() {
                    break;
                }
            } else {
                break;
            }
        }

        // Exit function scope.
        self.env = old_env;
        self.call_depth -= 1;

        let value = result?;
        match value {
            IfaValue::Return(v) => Ok((*v).clone()),
            other => Ok(other),
        }
    }

    fn call_ast_function_values(
        &mut self,
        params: &[String],
        body: &[Statement],
        env: EnvRef,
        args: Vec<IfaValue>,
    ) -> IfaResult<IfaValue> {
        if args.len() != params.len() {
            return Err(IfaError::ArityMismatch {
                expected: params.len(),
                got: args.len(),
            });
        }

        self.call_depth += 1;
        if let Some(limit) = self.call_depth_limit {
            if self.call_depth > limit {
                self.call_depth -= 1;
                return Err(IfaError::StackOverflow {
                    limit,
                    directive: match limit {
                        64 => crate::bytecode::OponSize::Kekere,
                        512 => crate::bytecode::OponSize::Arinrin,
                        4096 => crate::bytecode::OponSize::Nla,
                        _ => crate::bytecode::OponSize::Arinrin,
                    },
                });
            }
        }

        let old_env = self.env.clone();
        self.env = Environment::with_parent(env);

        for (param, value) in params.iter().zip(args.into_iter()) {
            Environment::define(&self.env, param, value);
        }

        let mut result = Ok(IfaValue::null());
        for stmt in body {
            result = self.execute_statement(stmt);
            if let Ok(val) = &result {
                if val.is_return() {
                    break;
                }
            } else {
                break;
            }
        }

        self.env = old_env;
        self.call_depth -= 1;

        let value = result?;
        match value {
            IfaValue::Return(v) => Ok((*v).clone()),
            other => Ok(other),
        }
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

fn collect_exports(program: &Program) -> Vec<String> {
    let mut out = Vec::new();
    for stmt in &program.statements {
        match stmt {
            Statement::VarDecl {
                name,
                visibility: Visibility::Public,
                ..
            } => out.push(name.clone()),
            Statement::Const {
                name,
                visibility: Visibility::Public,
                ..
            } => out.push(name.clone()),
            Statement::EseDef {
                name,
                visibility: Visibility::Public,
                ..
            } => out.push(name.clone()),
            Statement::OduDef {
                name,
                visibility: Visibility::Public,
                ..
            } => out.push(name.clone()),
            _ => {}
        }
    }
    out
}

fn parse_odu_mod_marker(s: &str) -> Option<OduDomain> {
    const PREFIX: &str = "__odu_mod__:";
    if let Some(rest) = s.strip_prefix(PREFIX) {
        if let Ok(id) = rest.parse::<u8>() {
            return odu_domain_from_id(id);
        }
        return odu_domain_from_name(rest);
    }
    None
}

fn parse_odu_fn_marker(s: &str) -> Option<(OduDomain, String)> {
    const PREFIX: &str = "__odu_fn__:";
    if let Some(rest) = s.strip_prefix(PREFIX) {
        let mut parts = rest.splitn(2, ':');
        let domain = parts.next()?;
        let method = parts.next()?.to_string();
        if let Ok(id) = domain.parse::<u8>() {
            return odu_domain_from_id(id).map(|d| (d, method));
        }
        return odu_domain_from_name(domain).map(|d| (d, method));
    }
    None
}

fn odu_domain_from_id(id: u8) -> Option<OduDomain> {
    match id {
        0 => Some(OduDomain::Ogbe),
        1 => Some(OduDomain::Oyeku),
        2 => Some(OduDomain::Iwori),
        3 => Some(OduDomain::Odi),
        4 => Some(OduDomain::Irosu),
        5 => Some(OduDomain::Owonrin),
        6 => Some(OduDomain::Obara),
        7 => Some(OduDomain::Okanran),
        8 => Some(OduDomain::Ogunda),
        9 => Some(OduDomain::Osa),
        10 => Some(OduDomain::Ika),
        11 => Some(OduDomain::Oturupon),
        12 => Some(OduDomain::Otura),
        13 => Some(OduDomain::Irete),
        14 => Some(OduDomain::Ose),
        15 => Some(OduDomain::Ofun),
        16 => Some(OduDomain::Coop),
        17 => Some(OduDomain::Opele),
        18 => Some(OduDomain::Cpu),
        19 => Some(OduDomain::Gpu),
        20 => Some(OduDomain::Storage),
        21 => Some(OduDomain::Backend),
        22 => Some(OduDomain::Frontend),
        23 => Some(OduDomain::Crypto),
        24 => Some(OduDomain::Ml),
        25 => Some(OduDomain::GameDev),
        26 => Some(OduDomain::Iot),
        27 => Some(OduDomain::Ohun),
        28 => Some(OduDomain::Fidio),
        29 => Some(OduDomain::Sys),
        _ => None,
    }
}

fn odu_domain_from_name(name: &str) -> Option<OduDomain> {
    match name.to_lowercase().as_str() {
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
        "coop" => Some(OduDomain::Coop),
        "opele" => Some(OduDomain::Opele),
        "cpu" => Some(OduDomain::Cpu),
        "gpu" => Some(OduDomain::Gpu),
        "storage" => Some(OduDomain::Storage),
        "backend" => Some(OduDomain::Backend),
        "frontend" => Some(OduDomain::Frontend),
        "crypto" => Some(OduDomain::Crypto),
        "ml" => Some(OduDomain::Ml),
        "gamedev" => Some(OduDomain::GameDev),
        "iot" => Some(OduDomain::Iot),
        "ohun" => Some(OduDomain::Ohun),
        "fidio" => Some(OduDomain::Fidio),
        "sys" => Some(OduDomain::Sys),
        _ => None,
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

        assert_eq!(Environment::get(&interp.env, "x"), Some(IfaValue::int(42)));
    }

    #[test]
    fn test_arithmetic_precedence() {
        // Test that * has higher precedence than +
        let program = parse("ayanmo x = 2 + 3 * 4;").unwrap();
        let mut interp = Interpreter::new();
        interp.execute(&program).unwrap();

        // Should be 2 + (3 * 4) = 14
        assert_eq!(Environment::get(&interp.env, "x"), Some(IfaValue::int(14)));
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
            Environment::get(&interp.env, "s"),
            Some(IfaValue::str("Hello World"))
        );
    }

    #[test]
    fn test_closure_captures_outer_variable() {
        let program = parse(
            r#"
            ese make_adder(x) {
                ese add(y) { pada x + y; }
                pada add;
            }

            ayanmo f = make_adder(5);
            ayanmo r = f(3);
            "#,
        )
        .expect("Parse failed");

        let mut interp = Interpreter::new();
        interp.execute(&program).unwrap();

        assert_eq!(Environment::get(&interp.env, "r"), Some(IfaValue::Int(8)));
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

        assert_eq!(Environment::get(&interp.env, "x"), Some(IfaValue::int(1)));
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

        assert_eq!(Environment::get(&interp.env, "x"), Some(IfaValue::int(5)));
    }

    #[test]
    fn test_function_def_and_call() {
        let program = parse(
            r#"
            ese add(a, b) { da a + b; }
            ayanmo x = add(1, 2);
        "#,
        )
        .unwrap();
        let mut interp = Interpreter::new();
        interp.execute(&program).unwrap();

        assert_eq!(Environment::get(&interp.env, "x"), Some(IfaValue::int(3)));
    }

    #[test]
    fn test_return_propagates_through_if() {
        let program = parse(
            r#"
            ese f(x) {
                ti x {
                    da 1;
                }
                da 2;
            }
            ayanmo a = f(1);
            ayanmo b = f(0);
        "#,
        )
        .unwrap();
        let mut interp = Interpreter::new();
        interp.execute(&program).unwrap();

        assert_eq!(Environment::get(&interp.env, "a"), Some(IfaValue::int(1)));
        assert_eq!(Environment::get(&interp.env, "b"), Some(IfaValue::int(2)));
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

        assert_eq!(Environment::get(&interp.env, "len"), Some(IfaValue::int(3)));
    }

    #[test]
    fn test_string_upper() {
        let program = parse(r#"ayanmo s = Ika.upper("hello");"#).unwrap();
        let mut interp = Interpreter::new();
        interp.execute(&program).unwrap();

        assert_eq!(
            Environment::get(&interp.env, "s"),
            Some(IfaValue::str("HELLO"))
        );
    }

    #[test]
    fn test_math_add() {
        let program = parse(r#"ayanmo x = Obara.add(1, 2, 3, 4);"#).unwrap();
        let mut interp = Interpreter::new();
        interp.execute(&program).unwrap();

        assert_eq!(Environment::get(&interp.env, "x"), Some(IfaValue::int(10)));
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

        assert_eq!(
            Environment::get(&interp.env, "a"),
            Some(IfaValue::bool(true))
        );
        assert_eq!(
            Environment::get(&interp.env, "b"),
            Some(IfaValue::bool(true))
        );
        assert_eq!(
            Environment::get(&interp.env, "c"),
            Some(IfaValue::bool(true))
        );
        assert_eq!(
            Environment::get(&interp.env, "d"),
            Some(IfaValue::bool(true))
        );
    }

    #[test]
    fn test_logical_ops_return_operands_and_short_circuit() {
        let program = parse(
            r#"
            ayanmo a = 0 && (1 / 0);          # should short-circuit: a == 0
            ayanmo b = 1 || (1 / 0);          # should short-circuit: b == 1
            ayanmo c = ofo || "default";      # should return RHS: c == "default"
        "#,
        )
        .unwrap();

        let mut interp = Interpreter::new();
        interp.execute(&program).unwrap();

        assert_eq!(Environment::get(&interp.env, "a"), Some(IfaValue::int(0)));
        assert_eq!(Environment::get(&interp.env, "b"), Some(IfaValue::int(1)));
        assert_eq!(
            Environment::get(&interp.env, "c"),
            Some(IfaValue::str("default"))
        );
    }

    #[test]
    fn test_try_finally_runs_on_return() {
        let program = parse(
            r#"
            ayanmo y = 0;
            ese f() {
                gbiyanju {
                    pada 1;
                } gba (e) {
                    pada 2;
                } nipari {
                    y = 3;
                }
            }
            ayanmo out = f();
            "#,
        )
        .unwrap();

        let mut interp = Interpreter::new();
        interp.execute(&program).unwrap();

        assert_eq!(Environment::get(&interp.env, "out"), Some(IfaValue::int(1)));
        assert_eq!(Environment::get(&interp.env, "y"), Some(IfaValue::int(3)));
    }

    #[test]
    fn test_runtime_directives_are_captured_in_output_and_opon() {
        let program = parse(
            r#"
            taboo: Ogbe -> Oyeku;
            ebo "server";
            jowo 10;
            "#,
        )
        .unwrap();

        let mut interp = Interpreter::new();
        interp.execute(&program).unwrap();

        let output = interp.get_output().join("\n");
        assert!(output.contains("[taboo] Ogbe -> Oyeku forbidden"));
        assert!(output.contains("[ẹbọ/sacrifice] Aspect initiated: server"));
        assert!(output.contains("[yield] requested 10 microseconds"));

        let history = interp.opon.get_history();
        assert!(history.iter().any(|event| event.action == "declare"));
        assert!(history.iter().any(|event| event.action == "initiate"));
        assert!(history.iter().any(|event| event.action == "yield"));
    }

    #[test]
    fn test_osa_sleep_records_without_blocking() {
        let program = parse(r#"ayanmo pending = Osa.sleep(5);"#).unwrap();
        let mut interp = Interpreter::new();
        interp.execute(&program).unwrap();

        assert!(
            interp
                .get_output()
                .iter()
                .any(|line| line.contains("[osa.sleep] requested 5 milliseconds"))
        );
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
