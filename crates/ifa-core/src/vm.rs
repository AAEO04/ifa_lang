//! # Ifá-Lang Virtual Machine
//!
//! Stack-based bytecode interpreter for Ifá-Lang.
//!
//! ### ✅ ARCHITECTURAL STATUS (String Operations)
//! `OpCode::Add` is now PURE NUMERIC (Int/Float only). String concatenation uses
//! the dedicated `OpCode::Concat (0x27)`, which is strict `Str + Str` only.
//! The `text += " more"` compiler path emits `ToString` + `Concat` as appropriate.
//!
//! Refer to `patch.md` for the Phase 7 Hardening Roadmap.

use crate::bytecode::{Bytecode, OpCode};
use crate::error::{IfaError, IfaResult};
use crate::native::{OduRegistry, VmContext};
use crate::opon::Opon;
use ifa_types::value_union::{ClosureData, FutureState, IfaValue, ResultPayload, UpvalueCell};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::sync::Arc;

/// Call frame for function calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallFrame {
    /// Return address (instruction pointer to return to)
    pub return_addr: usize,
    /// Base pointer (stack index where this frame starts)
    pub base_ptr: usize,
    /// Local variable count
    pub local_count: usize,
    /// Captured closure environment for this frame (if executing a closure).
    pub closure_env: Option<Arc<Vec<UpvalueCell>>>,
    /// Whether this frame returns an async value (wrap in Future)
    pub async_return: bool,
}

impl CallFrame {
    fn new(
        return_addr: usize,
        base_ptr: usize,
        closure_env: Option<Arc<Vec<UpvalueCell>>>,
        async_return: bool,
    ) -> Self {
        Self {
            return_addr,
            base_ptr,
            local_count: 0,
            closure_env,
            async_return,
        }
    }
}

/// Recovery frame for exception handling (The Shield of Ọ̀kànràn)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RecoveryFrame {
    /// Stack depth to restore to
    pub stack_depth: usize,
    /// Call frame depth to restore to
    pub call_depth: usize,
    /// Instruction pointer to jump to (Catch Handler)
    pub catch_ip: usize,
    /// Absolute IP of the finally block, if one exists.
    /// §12.4: MUST execute on all exit paths.
    pub finally_ip: Option<usize>,
    /// Whether this frame can still catch and enter its `gba` block.
    /// After control has entered catch, the frame remains only to ensure
    /// `nipari` still runs on return/throw/error from the catch body.
    pub can_catch: bool,
}

/// Typed continuation stored when a `Return` or `Throw` is intercepted
/// by a `finally` block. `FinallyEnd` reads this to complete the operation
/// after cleanup has run. No value-stack pollution.
#[derive(Debug, Clone)]
pub enum FinallyResumption {
    /// A `pada` (return) was intercepted. Execute the frame pop after cleanup.
    Return { return_value: IfaValue },
    /// A `ta` (throw) was intercepted. Re-propagate after cleanup.
    Propagate { error: IfaError },
}

use crate::vm_ikin::Ikin;
use crate::vm_iroke;

/// The Ifá Virtual Machine
#[derive(Serialize, Deserialize)]
pub struct IfaVM {
    /// Value stack
    stack: Vec<IfaValue>,
    /// Call stack
    frames: Vec<CallFrame>,
    /// Instruction pointer
    pub ip: usize,

    /// Global variables
    globals: std::collections::HashMap<String, IfaValue>,
    /// Memory (Opon)
    pub opon: Opon,
    /// Stack capacity limit
    stack_limit: Option<usize>,
    /// Call frame capacity limit
    frame_limit: Option<usize>,
    /// Active memory directive
    pub opon_size: crate::bytecode::OponSize,

    /// Function Registry (Standard Library)
    #[serde(skip)]
    pub registry: Option<Box<dyn OduRegistry>>,
    /// Halt flag
    halted: bool,
    /// Execution ticks (for GC/Interrupts)
    pub ticks: usize,

    /// Recovery stack (for Try/Catch)
    recovery_stack: Vec<RecoveryFrame>,

    /// The Sacred Nuts - Runtime Constant Pool
    pub ikin: Ikin,

    /// Async task queue (cooperative scheduler)
    #[serde(skip)]
    task_queue: VecDeque<Task>,

    /// Already imported modules
    #[serde(skip)]
    imported: std::collections::HashSet<String>,
    /// Circular import guard
    #[serde(skip)]
    import_guard: crate::module_resolver::ImportGuard,
    /// Canonical resolver - used for every import
    #[serde(skip)]
    resolver: crate::module_resolver::ModuleResolver,
    /// Cached compiled modules (path -> {hash, bytecode})
    #[serde(skip)]
    module_cache: std::collections::HashMap<String, CachedModule>,
    /// Cached module exports (path -> exports map)
    #[serde(skip)]
    module_exports: std::collections::HashMap<String, IfaValue>,
    /// Cached module bytecode by logical import key.
    #[serde(skip)]
    module_bytecode: std::collections::HashMap<String, Bytecode>,
    /// Persistent module globals by logical import key.
    #[serde(skip)]
    module_globals: std::collections::HashMap<String, std::collections::HashMap<String, IfaValue>>,
    /// Current file being executed (for relative imports)
    #[serde(skip)]
    current_file: Option<std::path::PathBuf>,

    /// Pending finally continuation (§12.4).

    /// Set by `Return`/`Throw` when they are pre-empted by a finally block.
    /// Cleared and executed by `FinallyEnd`.
    #[serde(skip)]
    pending_finally: Option<FinallyResumption>,
}

#[derive(Clone)]
struct CachedModule {
    hash: u64,
    bytecode: Bytecode,
}

#[derive(Clone)]
struct Task {
    func: IfaValue,
    args: Vec<IfaValue>,
    future: ifa_types::value_union::FutureCell,
    state: TaskState,
    started: bool,
    base_depth: usize,
}

#[derive(Clone, Default)]
struct TaskState {
    stack: Vec<IfaValue>,
    frames: Vec<CallFrame>,
    ip: usize,
    halted: bool,
    recovery_stack: Vec<RecoveryFrame>,
}

impl IfaVM {
    fn error_to_catch_value(error: &IfaError) -> IfaValue {
        error
            .user_value()
            .cloned()
            .unwrap_or_else(|| IfaValue::str(error.to_string()))
    }

    /// Get a global variable by name
    pub fn get_global(&self, name: &str) -> Option<&IfaValue> {
        self.globals.get(name)
    }

    /// Set or replace a global variable.
    pub fn set_global(&mut self, name: impl Into<String>, value: IfaValue) {
        self.globals.insert(name.into(), value);
    }

    /// Create new VM
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
        IfaVM {
            stack: Vec::new(),
            frames: Vec::new(),
            ip: 0,
            globals: std::collections::HashMap::new(),
            opon: Opon::create_default(),
            stack_limit: None,
            frame_limit: None,
            opon_size: crate::bytecode::OponSize::Ailopin,
            registry: None,
            halted: false,
            ticks: 0,
            recovery_stack: Vec::with_capacity(32),
            ikin: Ikin::new(),
            task_queue: VecDeque::new(),
            imported: std::collections::HashSet::new(),
            import_guard: crate::module_resolver::ImportGuard::new(),
            module_cache: std::collections::HashMap::new(),
            module_exports: std::collections::HashMap::new(),
            module_bytecode: std::collections::HashMap::new(),
            module_globals: std::collections::HashMap::new(),
            resolver: crate::module_resolver::ModuleResolver::new(module_paths),
            current_file: None,
            pending_finally: None,
        }

    }

    /// Attach a function registry (Standard Library)
    pub fn with_registry(mut self, registry: Box<dyn OduRegistry>) -> Self {
        self.registry = Some(registry);
        self
    }

    /// Create VM with custom Opon size
    pub fn with_opon(opon: Opon) -> Self {
        let mut module_paths = Vec::new();
        if let Ok(cwd) = std::env::current_dir() {
            module_paths.push(cwd);
        }
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                module_paths.push(dir.join("lib"));
            }
        }
        IfaVM {
            stack: Vec::new(),
            frames: Vec::new(),
            ip: 0,
            globals: std::collections::HashMap::new(),
            opon,
            stack_limit: None,
            frame_limit: None,
            opon_size: crate::bytecode::OponSize::Ailopin,
            registry: None,
            halted: false,
            ticks: 0,
            recovery_stack: Vec::with_capacity(32),
            ikin: Ikin::new(),
            task_queue: VecDeque::new(),
            imported: std::collections::HashSet::new(),
            import_guard: crate::module_resolver::ImportGuard::new(),
            module_cache: std::collections::HashMap::new(),
            module_exports: std::collections::HashMap::new(),
            module_bytecode: std::collections::HashMap::new(),
            module_globals: std::collections::HashMap::new(),
            resolver: crate::module_resolver::ModuleResolver::new(module_paths),
            current_file: None,
            pending_finally: None,
        }

    }

    /// Create VM with custom file path (for module resolution)
    pub fn with_file(file: impl AsRef<std::path::Path>) -> Self {
        let mut vm = Self::new();
        let path = file.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            vm.resolver.search_paths.insert(0, parent.to_path_buf());
        }
        vm.current_file = Some(path);
        vm
    }


    // =========================================================================
    // PERSISTENT STATE (SNAPSHOTS)
    // =========================================================================

    /// Create a binary snapshot of the VM state.
    /// Requires the original Bytecode to stamp the snapshot with an execution hash.
    pub fn snapshot(&self, bytecode: &Bytecode) -> IfaResult<Vec<u8>> {
        bincode::serialize(&(bytecode.hash(), self))
            .map_err(|e| IfaError::Custom(format!("Snapshot failed: {}", e)))
    }

    /// Create a JSON snapshot of the VM state (Inspection only)
    pub fn snapshot_json(&self) -> IfaResult<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| IfaError::Custom(format!("JSON snapshot failed: {}", e)))
    }

    /// Resume VM execution from a binary snapshot.
    /// The `bytecode` provided MUST exactly match the bytecode that was executing when snapshotted.
    pub fn resume(snapshot: &[u8], bytecode: &Bytecode) -> IfaResult<Self> {
        let (saved_hash, vm): (u64, IfaVM) = bincode::deserialize(snapshot)
            .map_err(|e| IfaError::Custom(format!("Corrupted snapshot: {}", e)))?;

        if saved_hash != bytecode.hash() {
            return Err(IfaError::Custom(
                "InvalidSnapshot: The bytecode provided does not match the active bytecode at the time of the snapshot. Resuming would cause a VM segfault.".to_string()
            ));
        }

        Ok(vm)
    }

    // =========================================================================
    // STACK OPERATIONS
    // =========================================================================

    /// Push value onto stack
    pub fn push(&mut self, value: IfaValue) -> IfaResult<()> {
        if let Some(limit) = self.stack_limit {
            if self.stack.len() >= limit {
                return Err(IfaError::StackOverflow {
                    limit,
                    directive: self.opon_size,
                });
            }
        }
        self.stack.push(value);
        Ok(())
    }

    /// Push CallFrame onto execution stack
    pub fn push_frame(&mut self, frame: CallFrame) -> IfaResult<()> {
        if let Some(limit) = self.frame_limit {
            if self.frames.len() >= limit {
                return Err(IfaError::StackOverflow {
                    limit,
                    directive: self.opon_size,
                });
            }
        }
        self.frames.push(frame);
        Ok(())
    }

    /// Pop value from stack
    pub fn pop(&mut self) -> IfaResult<IfaValue> {
        self.stack.pop().ok_or(IfaError::StackUnderflow)
    }

    /// Peek at top of stack
    pub fn peek(&self) -> IfaResult<&IfaValue> {
        self.stack.last().ok_or(IfaError::StackUnderflow)
    }

    /// Pop an integer from the stack

    // =========================================================================
    // BYTECODE EXECUTION
    // =========================================================================

    /// Execute bytecode
    pub fn execute(&mut self, bytecode: &Bytecode) -> IfaResult<IfaValue> {
        self.set_current_file_from_source(&bytecode.source_name);
        self.ip = 0;
        self.halted = false;
        self.task_queue.clear();

        // Phase 1: Consult the Nuts (Load Constants)
        self.ikin.load_from_bytecode(bytecode);

        let (stack_cap, frame_cap) = bytecode.opon_size.limits();
        self.stack_limit = stack_cap;
        self.frame_limit = frame_cap;
        self.opon_size = bytecode.opon_size;

        if let Some(cap) = stack_cap {
            if self.stack.capacity() < cap {
                self.stack.reserve(cap - self.stack.len());
            }
        }

        self.resume_execution(bytecode)
    }

    fn set_current_file_from_source(&mut self, source_name: &str) {
        let path = std::path::Path::new(source_name);
        if path.exists() {
            if let Some(parent) = path.parent() {
                if !self.resolver.search_paths.iter().any(|p| p == parent) {
                    self.resolver.search_paths.insert(0, parent.to_path_buf());
                }
            }
            self.current_file = Some(path.to_path_buf());
        }
    }


    fn hash_source(source: &str) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        source.hash(&mut hasher);
        hasher.finish()
    }

    fn hash_bytes(bytes: &[u8]) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        bytes.hash(&mut hasher);
        hasher.finish()
    }

    fn module_fn_marker(module_key: &str, name: &str) -> String {
        format!("__module_fn__:{}:{}", module_key, name)
    }

    fn export_value_for_import(module_key: &str, name: &str, value: &IfaValue) -> IfaValue {
        match value {
            IfaValue::Fn(_) | IfaValue::Closure(_) => {
                IfaValue::str(Self::module_fn_marker(module_key, name))
            }
            other => other.clone(),
        }
    }

    fn invoke_module_function(
        &mut self,
        module_key: &str,
        function_name: &str,
        args: Vec<IfaValue>,
    ) -> IfaResult<IfaValue> {
        let bytecode = self
            .module_bytecode
            .get(module_key)
            .cloned()
            .ok_or_else(|| {
                IfaError::Runtime(format!("Module bytecode missing for '{}'", module_key))
            })?;
        let module_globals = self
            .module_globals
            .get(module_key)
            .cloned()
            .ok_or_else(|| {
                IfaError::Runtime(format!("Module state missing for '{}'", module_key))
            })?;
        let func = module_globals
            .get(function_name)
            .cloned()
            .ok_or_else(|| IfaError::UndefinedVariable(function_name.to_string()))?;

        let saved_ip = self.ip;
        let saved_halted = self.halted;
        let saved_stack_len = self.stack.len();
        let saved_frames_len = self.frames.len();
        let saved_recovery_len = self.recovery_stack.len();
        let saved_limits = (self.stack_limit, self.frame_limit, self.opon_size);
        let saved_ikin = self.ikin.clone();
        let saved_globals = std::mem::replace(&mut self.globals, module_globals);

        let result = (|| {
            self.halted = false;
            self.ikin.load_from_bytecode(&bytecode);
            let (stack_cap, frame_cap) = bytecode.opon_size.limits();
            self.stack_limit = stack_cap;
            self.frame_limit = frame_cap;
            self.opon_size = bytecode.opon_size;
            let return_addr = bytecode.code.len();
            match func {
                IfaValue::Fn(data) => {
                    if args.len() != data.arity as usize {
                        return Err(IfaError::ArityMismatch {
                            expected: data.arity as usize,
                            got: args.len(),
                        });
                    }
                    self.push_frame(CallFrame::new(return_addr, self.stack.len(), None, false))?;
                    for arg in args {
                        self.push(arg)?;
                    }
                    self.ip = data.start_ip;
                }
                IfaValue::Closure(closure) => {
                    let data = &closure.fn_data;
                    if args.len() != data.arity as usize {
                        return Err(IfaError::ArityMismatch {
                            expected: data.arity as usize,
                            got: args.len(),
                        });
                    }
                    self.push_frame(CallFrame::new(
                        return_addr,
                        self.stack.len(),
                        Some(closure.env.clone()),
                        false,
                    ))?;
                    for arg in args {
                        self.push(arg)?;
                    }
                    self.ip = data.start_ip;
                }
                other => {
                    return Err(IfaError::TypeError {
                        expected: "Function".into(),
                        got: other.type_name().into(),
                    });
                }
            }
            self.resume_execution(&bytecode)
        })();

        let updated_module_globals = self.globals.clone();
        self.globals = saved_globals;
        self.module_globals
            .insert(module_key.to_string(), updated_module_globals);
        self.stack.truncate(saved_stack_len);
        self.frames.truncate(saved_frames_len);
        self.recovery_stack.truncate(saved_recovery_len);
        self.stack_limit = saved_limits.0;
        self.frame_limit = saved_limits.1;
        self.opon_size = saved_limits.2;
        self.ikin = saved_ikin;
        self.ip = saved_ip;
        self.halted = saved_halted;

        result
    }

    fn import_module(&mut self, path: &str) -> IfaResult<IfaValue> {
        let module_key = path.replace('\\', "/");

        if module_key.starts_with("std.") || module_key.starts_with("std/") {
            if let Some(registry) = &self.registry {
                return registry.import(&module_key);
            }
            return Err(IfaError::RegistryNotAttached(
                "Standard library registry not attached".into(),
            ));
        }

        // Circular import detection using unified guard
        self.import_guard.enter(&module_key)?;

        let resolved = self.resolver.resolve(&module_key)?;
        let file_path = resolved.path;
        let is_ifab = resolved.is_binary;

        let cache_key = file_path.to_string_lossy().to_string();
        let cached_hash_before = self.module_cache.get(&cache_key).map(|cached| cached.hash);

        let (bytecode, export_names, content_hash) = if is_ifab {
            let bytes = std::fs::read(&file_path).map_err(|e| {
                IfaError::IoError(format!("Cannot read module '{}': {}", module_key, e))
            })?;
            let hash = Self::hash_bytes(&bytes);
            let bc = Bytecode::from_bytes(&bytes)?;
            let exports = bc.exports.clone();
            (bc, exports, hash)
        } else {
            let source = std::fs::read_to_string(&file_path).map_err(|e| {
                IfaError::IoError(format!("Cannot read module '{}': {}", module_key, e))
            })?;
            let source_hash = Self::hash_source(&source);
            let program = crate::parser::parse(&source).map_err(|e| {
                IfaError::Runtime(format!("Parse error in module '{}': {}", module_key, e))
            })?;
            let export_names = collect_exports_vm(&program);

            let bytecode = if let Some(cached) = self.module_cache.get(&cache_key) {
                if cached.hash == source_hash {
                    cached.bytecode.clone()
                } else {
                    let compiler =
                        crate::compiler::Compiler::new(file_path.to_string_lossy().as_ref());
                    let bytecode = compiler.compile(&program)?;
                    self.module_cache.insert(
                        cache_key.clone(),
                        CachedModule {
                            hash: source_hash,
                            bytecode: bytecode.clone(),
                        },
                    );
                    bytecode
                }
            } else {
                let compiler = crate::compiler::Compiler::new(file_path.to_string_lossy().as_ref());
                let bytecode = compiler.compile(&program)?;
                self.module_cache.insert(
                    cache_key.clone(),
                    CachedModule {
                        hash: source_hash,
                        bytecode: bytecode.clone(),
                    },
                );
                bytecode
            };
            (bytecode, export_names, source_hash)
        };

        if self.imported.contains(&module_key) && cached_hash_before == Some(content_hash) {
            if let Some(exports) = self.module_exports.get(&module_key) {
                self.import_guard.exit(&module_key);
                return Ok(exports.clone());
            }
        }

        let prev_file = self.current_file.take();
        let prev_paths = self.resolver.search_paths.clone();
        if let Some(parent) = file_path.parent() {
            if !self.resolver.search_paths.iter().any(|p| p == parent) {
                self.resolver.search_paths.insert(0, parent.to_path_buf());
            }
        }
        self.current_file = Some(file_path.clone());

        let prev_globals = std::mem::take(&mut self.globals);
        self.globals = std::collections::HashMap::new();
        let result = self.execute_module(&bytecode);

        let mut exports = std::collections::HashMap::new();
        if result.is_ok() {
            for name in export_names {
                if let Some(val) = self.globals.get(&name).cloned() {
                    let export_val = Self::export_value_for_import(&module_key, &name, &val);
                    exports.insert(name, export_val);
                }
            }
        }
        let exports_val = IfaValue::map(exports);
        let module_globals = self.globals.clone();

        self.current_file = prev_file;
        self.resolver.search_paths = prev_paths;
        self.globals = prev_globals;

        self.import_guard.exit(&module_key);
        if result.is_ok() {
            self.imported.insert(module_key.clone());
            self.module_exports
                .insert(module_key.clone(), exports_val.clone());
            self.module_bytecode
                .insert(module_key.clone(), bytecode.clone());
            self.module_globals
                .insert(module_key.clone(), module_globals);
            self.module_cache.insert(
                cache_key,
                CachedModule {
                    hash: content_hash,
                    bytecode: bytecode.clone(),
                },
            );
        }
        result.map(|_| exports_val)
    }


    fn execute_module(&mut self, bytecode: &Bytecode) -> IfaResult<()> {
        let saved_ip = self.ip;
        let saved_halted = self.halted;
        let saved_stack_len = self.stack.len();
        let saved_frames_len = self.frames.len();
        let saved_recovery_len = self.recovery_stack.len();
        let saved_limits = (self.stack_limit, self.frame_limit, self.opon_size);
        let saved_ikin = self.ikin.clone();

        let result = (|| {
            self.ip = 0;
            self.halted = false;
            self.ikin.load_from_bytecode(bytecode);
            let (stack_cap, frame_cap) = bytecode.opon_size.limits();
            self.stack_limit = stack_cap;
            self.frame_limit = frame_cap;
            self.opon_size = bytecode.opon_size;

            self.resume_execution(bytecode).map(|_| ())
        })();

        self.stack.truncate(saved_stack_len);
        self.frames.truncate(saved_frames_len);
        self.recovery_stack.truncate(saved_recovery_len);
        self.stack_limit = saved_limits.0;
        self.frame_limit = saved_limits.1;
        self.opon_size = saved_limits.2;
        self.ikin = saved_ikin;
        self.ip = saved_ip;
        self.halted = saved_halted;

        result
    }

    /// Resume bytecode execution from current instruction pointer
    pub fn resume_execution(&mut self, bytecode: &Bytecode) -> IfaResult<IfaValue> {
        self.halted = false;

        while !self.halted && self.ip < bytecode.code.len() {
            if let Err(e) = self.step(bytecode) {
                if matches!(e, IfaError::Yielded) {
                    return Err(e);
                }

                // The Shield of Ọ̀kànràn: Attempt recovery before crashing
                // Pass reference to avoid cloning unless we actually recover
                if self.attempt_recovery(&e)? {
                    continue;
                }
                return Err(e);
            }
        }

        // Return top of stack or Null
        Ok(self.stack.pop().unwrap_or(IfaValue::null()))
    }

    fn swap_task_state(&mut self, task: &mut Task) {
        std::mem::swap(&mut self.stack, &mut task.state.stack);
        std::mem::swap(&mut self.frames, &mut task.state.frames);
        std::mem::swap(&mut self.ip, &mut task.state.ip);
        std::mem::swap(&mut self.halted, &mut task.state.halted);
        std::mem::swap(&mut self.recovery_stack, &mut task.state.recovery_stack);
    }

    fn call_value_task(&mut self, func: IfaValue, args: Vec<IfaValue>) -> IfaResult<()> {
        match func {
            IfaValue::Fn(data) => {
                if args.len() != data.arity as usize {
                    return Err(IfaError::ArityMismatch {
                        expected: data.arity as usize,
                        got: args.len(),
                    });
                }
                self.push_frame(CallFrame::new(self.ip, self.stack.len(), None, false))?;
                for arg in args {
                    self.push(arg)?;
                }
                self.ip = data.start_ip;
            }
            IfaValue::Closure(closure) => {
                let data = &closure.fn_data;
                if args.len() != data.arity as usize {
                    return Err(IfaError::ArityMismatch {
                        expected: data.arity as usize,
                        got: args.len(),
                    });
                }
                self.push_frame(CallFrame::new(
                    self.ip,
                    self.stack.len(),
                    Some(closure.env.clone()),
                    false,
                ))?;
                for arg in args {
                    self.push(arg)?;
                }
                self.ip = data.start_ip;
            }
            other => {
                return Err(IfaError::TypeError {
                    expected: "Function".into(),
                    got: other.type_name().into(),
                });
            }
        }
        Ok(())
    }

    fn run_task_slice(
        &mut self,
        task: &mut Task,
        bytecode: &Bytecode,
    ) -> IfaResult<Option<IfaValue>> {
        self.swap_task_state(task);

        if !task.started {
            task.base_depth = self.frames.len();
            self.call_value_task(task.func.clone(), task.args.clone())?;
            task.started = true;
        }

        loop {
            if self.frames.len() == task.base_depth {
                let result = self.stack.pop().unwrap_or(IfaValue::null());
                self.swap_task_state(task);
                return Ok(Some(result));
            }

            match self.step(bytecode) {
                Ok(()) => {}
                Err(IfaError::Yielded) => {
                    self.swap_task_state(task);
                    return Ok(None);
                }
                Err(e) => {
                    self.swap_task_state(task);
                    return Err(e);
                }
            }
        }
    }

    fn poll_one_task(&mut self, bytecode: &Bytecode) -> IfaResult<bool> {
        let mut task = match self.task_queue.pop_front() {
            Some(t) => t,
            None => return Ok(false),
        };
        let maybe_result = self.run_task_slice(&mut task, bytecode)?;
        if let Some(result) = maybe_result {
            let mut state = task
                .future
                .lock()
                .map_err(|_| IfaError::Runtime("Future lock poisoned".into()))?;
            *state = FutureState::Ready(result);
        } else {
            self.task_queue.push_back(task);
        }
        Ok(true)
    }

    pub(crate) fn await_future(
        &mut self,
        cell: &ifa_types::value_union::FutureCell,
        bytecode: &Bytecode,
    ) -> IfaResult<IfaValue> {
        loop {
            let ready = {
                let state = cell
                    .lock()
                    .map_err(|_| IfaError::Runtime("Future lock poisoned".into()))?;
                match &*state {
                    FutureState::Ready(v) => Some(v.clone()),
                    FutureState::Pending => None,
                }
            };
            if let Some(v) = ready {
                return Ok(v);
            }
            if !self.poll_one_task(bytecode)? {
                return Err(IfaError::Runtime(
                    "Future pending with no runnable tasks".into(),
                ));
            }
        }
    }

    fn call_registry(
        &mut self,
        domain_id: u8,
        method_name: &str,
        args: Vec<IfaValue>,
        bytecode: &Bytecode,
    ) -> IfaResult<IfaValue> {
        let Some(registry) = self.registry.take() else {
            return Err(IfaError::RegistryNotAttached(method_name.to_string()));
        };
        let mut ctx = VmContext { vm: self, bytecode };
        let result = registry.call(domain_id, method_name, args, &mut ctx);
        self.registry = Some(registry);
        result
    }

    pub fn spawn_task(&mut self, func: IfaValue, args: Vec<IfaValue>) -> IfaResult<IfaValue> {
        let cell = match IfaValue::future_pending() {
            IfaValue::Future(cell) => cell,
            _ => unreachable!(),
        };
        let task = Task {
            func,
            args,
            future: cell.clone(),
            state: TaskState::default(),
            started: false,
            base_depth: 0,
        };
        self.task_queue.push_back(task);
        Ok(IfaValue::Future(cell))
    }

    /// Attempt to recover from a runtime error using the Shield of Ọ̀kànràn
    fn attempt_recovery(&mut self, error: &IfaError) -> IfaResult<bool> {
        if let Some(frame) = self.recovery_stack.pop() {
            // If this frame already consumed its catch arm, the only remaining
            // obligation is to run its finally block before propagating outward.
            if !frame.can_catch {
                if let Some(finally_ip) = frame.finally_ip {
                    if self.stack.len() > frame.stack_depth {
                        self.stack.truncate(frame.stack_depth);
                    }
                    if self.frames.len() > frame.call_depth {
                        self.frames.truncate(frame.call_depth);
                    }
                    self.pending_finally = Some(FinallyResumption::Propagate {
                        error: error.clone(),
                    });
                    self.ip = finally_ip;
                    return Ok(true);
                }
                return Ok(false);
            }

            // 1. Restore stacks
            if self.stack.len() > frame.stack_depth {
                self.stack.truncate(frame.stack_depth); // Drop triggers Ebo cleanup
            }
            if self.frames.len() > frame.call_depth {
                self.frames.truncate(frame.call_depth);
            }

            // 2. Convert the trapped control-flow error into the catch binding value.
            // User-thrown values must arrive unchanged; VM/runtime errors still degrade
            // to their display string until structured VM errors are introduced.
            self.push(Self::error_to_catch_value(error))?;

            // Catch has consumed the exception arm. If a finally exists, keep a
            // sentinel frame so return/throw/error from the catch still runs it.
            if frame.finally_ip.is_some() {
                self.recovery_stack.push(RecoveryFrame {
                    stack_depth: frame.stack_depth,
                    call_depth: frame.call_depth,
                    catch_ip: frame.catch_ip,
                    finally_ip: frame.finally_ip,
                    can_catch: false,
                });
            }
            self.ip = frame.catch_ip;

            Ok(true) // Recovered
        } else {
            Ok(false) // No shield found, crash
        }
    }
    /// Execute single instruction (The Step of Iroke)
    fn step(&mut self, bytecode: &Bytecode) -> IfaResult<()> {
        let opcode = vm_iroke::tap(self, bytecode)?;

        match opcode {
            // Stack operations
            OpCode::PushNull => self.push(IfaValue::null())?,
            OpCode::PushTrue => self.push(IfaValue::bool(true))?,
            OpCode::PushFalse => self.push(IfaValue::bool(false))?,
            OpCode::PushList => self.push(IfaValue::list(Vec::new()))?,
            OpCode::PushMap => self.push(IfaValue::map(std::collections::HashMap::new()))?,

            OpCode::LoadUpvalue => {
                let slot = self.read_u16(bytecode)? as usize;
                let env = self
                    .frames
                    .last()
                    .and_then(|f| f.closure_env.clone())
                    .ok_or_else(|| {
                        IfaError::Runtime("No closure environment in current frame".into())
                    })?;

                let cell = env
                    .get(slot)
                    .cloned()
                    .ok_or_else(|| IfaError::UndefinedVariable(format!("<upvalue:{}>", slot)))?;

                let value = cell
                    .try_borrow()
                    .map_err(|_| IfaError::Runtime("Upvalue borrow failed".into()))?
                    .clone();
                self.push(value)?;
            }

            OpCode::StoreUpvalue => {
                let slot = self.read_u16(bytecode)? as usize;
                let value = self.pop()?;
                let env = self
                    .frames
                    .last()
                    .and_then(|f| f.closure_env.clone())
                    .ok_or_else(|| {
                        IfaError::Runtime("No closure environment in current frame".into())
                    })?;

                let cell = env
                    .get(slot)
                    .cloned()
                    .ok_or_else(|| IfaError::UndefinedVariable(format!("<upvalue:{}>", slot)))?;

                *cell
                    .try_borrow_mut()
                    .map_err(|_| IfaError::Runtime("Upvalue borrow failed".into()))? = value;
            }

            OpCode::PushFn => {
                // Read function metadata from bytecode
                let name_idx = self.read_u16(bytecode)? as usize;
                let name = bytecode
                    .strings
                    .get(name_idx)
                    .cloned()
                    .unwrap_or_else(|| format!("<fn#{}>", name_idx));

                // Read start_ip as u32 little-endian (4 bytes)
                let b0 = self.read_u8(bytecode)? as u32;
                let b1 = self.read_u8(bytecode)? as u32;
                let b2 = self.read_u8(bytecode)? as u32;
                let b3 = self.read_u8(bytecode)? as u32;
                let start_ip = (b0 | (b1 << 8) | (b2 << 16) | (b3 << 24)) as usize;

                let arity = self.read_u8(bytecode)?;
                let is_async = if bytecode.version >= 3 {
                    self.read_u8(bytecode)? != 0
                } else {
                    false
                };

                self.push(IfaValue::bytecode_fn(name, start_ip, arity, is_async))?;
            }

            OpCode::MakeClosure => {
                let capture_count = self.read_u8(bytecode)? as usize;

                let fn_template = self.pop()?;
                let fn_data = match fn_template {
                    IfaValue::Fn(data) => data,
                    _ => {
                        return Err(IfaError::TypeError {
                            expected: "Function template".into(),
                            got: fn_template.type_name().into(),
                        });
                    }
                };

                let base = self.frames.last().map(|f| f.base_ptr).unwrap_or(0);
                let parent_env = self.frames.last().and_then(|f| f.closure_env.clone());
                let mut env: Vec<UpvalueCell> = Vec::with_capacity(capture_count);

                for _ in 0..capture_count {
                    let kind = self.read_u8(bytecode)?;
                    let idx = self.read_u16(bytecode)? as usize;

                    match kind {
                        // Capture local slot
                        0 => {
                            let slot_index = base + idx;
                            let slot = self.stack.get(slot_index).cloned().ok_or_else(|| {
                                IfaError::UndefinedVariable(format!("<local:{}>", idx))
                            })?;

                            let cell = match slot {
                                IfaValue::Upvalue(cell) => cell,
                                value => {
                                    let cell: UpvalueCell = Rc::new(RefCell::new(value));
                                    // Box the local slot so future mutations share the cell.
                                    if slot_index < self.stack.len() {
                                        self.stack[slot_index] = IfaValue::Upvalue(cell.clone());
                                    }
                                    cell
                                }
                            };

                            env.push(cell);
                        }

                        // Capture upvalue slot from parent closure
                        1 => {
                            let parent_env = parent_env.clone().ok_or_else(|| {
                                IfaError::Runtime(
                                    "Attempted to capture upvalue without an enclosing closure"
                                        .into(),
                                )
                            })?;
                            let cell = parent_env.get(idx).cloned().ok_or_else(|| {
                                IfaError::UndefinedVariable(format!("<upvalue:{}>", idx))
                            })?;
                            env.push(cell);
                        }

                        _ => {
                            return Err(IfaError::Runtime(format!(
                                "MakeClosure: invalid capture kind {}",
                                kind
                            )));
                        }
                    }
                }

                self.push(IfaValue::Closure(Arc::new(ClosureData {
                    fn_data,
                    env: Arc::new(env),
                })))?;
            }

            OpCode::PushInt => {
                let value = self.read_i64(bytecode)?;
                self.push(IfaValue::int(value))?;
            }

            OpCode::PushFloat => {
                let value = self.read_f64(bytecode)?;
                self.push(IfaValue::float(value))?;
            }

            OpCode::PushStr => {
                let idx = self.read_u16(bytecode)? as usize;
                // Use Ikin for O(1) Arc access
                let arc = self.ikin.consult_string(idx).ok_or_else(|| {
                    IfaError::Custom("Invalid string constant index in Ikin".into())
                })?;
                self.push(IfaValue::Str(arc.clone()))?;
            }

            OpCode::Pop => {
                self.pop()?;
            }

            OpCode::Dup => {
                let value = self.peek()?.clone();
                self.push(value)?;
            }

            OpCode::Swap => {
                let len = self.stack.len();
                if len < 2 {
                    return Err(IfaError::StackUnderflow);
                }
                self.stack.swap(len - 1, len - 2);
            }

            // Arithmetic: Add is now PURE NUMERIC. Strings use OpCode::Concat.
            OpCode::Add => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (a.clone(), b.clone()) {
                    (IfaValue::Int(ia), IfaValue::Int(ib)) => {
                        // D6: Checked arithmetic — promote to Float on overflow
                        match ia.checked_add(ib) {
                            Some(result) => self.push(IfaValue::int(result))?,
                            None => self.push(IfaValue::float(ia as f64 + ib as f64))?,
                        }
                    }
                    (IfaValue::Float(fa), IfaValue::Float(fb)) => {
                        self.push(IfaValue::float(fa + fb))?
                    }
                    (IfaValue::Int(ia), IfaValue::Float(fb)) => {
                        self.push(IfaValue::float(ia as f64 + fb))?
                    }
                    (IfaValue::Float(fa), IfaValue::Int(ib)) => {
                        self.push(IfaValue::float(fa + ib as f64))?
                    }
                    _ => {
                        return Err(IfaError::TypeError {
                            expected: "Int or Float (use += for strings)".into(),
                            got: format!("{} + {}", a.type_name(), b.type_name()),
                        });
                    }
                }
            }

            OpCode::Concat => {
                let rhs = self.pop()?;
                let lhs = self.pop()?;
                match (lhs, rhs) {
                    (IfaValue::Str(lhs), IfaValue::Str(rhs)) => {
                        let mut s = String::with_capacity(lhs.len() + rhs.len());
                        s.push_str(&lhs);
                        s.push_str(&rhs);
                        self.push(IfaValue::str(s))?;
                    }
                    (lhs, rhs) => {
                        return Err(IfaError::TypeError {
                            expected: "Str + Str".into(),
                            got: format!("{} ++ {}", lhs.type_name(), rhs.type_name()),
                        });
                    }
                }
            }

            OpCode::Sub => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (a, b) {
                    (IfaValue::Int(ia), IfaValue::Int(ib)) => {
                        // D6: Checked arithmetic — promote to Float on overflow
                        match ia.checked_sub(ib) {
                            Some(result) => self.push(IfaValue::int(result))?,
                            None => self.push(IfaValue::float(ia as f64 - ib as f64))?,
                        }
                    }
                    (IfaValue::Float(fa), IfaValue::Float(fb)) => {
                        self.push(IfaValue::float(fa - fb))?
                    }
                    _ => {
                        return Err(IfaError::TypeError {
                            expected: "Int/Float".into(),
                            got: "Mismatch".into(),
                        });
                    }
                }
            }

            OpCode::Mul => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (a, b) {
                    (IfaValue::Int(ia), IfaValue::Int(ib)) => {
                        // D6: Checked arithmetic — promote to Float on overflow
                        match ia.checked_mul(ib) {
                            Some(result) => self.push(IfaValue::int(result))?,
                            None => self.push(IfaValue::float(ia as f64 * ib as f64))?,
                        }
                    }
                    (IfaValue::Float(fa), IfaValue::Float(fb)) => {
                        self.push(IfaValue::float(fa * fb))?
                    }
                    _ => {
                        return Err(IfaError::TypeError {
                            expected: "Int/Float".into(),
                            got: "Mismatch".into(),
                        });
                    }
                }
            }

            OpCode::Div => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (a, b) {
                    (IfaValue::Int(ia), IfaValue::Int(ib)) => {
                        if ib == 0 {
                            return Err(IfaError::DivisionByZero("Cannot divide by zero".into()));
                        }
                        // R1: Int/Int division truncates toward zero per spec §4.4
                        self.push(IfaValue::int(ia / ib))?
                    }
                    (IfaValue::Float(fa), IfaValue::Float(fb)) => {
                        // R2: Float 0.0/0.0 produces NaN per spec §4.5, never an error
                        self.push(IfaValue::float(fa / fb))?
                    }
                    _ => {
                        return Err(IfaError::TypeError {
                            expected: "Int/Float".into(),
                            got: "Mismatch".into(),
                        });
                    }
                }
            }

            // Variables
            OpCode::LoadLocal => {
                let idx = self.read_u16(bytecode)? as usize;
                let base = self.frames.last().map(|f| f.base_ptr).unwrap_or(0);

                let slot = self
                    .stack
                    .get(base + idx)
                    .cloned()
                    .ok_or_else(|| IfaError::UndefinedVariable(format!("<local:{}>", idx)))?;

                match slot {
                    IfaValue::Upvalue(cell) => {
                        let value = cell
                            .try_borrow()
                            .map_err(|_| IfaError::Runtime("Upvalue borrow failed".into()))?
                            .clone();
                        self.push(value)?;
                    }
                    value => self.push(value)?,
                }
            }

            OpCode::StoreLocal => {
                let idx = self.read_u16(bytecode)? as usize;
                let value = self.pop()?;
                let base = self.frames.last().map(|f| f.base_ptr).unwrap_or(0);
                if base + idx >= self.stack.len() {
                    return Err(IfaError::UndefinedVariable(format!("<local:{}>", idx)));
                }
                match self.stack[base + idx].clone() {
                    IfaValue::Upvalue(cell) => {
                        *cell
                            .try_borrow_mut()
                            .map_err(|_| IfaError::Runtime("Upvalue borrow failed".into()))? =
                            value;
                    }
                    _ => self.stack[base + idx] = value,
                }
            }

            OpCode::LoadGlobal => {
                let idx = self.read_u16(bytecode)? as usize;
                let name = bytecode
                    .strings
                    .get(idx)
                    .cloned()
                    .ok_or(IfaError::Custom("Invalid global name index".into()))?;
                // D14: Error on undefined globals instead of silent Null
                let value = self
                    .globals
                    .get(&name)
                    .cloned()
                    .ok_or_else(|| IfaError::UndefinedVariable(name.clone()))?;
                self.push(value)?;
            }

            OpCode::StoreGlobal => {
                let idx = self.read_u16(bytecode)? as usize;
                let name = bytecode
                    .strings
                    .get(idx)
                    .cloned()
                    .ok_or(IfaError::Custom("Invalid global name index".into()))?;
                let value = self.pop()?;
                self.globals.insert(name, value);
            }

            // Control flow
            OpCode::Jump => {
                // D2: Use u32 offset per ifa-bytecode operand_bytes()
                let offset = self.read_u32(bytecode)?;
                self.ip = offset as usize;
            }

            OpCode::JumpIfFalse => {
                // D2: Use u32 offset per ifa-bytecode operand_bytes()
                let offset = self.read_u32(bytecode)?;
                let cond = self.pop()?;
                if !cond.is_truthy() {
                    self.ip = offset as usize;
                }
            }

            OpCode::JumpIfTrue => {
                // D2: Use u32 offset per ifa-bytecode operand_bytes()
                let offset = self.read_u32(bytecode)?;
                let cond = self.pop()?;
                if cond.is_truthy() {
                    self.ip = offset as usize;
                }
            }

            // Functions
            OpCode::Call => {
                let arg_count = self.read_u8(bytecode)? as usize;

                // 1. Pop arguments
                let mut args = Vec::with_capacity(arg_count);
                for _ in 0..arg_count {
                    args.push(self.pop()?);
                }
                // Args are popped in reverse order (stack LIFO), so reverse them to get (arg1, arg2...)
                args.reverse();

                // 2. Pop function
                let func = self.pop()?;

                match func {
                    IfaValue::Fn(data) => {
                        if args.len() != data.arity as usize {
                            return Err(IfaError::ArityMismatch {
                                expected: data.arity as usize,
                                got: args.len(),
                            });
                        }
                        if data.is_async {
                            let future = self.spawn_task(IfaValue::Fn(data.clone()), args)?;
                            self.push(future)?;
                        } else {
                            self.push_frame(CallFrame::new(
                                self.ip,
                                self.stack.len(),
                                None,
                                data.is_async,
                            ))?;

                            for arg in args {
                                self.push(arg)?;
                            }

                            self.ip = data.start_ip;
                        }
                    }
                    IfaValue::Closure(closure) => {
                        let data = &closure.fn_data;
                        if args.len() != data.arity as usize {
                            return Err(IfaError::ArityMismatch {
                                expected: data.arity as usize,
                                got: args.len(),
                            });
                        }
                        if data.is_async {
                            let future =
                                self.spawn_task(IfaValue::Closure(closure.clone()), args)?;
                            self.push(future)?;
                        } else {
                            self.push_frame(CallFrame::new(
                                self.ip,
                                self.stack.len(),
                                Some(closure.env.clone()),
                                data.is_async,
                            ))?;

                            for arg in args {
                                self.push(arg)?;
                            }

                            self.ip = data.start_ip;
                        }
                    }
                    IfaValue::Str(s) => {
                        if let Some((domain_id, method)) = parse_odu_fn_marker(&s) {
                            let result = self.call_registry(domain_id, &method, args, bytecode)?;
                            self.push(result)?;
                        } else if let Some((module_key, function_name)) = parse_module_fn_marker(&s)
                        {
                            let result =
                                self.invoke_module_function(&module_key, &function_name, args)?;
                            self.push(result)?;
                        } else {
                            return Err(IfaError::TypeError {
                                expected: "Function".into(),
                                got: "Str".into(),
                            });
                        }
                    }
                    _ => {
                        return Err(IfaError::TypeError {
                            expected: "Function".into(),
                            got: func.type_name().into(),
                        });
                    }
                }
            }

            OpCode::TailCall => {
                let arg_count = self.read_u8(bytecode)? as usize;

                // 1. Pop arguments
                let mut args = Vec::with_capacity(arg_count);
                for _ in 0..arg_count {
                    args.push(self.pop()?);
                }
                args.reverse();

                // 2. Pop function
                let func = self.pop()?;

                let (start_ip, arity, env, async_return) = match func {
                    IfaValue::Fn(ref data) => {
                        (data.start_ip, data.arity as usize, None, data.is_async)
                    }
                    IfaValue::Closure(ref closure) => (
                        closure.fn_data.start_ip,
                        closure.fn_data.arity as usize,
                        Some(closure.env.clone()),
                        closure.fn_data.is_async,
                    ),
                    _ => {
                        return Err(IfaError::TypeError {
                            expected: "Function".into(),
                            got: func.type_name().into(),
                        });
                    }
                };

                if args.len() != arity {
                    return Err(IfaError::ArityMismatch {
                        expected: arity,
                        got: args.len(),
                    });
                }

                if async_return {
                    let future = self.spawn_task(func, args)?;
                    if let Some(frame) = self.frames.pop() {
                        if self.stack.len() > frame.base_ptr {
                            self.stack.truncate(frame.base_ptr);
                        }
                        self.push(future)?;
                        self.ip = frame.return_addr;
                    } else {
                        self.push(future)?;
                        self.halted = true;
                    }
                    return Ok(());
                }

                if let Some(frame) = self.frames.last_mut() {
                    if self.stack.len() > frame.base_ptr {
                        self.stack.truncate(frame.base_ptr);
                    }
                    frame.local_count = 0;
                    frame.closure_env = env;
                    frame.async_return = async_return;

                    for arg in args {
                        self.push(arg)?;
                    }
                } else {
                    self.push_frame(CallFrame::new(self.ip, self.stack.len(), env, async_return))?;
                    for arg in args {
                        self.push(arg)?;
                    }
                }

                self.ip = start_ip;
            }

            OpCode::Return => {
                // §12.4: If there is a pending finally block on the nearest recovery frame,
                // stash the continuation and divert to it. FinallyEnd will complete the return.
                if let Some(finally_ip) = self.recovery_stack.last().and_then(|f| f.finally_ip) {
                    let return_value = self.pop().unwrap_or(IfaValue::null());
                    // Pop the recovery frame that owns this finally block — cleanup is about to run.
                    self.recovery_stack.pop();
                    self.pending_finally = Some(FinallyResumption::Return { return_value });
                    self.ip = finally_ip;
                    return Ok(());
                }

                if let Some(frame) = self.frames.pop() {
                    let return_value = self.pop().unwrap_or(IfaValue::null());
                    if self.stack.len() > frame.base_ptr {
                        self.stack.truncate(frame.base_ptr);
                    }
                    if frame.async_return {
                        self.push(IfaValue::future_ready(return_value))?;
                    } else {
                        self.push(return_value)?;
                    }
                    self.ip = frame.return_addr;
                } else {
                    self.halted = true;
                }
            }

            OpCode::Await => {
                let value = self.pop()?;
                match value {
                    IfaValue::Future(cell) => {
                        let v = self.await_future(&cell, bytecode)?;
                        self.push(v)?;
                    }
                    other => {
                        return Err(IfaError::TypeError {
                            expected: "Future".into(),
                            got: other.type_name().into(),
                        });
                    }
                }
            }

            OpCode::CallOdu => {
                let domain_id = self.read_u8(bytecode)?;
                let idx = self.read_u16(bytecode)? as usize;
                let method_name = bytecode.strings.get(idx).cloned().ok_or_else(|| {
                    IfaError::Custom(format!("CallOdu: invalid string pool index {}", idx))
                })?;

                let arity = self.read_u8(bytecode)?;

                let mut args = Vec::with_capacity(arity as usize);
                for _ in 0..arity {
                    args.push(self.pop()?);
                }
                args.reverse();

                let result = self.call_registry(domain_id, &method_name, args, bytecode)?;
                self.push(result)?;
            }

            OpCode::CallMethod => {
                let method_idx = self.read_u16(bytecode)?;
                let arg_count = self.read_u8(bytecode)?;

                let mut args = Vec::with_capacity(arg_count as usize);
                for _ in 0..arg_count {
                    args.push(self.pop()?);
                }
                args.reverse();

                let object = self.pop()?;

                // Resolve method name from string pool
                let method_name = bytecode
                    .strings
                    .get(method_idx as usize)
                    .cloned()
                    .ok_or_else(|| {
                        IfaError::Custom(format!("Invalid method name index: {}", method_idx))
                    })?;

                if let IfaValue::Str(s) = &object {
                    if let Some(domain_id) = parse_odu_mod_marker(s) {
                        let result = self.call_registry(domain_id, &method_name, args, bytecode)?;
                        self.push(result)?;
                        return Ok(());
                    }
                }

                // Try map-based dispatch first (kiri pattern: app = {"main": fn, ...}; app.main())
                match object {
                    IfaValue::Map(map) => {
                        let key: std::sync::Arc<str> = std::sync::Arc::from(method_name.as_str());
                        if let Some(func) = map.get(&key) {
                            match func {
                                IfaValue::Fn(data) => {
                                    if args.len() != data.arity as usize {
                                        return Err(IfaError::ArityMismatch {
                                            expected: data.arity as usize,
                                            got: args.len(),
                                        });
                                    }
                                    if data.is_async {
                                        let future =
                                            self.spawn_task(IfaValue::Fn(data.clone()), args)?;
                                        self.push(future)?;
                                    } else {
                                        // Push CallFrame
                                        self.push_frame(CallFrame::new(
                                            self.ip,
                                            self.stack.len(),
                                            None,
                                            data.is_async,
                                        ))?;
                                        // Push arguments as locals
                                        for arg in args {
                                            self.push(arg)?;
                                        }
                                        // Jump to function body
                                        self.ip = data.start_ip;
                                    }
                                }
                                IfaValue::Closure(closure) => {
                                    let data = &closure.fn_data;
                                    if args.len() != data.arity as usize {
                                        return Err(IfaError::ArityMismatch {
                                            expected: data.arity as usize,
                                            got: args.len(),
                                        });
                                    }
                                    if data.is_async {
                                        let future = self
                                            .spawn_task(IfaValue::Closure(closure.clone()), args)?;
                                        self.push(future)?;
                                    } else {
                                        self.push_frame(CallFrame::new(
                                            self.ip,
                                            self.stack.len(),
                                            Some(closure.env.clone()),
                                            data.is_async,
                                        ))?;
                                        for arg in args {
                                            self.push(arg)?;
                                        }
                                        self.ip = data.start_ip;
                                    }
                                }
                                IfaValue::Str(s) => {
                                    if let Some((domain_id, method)) = parse_odu_fn_marker(s) {
                                        let result =
                                            self.call_registry(domain_id, &method, args, bytecode)?;
                                        self.push(result)?;
                                    } else if let Some((module_key, function_name)) =
                                        parse_module_fn_marker(s)
                                    {
                                        let result = self.invoke_module_function(
                                            &module_key,
                                            &function_name,
                                            args,
                                        )?;
                                        self.push(result)?;
                                    } else {
                                        self.push(IfaValue::Str(s.clone()))?;
                                    }
                                }
                                other => {
                                    // Map has the key but it's not a function — return the value
                                    self.push(other.clone())?;
                                }
                            }
                        } else {
                            // Key not found in map
                            return Err(IfaError::Custom(format!(
                                "Map has no method '{}'",
                                method_name
                            )));
                        }
                    }
                    IfaValue::List(mut l) => {
                        if method_name == "fikun" || method_name == "append" || method_name == "push" {
                           let val = args.get(0).ok_or_else(|| IfaError::ArityMismatch { expected: 1, got: args.len() })?;
                           let vec = Arc::make_mut(&mut l);
                           vec.push(val.clone());
                           self.push(IfaValue::null())?;
                           return Ok(());
                        } else {
                             return Err(IfaError::Custom(format!("List has no method '{}'", method_name)));
                        }
                    }
                    obj => {
                        // Fall back to registry for class instances / native objects
                        if let Some(registry) = self.registry.take() {
                            let result = registry.call_method(&obj, method_idx, args)?;
                            self.push(result)?;
                            self.registry = Some(registry);
                        } else {
                            return Err(IfaError::Custom(format!(
                                "Cannot call method '{}' on {}",
                                method_name,
                                obj.type_name()
                            )));
                        }
                    }
                }
            }

            // Collections
            OpCode::GetIndex => {
                // Indexing
                let index = self.pop()?;
                let collection = self.pop()?;

                match collection {
                    IfaValue::Map(m) => {
                        let key = match index {
                            IfaValue::Str(s) => s,
                            _ => {
                                return Err(IfaError::TypeError {
                                    expected: "Str".into(),
                                    got: index.type_name().into(),
                                });
                            }
                        };
                        match m.get(&key) {
                            Some(v) => self.push(v.clone())?,
                            None => self.push(IfaValue::null())?,
                        }
                    }
                    IfaValue::List(l) => {
                        let idx = match index {
                            IfaValue::Int(i) => i as usize,
                            _ => {
                                return Err(IfaError::TypeError {
                                    expected: "Int".into(),
                                    got: index.type_name().into(),
                                });
                            }
                        };
                        if idx >= l.len() {
                            return Err(IfaError::Runtime("Index out of bounds".into()));
                        }
                        self.push(l[idx].clone())?
                    }
                    IfaValue::Str(s) => {
                        let idx = match index {
                            IfaValue::Int(i) => i as usize,
                            _ => {
                                return Err(IfaError::TypeError {
                                    expected: "Int".into(),
                                    got: index.type_name().into(),
                                });
                            }
                        };
                        // Very inefficient char access, but functional
                        if let Some(c) = s.chars().nth(idx) {
                            self.push(IfaValue::str(c.to_string()))?;
                        } else {
                            return Err(IfaError::Runtime("Index out of bounds".into()));
                        }
                    }
                    _ => {
                        return Err(IfaError::TypeError {
                            expected: "Collection".into(),
                            got: collection.type_name().into(),
                        });
                    }
                }
            }

            OpCode::SetIndex => {
                let val = self.pop()?;
                let index = self.pop()?;
                let mut collection = self.pop()?;

                match collection {
                    IfaValue::List(ref mut vec_arc) => {
                        let i = match index {
                            IfaValue::Int(n) => n as usize,
                            _ => {
                                return Err(IfaError::TypeError {
                                    expected: "Int".into(),
                                    got: index.type_name().into(),
                                });
                            }
                        };
                        // HIGH PERFORMANCE: CoW using make_mut
                        let vec = std::sync::Arc::make_mut(vec_arc);
                        if i >= vec.len() {
                            return Err(IfaError::Runtime("Index out of bounds".into()));
                        }
                        vec[i] = val;
                    }
                    IfaValue::Map(ref mut map_arc) => {
                        let k = match index {
                            IfaValue::Str(s) => s.clone(),
                            _ => {
                                return Err(IfaError::TypeError {
                                    expected: "Str".into(),
                                    got: index.type_name().into(),
                                });
                            }
                        };
                        let map = std::sync::Arc::make_mut(map_arc);
                        map.insert(k, val);
                    }
                    _ => {
                        return Err(IfaError::TypeError {
                            expected: "List/Map".into(),
                            got: collection.type_name().into(),
                        });
                    }
                }
            }

            OpCode::BuildList => {
                let count = self.read_u8(bytecode)? as usize;
                let mut items = Vec::with_capacity(count);
                for _ in 0..count {
                    items.push(self.pop()?);
                }
                items.reverse();
                self.push(IfaValue::list(items))?;
            }

            OpCode::BuildMap => {
                let count = self.read_u8(bytecode)? as usize;
                // IfaValue::map expects HashMap<String, IfaValue> for input constructor convenience
                let mut map = std::collections::HashMap::with_capacity(count);
                for _ in 0..count {
                    let value = self.pop()?;
                    let key = self.pop()?;
                    if let IfaValue::Str(k) = key {
                        map.insert(k.to_string(), value);
                    }
                }
                self.push(IfaValue::map(map))?;
            }

            // I/O
            OpCode::Print => {
                let value = self.pop()?;
                self.opon.record("Ìrosù", "fọ̀ (spoke)", &value);
            }

            OpCode::PrintRaw => {
                let value = self.pop()?;
                self.opon.record("Ìrosù", "fọ̀ (spoke_raw)", &value);
            }

            OpCode::Input => {
                use std::io::{self, BufRead, Write};
                print!("> ");
                io::stdout().flush().ok();
                let mut input = String::new();
                io::stdin().lock().read_line(&mut input).ok();
                let result = IfaValue::str(input.trim());
                self.opon.record("Ogbè", "gbà (received)", &result);
                self.push(result)?;
            }

            OpCode::Import => {
                let path_idx = self.read_u16(bytecode)? as usize;
                let path = bytecode
                    .strings
                    .get(path_idx)
                    .cloned()
                    .ok_or(IfaError::Custom("Invalid import path index".into()))?;
                let exports = self.import_module(&path)?;
                self.push(exports)?;
            }

            OpCode::DefineClass => {
                // DESIGN DECISION (2026-04-07): Classes are formally removed.
                // This opcode must never be emitted by a current compiler. If it is
                // reached, bytecode is either stale or from a pre-decision build.
                // Fail loudly rather than silently pushing a corrupt IfaValue::Class.
                return Err(IfaError::Custom(
                    "DefineClass opcode reached at runtime. \
                     Class-based OOP has been formally removed from Ifá-Lang. \
                     Recompile your source with `ifa build` — the compiler will \
                     guide you toward Map + Domain Protocol design instead."
                        .into(),
                ));
            }

            // Exception Handling
            OpCode::TryBegin => {
                let offset = self.read_u32(bytecode)? as usize;
                let catch_ip = self.ip + offset;

                self.recovery_stack.push(RecoveryFrame {
                    stack_depth: self.stack.len(),
                    call_depth: self.frames.len(),
                    catch_ip,
                    finally_ip: None, // Populated by FinallyBegin if present
                    can_catch: true,
                });
            }

            OpCode::TryEnd => {
                // Happy path: pop the unused recovery frame
                self.recovery_stack.pop();
            }

            OpCode::Throw => {
                let err_val = self.pop()?;
                // §12.4: If there is a recovery frame with a finally block, divert to it.
                if let Some(finally_ip) = self.recovery_stack.last().and_then(|f| f.finally_ip) {
                    let error = IfaError::UserError(Box::new(err_val));
                    self.recovery_stack.pop();
                    self.pending_finally = Some(FinallyResumption::Propagate { error });
                    self.ip = finally_ip;
                    return Ok(());
                }
                // No finally block — convert to control-flow error immediately.
                return Err(IfaError::UserError(Box::new(err_val)));
            }

            // === Finally Handling ===
            OpCode::FinallyBegin => {
                // Operand: absolute IP of the shared (canonical) finally block.
                let finally_ip = self.read_u32(bytecode)? as usize;
                if let Some(frame) = self.recovery_stack.last_mut() {
                    frame.finally_ip = Some(finally_ip);
                }
            }

            OpCode::FinallyEnd => {
                // Cleanup has finished. Execute the stashed continuation, if any.
                match self.pending_finally.take() {
                    Some(FinallyResumption::Return { return_value }) => {
                        if let Some(finally_ip) =
                            self.recovery_stack.last().and_then(|f| f.finally_ip)
                        {
                            // Another outer finally still needs to run before the frame is popped.
                            self.recovery_stack.pop();
                            self.pending_finally = Some(FinallyResumption::Return { return_value });
                            self.ip = finally_ip;
                            return Ok(());
                        }

                        // Complete the return that was pre-empted by the finally block.
                        let frame = self
                            .frames
                            .pop()
                            .unwrap_or_else(|| CallFrame::new(0, 0, None, false));
                        if self.stack.len() > frame.base_ptr {
                            self.stack.truncate(frame.base_ptr);
                        }
                        if frame.async_return {
                            self.push(IfaValue::future_ready(return_value))?;
                        } else {
                            self.push(return_value)?;
                        }
                        self.ip = frame.return_addr;
                    }
                    Some(FinallyResumption::Propagate { error }) => {
                        if let Some(finally_ip) =
                            self.recovery_stack.last().and_then(|f| f.finally_ip)
                        {
                            self.recovery_stack.pop();
                            self.pending_finally = Some(FinallyResumption::Propagate { error });
                            self.ip = finally_ip;
                            return Ok(());
                        }
                        // Re-raise the error that was pre-empted by the finally block.
                        return Err(error);
                    }
                    None => {
                        // Normal flow (the finally ran as part of happy-path TryEnd).
                        // No action needed — fall through.
                    }
                }
            }

            OpCode::PropagateError => {
                let value = self.pop()?;
                match value {
                    IfaValue::Result(payload) => match *payload {
                        ResultPayload::Ok(ok) => self.push(ok)?,
                        ResultPayload::Err(err) => {
                            return Err(IfaError::UserError(Box::new(err)));
                        }
                    },
                    other => {
                        // Preserve backwards-compatibility for existing call sites
                        // that still compile bare values before full Result lowering.
                        self.push(other)?;
                    }
                }
            }

            // System
            OpCode::Halt => {
                self.halted = true;
            }
            OpCode::Yield => {
                return Err(IfaError::Yielded);
            }

            OpCode::Ref => {
                let addr = self.read_u32(bytecode)? as usize;
                self.push(IfaValue::Int(addr as i64))?;
            }

            OpCode::Load8 => {
                let ptr = self.pop()?;
                match ptr {
                    IfaValue::Int(addr) => {
                        let addr = addr as usize;
                        // Mock MMIO behavior validation for simulation or normal Opon read
                        let val = if addr >= 0x4000_0000 {
                            // Simulation: Reads from "hardware" return 0
                            IfaValue::int(0)
                        } else {
                            // Regular Opon access
                            self.opon.get(addr).cloned().unwrap_or(IfaValue::null())
                        };
                        self.push(val)?;
                    }
                    _ => {
                        return Err(IfaError::TypeError {
                            expected: "Pointer".into(),
                            got: ptr.type_name().into(),
                        });
                    }
                }
            }

            // === Pointers & Memory ===
            OpCode::Store8 => {
                let ptr = self.pop()?;
                let val = self.pop()?;

                match ptr {
                    IfaValue::Int(addr_i) => {
                        let addr = addr_i as usize;
                        if addr >= 0x4000_0000 {
                            // Simulation: Log the "hardware" write
                            self.opon.record("MMIO", "write", &val);
                        } else {
                            let _ = self
                                .opon
                                .try_set(addr, val)
                                .map_err(|e| IfaError::Runtime(e.to_string()))?;
                        }
                    }
                    _ => {
                        return Err(IfaError::TypeError {
                            expected: "Pointer (Int)".into(),
                            got: ptr.type_name().into(),
                        });
                    }
                }
            }

            OpCode::Store16 => {
                let ptr = self.pop()?;
                let val = self.pop()?;
                match ptr {
                    IfaValue::Int(addr_i) => {
                        let addr = addr_i as usize;
                        if addr >= 0x4000_0000 {
                            self.opon.record("MMIO", "write", &val);
                        } else {
                            let _ = self
                                .opon
                                .try_set(addr, val)
                                .map_err(|e| IfaError::Runtime(e.to_string()))?;
                        }
                    }
                    _ => {
                        return Err(IfaError::TypeError {
                            expected: "Pointer (Int)".into(),
                            got: ptr.type_name().into(),
                        });
                    }
                }
            }
            OpCode::Load16 => {
                let ptr = self.pop()?;
                match ptr {
                    IfaValue::Int(addr_i) => {
                        let addr = addr_i as usize;
                        let val = if addr >= 0x4000_0000 {
                            IfaValue::int(0)
                        } else {
                            self.opon.get(addr).cloned().unwrap_or(IfaValue::null())
                        };
                        self.push(val)?;
                    }
                    _ => {
                        return Err(IfaError::TypeError {
                            expected: "Pointer (Int)".into(),
                            got: ptr.type_name().into(),
                        });
                    }
                }
            }

            OpCode::Load32 => {
                let ptr = self.pop()?;
                match ptr {
                    IfaValue::Int(addr_i) => {
                        let addr = addr_i as usize;
                        let val = if addr >= 0x4000_0000 {
                            IfaValue::int(0)
                        } else {
                            self.opon.get(addr).cloned().unwrap_or(IfaValue::null())
                        };
                        self.push(val)?;
                    }
                    _ => {
                        return Err(IfaError::TypeError {
                            expected: "Ptr (Int)".into(),
                            got: ptr.type_name().into(),
                        });
                    }
                }
            }
            OpCode::Load64 => {
                let ptr = self.pop()?;
                match ptr {
                    IfaValue::Int(addr_i) => {
                        let addr = addr_i as usize;
                        let val = if addr >= 0x4000_0000 {
                            IfaValue::int(0)
                        } else {
                            self.opon.get(addr).cloned().unwrap_or(IfaValue::null())
                        };
                        self.push(val)?;
                    }
                    _ => {
                        return Err(IfaError::TypeError {
                            expected: "Ptr (Int)".into(),
                            got: ptr.type_name().into(),
                        });
                    }
                }
            }

            OpCode::Store32 => {
                let ptr = self.pop()?;
                let val = self.pop()?;
                match ptr {
                    IfaValue::Int(addr_i) => {
                        let addr = addr_i as usize;
                        if addr >= 0x4000_0000 {
                            self.opon.record("MMIO", "write", &val);
                        } else {
                            let _ = self
                                .opon
                                .try_set(addr, val)
                                .map_err(|e| IfaError::Runtime(e.to_string()))?;
                        }
                    }
                    _ => {
                        return Err(IfaError::TypeError {
                            expected: "Ptr (Int)".into(),
                            got: ptr.type_name().into(),
                        });
                    }
                }
            }
            OpCode::Store64 => {
                let ptr = self.pop()?;
                let val = self.pop()?;
                match ptr {
                    IfaValue::Int(addr_i) => {
                        let addr = addr_i as usize;
                        if addr >= 0x4000_0000 {
                            self.opon.record("MMIO", "write", &val);
                        } else {
                            let _ = self
                                .opon
                                .try_set(addr, val)
                                .map_err(|e| IfaError::Runtime(e.to_string()))?;
                        }
                    }
                    _ => {
                        return Err(IfaError::TypeError {
                            expected: "Pointer".into(),
                            got: ptr.type_name().into(),
                        });
                    }
                }
            }
            // Bitwise operations
            OpCode::And => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (a, b) {
                    (IfaValue::Int(i1), IfaValue::Int(i2)) => self.push(IfaValue::int(i1 & i2))?,
                    (IfaValue::Bool(b1), IfaValue::Bool(b2)) => {
                        self.push(IfaValue::bool(b1 && b2))?
                    }
                    (a, _) => {
                        return Err(IfaError::TypeError {
                            expected: "Int or Bool".into(),
                            got: a.type_name().into(),
                        });
                    }
                }
            }

            OpCode::Or => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (a, b) {
                    (IfaValue::Int(i1), IfaValue::Int(i2)) => self.push(IfaValue::int(i1 | i2))?,
                    (IfaValue::Bool(b1), IfaValue::Bool(b2)) => {
                        self.push(IfaValue::bool(b1 || b2))?
                    }
                    (a, _) => {
                        return Err(IfaError::TypeError {
                            expected: "Int or Bool".into(),
                            got: a.type_name().into(),
                        });
                    }
                }
            }

            OpCode::Xor => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (a, b) {
                    (IfaValue::Int(i1), IfaValue::Int(i2)) => self.push(IfaValue::int(i1 ^ i2))?,
                    (IfaValue::Bool(b1), IfaValue::Bool(b2)) => {
                        self.push(IfaValue::bool(b1 ^ b2))?
                    }
                    (a, _) => {
                        return Err(IfaError::TypeError {
                            expected: "Int/Bool".into(),
                            got: a.type_name().into(),
                        });
                    }
                }
            }

            OpCode::Len => {
                let val = self.pop()?;
                match val {
                    // R3: String length is Unicode code points, not bytes
                    IfaValue::Str(s) => self.push(IfaValue::int(s.chars().count() as i64))?,
                    IfaValue::List(l) => self.push(IfaValue::int(l.len() as i64))?,
                    IfaValue::Map(m) => self.push(IfaValue::int(m.len() as i64))?,
                    _ => {
                        return Err(IfaError::TypeError {
                            expected: "Collection".into(),
                            got: val.type_name().into(),
                        });
                    }
                }
            }

            OpCode::Not => {
                let a = self.pop()?;
                match a {
                    IfaValue::Int(i) => self.push(IfaValue::int(!i))?,
                    IfaValue::Bool(b) => self.push(IfaValue::bool(!b))?,
                    _ => {
                        return Err(IfaError::TypeError {
                            expected: "Int/Bool".into(),
                            got: a.type_name().into(),
                        });
                    }
                }
            }

            OpCode::Shl => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (a, b) {
                    (IfaValue::Int(val), IfaValue::Int(shift)) => {
                        self.push(IfaValue::int(val << shift))?
                    }
                    (a, _) => {
                        return Err(IfaError::TypeError {
                            expected: "Int".into(),
                            got: a.type_name().into(),
                        });
                    }
                }
            }

            OpCode::Shr => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (a, b) {
                    (IfaValue::Int(val), IfaValue::Int(shift)) => {
                        self.push(IfaValue::int(val >> shift))?
                    }
                    (a, _) => {
                        return Err(IfaError::TypeError {
                            expected: "Int".into(),
                            got: a.type_name().into(),
                        });
                    }
                }
            }

            // Type Casting
            OpCode::ToInt => {
                let val = self.pop()?;
                match val {
                    IfaValue::Int(i) => self.push(IfaValue::int(i))?,
                    IfaValue::Float(f) => self.push(IfaValue::int(f as i64))?,
                    IfaValue::Bool(b) => self.push(IfaValue::int(if b { 1 } else { 0 }))?,
                    // Parse string as integer; treat malformed input as 0 (same as JS parseInt).
                    IfaValue::Str(s) => {
                        let n = s.trim().parse::<i64>().unwrap_or(0);
                        self.push(IfaValue::int(n))?
                    }
                    _ => self.push(IfaValue::int(0))?,
                }
            }

            OpCode::ToFloat => {
                let val = self.pop()?;
                match val {
                    IfaValue::Int(i) => self.push(IfaValue::float(i as f64))?,
                    IfaValue::Float(f) => self.push(IfaValue::float(f))?,
                    _ => self.push(IfaValue::float(0.0))?,
                }
            }

            OpCode::ToString => {
                let val = self.pop()?;
                self.push(IfaValue::str(val.to_string()))?;
            }

            OpCode::ToBool => {
                let val = self.pop()?;
                self.push(IfaValue::bool(val.is_truthy()))?;
            }

            OpCode::Neg => {
                let val = self.pop()?;
                match val {
                    IfaValue::Int(n) => self.push(IfaValue::int(-n))?,
                    IfaValue::Float(f) => self.push(IfaValue::float(-f))?,
                    _ => return Err(IfaError::Runtime("Invalid type for negation".into())),
                }
            }
            OpCode::Pow => {
                let exp = self.pop()?;
                let base = self.pop()?;
                match (base, exp) {
                    (IfaValue::Int(b), IfaValue::Int(e)) => {
                        self.push(IfaValue::int(b.pow(e as u32)))?
                    }
                    (IfaValue::Float(b), IfaValue::Float(e)) => {
                        self.push(IfaValue::float(b.powf(e)))?
                    }
                    _ => return Err(IfaError::Runtime("Invalid types for power".into())),
                }
            }
            OpCode::Mod => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (a, b) {
                    (IfaValue::Int(a), IfaValue::Int(b)) => self.push(IfaValue::int(a % b))?,
                    _ => return Err(IfaError::Runtime("Modulus requires integers".into())),
                }
            }

            OpCode::Lt => {
                let b = self.pop()?;
                let a = self.pop()?;
                let a_type = a.type_name().to_string();
                let b_type = b.type_name().to_string();
                let result = match (a, b) {
                    (IfaValue::Int(ia), IfaValue::Int(ib)) => ia < ib,
                    (IfaValue::Float(fa), IfaValue::Float(fb)) => fa < fb,
                    (IfaValue::Int(ia), IfaValue::Float(fb)) => (ia as f64) < fb,
                    (IfaValue::Float(fa), IfaValue::Int(ib)) => fa < (ib as f64),
                    (IfaValue::Str(ref sa), IfaValue::Str(ref sb)) => sa < sb,
                    _ => {
                        return Err(IfaError::TypeError {
                            expected: "Int/Float/String".into(),
                            got: format!("{} and {}", a_type, b_type),
                        });
                    }
                };
                self.push(IfaValue::bool(result))?;
            }
            OpCode::Le => {
                let b = self.pop()?;
                let a = self.pop()?;
                let a_type = a.type_name().to_string();
                let b_type = b.type_name().to_string();
                let result = match (a, b) {
                    (IfaValue::Int(ia), IfaValue::Int(ib)) => ia <= ib,
                    (IfaValue::Float(fa), IfaValue::Float(fb)) => fa <= fb,
                    (IfaValue::Int(ia), IfaValue::Float(fb)) => (ia as f64) <= fb,
                    (IfaValue::Float(fa), IfaValue::Int(ib)) => fa <= (ib as f64),
                    (IfaValue::Str(ref sa), IfaValue::Str(ref sb)) => sa <= sb,
                    _ => {
                        return Err(IfaError::TypeError {
                            expected: "Int/Float/String".into(),
                            got: format!("{} and {}", a_type, b_type),
                        });
                    }
                };
                self.push(IfaValue::bool(result))?;
            }
            OpCode::Gt => {
                let b = self.pop()?;
                let a = self.pop()?;
                let a_type = a.type_name().to_string();
                let b_type = b.type_name().to_string();
                let result = match (a, b) {
                    (IfaValue::Int(ia), IfaValue::Int(ib)) => ia > ib,
                    (IfaValue::Float(fa), IfaValue::Float(fb)) => fa > fb,
                    (IfaValue::Int(ia), IfaValue::Float(fb)) => (ia as f64) > fb,
                    (IfaValue::Float(fa), IfaValue::Int(ib)) => fa > (ib as f64),
                    (IfaValue::Str(ref sa), IfaValue::Str(ref sb)) => sa > sb,
                    _ => {
                        return Err(IfaError::TypeError {
                            expected: "Int/Float/String".into(),
                            got: format!("{} and {}", a_type, b_type),
                        });
                    }
                };
                self.push(IfaValue::bool(result))?;
            }
            OpCode::Ge => {
                let b = self.pop()?;
                let a = self.pop()?;
                let a_type = a.type_name().to_string();
                let b_type = b.type_name().to_string();
                let result = match (a, b) {
                    (IfaValue::Int(ia), IfaValue::Int(ib)) => ia >= ib,
                    (IfaValue::Float(fa), IfaValue::Float(fb)) => fa >= fb,
                    (IfaValue::Int(ia), IfaValue::Float(fb)) => (ia as f64) >= fb,
                    (IfaValue::Float(fa), IfaValue::Int(ib)) => fa >= (ib as f64),
                    (IfaValue::Str(ref sa), IfaValue::Str(ref sb)) => sa >= sb,
                    _ => {
                        return Err(IfaError::TypeError {
                            expected: "Int/Float/String".into(),
                            got: format!("{} and {}", a_type, b_type),
                        });
                    }
                };
                self.push(IfaValue::bool(result))?;
            }
            OpCode::Eq => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(IfaValue::bool(a == b))?;
            }
            OpCode::Ne => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(IfaValue::bool(a != b))?;
            }

            OpCode::Push => {
                let idx = self.read_u32(bytecode)? as usize;
                let value = bytecode.constants.get(idx).cloned().ok_or_else(|| {
                    IfaError::Custom(format!("Invalid constant pool index {}", idx))
                })?;
                self.push(value)?;
            }
            _ => {
                return Err(IfaError::Custom(
                    format!("Unimplemented opcode: {:?}", opcode).into(),
                ));
            }
        }

        Ok(())
    }

    // =========================================================================
    // BYTECODE READING HELPERS
    // =========================================================================

    fn read_u8(&mut self, bytecode: &Bytecode) -> IfaResult<u8> {
        if self.ip >= bytecode.code.len() {
            return Err(IfaError::Custom("Unexpected end of bytecode".to_string()));
        }
        let value = bytecode.code[self.ip];
        self.ip += 1;
        Ok(value)
    }

    fn read_u16(&mut self, bytecode: &Bytecode) -> IfaResult<u16> {
        let mut bytes = [0u8; 2];
        bytes[0] = self.read_u8(bytecode)?;
        bytes[1] = self.read_u8(bytecode)?;
        Ok(u16::from_le_bytes(bytes))
    }

    fn read_u32(&mut self, bytecode: &Bytecode) -> IfaResult<u32> {
        let mut bytes = [0u8; 4];
        bytes[0] = self.read_u8(bytecode)?;
        bytes[1] = self.read_u8(bytecode)?;
        bytes[2] = self.read_u8(bytecode)?;
        bytes[3] = self.read_u8(bytecode)?;
        Ok(u32::from_le_bytes(bytes))
    }

    fn read_i64(&mut self, bytecode: &Bytecode) -> IfaResult<i64> {
        let mut bytes = [0u8; 8];
        for byte in &mut bytes {
            *byte = self.read_u8(bytecode)?;
        }
        Ok(i64::from_le_bytes(bytes))
    }

    fn read_f64(&mut self, bytecode: &Bytecode) -> IfaResult<f64> {
        let mut bytes = [0u8; 8];
        for byte in &mut bytes {
            *byte = self.read_u8(bytecode)?;
        }
        Ok(f64::from_le_bytes(bytes))
    }
}

fn collect_exports_vm(program: &crate::ast::Program) -> Vec<String> {
    let mut out = Vec::new();
    for stmt in &program.statements {
        match stmt {
            crate::ast::Statement::VarDecl {
                name,
                visibility: crate::ast::Visibility::Public,
                ..
            } => out.push(name.clone()),
            crate::ast::Statement::Const {
                name,
                visibility: crate::ast::Visibility::Public,
                ..
            } => out.push(name.clone()),
            crate::ast::Statement::EseDef {
                name,
                visibility: crate::ast::Visibility::Public,
                ..
            } => out.push(name.clone()),
            crate::ast::Statement::OduDef {
                name,
                visibility: crate::ast::Visibility::Public,
                ..
            } => out.push(name.clone()),
            _ => {}
        }
    }
    out
}

impl Default for IfaVM {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_odu_mod_marker(s: &str) -> Option<u8> {
    const PREFIX: &str = "__odu_mod__:";
    if let Some(rest) = s.strip_prefix(PREFIX) {
        if let Ok(id) = rest.parse::<u8>() {
            return Some(id);
        }
        if let Some(id) = odu_domain_id(rest) {
            return Some(id);
        }
    }
    None
}

fn parse_odu_fn_marker(s: &str) -> Option<(u8, String)> {
    const PREFIX: &str = "__odu_fn__:";
    if let Some(rest) = s.strip_prefix(PREFIX) {
        let mut parts = rest.splitn(2, ':');
        let domain = parts.next()?;
        let method = parts.next()?.to_string();
        if let Ok(id) = domain.parse::<u8>() {
            return Some((id, method));
        }
        if let Some(id) = odu_domain_id(domain) {
            return Some((id, method));
        }
    }
    None
}

fn parse_module_fn_marker(s: &str) -> Option<(String, String)> {
    const PREFIX: &str = "__module_fn__:";
    let rest = s.strip_prefix(PREFIX)?;
    let split_at = rest.rfind(':')?;
    let module_key = rest[..split_at].to_string();
    let function_name = rest[split_at + 1..].to_string();
    if module_key.is_empty() || function_name.is_empty() {
        return None;
    }
    Some((module_key, function_name))
}

fn odu_domain_id(name: &str) -> Option<u8> {
    match name.to_lowercase().as_str() {
        "ogbe" => Some(0),
        "oyeku" => Some(1),
        "iwori" => Some(2),
        "odi" => Some(3),
        "irosu" => Some(4),
        "owonrin" => Some(5),
        "obara" => Some(6),
        "okanran" => Some(7),
        "ogunda" => Some(8),
        "osa" => Some(9),
        "ika" => Some(10),
        "oturupon" => Some(11),
        "otura" => Some(12),
        "irete" => Some(13),
        "ose" => Some(14),
        "ofun" => Some(15),
        "coop" => Some(16),
        "opele" => Some(17),
        "cpu" => Some(18),
        "gpu" => Some(19),
        "storage" => Some(20),
        "backend" => Some(21),
        "frontend" => Some(22),
        "crypto" => Some(23),
        "ml" => Some(24),
        "gamedev" => Some(25),
        "iot" => Some(26),
        "ohun" => Some(27),
        "fidio" => Some(28),
        "sys" => Some(29),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::Compiler;
    use crate::parser::parse;

    #[test]
    fn test_simple_arithmetic() {
        let mut vm = IfaVM::new();

        // Push 5, Push 3, Add -> 8
        let mut bc = Bytecode::new("test");
        bc.code = vec![
            OpCode::PushInt as u8,
            5,
            0,
            0,
            0,
            0,
            0,
            0,
            0, // 5 as i64 LE
            OpCode::PushInt as u8,
            3,
            0,
            0,
            0,
            0,
            0,
            0,
            0, // 3 as i64 LE
            OpCode::Add as u8,
            OpCode::Halt as u8,
        ];

        let result = vm.execute(&bc).unwrap();
        assert_eq!(result, IfaValue::Int(8));
    }

    #[test]
    fn test_stack_operations() {
        let mut vm = IfaVM::new();

        vm.push(IfaValue::Int(1)).unwrap();
        vm.push(IfaValue::Int(2)).unwrap();
        vm.push(IfaValue::Int(3)).unwrap();

        assert_eq!(vm.pop().unwrap(), IfaValue::Int(3));
        assert_eq!(vm.pop().unwrap(), IfaValue::Int(2));
        assert_eq!(vm.pop().unwrap(), IfaValue::Int(1));
        assert!(vm.pop().is_err());
    }

    #[test]
    fn test_snapshot_yield() {
        let mut vm = IfaVM::new();

        let mut bc = Bytecode::new("test_yield");
        bc.code = vec![
            OpCode::PushInt as u8,
            5,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            OpCode::Yield as u8,
            OpCode::PushInt as u8,
            3,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            OpCode::Add as u8,
            OpCode::Halt as u8,
        ];

        // 1. Execute up to Yield
        let res = vm.execute(&bc);
        assert!(matches!(res, Err(IfaError::Yielded)));

        assert_eq!(vm.stack.len(), 1);
        assert_eq!(vm.stack[0], IfaValue::Int(5));

        // 2. Snapshot
        let snap = vm.snapshot(&bc).expect("Failed to create snapshot");

        // 3. Resume in a fresh VM
        let mut vm2 = IfaVM::resume(&snap, &bc).expect("Failed to resume snapshot");

        // 4. Continue execution
        let final_res = vm2.resume_execution(&bc).unwrap();

        assert_eq!(final_res, IfaValue::Int(8)); // 5 + 3 = 8
        assert_eq!(vm2.stack.len(), 0);
    }

    #[test]
    fn test_return_from_catch_runs_finally() {
        let source = r#"
        ayanmo y = 0;
        ese f() {
            gbiyanju {
                ayanmo _boom = 1 / 0;
            } gba (e) {
                pada 1;
            } nipari {
                y = 2;
            }
        }
        ayanmo _r = f();
        pada y;
        "#;

        let program = parse(source).expect("parse failed");
        let bytecode = Compiler::new("test_return_from_catch_runs_finally")
            .compile(&program)
            .expect("compile failed");
        let mut vm = IfaVM::new();
        let got = vm.execute(&bytecode).expect("vm failed");
        assert_eq!(got, IfaValue::Int(2));
    }

    #[test]
    fn test_nested_finally_runs_before_return_completes() {
        let source = r#"
        ayanmo y = 0;
        ese f() {
            gbiyanju {
                gbiyanju {
                    ayanmo _boom = 1 / 0;
                } gba (e) {
                    pada 7;
                } nipari {
                    y = 1;
                }
            } gba (outer) {
                pada 9;
            } nipari {
                y = 2;
            }
        }
        ayanmo _r = f();
        pada y;
        "#;

        let program = parse(source).expect("parse failed");
        let bytecode = Compiler::new("test_nested_finally_runs_before_return_completes")
            .compile(&program)
            .expect("compile failed");
        let mut vm = IfaVM::new();
        let got = vm.execute(&bytecode).expect("vm failed");
        assert_eq!(got, IfaValue::Int(2));
    }

    #[test]
    fn test_propagate_error_unwraps_ok_and_throws_err() {
        let mut ok_vm = IfaVM::new();
        ok_vm
            .globals
            .insert("okv".to_string(), IfaValue::ok(IfaValue::Int(41)));
        let mut ok_bytecode = Bytecode::new("test_propagate_error_unwraps_ok");
        ok_bytecode.strings.push("okv".to_string());
        ok_bytecode.code = vec![
            OpCode::LoadGlobal as u8,
            0,
            0,
            OpCode::PropagateError as u8,
            OpCode::Return as u8,
        ];
        let ok_got = ok_vm.execute(&ok_bytecode).expect("vm failed");
        assert_eq!(ok_got, IfaValue::Int(41));

        let mut err_vm = IfaVM::new();
        err_vm
            .globals
            .insert("failv".to_string(), IfaValue::err(IfaValue::str("boom")));
        let mut err_bytecode = Bytecode::new("test_propagate_error_throws_err");
        err_bytecode.strings.push("failv".to_string());
        err_bytecode.code = vec![
            OpCode::TryBegin as u8,
            5,
            0,
            0,
            0,
            OpCode::LoadGlobal as u8,
            0,
            0,
            OpCode::PropagateError as u8,
            OpCode::TryEnd as u8,
            OpCode::PushInt as u8,
            7,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            OpCode::Return as u8,
        ];
        let err_got = err_vm.execute(&err_bytecode).expect("vm failed");
        assert_eq!(err_got, IfaValue::Int(7));
    }
}
