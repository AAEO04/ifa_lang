//! # Ifá-Lang Virtual Machine
//!
//! Stack-based bytecode interpreter for Ifá-Lang.
//!
//! ### ✅ ARCHITECTURAL STATUS (String Operations)
//! `OpCode::Add` is polymorphic (supporting Int/Float arithmetic, List concatenation, and String concatenation).
//! `OpCode::Concat (0x27)` is a strict `Str + Str` Concat-only instruction.
//!
//! Refer to `patch.md` for the Phase 7 Hardening Roadmap.

use crate::actor::ActorTable;
use crate::bytecode::{Bytecode, OpCode};
use crate::error::{IfaError, IfaResult};
use crate::native::{OduRegistry, VmContext};
use crate::opon::Opon;
use bincode::Options;
use ifa_types::registry::ResourceRegistry;
use ifa_types::value_union::{ClosureData, FutureState, IfaValue, ResultPayload, UpvalueCell};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc as RegistryArc;
use std::sync::{Arc, Mutex};

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
    #[serde(skip)]
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

pub trait BytecodeCache: Send + Sync {
    fn get_bytecode(&self, cache_key: &str) -> Option<(u64, Bytecode)>;
    fn put_bytecode(&self, cache_key: &str, hash: u64, bytecode: &Bytecode);
}

const MAX_SNAPSHOT_BYTES: u64 = 64 * 1024 * 1024;
/// E2: Linker/module state extracted from hot IfaVM body for cache locality.
pub struct ModuleState {
    pub imported: std::collections::HashSet<String>,
    pub import_guard: crate::module_resolver::ImportGuard,
    pub resolver: crate::module_resolver::ModuleResolver,
    pub module_cache: std::collections::HashMap<String, CachedModule>,
    pub module_exports: std::collections::HashMap<String, IfaValue>,
    pub module_bytecode: std::collections::HashMap<String, Bytecode>,
    pub module_globals: std::collections::HashMap<String, GlobalState>,
    pub current_file: Option<std::path::PathBuf>,
    pub external_cache: Option<std::sync::Arc<dyn BytecodeCache>>,
}

impl Default for ModuleState {
    fn default() -> Self {
        ModuleState {
            imported: std::collections::HashSet::new(),
            import_guard: crate::module_resolver::ImportGuard::new(),
            resolver: crate::module_resolver::ModuleResolver::new(vec![]),
            module_cache: std::collections::HashMap::new(),
            module_exports: std::collections::HashMap::new(),
            module_bytecode: std::collections::HashMap::new(),
            module_globals: std::collections::HashMap::new(),
            current_file: None,
            external_cache: None,
        }
    }
}

#[derive(Clone)]
pub struct CachedModule {
    mtime: std::time::SystemTime,
    size: u64,
    bytecode: Bytecode,
    export_names: Vec<String>,
}

/// The Ifá Virtual Machine
#[derive(Serialize, Deserialize)]
pub struct IfaVM {
    /// Execution Context (State Locality)
    pub ctx: ExecutionContext,

    /// Global variables
    globals: GlobalState,
    /// Bytecode string index -> GlobalState slot.
    /// I1: Direct Vec index replaces HashMap. Populated on first access per bytecode string index.
    #[serde(skip)]
    global_string_slots: Vec<Option<usize>>,
    /// Name -> GlobalState slot. O(1) lookup for the public get_global/set_global API.
    /// Rebuilt lazily after deserialization via ensure_global_slot.
    #[serde(skip)]
    global_names_index: HashMap<String, usize>,
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
    /// Per-VM resource handle store (replaces global REGISTRY).
    /// Arc-wrapped so background dispatch threads (storage, GPU init) can
    /// clone the pointer and register handles without borrowing the VM.
    #[serde(skip)]
    pub resource_registry: RegistryArc<ResourceRegistry>,
    /// Execution ticks (for GC/Interrupts)
    pub ticks: usize,
    /// Execution fuel budget
    pub fuel: Option<u64>,

    /// The Sacred Nuts - Runtime Constant Pool
    pub ikin: Ikin,

    /// Async task queue (cooperative scheduler)
    #[serde(skip)]
    task_queue: VecDeque<Task>,

    /// H2: Process-wide actor handle registry. Shared across VM + spawned actors.
    #[serde(skip)]
    pub actor_table: Arc<ActorTable>,

    /// E2: All module/linker state in one cache-friendly sub-struct.
    /// Access via `self.module.resolver`, `self.module.module_cache`, etc.
    #[serde(skip)]
    pub module: ModuleState,

    /// H2: Actor ID of this VM if executing inside an actor context
    #[serde(skip)]
    pub actor_id: Option<u64>,

    /// Active epoch cleanups to manage reactivity subscription lifespans
    #[serde(skip)]
    pub epoch_guards: Vec<crate::ajose::EpochCleanupGuard>,

    /// Pending finally continuation (§12.4).

    /// Set by `Return`/`Throw` when they are pre-empted by a finally block.
    /// Cleared and executed by `FinallyEnd`.
    #[serde(skip)]
    pending_finally: Option<FinallyResumption>,

    /// Spiritual Capacity/Time Limit (Ori limit)
    #[serde(skip)]
    pub ori_limit: Option<u64>,
}

#[derive(Clone)]
struct Task {
    func: IfaValue,
    args: Vec<IfaValue>,
    future: ifa_types::value_union::FutureCell,
    ctx: ExecutionContext,
    started: bool,
    base_depth: usize,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub stack: Vec<IfaValue>,
    pub frames: Vec<CallFrame>,
    pub ip: usize,
    pub halted: bool,
    pub recovery_stack: Vec<RecoveryFrame>,
    /// Stack of (continue_ip, break_ip) for each active loop.
    /// Pushed when a loop starts (at the JumpIfFalse checking the condition),
    /// popped when the loop exits naturally or via break.
    pub loop_stack: Vec<(usize, usize)>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct GlobalState {
    pub names: Vec<String>,
    pub values: Vec<Option<IfaValue>>,
}

impl IfaVM {
    fn error_to_catch_value(error: &IfaError) -> IfaValue {
        error
            .user_value()
            .map(|v| v.thaw())
            .unwrap_or_else(|| IfaValue::str(error.to_string()))
    }

    fn find_global_slot(&self, name: &str) -> Option<usize> {
        // O(1) index first; fall back to linear scan only after deserialization
        // before any ensure_global_slot has been called for this name.
        self.global_names_index
            .get(name)
            .copied()
            .or_else(|| self.globals.names.iter().position(|n| n == name))
    }

    fn reset_global_string_slots(&mut self, string_count: usize) {
        self.global_string_slots.clear();
        self.global_string_slots.resize(string_count, None);
        // Do NOT clear global_names_index here — the name->slot mapping survives
        // across module switches; only the bytecode-index cache is invalidated.
    }

    fn ensure_global_slot(&mut self, name: &str) -> usize {
        if let Some(slot) = self.find_global_slot(name) {
            // Warm the index in case we reached here via the linear fallback.
            self.global_names_index
                .entry(name.to_string())
                .or_insert(slot);
            return slot;
        }

        let slot = self.globals.names.len();
        self.globals.names.push(name.to_string());
        self.globals.values.push(None);
        self.global_names_index.insert(name.to_string(), slot);
        slot
    }

    fn resolve_global_slot(&mut self, bytecode: &Bytecode, idx: usize) -> IfaResult<usize> {
        if idx >= bytecode.strings.len() {
            return Err(IfaError::Custom("Invalid global name index".into()));
        }

        if idx >= self.global_string_slots.len() {
            self.global_string_slots
                .resize(bytecode.strings.len(), None);
        }

        // Fast path: slot already resolved for this string index.
        if let Some(slot) = self.global_string_slots[idx] {
            return Ok(slot);
        }

        // Slow path: first access for this string index — resolve name -> slot.
        let name = bytecode
            .strings
            .get(idx)
            .ok_or(IfaError::Custom("Invalid global name index".into()))?;
        let slot = self.ensure_global_slot(name);
        self.global_string_slots[idx] = Some(slot);
        Ok(slot)
    }

    fn load_global_slot(&mut self, bytecode: &Bytecode, idx: usize) -> IfaResult<()> {
        let slot = self.resolve_global_slot(bytecode, idx)?;
        let value = self
            .globals
            .values
            .get(slot)
            .and_then(Option::as_ref)
            .cloned()
            .ok_or_else(|| IfaError::UndefinedVariable(self.globals.names[slot].clone()))?;
        if matches!(value, IfaValue::Moved) {
            return Err(IfaError::UndefinedVariable(format!(
                "Use of moved global variable '{}'",
                self.globals.names[slot]
            )));
        }
        self.push(value)
    }

    fn move_global_slot(&mut self, bytecode: &Bytecode, idx: usize) -> IfaResult<()> {
        let slot = self.resolve_global_slot(bytecode, idx)?;
        let value = self
            .globals
            .values
            .get_mut(slot)
            .and_then(|val| std::mem::replace(val, Some(IfaValue::Moved)))
            .ok_or_else(|| IfaError::UndefinedVariable(self.globals.names[slot].clone()))?;
        if matches!(value, IfaValue::Moved) {
            return Err(IfaError::UndefinedVariable(format!(
                "Use of moved global variable '{}'",
                self.globals.names[slot]
            )));
        }
        self.push(value)
    }

    fn store_global_slot(&mut self, bytecode: &Bytecode, idx: usize) -> IfaResult<()> {
        let slot = self.resolve_global_slot(bytecode, idx)?;
        let value = self.pop()?;
        if slot >= self.globals.values.len() {
            return Err(IfaError::Custom("Global slot out of bounds".into()));
        }
        self.globals.values[slot] = Some(value);
        Ok(())
    }

    /// Get a global variable by name.
    pub fn get_global(&self, name: &str) -> Option<&IfaValue> {
        self.find_global_slot(name)
            .and_then(|slot| self.globals.values.get(slot))
            .and_then(Option::as_ref)
    }

    /// Get all globals
    pub fn globals(&self) -> &GlobalState {
        &self.globals
    }

    /// Set or replace a global variable.
    pub fn set_global(&mut self, name: impl Into<String>, value: IfaValue) {
        let name = name.into();
        let slot = self.ensure_global_slot(&name);
        self.globals.values[slot] = Some(value);
    }

    /// Create new VM
    pub fn new() -> Self {
        let mut module_paths = Vec::new();
        if let Ok(cwd) = std::env::current_dir() {
            module_paths.push(cwd);
        }
        if let Ok(exe) = std::env::current_exe()
            && let Some(dir) = exe.parent()
        {
            module_paths.push(dir.join("lib"));
        }
        IfaVM {
            ctx: ExecutionContext {
                stack: Vec::new(),
                frames: Vec::new(),
                ip: 0,
                halted: false,
                recovery_stack: Vec::with_capacity(32),
                loop_stack: Vec::new(),
            },
            globals: GlobalState::default(),
            global_string_slots: Vec::new(),
            global_names_index: HashMap::new(),
            opon: Opon::create_default(),
            stack_limit: Some(4096),
            frame_limit: Some(512),
            opon_size: crate::bytecode::OponSize::Ailopin,
            registry: None,
            resource_registry: RegistryArc::new(ResourceRegistry::new()),
            ticks: 0,
            fuel: None,
            ikin: Ikin::new(),
            task_queue: VecDeque::new(),
            actor_table: ActorTable::new(),
            module: ModuleState {
                imported: std::collections::HashSet::new(),
                import_guard: crate::module_resolver::ImportGuard::new(),
                module_cache: std::collections::HashMap::new(),
                module_exports: std::collections::HashMap::new(),
                module_bytecode: std::collections::HashMap::new(),
                module_globals: std::collections::HashMap::new(),
                resolver: crate::module_resolver::ModuleResolver::new(module_paths),
                current_file: None,
                external_cache: None,
            },
            actor_id: None,
            epoch_guards: Vec::new(),
            pending_finally: None,
            ori_limit: None,
        }
    }

    /// Attach a function registry (Standard Library)
    pub fn with_registry(mut self, registry: Box<dyn OduRegistry>) -> Self {
        self.registry = Some(registry);
        self
    }

    /// Create a VM configured for sandboxed execution with a fixed fuel budget.
    pub fn sandboxed(fuel: u64) -> Self {
        let mut vm = Self::new();
        vm.stack_limit = Some(1024);
        vm.frame_limit = Some(128);
        vm.fuel = Some(fuel);
        vm
    }

    /// Create VM with custom Opon size
    pub fn with_opon(opon: Opon) -> Self {
        let mut module_paths = Vec::new();
        if let Ok(cwd) = std::env::current_dir() {
            module_paths.push(cwd);
        }
        if let Ok(exe) = std::env::current_exe()
            && let Some(dir) = exe.parent()
        {
            module_paths.push(dir.join("lib"));
        }
        IfaVM {
            ctx: ExecutionContext {
                stack: Vec::new(),
                frames: Vec::new(),
                ip: 0,
                halted: false,
                recovery_stack: Vec::with_capacity(32),
                loop_stack: Vec::new(),
            },
            globals: GlobalState::default(),
            global_string_slots: Vec::new(),
            global_names_index: HashMap::new(),
            opon,
            stack_limit: Some(4096),
            frame_limit: Some(512),
            opon_size: crate::bytecode::OponSize::Ailopin,
            registry: None,
            resource_registry: RegistryArc::new(ResourceRegistry::new()),
            ticks: 0,
            fuel: None,
            ikin: Ikin::new(),
            task_queue: VecDeque::new(),
            actor_table: ActorTable::new(),
            module: ModuleState {
                imported: std::collections::HashSet::new(),
                import_guard: crate::module_resolver::ImportGuard::new(),
                module_cache: std::collections::HashMap::new(),
                module_exports: std::collections::HashMap::new(),
                module_bytecode: std::collections::HashMap::new(),
                module_globals: std::collections::HashMap::new(),
                resolver: crate::module_resolver::ModuleResolver::new(module_paths),
                current_file: None,
                external_cache: None,
            },
            actor_id: None,
            epoch_guards: Vec::new(),
            pending_finally: None,
            ori_limit: None,
        }
    }

    /// Create VM with custom file path (for module resolution)
    pub fn with_file(file: impl AsRef<std::path::Path>) -> Self {
        let mut vm = Self::new();
        let path = file.as_ref().to_path_buf();
        if let Some(parent) = path.parent() {
            vm.module
                .resolver
                .search_paths
                .insert(0, parent.to_path_buf());
        }
        vm.module.current_file = Some(path);
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
        let opts = bincode::DefaultOptions::new()
            .with_limit(MAX_SNAPSHOT_BYTES)
            .with_fixint_encoding()
            .allow_trailing_bytes();
        let (saved_hash, mut vm): (u64, IfaVM) = opts
            .deserialize(snapshot)
            .map_err(|e| IfaError::Custom(format!("Corrupted snapshot: {}", e)))?;

        if saved_hash != bytecode.hash() {
            return Err(IfaError::Custom(
                "InvalidSnapshot: The bytecode provided does not match the active bytecode at the time of the snapshot. Resuming would cause a VM segfault.".to_string()
            ));
        }

        vm.ikin.rebuild_bytecode_mapping(bytecode)?;

        Ok(vm)
    }

    // =========================================================================
    // STACK OPERATIONS
    // =========================================================================

    /// Push value onto stack
    pub fn push(&mut self, value: IfaValue) -> IfaResult<()> {
        if let Some(limit) = self.stack_limit
            && self.ctx.stack.len() >= limit
        {
            return Err(IfaError::StackOverflow {
                limit,
                directive: self.opon_size,
            });
        }
        self.ctx.stack.push(value);
        Ok(())
    }

    /// Push CallFrame onto execution stack
    pub fn push_frame(&mut self, frame: CallFrame) -> IfaResult<()> {
        if let Some(limit) = self.frame_limit
            && self.ctx.frames.len() >= limit
        {
            return Err(IfaError::StackOverflow {
                limit,
                directive: self.opon_size,
            });
        }
        self.ctx.frames.push(frame);
        Ok(())
    }

    /// Pop value from stack
    pub fn pop(&mut self) -> IfaResult<IfaValue> {
        self.ctx.stack.pop().ok_or(IfaError::StackUnderflow)
    }

    /// Peek at top of stack
    pub fn peek(&self) -> IfaResult<&IfaValue> {
        self.ctx.stack.last().ok_or(IfaError::StackUnderflow)
    }

    // /// Pop an integer from the stack

    // =========================================================================
    // BYTECODE EXECUTION
    // =========================================================================

    /// Execute bytecode
    pub fn execute(&mut self, bytecode: &Bytecode) -> IfaResult<IfaValue> {
        // Phase 0: Validate bytecode integrity
        ifa_bytecode::validate_bytecode(&bytecode.code)
            .map_err(|e| IfaError::Compile(format!("Invalid Bytecode: {:?}", e)))?;

        self.set_current_file_from_source(&bytecode.source_name);
        self.ctx.ip = 0;
        self.ctx.halted = false;
        self.task_queue.clear();
        self.reset_global_string_slots(bytecode.strings.len());

        // Phase 1: Consult the Nuts (Load Constants)
        self.ikin.load_from_bytecode(bytecode)?;

        let (stack_cap, frame_cap) = bytecode.opon_size.limits();
        self.stack_limit = stack_cap;
        self.frame_limit = frame_cap;
        self.opon_size = bytecode.opon_size;

        if let Some(cap) = stack_cap
            && self.ctx.stack.capacity() < cap
        {
            self.ctx.stack.reserve(cap - self.ctx.stack.len());
        }

        self.resume_execution(bytecode)
    }

    fn set_current_file_from_source(&mut self, source_name: &str) {
        let path = std::path::Path::new(source_name);
        if path.exists() {
            if let Some(parent) = path.parent()
                && !self
                    .module
                    .resolver
                    .search_paths
                    .iter()
                    .any(|p| p == parent)
            {
                self.module
                    .resolver
                    .search_paths
                    .insert(0, parent.to_path_buf());
            }
            self.module.current_file = Some(path.to_path_buf());
        }
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
            .module
            .module_bytecode
            .get(module_key)
            .cloned()
            .ok_or_else(|| {
                IfaError::Runtime(format!("Module bytecode missing for '{}'", module_key))
            })?;
        let module_globals = self
            .module
            .module_globals
            .get(module_key)
            .cloned()
            .ok_or_else(|| {
                IfaError::Runtime(format!("Module state missing for '{}'", module_key))
            })?;
        let func_slot = module_globals
            .names
            .iter()
            .position(|name| name == function_name)
            .ok_or_else(|| IfaError::UndefinedVariable(function_name.to_string()))?;
        let func = module_globals
            .values
            .get(func_slot)
            .and_then(Option::as_ref)
            .cloned()
            .ok_or_else(|| IfaError::UndefinedVariable(function_name.to_string()))?;

        let saved_ip = self.ctx.ip;
        let saved_halted = self.ctx.halted;
        let saved_stack_len = self.ctx.stack.len();
        let saved_frames_len = self.ctx.frames.len();
        let saved_recovery_len = self.ctx.recovery_stack.len();
        let saved_limits = (self.stack_limit, self.frame_limit, self.opon_size);
        let saved_ikin_mapping = self.ikin.take_mapping();
        let saved_globals = std::mem::replace(&mut self.globals, module_globals);
        self.reset_global_string_slots(bytecode.strings.len());

        let result = (|| {
            self.ctx.halted = false;
            self.ikin.load_from_bytecode(&bytecode)?;
            let (stack_cap, frame_cap) = bytecode.opon_size.limits();
            self.stack_limit = stack_cap;
            self.frame_limit = frame_cap;
            self.opon_size = bytecode.opon_size;
            let return_addr = bytecode.code.len();
            self.ctx.ip = return_addr;
            let arg_count = args.len();
            for arg in args {
                self.push(arg)?;
            }
            self.call_value(func, arg_count, false, Some(&bytecode))?;
            self.resume_execution(&bytecode)
        })();

        let updated_module_globals = self.globals.clone();
        self.globals = saved_globals;
        self.global_string_slots.clear();
        self.global_names_index.clear();
        self.module
            .module_globals
            .insert(module_key.to_string(), updated_module_globals);
        self.ctx.stack.truncate(saved_stack_len);
        self.ctx.frames.truncate(saved_frames_len);
        self.ctx.recovery_stack.truncate(saved_recovery_len);
        self.stack_limit = saved_limits.0;
        self.frame_limit = saved_limits.1;
        self.opon_size = saved_limits.2;
        self.ikin.restore_mapping(saved_ikin_mapping);
        self.ctx.ip = saved_ip;
        self.ctx.halted = saved_halted;

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
        self.module.import_guard.enter(&module_key)?;

        let resolved = self.module.resolver.resolve(&module_key)?;
        let file_path = resolved.path;
        let is_ifab = resolved.is_binary;

        let cache_key = file_path.to_string_lossy().to_string();
        let mut cached_mtime_before = None;
        let mut cached_size_before = None;
        if let Some(cached) = self.module.module_cache.get(&cache_key) {
            cached_mtime_before = Some(cached.mtime);
            cached_size_before = Some(cached.size);
        }

        let (bytecode, export_names) = if is_ifab {
            let metadata = std::fs::metadata(&file_path).map_err(|e| {
                IfaError::IoError(format!(
                    "Cannot read module metadata '{}': {}",
                    module_key, e
                ))
            })?;
            let mtime = metadata
                .modified()
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            let size = metadata.len();

            if let Some(cached) = self.module.module_cache.get(&cache_key) {
                if cached.mtime == mtime && cached.size == size {
                    (cached.bytecode.clone(), cached.export_names.clone())
                } else {
                    let bytes = std::fs::read(&file_path).map_err(|e| {
                        IfaError::IoError(format!("Cannot read module '{}': {}", module_key, e))
                    })?;
                    let bc = Bytecode::from_bytes(&bytes)?;
                    let exports = bc.exports.clone();
                    self.module.module_cache.insert(
                        cache_key.clone(),
                        CachedModule {
                            mtime,
                            size,
                            bytecode: bc.clone(),
                            export_names: exports.clone(),
                        },
                    );
                    (bc, exports)
                }
            } else {
                let bytes = std::fs::read(&file_path).map_err(|e| {
                    IfaError::IoError(format!("Cannot read module '{}': {}", module_key, e))
                })?;
                let bc = Bytecode::from_bytes(&bytes)?;
                let exports = bc.exports.clone();
                self.module.module_cache.insert(
                    cache_key.clone(),
                    CachedModule {
                        mtime,
                        size,
                        bytecode: bc.clone(),
                        export_names: exports.clone(),
                    },
                );
                (bc, exports)
            }
        } else {
            #[cfg(feature = "compiler")]
            {
                let metadata = std::fs::metadata(&file_path).map_err(|e| {
                    IfaError::IoError(format!(
                        "Cannot read module metadata '{}': {}",
                        module_key, e
                    ))
                })?;
                let mtime = metadata
                    .modified()
                    .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                let size = metadata.len();

                if let Some(cached) = self.module.module_cache.get(&cache_key) {
                    if cached.mtime == mtime && cached.size == size {
                        (cached.bytecode.clone(), cached.export_names.clone())
                    } else {
                        let source = std::fs::read_to_string(&file_path).map_err(|e| {
                            IfaError::IoError(format!("Cannot read module '{}': {}", module_key, e))
                        })?;
                        let program = ifa_parser::parse(&source).map_err(|e| {
                            IfaError::Runtime(format!(
                                "Parse error in module '{}': {}",
                                module_key, e
                            ))
                        })?;
                        let export_names = collect_exports_vm(&program);
                        let compiler =
                            ifa_compiler::Compiler::new(file_path.to_string_lossy().as_ref());
                        let bytecode = compiler.compile(&program)?;
                        self.module.module_cache.insert(
                            cache_key.clone(),
                            CachedModule {
                                mtime,
                                size,
                                bytecode: bytecode.clone(),
                                export_names: export_names.clone(),
                            },
                        );
                        (bytecode, export_names)
                    }
                } else {
                    let source = std::fs::read_to_string(&file_path).map_err(|e| {
                        IfaError::IoError(format!("Cannot read module '{}': {}", module_key, e))
                    })?;
                    let program = ifa_parser::parse(&source).map_err(|e| {
                        IfaError::Runtime(format!("Parse error in module '{}': {}", module_key, e))
                    })?;
                    let export_names = collect_exports_vm(&program);
                    let compiler =
                        ifa_compiler::Compiler::new(file_path.to_string_lossy().as_ref());
                    let bytecode = compiler.compile(&program)?;
                    self.module.module_cache.insert(
                        cache_key.clone(),
                        CachedModule {
                            mtime,
                            size,
                            bytecode: bytecode.clone(),
                            export_names: export_names.clone(),
                        },
                    );
                    (bytecode, export_names)
                }
            }
            #[cfg(not(feature = "compiler"))]
            {
                return Err(IfaError::Runtime(format!(
                    "Cannot load source module '{}': compiler feature is disabled in this build",
                    module_key
                )));
            }
        };

        let metadata_matches = if let Some(cached) = self.module.module_cache.get(&cache_key) {
            Some(cached.mtime) == cached_mtime_before && Some(cached.size) == cached_size_before
        } else {
            false
        };

        if self.module.imported.contains(&module_key)
            && metadata_matches
            && let Some(exports) = self.module.module_exports.get(&module_key)
        {
            self.module.import_guard.exit(&module_key);
            return Ok(exports.clone());
        }

        let prev_file = self.module.current_file.take();
        let prev_paths = self.module.resolver.search_paths.clone();
        if let Some(parent) = file_path.parent()
            && !self
                .module
                .resolver
                .search_paths
                .iter()
                .any(|p| p == parent)
        {
            self.module
                .resolver
                .search_paths
                .insert(0, parent.to_path_buf());
        }
        self.module.current_file = Some(file_path.clone());

        let prev_globals = std::mem::take(&mut self.globals);
        self.globals = GlobalState::default();
        self.global_string_slots.clear();
        self.global_names_index.clear();
        let result = self.execute_module(&bytecode);

        let mut exports = std::collections::HashMap::new();
        if result.is_ok() {
            for name in export_names {
                if let Some(val) = self.get_global(&name).cloned() {
                    let export_val = Self::export_value_for_import(&module_key, &name, &val);
                    exports.insert(name.to_string(), export_val);
                }
            }
        }
        let exports_val = IfaValue::map(exports);
        let module_globals = self.globals.clone();

        self.module.current_file = prev_file;
        self.module.resolver.search_paths = prev_paths;
        self.globals = prev_globals;
        self.global_string_slots.clear();
        self.global_names_index.clear();

        self.module.import_guard.exit(&module_key);
        if result.is_ok() {
            self.module.imported.insert(module_key.clone());
            self.module
                .module_exports
                .insert(module_key.clone(), exports_val.clone());
            self.module
                .module_bytecode
                .insert(module_key.clone(), bytecode.clone());
            self.module
                .module_globals
                .insert(module_key.clone(), module_globals);
            // Note: module_cache was already updated during bytecode loading above.
        }
        result.map(|_| exports_val)
    }

    fn execute_module(&mut self, bytecode: &Bytecode) -> IfaResult<()> {
        let saved_ip = self.ctx.ip;
        let saved_halted = self.ctx.halted;
        let saved_stack_len = self.ctx.stack.len();
        let saved_frames_len = self.ctx.frames.len();
        let saved_recovery_len = self.ctx.recovery_stack.len();
        let saved_limits = (self.stack_limit, self.frame_limit, self.opon_size);
        let saved_ikin_mapping = self.ikin.take_mapping();

        let result = (|| {
            self.ctx.ip = 0;
            self.ctx.halted = false;
            self.ikin.load_from_bytecode(bytecode)?;
            let (stack_cap, frame_cap) = bytecode.opon_size.limits();
            self.stack_limit = stack_cap;
            self.frame_limit = frame_cap;
            self.opon_size = bytecode.opon_size;

            ifa_infra::cpu::profile_with_ori(&bytecode.source_name, self.ori_limit, || {
                self.resume_execution(bytecode).map(|_| ())
            })
        })();

        self.ctx.stack.truncate(saved_stack_len);
        self.ctx.frames.truncate(saved_frames_len);
        self.ctx.recovery_stack.truncate(saved_recovery_len);
        self.stack_limit = saved_limits.0;
        self.frame_limit = saved_limits.1;
        self.opon_size = saved_limits.2;
        self.ikin.restore_mapping(saved_ikin_mapping);
        self.ctx.ip = saved_ip;
        self.ctx.halted = saved_halted;

        result
    }

    /// Resume bytecode execution from current instruction pointer
    pub fn resume_execution(&mut self, bytecode: &Bytecode) -> IfaResult<IfaValue> {
        self.ctx.halted = false;
        if self.global_string_slots.len() != bytecode.strings.len() {
            self.reset_global_string_slots(bytecode.strings.len());
        }

        let mut ip = self.ctx.ip;
        while !self.ctx.halted && ip < bytecode.code.len() {
            self.ctx.ip = ip;
            if let Err(e) = self.step(bytecode) {
                if matches!(e, IfaError::Yielded) {
                    return Err(e);
                }

                // The Shield of Ọ̀kànràn: Attempt recovery before crashing
                // Pass reference to avoid cloning unless we actually recover
                if self.attempt_recovery(&e)? {
                    ip = self.ctx.ip;
                    continue;
                }
                self.ctx.ip = ip;

                let enriched_error = if let Some(line) = bytecode.get_line(ip) {
                    let err_str = e.to_string();
                    if err_str.contains(" [at ") {
                        e
                    } else {
                        let loc = format!(" [at {}:{}]", bytecode.source_name, line);
                        match e {
                            IfaError::Runtime(msg) => IfaError::Runtime(format!("{}{}", msg, loc)),
                            IfaError::TypeError { expected, got } => IfaError::TypeError {
                                expected,
                                got: format!("{} {}", got, loc),
                            },
                            IfaError::ArityMismatch { expected, got } => IfaError::Custom(format!(
                                "Arity mismatch: expected {} arguments, got {}{}",
                                expected, got, loc
                            )),
                            IfaError::DivisionByZero(msg) => {
                                IfaError::DivisionByZero(format!("{}{}", msg, loc))
                            }
                            IfaError::Overflow(msg) => {
                                IfaError::Overflow(format!("{}{}", msg, loc))
                            }
                            IfaError::Underflow(msg) => {
                                IfaError::Underflow(format!("{}{}", msg, loc))
                            }
                            IfaError::FileNotFound(msg) => {
                                IfaError::FileNotFound(format!("{}{}", msg, loc))
                            }
                            IfaError::PermissionDenied(msg) => {
                                IfaError::PermissionDenied(format!("{}{}", msg, loc))
                            }
                            IfaError::IoError(msg) => IfaError::IoError(format!("{}{}", msg, loc)),
                            IfaError::ConnectionFailed(msg) => {
                                IfaError::ConnectionFailed(format!("{}{}", msg, loc))
                            }
                            IfaError::SsrfBlocked(msg) => {
                                IfaError::SsrfBlocked(format!("{}{}", msg, loc))
                            }
                            IfaError::UndefinedVariable(msg) => {
                                IfaError::UndefinedVariable(format!("{}{}", msg, loc))
                            }
                            IfaError::UndefinedFunction(msg) => {
                                IfaError::UndefinedFunction(format!("{}{}", msg, loc))
                            }
                            other => IfaError::Custom(format!("{}{}", other, loc)),
                        }
                    }
                } else {
                    e
                };
                return Err(enriched_error);
            }
            ip = self.ctx.ip;
        }

        self.ctx.ip = ip;

        // Return top of stack or Null
        Ok(self.ctx.stack.pop().unwrap_or(IfaValue::null()))
    }

    fn swap_task_state(&mut self, task: &mut Task) {
        std::mem::swap(&mut self.ctx, &mut task.ctx);
    }

    fn call_value(
        &mut self,
        func: IfaValue,
        arg_count: usize,
        is_tail_call: bool,
        bytecode: Option<&Bytecode>,
    ) -> IfaResult<()> {
        let (start_ip, arity, env, async_return, is_str) = match &func {
            IfaValue::Fn(data) => (
                data.start_ip,
                data.arity as usize,
                None,
                data.is_async,
                false,
            ),
            IfaValue::Closure(closure) => (
                closure.fn_data.start_ip,
                closure.fn_data.arity as usize,
                Some(closure.env.clone()),
                closure.fn_data.is_async,
                false,
            ),
            IfaValue::Str(_) => (0, 0, None, false, true),
            _ => {
                return Err(IfaError::TypeError {
                    expected: "Function".into(),
                    got: func.type_name().into(),
                });
            }
        };

        if is_str {
            let s = match func {
                IfaValue::Str(s) => s,
                _ => unreachable!(),
            };
            if let Some((domain_id, method)) = parse_odu_fn_marker(&s) {
                if let Some(bc) = bytecode {
                    let args = self.ctx.stack.split_off(self.ctx.stack.len() - arg_count);
                    let result = self.call_registry(domain_id, &method, args, bc)?;
                    self.push(result)?;
                    return Ok(());
                } else {
                    return Err(IfaError::Runtime(
                        "Cannot call registry without bytecode".into(),
                    ));
                }
            } else if let Some((module_key, function_name)) = parse_module_fn_marker(&s) {
                let args = self.ctx.stack.split_off(self.ctx.stack.len() - arg_count);
                let result = self.invoke_module_function(&module_key, &function_name, args)?;
                self.push(result)?;
                return Ok(());
            } else {
                return Err(IfaError::TypeError {
                    expected: "Function".into(),
                    got: "Str".into(),
                });
            }
        }

        if arg_count != arity {
            return Err(IfaError::ArityMismatch {
                expected: arity,
                got: arg_count,
            });
        }

        if async_return {
            let args = self.ctx.stack.split_off(self.ctx.stack.len() - arg_count);
            let future = self.spawn_task(func, args)?;
            if is_tail_call {
                if let Some(frame) = self.ctx.frames.pop() {
                    if self.ctx.stack.len() > frame.base_ptr {
                        self.ctx.stack.truncate(frame.base_ptr);
                    }
                    self.push(future)?;
                    self.ctx.ip = frame.return_addr;
                } else {
                    self.push(future)?;
                    self.ctx.halted = true;
                }
            } else {
                self.push(future)?;
            }
            return Ok(());
        }

        if is_tail_call {
            if let Some(frame) = self.ctx.frames.last_mut() {
                let base_ptr = frame.base_ptr;
                let stack_len = self.ctx.stack.len();

                // Shift arguments down to base_ptr
                for i in 0..arg_count {
                    self.ctx.stack[base_ptr + i] =
                        self.ctx.stack[stack_len - arg_count + i].clone();
                }
                self.ctx.stack.truncate(base_ptr + arg_count);

                frame.local_count = 0;
                frame.closure_env = env;
                frame.async_return = async_return;
            } else {
                self.push_frame(CallFrame::new(
                    self.ctx.ip,
                    self.ctx.stack.len() - arg_count,
                    env,
                    async_return,
                ))?;
            }
        } else {
            self.push_frame(CallFrame::new(
                self.ctx.ip,
                self.ctx.stack.len() - arg_count,
                env,
                async_return,
            ))?;
        }

        self.ctx.ip = start_ip;
        Ok(())
    }

    fn run_task_slice(
        &mut self,
        task: &mut Task,
        bytecode: &Bytecode,
    ) -> IfaResult<Option<IfaValue>> {
        self.swap_task_state(task);

        if !task.started {
            task.base_depth = self.ctx.frames.len();
            // Push arguments onto the task's stack context
            let arg_count = task.args.len();
            for arg in &task.args {
                self.push(arg.clone())?;
            }
            // Call the function body directly — do NOT use call_value here.
            // call_value would see is_async=true and spawn ANOTHER task, creating
            // a recursive pending chain. The task runner is already the async context:
            // we execute the body synchronously within this task slice.
            let task_func_data = match &task.func {
                IfaValue::Fn(data) => Ok((data.start_ip, data.arity as usize, None)),
                IfaValue::Closure(closure) => Ok((
                    closure.fn_data.start_ip,
                    closure.fn_data.arity as usize,
                    Some(closure.env.clone()),
                )),
                other => Err(other.type_name().to_string()),
            };
            let (start_ip, arity, env) = match task_func_data {
                Ok(data) => data,
                Err(got) => {
                    self.swap_task_state(task);
                    return Err(IfaError::TypeError {
                        expected: "Function".into(),
                        got,
                    });
                }
            };
            if arg_count != arity {
                self.swap_task_state(task);
                return Err(IfaError::ArityMismatch {
                    expected: arity,
                    got: arg_count,
                });
            }
            // async_return: false — the task runner collects the raw return value
            // and stores it in FutureState::Ready. No wrapping needed.
            self.push_frame(CallFrame::new(
                self.ctx.ip,
                self.ctx.stack.len() - arg_count,
                env,
                false,
            ))?;
            self.ctx.ip = start_ip;
            task.started = true;
        }

        loop {
            if self.ctx.frames.len() == task.base_depth {
                let result = self.ctx.stack.pop().unwrap_or(IfaValue::null());
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
                .write()
                .map_err(|_| IfaError::Runtime("Future lock poisoned".into()))?;
            *state = FutureState::Ready(result);
        } else {
            self.task_queue.push_back(task);
        }
        Ok(true)
    }

    pub(crate) fn await_future(
        &mut self,
        val: &IfaValue,
        bytecode: &Bytecode,
    ) -> IfaResult<IfaValue> {
        match val {
            IfaValue::Future(cell) => loop {
                let ready = {
                    let state = cell
                        .read()
                        .map_err(|_| IfaError::Runtime("Future lock poisoned".into()))?;
                    match &*state {
                        ifa_types::value_union::FutureState::Ready(v) => Some(v.clone()),
                        ifa_types::value_union::FutureState::Pending => None,
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
            },
            IfaValue::NativeFuture(cell) => loop {
                let ready = {
                    let state = cell
                        .read()
                        .map_err(|_| IfaError::Runtime("Native future lock poisoned".into()))?;
                    state.clone()
                };
                match ready {
                    ifa_types::value_union::NativeFutureState::Ready(bytes) => {
                        return bincode::deserialize(&bytes).map_err(|e| {
                            IfaError::Runtime(format!("NativeFuture deserialize failed: {}", e))
                        });
                    }
                    ifa_types::value_union::NativeFutureState::Error(err) => {
                        return Err(IfaError::Runtime(err));
                    }
                    _ => {}
                }
                if !self.poll_one_task(bytecode)? {
                    return Err(IfaError::Runtime(
                        "Future pending with no runnable tasks".into(),
                    ));
                }
            },
            other => Err(IfaError::TypeError {
                expected: "Future".into(),
                got: other.type_name().into(),
            }),
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
        let ori_limit = ctx.vm.ori_limit;
        let result = ifa_infra::cpu::profile_with_ori(method_name, ori_limit, || {
            registry.call(domain_id, method_name, args, &mut ctx)
        });
        self.registry = Some(registry);
        result
    }

    /// E6: Fast path — no string pool; delegates to OduRegistry::call_fast.
    fn call_registry_fast(
        &mut self,
        domain_id: u8,
        method_id: u16,
        args: Vec<IfaValue>,
        bytecode: &Bytecode,
    ) -> IfaResult<IfaValue> {
        let Some(registry) = self.registry.take() else {
            return Err(IfaError::Custom(format!(
                "CallOduFast: no registry attached for domain_id={} method_id={:#06x}",
                domain_id, method_id
            )));
        };
        let mut ctx = VmContext { vm: self, bytecode };
        let ori_limit = ctx.vm.ori_limit;
        let result = ifa_infra::cpu::profile_with_ori("fast_dispatch", ori_limit, || {
            registry.call_fast(domain_id, method_id, args, &mut ctx)
        });
        self.registry = Some(registry);
        result
    }

    pub fn spawn_task(&mut self, func: IfaValue, args: Vec<IfaValue>) -> IfaResult<IfaValue> {
        if self.task_queue.len() >= 10_000 {
            return Err(IfaError::Runtime(
                "Task queue overflow (limit 10,000)".into(),
            ));
        }

        let cell = match IfaValue::future_pending() {
            IfaValue::Future(cell) => cell,
            _ => {
                return Err(IfaError::Runtime(
                    "Internal error: future_pending did not return a Future".into(),
                ));
            }
        };
        let task = Task {
            func,
            args,
            future: cell.clone(),
            ctx: ExecutionContext::default(),
            started: false,
            base_depth: 0,
        };
        self.task_queue.push_back(task);
        Ok(IfaValue::Future(cell))
    }

    /// Attempt to recover from a runtime error using the Shield of Ọ̀kànràn
    fn attempt_recovery(&mut self, error: &IfaError) -> IfaResult<bool> {
        if let Some(frame) = self.ctx.recovery_stack.pop() {
            // If this frame already consumed its catch arm, the only remaining
            // obligation is to run its finally block before propagating outward.
            if !frame.can_catch {
                if let Some(finally_ip) = frame.finally_ip {
                    if self.ctx.stack.len() > frame.stack_depth {
                        self.ctx.stack.truncate(frame.stack_depth);
                    }
                    if self.ctx.frames.len() > frame.call_depth {
                        self.ctx.frames.truncate(frame.call_depth);
                    }
                    self.pending_finally = Some(FinallyResumption::Propagate {
                        error: error.clone(),
                    });
                    self.ctx.ip = finally_ip;
                    return Ok(true);
                }
                return Ok(false);
            }

            // 1. Restore stacks
            if self.ctx.stack.len() > frame.stack_depth {
                self.ctx.stack.truncate(frame.stack_depth); // Drop triggers Ebo cleanup
            }
            if self.ctx.frames.len() > frame.call_depth {
                self.ctx.frames.truncate(frame.call_depth);
            }

            // 2. Convert the trapped control-flow error into the catch binding value.
            // User-thrown values must arrive unchanged; VM/runtime errors still degrade
            // to their display string until structured VM errors are introduced.
            self.push(Self::error_to_catch_value(error))?;

            // Catch has consumed the exception arm. If a finally exists, keep a
            // sentinel frame so return/throw/error from the catch still runs it.
            if frame.finally_ip.is_some() {
                self.ctx.recovery_stack.push(RecoveryFrame {
                    stack_depth: frame.stack_depth,
                    call_depth: frame.call_depth,
                    catch_ip: frame.catch_ip,
                    finally_ip: frame.finally_ip,
                    can_catch: false,
                });
            }
            self.ctx.ip = frame.catch_ip;

            Ok(true) // Recovered
        } else {
            Ok(false) // No shield found, crash
        }
    }

    fn boxed_i64(value: &IfaValue) -> Option<i64> {
        match value {
            IfaValue::Int(i) => Some(*i),
            _ => None,
        }
    }

    /// Execute single instruction (The Step of Iroke)
    pub fn step(&mut self, bytecode: &Bytecode) -> IfaResult<()> {
        let opcode = vm_iroke::tap(self, bytecode)?;

        // OduDomain Capability Enforcement for effectful opcodes bypassing CallOdu
        if let Some(domain) = ifa_types::domain::OduDomain::classify_effect(opcode) {
            if let Some(registry) = &self.registry {
                if let Some(id) = domain.dispatch_id() {
                    registry.check_effect(id)?;
                }
            }
        }

        match opcode {
            OpCode::PushNull => self.push(IfaValue::null())?,
            OpCode::PushTrue => self.push(IfaValue::bool(true))?,
            OpCode::PushFalse => self.push(IfaValue::bool(false))?,
            OpCode::PushList => self.push(IfaValue::list(Vec::new()))?,
            OpCode::PushMap => self.push(IfaValue::map(HashMap::new()))?,
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
                let arc = self.ikin.consult_string(idx).ok_or_else(|| {
                    IfaError::Custom("Invalid string constant index in Ikin".into())
                })?;
                self.push(IfaValue::Str(Box::new(arc.to_string())))?;
            }
            OpCode::Pop => {
                self.pop()?;
            }
            OpCode::Dup => {
                let value = self.peek()?.clone();
                self.push(value)?;
            }
            OpCode::Swap => {
                let len = self.ctx.stack.len();
                if len < 2 {
                    return Err(IfaError::StackUnderflow);
                }
                self.ctx.stack.swap(len - 1, len - 2);
            }
            OpCode::Push => {
                let idx = self.read_u32(bytecode)? as usize;
                let value = bytecode.constants.get(idx).cloned().ok_or_else(|| {
                    IfaError::Custom(format!("Invalid constant pool index {}", idx))
                })?;
                self.push(value)?;
            }
            OpCode::LoadUpvalue => {
                let slot = self.read_u16(bytecode)? as usize;
                let env = self
                    .ctx
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
                    .try_lock()
                    .map_err(|_| IfaError::Runtime("Upvalue lock failed".into()))?
                    .clone();
                self.push(value)?;
            }
            OpCode::StoreUpvalue => {
                let slot = self.read_u16(bytecode)? as usize;
                let value = self.pop()?;
                let env = self
                    .ctx
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
                    .try_lock()
                    .map_err(|_| IfaError::Runtime("Upvalue lock failed".into()))? = value;
            }
            OpCode::MoveUpvalue => {
                let slot = self.read_u16(bytecode)? as usize;
                let env = self
                    .ctx
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

                let value = {
                    let mut lock = cell
                        .try_lock()
                        .map_err(|_| IfaError::Runtime("Upvalue lock failed".into()))?;
                    std::mem::replace(&mut *lock, IfaValue::Moved)
                };
                if matches!(value, IfaValue::Moved) {
                    return Err(IfaError::UndefinedVariable("Use of moved variable".into()));
                }
                self.push(value)?;
            }
            OpCode::LoadLocal => {
                let idx = self.read_u16(bytecode)? as usize;
                let base = self.ctx.frames.last().map(|f| f.base_ptr).unwrap_or(0);

                let slot = self
                    .ctx
                    .stack
                    .get(base + idx)
                    .cloned()
                    .ok_or_else(|| IfaError::UndefinedVariable(format!("<local:{}>", idx)))?;

                match slot {
                    IfaValue::Moved => {
                        return Err(IfaError::UndefinedVariable("Use of moved variable".into()));
                    }
                    IfaValue::Upvalue(cell) => {
                        let value = cell
                            .try_lock()
                            .map_err(|_| IfaError::Runtime("Upvalue lock failed".into()))?
                            .clone();
                        if matches!(value, IfaValue::Moved) {
                            return Err(IfaError::UndefinedVariable(
                                "Use of moved variable".into(),
                            ));
                        }
                        self.push(value)?;
                    }
                    value => self.push(value)?,
                }
            }
            OpCode::StoreLocal => {
                let idx = self.read_u16(bytecode)? as usize;
                let value = self.pop()?;
                let base = self.ctx.frames.last().map(|f| f.base_ptr).unwrap_or(0);
                if base + idx >= self.ctx.stack.len() {
                    return Err(IfaError::UndefinedVariable(format!("<local:{}>", idx)));
                }
                if let IfaValue::Upvalue(cell) = &self.ctx.stack[base + idx] {
                    *cell
                        .try_lock()
                        .map_err(|_| IfaError::Runtime("Upvalue lock failed".into()))? = value;
                } else {
                    self.ctx.stack[base + idx] = value;
                }
            }
            OpCode::MoveLocal => {
                let idx = self.read_u16(bytecode)? as usize;
                let base = self.ctx.frames.last().map(|f| f.base_ptr).unwrap_or(0);
                if base + idx >= self.ctx.stack.len() {
                    return Err(IfaError::UndefinedVariable(format!("<local:{}>", idx)));
                }

                let slot = std::mem::replace(&mut self.ctx.stack[base + idx], IfaValue::Moved);

                match slot {
                    IfaValue::Moved => {
                        return Err(IfaError::UndefinedVariable("Use of moved variable".into()));
                    }
                    IfaValue::Upvalue(cell) => {
                        let value = {
                            let mut lock = cell
                                .try_lock()
                                .map_err(|_| IfaError::Runtime("Upvalue lock failed".into()))?;
                            std::mem::replace(&mut *lock, IfaValue::Moved)
                        };
                        if matches!(value, IfaValue::Moved) {
                            return Err(IfaError::UndefinedVariable(
                                "Use of moved variable".into(),
                            ));
                        }
                        self.push(value)?;
                    }
                    value => self.push(value)?,
                }
            }
            OpCode::LoadGlobal => {
                let idx = self.read_u16(bytecode)? as usize;
                self.load_global_slot(bytecode, idx)?;
            }
            OpCode::StoreGlobal => {
                let idx = self.read_u16(bytecode)? as usize;
                self.store_global_slot(bytecode, idx)?;
            }
            OpCode::MoveGlobal => {
                let idx = self.read_u16(bytecode)? as usize;
                self.move_global_slot(bytecode, idx)?;
            }
            OpCode::Jump => {
                self.ctx.ip = self.read_u32(bytecode)? as usize;
            }
            OpCode::JumpIfFalse => {
                let offset = self.read_u32(bytecode)? as usize;
                let cond = self.pop()?;
                if !cond.is_truthy() {
                    self.ctx.ip = offset;
                    // Leaving a loop naturally — pop its frame
                    self.ctx.loop_stack.pop();
                } else {
                    // Entering (or re-entering) a loop body; record loop bounds:
                    // continue_ip = ip before the JumpIfFalse (loop header)
                    // break_ip    = offset (the exit target of the JumpIfFalse)
                    // Only push if this JumpIfFalse is actually a loop guard.
                    // Heuristic: if the back-edge Jump at end of loop points before here,
                    // we know we're in a loop. For correctness we always push here and pop
                    // on exit — extra frames from non-loop JumpIfFalse are harmless because
                    // Break/Continue can only appear inside compiled loop bodies.
                    let continue_ip = self.ctx.ip.saturating_sub(5); // re-eval condition
                    self.ctx.loop_stack.push((continue_ip, offset));
                }
            }
            OpCode::JumpIfTrue => {
                let offset = self.read_u32(bytecode)? as usize;
                let cond = self.pop()?;
                if cond.is_truthy() {
                    self.ctx.ip = offset;
                }
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

                let base = self.ctx.frames.last().map(|f| f.base_ptr).unwrap_or(0);
                let parent_env = self.ctx.frames.last().and_then(|f| f.closure_env.clone());
                let mut env: Vec<UpvalueCell> = Vec::with_capacity(capture_count);

                for _ in 0..capture_count {
                    let kind = self.read_u8(bytecode)?;
                    let idx = self.read_u16(bytecode)? as usize;

                    match kind {
                        0 => {
                            let slot_index = base + idx;
                            let slot =
                                self.ctx.stack.get(slot_index).cloned().ok_or_else(|| {
                                    IfaError::UndefinedVariable(format!("<local:{}>", idx))
                                })?;

                            let cell = match slot {
                                IfaValue::Upvalue(cell) => cell,
                                value => {
                                    let cell: UpvalueCell =
                                        ifa_types::gc::IfaGc::new(Mutex::new(value));
                                    if slot_index < self.ctx.stack.len() {
                                        self.ctx.stack[slot_index] =
                                            IfaValue::Upvalue(cell.clone());
                                    }
                                    cell
                                }
                            };

                            env.push(cell);
                        }
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

                self.push(IfaValue::Closure(ifa_types::gc::IfaGc::new(ClosureData {
                    fn_data,
                    env: Arc::new(env),
                })))?;
            }

            OpCode::Call => {
                self.dispatch_call(bytecode)?;
            }

            OpCode::TailCall => {
                self.dispatch_tail_call(bytecode)?;
            }

            OpCode::Return => {
                self.dispatch_return()?;
            }

            OpCode::Await => {
                let value = self.pop()?;
                match value {
                    IfaValue::Future(_) | IfaValue::NativeFuture(_) => {
                        let v = self.await_future(&value, bytecode)?;
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

            // E6: CallOduFast — statically resolved dispatch; no string pool access.
            // Encoding: [domain_id: u8 | method_id_hi: u8 | method_id_lo: u8 | arity: u8]
            OpCode::CallOduFast => {
                let domain_id = self.read_u8(bytecode)?;
                let method_id_hi = self.read_u8(bytecode)?;
                let method_id_lo = self.read_u8(bytecode)?;
                let method_id = ((method_id_hi as u16) << 8) | (method_id_lo as u16);
                let arity = self.read_u8(bytecode)?;

                let mut args = Vec::with_capacity(arity as usize);
                for _ in 0..arity {
                    args.push(self.pop()?);
                }
                args.reverse();

                let result = self.call_registry_fast(domain_id, method_id, args, bytecode)?;
                self.push(result)?;
            }

            OpCode::ParallelFor => {
                self.dispatch_parallel_for(bytecode)?;
            }

            OpCode::EpochBegin => {
                let name_val = self.pop()?;
                let name = name_val.to_string();
                self.opon.begin_epoch(&name);
                if let Some(epoch) = self.opon.current_epoch() {
                    let guard = crate::ajose::EpochCleanupGuard::new(epoch.cleanups.clone());
                    self.epoch_guards.push(guard);
                }
            }

            OpCode::EpochEnd => {
                self.epoch_guards.pop();
                self.opon
                    .end_epoch()
                    .map_err(|e| IfaError::Runtime(format!("Ẹbọ epoch error: {}", e)))?;
            }

            OpCode::CallMethod => {
                self.dispatch_call_method(bytecode)?;
            }
            OpCode::GetIndex => {
                let index = self.pop()?;
                let collection = self.pop()?;

                match collection {
                    IfaValue::Map(m) => {
                        let key = match index {
                            IfaValue::Str(s) => ifa_types::CompactString::new(&*s),
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
                        let raw = match index {
                            IfaValue::Int(i) => i,
                            _ => {
                                return Err(IfaError::TypeError {
                                    expected: "Int".into(),
                                    got: index.type_name().into(),
                                });
                            }
                        };
                        let len = l.len() as i64;
                        // Negative index: -1 = last, -2 = second-to-last, etc.
                        let idx = if raw < 0 { len + raw } else { raw };
                        if idx < 0 || idx >= len {
                            // OOB → null (consistent with Map miss)
                            self.push(IfaValue::null())?;
                        } else {
                            self.push(l[idx as usize].clone())?;
                        }
                    }
                    IfaValue::Str(s) => {
                        let raw = match index {
                            IfaValue::Int(i) => i,
                            _ => {
                                return Err(IfaError::TypeError {
                                    expected: "Int".into(),
                                    got: index.type_name().into(),
                                });
                            }
                        };
                        let chars: Vec<char> = s.chars().collect();
                        let len = chars.len() as i64;
                        let idx = if raw < 0 { len + raw } else { raw };
                        if idx < 0 || idx >= len {
                            return Err(IfaError::Runtime(format!(
                                "String index {} out of bounds (len {})",
                                raw, len
                            )));
                        }
                        self.push(IfaValue::str(chars[idx as usize].to_string()))?;
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
                        let vec = ifa_types::gc::IfaGc::make_mut(vec_arc);
                        if i >= vec.len() {
                            return Err(IfaError::Runtime("Index out of bounds".into()));
                        }
                        vec[i] = val;
                    }
                    IfaValue::Map(ref mut map_arc) => {
                        let k = match index {
                            IfaValue::Str(s) => ifa_types::CompactString::new(&*s),
                            _ => {
                                return Err(IfaError::TypeError {
                                    expected: "Str".into(),
                                    got: index.type_name().into(),
                                });
                            }
                        };
                        let map = ifa_types::gc::IfaGc::make_mut(map_arc);
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

            OpCode::BuildSet => {
                let count = self.read_u8(bytecode)? as usize;
                #[allow(clippy::mutable_key_type)]
                let mut set = std::collections::HashSet::with_capacity(count);
                for _ in 0..count {
                    set.insert(self.pop()?);
                }
                self.push(IfaValue::set(set))?;
            }

            OpCode::SetAdd => {
                let value = self.pop()?;
                let set_val = self.pop()?;
                if let IfaValue::Set(mut set_arc) = set_val {
                    #[allow(clippy::mutable_key_type)]
                    let set = Arc::make_mut(&mut set_arc);
                    set.insert(value);
                    self.push(IfaValue::Set(set_arc))?;
                } else {
                    return Err(IfaError::TypeError {
                        expected: "Set".into(),
                        got: set_val.type_name().into(),
                    });
                }
            }

            OpCode::SetHas => {
                let value = self.pop()?;
                let set_val = self.pop()?;
                if let IfaValue::Set(set_arc) = &set_val {
                    self.push(IfaValue::Bool(set_arc.contains(&value)))?;
                } else {
                    return Err(IfaError::TypeError {
                        expected: "Set".into(),
                        got: set_val.type_name().into(),
                    });
                }
            }

            OpCode::SetRemove => {
                let value = self.pop()?;
                let set_val = self.pop()?;
                if let IfaValue::Set(mut set_arc) = set_val {
                    #[allow(clippy::mutable_key_type)]
                    let set = Arc::make_mut(&mut set_arc);
                    set.remove(&value);
                    self.push(IfaValue::Set(set_arc))?;
                } else {
                    return Err(IfaError::TypeError {
                        expected: "Set".into(),
                        got: set_val.type_name().into(),
                    });
                }
            }
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
                return Err(IfaError::Custom(
                    "DefineClass opcode reached at runtime. \
                     Class-based OOP has been formally removed from Ifá-Lang. \
                     Recompile your source with `ifa build` — the compiler will \
                     guide you toward Map + Domain Protocol design instead."
                        .into(),
                ));
            }

            OpCode::Halt => {
                self.ctx.halted = true;
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
                        let val = if addr >= 0x4000_0000 {
                            IfaValue::int(0)
                        } else {
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

            OpCode::Store8 => {
                let ptr = self.pop()?;
                let val = self.pop()?;
                match ptr {
                    IfaValue::Int(addr_i) => {
                        let addr = addr_i as usize;
                        if addr >= 0x4000_0000 {
                            self.opon.record("MMIO", "write", &val);
                        } else {
                            self.opon
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
                            self.opon
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
                            self.opon
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
                            self.opon
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
            OpCode::And => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (a, b) {
                    (IfaValue::Int(i1), IfaValue::Int(i2)) => {
                        self.push(IfaValue::int(i1 & i2))?;
                    }
                    (IfaValue::Bool(b1), IfaValue::Bool(b2)) => {
                        self.push(IfaValue::bool(b1 && b2))?;
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
                    (IfaValue::Int(i1), IfaValue::Int(i2)) => {
                        self.push(IfaValue::int(i1 | i2))?;
                    }
                    (IfaValue::Bool(b1), IfaValue::Bool(b2)) => {
                        self.push(IfaValue::bool(b1 || b2))?;
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
                    (IfaValue::Int(i1), IfaValue::Int(i2)) => {
                        self.push(IfaValue::int(i1 ^ i2))?;
                    }
                    (IfaValue::Bool(b1), IfaValue::Bool(b2)) => {
                        self.push(IfaValue::bool(b1 ^ b2))?;
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
                    IfaValue::Str(s) => {
                        let len = self.ikin.string_len(&s) as i64;
                        self.push(IfaValue::int(len))?
                    }
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
                    IfaValue::Int(i) => {
                        self.push(IfaValue::int(!i))?;
                    }
                    IfaValue::Bool(b) => {
                        self.push(IfaValue::bool(!b))?;
                    }
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
                if let (Some(val), Some(shift)) = (Self::boxed_i64(&a), Self::boxed_i64(&b)) {
                    self.push(IfaValue::int(val << shift))?;
                } else {
                    return Err(IfaError::TypeError {
                        expected: "Int".into(),
                        got: a.type_name().into(),
                    });
                }
            }
            OpCode::Shr => {
                let b = self.pop()?;
                let a = self.pop()?;
                if let (Some(val), Some(shift)) = (Self::boxed_i64(&a), Self::boxed_i64(&b)) {
                    self.push(IfaValue::int(val >> shift))?;
                } else {
                    return Err(IfaError::TypeError {
                        expected: "Int".into(),
                        got: a.type_name().into(),
                    });
                }
            }
            // Arithmetic (signed) right shift: preserves sign bit, unlike logical Shr.
            // Stack: [a: Int, b: Int] -> [a >> b: Int]
            OpCode::Sar => {
                let b = self.pop()?;
                let a = self.pop()?;
                if let (Some(val), Some(shift)) = (Self::boxed_i64(&a), Self::boxed_i64(&b)) {
                    // Rust's >> on i64 is already arithmetic (sign-extending), so this is correct.
                    let shift = shift.clamp(0, 63) as u32;
                    self.push(IfaValue::int(val >> shift))?;
                } else {
                    return Err(IfaError::TypeError {
                        expected: "Int".into(),
                        got: a.type_name().into(),
                    });
                }
            }
            // Append: in-place CoW push onto a List.
            // Stack: [list, val] -> [list]  (same list reference, mutated via CoW)
            OpCode::Append => {
                let val = self.pop()?;
                let mut list = self.pop()?;
                match list {
                    IfaValue::List(ref mut arc) => {
                        ifa_types::gc::IfaGc::make_mut(arc).push(val);
                        self.push(list)?;
                    }
                    _ => {
                        return Err(IfaError::TypeError {
                            expected: "List".into(),
                            got: list.type_name().into(),
                        });
                    }
                }
            }
            // GetField: load a named field from a Map-as-object.
            // Encoding: [2-byte string-pool index]
            // Stack: [obj: Map] -> [val]
            OpCode::GetField => {
                let name_idx = self.read_u16(bytecode)? as usize;
                let field_name = bytecode.strings.get(name_idx).cloned().ok_or_else(|| {
                    IfaError::Custom(format!("GetField: invalid string pool index {}", name_idx))
                })?;
                let obj = self.pop()?;
                match obj {
                    IfaValue::Map(ref m) => {
                        let key = ifa_types::CompactString::new(&field_name);
                        let v = m.get(&key).cloned().unwrap_or(IfaValue::null());
                        self.push(v)?;
                    }
                    _ => {
                        return Err(IfaError::TypeError {
                            expected: "Map (object)".into(),
                            got: obj.type_name().into(),
                        });
                    }
                }
            }
            // SetField: store a named field on a Map-as-object (CoW).
            // Encoding: [2-byte string-pool index]
            // Stack: [obj: Map, val] -> [obj]  (obj re-pushed for chaining)
            OpCode::SetField => {
                let name_idx = self.read_u16(bytecode)? as usize;
                let field_name = bytecode.strings.get(name_idx).cloned().ok_or_else(|| {
                    IfaError::Custom(format!("SetField: invalid string pool index {}", name_idx))
                })?;
                let val = self.pop()?;
                let mut obj = self.pop()?;
                match obj {
                    IfaValue::Map(ref mut arc) => {
                        let key = ifa_types::CompactString::new(&field_name);
                        ifa_types::gc::IfaGc::make_mut(arc).insert(key, val);
                        self.push(obj)?;
                    }
                    _ => {
                        return Err(IfaError::TypeError {
                            expected: "Map (object)".into(),
                            got: obj.type_name().into(),
                        });
                    }
                }
            }
            OpCode::SetOriLimit => {
                let limit = self.read_u64(bytecode)?;
                self.ori_limit = Some(limit);
            }
            OpCode::ToInt => {
                let val = self.pop()?;
                match val {
                    IfaValue::Int(i) => self.push(IfaValue::int(i))?,
                    IfaValue::Float(f) => self.push(IfaValue::int(f as i64))?,
                    IfaValue::Bool(b) => self.push(IfaValue::int(if b { 1 } else { 0 }))?,
                    IfaValue::Str(s) => {
                        let n = s.trim().parse::<i64>().map_err(|_| IfaError::TypeError {
                            expected: "numeric String".into(),
                            got: format!("\"{}\"", s),
                        })?;
                        self.push(IfaValue::int(n))?;
                    }
                    other => {
                        return Err(IfaError::TypeError {
                            expected: "Int, Float, Bool, or numeric String".into(),
                            got: other.type_name().into(),
                        })
                    }
                }
            }
            OpCode::ToFloat => {
                let val = self.pop()?;
                match val {
                    IfaValue::Int(i) => self.push(IfaValue::float(i as f64))?,
                    IfaValue::Float(f) => self.push(IfaValue::float(f))?,
                    other => {
                        return Err(IfaError::TypeError {
                            expected: "Int or Float".into(),
                            got: other.type_name().into(),
                        })
                    }
                }
            }
            OpCode::ToString => {
                let val = self.pop()?;
                self.push(IfaValue::str(val.to_string()))?;
            }
            OpCode::Add => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (a, b) {
                    (IfaValue::Int(ia), IfaValue::Int(ib)) => match ia.checked_add(ib) {
                        Some(r) => self.push(IfaValue::int(r))?,
                        None => {
                            return Err(IfaError::Overflow(format!(
                                "{} + {} (use explicit Float conversion for large numbers)",
                                ia, ib
                            )))
                        }
                    },
                    (IfaValue::Float(fa), IfaValue::Float(fb)) => {
                        self.push(IfaValue::float(fa + fb))?
                    }
                    (IfaValue::Int(ia), IfaValue::Float(fb)) => {
                        self.push(IfaValue::float(ia as f64 + fb))?
                    }
                    (IfaValue::Float(fa), IfaValue::Int(ib)) => {
                        self.push(IfaValue::float(fa + ib as f64))?
                    }
                    (IfaValue::Str(sa), IfaValue::Str(sb)) => {
                        let mut s = String::with_capacity(sa.len() + sb.len());
                        s.push_str(&sa);
                        s.push_str(&sb);
                        self.push(IfaValue::str(s))?
                    }
                    (IfaValue::Str(sa), other) => {
                        let sb = other.to_string();
                        let mut s = String::with_capacity(sa.len() + sb.len());
                        s.push_str(&sa);
                        s.push_str(&sb);
                        self.push(IfaValue::str(s))?
                    }
                    (other, IfaValue::Str(sb)) => {
                        let sa = other.to_string();
                        let mut s = String::with_capacity(sa.len() + sb.len());
                        s.push_str(&sa);
                        s.push_str(&sb);
                        self.push(IfaValue::str(s))?
                    }
                    (a, b) => {
                        return Err(IfaError::TypeError {
                            expected: "Int, Float, or String".into(),
                            got: format!("{} + {}", a.type_name(), b.type_name()),
                        });
                    }
                }
            }
            OpCode::Concat => {
                let rhs = self.pop()?;
                let lhs = self.pop()?;
                match (lhs, rhs) {
                    (IfaValue::Str(l), IfaValue::Str(r)) => {
                        let mut s = String::with_capacity(l.len() + r.len());
                        s.push_str(&l);
                        s.push_str(&r);
                        self.push(IfaValue::str(s))?;
                    }
                    (l, r) => {
                        return Err(IfaError::TypeError {
                            expected: "Str ++ Str".into(),
                            got: format!("{} ++ {}", l.type_name(), r.type_name()),
                        });
                    }
                }
            }
            OpCode::Sub => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (a, b) {
                    (IfaValue::Int(ia), IfaValue::Int(ib)) => match ia.checked_sub(ib) {
                        Some(r) => self.push(IfaValue::int(r))?,
                        None => {
                            return Err(IfaError::Overflow(format!(
                                "{} - {} (use explicit Float conversion for large numbers)",
                                ia, ib
                            )))
                        }
                    },
                    (IfaValue::Float(fa), IfaValue::Float(fb)) => {
                        self.push(IfaValue::float(fa - fb))?
                    }
                    (IfaValue::Int(ia), IfaValue::Float(fb)) => {
                        self.push(IfaValue::float(ia as f64 - fb))?
                    }
                    (IfaValue::Float(fa), IfaValue::Int(ib)) => {
                        self.push(IfaValue::float(fa - ib as f64))?
                    }
                    (a, b) => {
                        return Err(IfaError::TypeError {
                            expected: "Int/Float".into(),
                            got: format!("{} - {}", a.type_name(), b.type_name()),
                        });
                    }
                }
            }
            OpCode::Mul => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (a, b) {
                    (IfaValue::Int(ia), IfaValue::Int(ib)) => match ia.checked_mul(ib) {
                        Some(r) => self.push(IfaValue::int(r))?,
                        None => {
                            return Err(IfaError::Overflow(format!(
                                "{} * {} (use explicit Float conversion for large numbers)",
                                ia, ib
                            )))
                        }
                    },
                    (IfaValue::Float(fa), IfaValue::Float(fb)) => {
                        self.push(IfaValue::float(fa * fb))?
                    }
                    (IfaValue::Int(ia), IfaValue::Float(fb)) => {
                        self.push(IfaValue::float(ia as f64 * fb))?
                    }
                    (IfaValue::Float(fa), IfaValue::Int(ib)) => {
                        self.push(IfaValue::float(fa * ib as f64))?
                    }
                    (a, b) => {
                        return Err(IfaError::TypeError {
                            expected: "Int/Float".into(),
                            got: format!("{} * {}", a.type_name(), b.type_name()),
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
                        match ia.checked_div(ib) {
                            Some(r) => self.push(IfaValue::int(r))?,
                            None => {
                                return Err(IfaError::Overflow(format!(
                                    "{} / {} (use explicit Float conversion for large numbers)",
                                    ia, ib
                                )))
                            }
                        }
                    }
                    (IfaValue::Float(fa), IfaValue::Float(fb)) => {
                        if fb == 0.0 {
                            return Err(IfaError::DivisionByZero("Cannot divide by zero".into()));
                        }
                        self.push(IfaValue::float(fa / fb))?
                    }
                    (IfaValue::Int(ia), IfaValue::Float(fb)) => {
                        if fb == 0.0 {
                            return Err(IfaError::DivisionByZero("Cannot divide by zero".into()));
                        }
                        self.push(IfaValue::float(ia as f64 / fb))?
                    }
                    (IfaValue::Float(fa), IfaValue::Int(ib)) => {
                        if ib == 0 {
                            return Err(IfaError::DivisionByZero("Cannot divide by zero".into()));
                        }
                        self.push(IfaValue::float(fa / ib as f64))?
                    }
                    (a, b) => {
                        return Err(IfaError::TypeError {
                            expected: "Int/Float".into(),
                            got: format!("{} / {}", a.type_name(), b.type_name()),
                        });
                    }
                }
            }
            OpCode::ToBool => {
                let val = self.pop()?;
                self.push(IfaValue::bool(val.is_truthy()))?;
            }
            OpCode::Neg => {
                let val = self.pop()?;
                match val {
                    IfaValue::Int(i) => self.push(IfaValue::int(-i))?,
                    IfaValue::Float(f) => self.push(IfaValue::float(-f))?,
                    _ => return Err(IfaError::Runtime("Invalid type for negation".into())),
                }
            }
            OpCode::Pow => {
                let exp = self.pop()?;
                let base = self.pop()?;
                match (base, exp) {
                    (IfaValue::Int(b), IfaValue::Int(e)) => {
                        let e_u32 = u32::try_from(e).map_err(|_| {
                            IfaError::Runtime("Pow: negative exponent with integer base".into())
                        })?;
                        self.push(IfaValue::int(b.checked_pow(e_u32).ok_or_else(|| {
                            IfaError::Runtime("Pow: Integer overflow".into())
                        })?))?;
                    }
                    (IfaValue::Float(b), IfaValue::Float(e)) => {
                        self.push(IfaValue::float(b.powf(e)))?;
                    }
                    (IfaValue::Int(b), IfaValue::Float(e)) => {
                        self.push(IfaValue::float((b as f64).powf(e)))?;
                    }
                    (IfaValue::Float(b), IfaValue::Int(e)) => {
                        let e_i32 = i32::try_from(e).map_err(|_| {
                            IfaError::Runtime("Pow: exponent out of bounds for float".into())
                        })?;
                        self.push(IfaValue::float(b.powi(e_i32)))?;
                    }
                    _ => return Err(IfaError::Runtime("Invalid types for power".into())),
                }
            }
            OpCode::Mod => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (a, b) {
                    (IfaValue::Int(ia), IfaValue::Int(ib)) => {
                        if ib == 0 {
                            return Err(IfaError::DivisionByZero("Modulus by zero".into()));
                        }
                        self.push(IfaValue::int(ia % ib))?;
                    }
                    (a, b) => {
                        return Err(IfaError::TypeError {
                            expected: "Int % Int".into(),
                            got: format!("{} % {}", a.type_name(), b.type_name()),
                        });
                    }
                }
            }
            OpCode::Lt | OpCode::Le | OpCode::Gt | OpCode::Ge => {
                let b = self.pop()?;
                let a = self.pop()?;
                let a_type = a.type_name().to_string();
                let b_type = b.type_name().to_string();
                let result = match opcode {
                    OpCode::Lt => match (a, b) {
                        (IfaValue::Int(ia), IfaValue::Int(ib)) => ia < ib,
                        (IfaValue::Float(fa), IfaValue::Float(fb)) => fa < fb,
                        (IfaValue::Int(ia), IfaValue::Float(fb)) => (ia as f64) < fb,
                        (IfaValue::Float(fa), IfaValue::Int(ib)) => fa < (ib as f64),
                        (IfaValue::Str(sa), IfaValue::Str(sb)) => sa < sb,
                        _ => {
                            return Err(IfaError::TypeError {
                                expected: "Int/Float/String".into(),
                                got: format!("{} and {}", a_type, b_type),
                            });
                        }
                    },
                    OpCode::Le => match (a, b) {
                        (IfaValue::Int(ia), IfaValue::Int(ib)) => ia <= ib,
                        (IfaValue::Float(fa), IfaValue::Float(fb)) => fa <= fb,
                        (IfaValue::Int(ia), IfaValue::Float(fb)) => (ia as f64) <= fb,
                        (IfaValue::Float(fa), IfaValue::Int(ib)) => fa <= (ib as f64),
                        (IfaValue::Str(sa), IfaValue::Str(sb)) => sa <= sb,
                        _ => {
                            return Err(IfaError::TypeError {
                                expected: "Int/Float/String".into(),
                                got: format!("{} and {}", a_type, b_type),
                            });
                        }
                    },
                    OpCode::Gt => match (a, b) {
                        (IfaValue::Int(ia), IfaValue::Int(ib)) => ia > ib,
                        (IfaValue::Float(fa), IfaValue::Float(fb)) => fa > fb,
                        (IfaValue::Int(ia), IfaValue::Float(fb)) => (ia as f64) > fb,
                        (IfaValue::Float(fa), IfaValue::Int(ib)) => fa > (ib as f64),
                        (IfaValue::Str(sa), IfaValue::Str(sb)) => sa > sb,
                        _ => {
                            return Err(IfaError::TypeError {
                                expected: "Int/Float/String".into(),
                                got: format!("{} and {}", a_type, b_type),
                            });
                        }
                    },
                    OpCode::Ge => match (a, b) {
                        (IfaValue::Int(ia), IfaValue::Int(ib)) => ia >= ib,
                        (IfaValue::Float(fa), IfaValue::Float(fb)) => fa >= fb,
                        (IfaValue::Int(ia), IfaValue::Float(fb)) => (ia as f64) >= fb,
                        (IfaValue::Float(fa), IfaValue::Int(ib)) => fa >= (ib as f64),
                        (IfaValue::Str(sa), IfaValue::Str(sb)) => sa >= sb,
                        _ => {
                            return Err(IfaError::TypeError {
                                expected: "Int/Float/String".into(),
                                got: format!("{} and {}", a_type, b_type),
                            });
                        }
                    },
                    _ => return Err(IfaError::Runtime("Invalid comparison opcode".into())),
                };
                self.push(IfaValue::bool(result))?;
            }
            OpCode::Eq => {
                let b = self.pop()?;
                let a = self.pop()?;
                let result = match (&a, &b) {
                    (IfaValue::Int(ia), IfaValue::Float(fb)) => (*ia as f64) == *fb,
                    (IfaValue::Float(fa), IfaValue::Int(ib)) => *fa == (*ib as f64),
                    _ => a == b,
                };
                self.push(IfaValue::bool(result))?;
            }
            OpCode::Ne => {
                let b = self.pop()?;
                let a = self.pop()?;
                let result = match (&a, &b) {
                    (IfaValue::Int(ia), IfaValue::Float(fb)) => (*ia as f64) != *fb,
                    (IfaValue::Float(fa), IfaValue::Int(ib)) => *fa != (*ib as f64),
                    _ => a != b,
                };
                self.push(IfaValue::bool(result))?;
            }
            OpCode::TryBegin => {
                let catch_ip = self.read_u32(bytecode)? as usize;
                self.ctx.recovery_stack.push(RecoveryFrame {
                    stack_depth: self.ctx.stack.len(),
                    call_depth: self.ctx.frames.len(),
                    catch_ip,
                    finally_ip: None,
                    can_catch: true,
                });
            }
            OpCode::TryEnd => {
                self.ctx.recovery_stack.pop();
            }
            OpCode::Throw => {
                let err_val = self.pop()?;
                if let Some(finally_ip) = self.ctx.recovery_stack.last().and_then(|f| f.finally_ip)
                {
                    self.ctx.recovery_stack.pop();
                    self.pending_finally = Some(FinallyResumption::Propagate {
                        error: IfaError::UserError(err_val.freeze()?),
                    });
                    self.ctx.ip = finally_ip;
                    return Ok(());
                }
                return Err(IfaError::UserError(err_val.freeze()?));
            }
            OpCode::FinallyBegin => {
                let finally_ip = self.read_u32(bytecode)? as usize;
                if let Some(frame) = self.ctx.recovery_stack.last_mut() {
                    frame.finally_ip = Some(finally_ip);
                }
            }
            OpCode::FinallyEnd => match self.pending_finally.take() {
                Some(FinallyResumption::Return { return_value }) => {
                    if let Some(finally_ip) =
                        self.ctx.recovery_stack.last().and_then(|f| f.finally_ip)
                    {
                        self.ctx.recovery_stack.pop();
                        self.pending_finally = Some(FinallyResumption::Return { return_value });
                        self.ctx.ip = finally_ip;
                        return Ok(());
                    }
                    let frame = self
                        .ctx
                        .frames
                        .pop()
                        .unwrap_or_else(|| CallFrame::new(0, 0, None, false));
                    if self.ctx.stack.len() > frame.base_ptr {
                        self.ctx.stack.truncate(frame.base_ptr);
                    }
                    if frame.async_return {
                        self.push(IfaValue::future_ready(return_value))?;
                    } else {
                        self.push(return_value)?;
                    }
                    self.ctx.ip = frame.return_addr;
                }
                Some(FinallyResumption::Propagate { error }) => {
                    if let Some(finally_ip) =
                        self.ctx.recovery_stack.last().and_then(|f| f.finally_ip)
                    {
                        self.ctx.recovery_stack.pop();
                        self.pending_finally = Some(FinallyResumption::Propagate { error });
                        self.ctx.ip = finally_ip;
                        return Ok(());
                    }
                    return Err(error);
                }
                None => {}
            },
            OpCode::PropagateError => {
                let value = self.pop()?;
                match value {
                    IfaValue::Result(payload) => match *payload {
                        ResultPayload::Ire(ok) => self.push(ok)?,
                        ResultPayload::Ibi(err) => return Err(IfaError::UserError(err.freeze()?)),
                    },
                    other => self.push(other)?,
                }
            }
            OpCode::AssertType => {
                let type_id = self.read_u8(bytecode)?;
                let value = self.peek()?;
                let valid = match type_id {
                    0 => matches!(value, IfaValue::Int(_)),
                    1 => matches!(value, IfaValue::Float(_)),
                    2 => matches!(value, IfaValue::Str(_)),
                    3 => matches!(value, IfaValue::Bool(_)),
                    4 => matches!(value, IfaValue::List(_)),
                    5 => matches!(value, IfaValue::Map(_)),
                    6 => matches!(value, IfaValue::Fn(_) | IfaValue::Closure(_)),
                    255 => true, // Any
                    _ => false,
                };
                if !valid {
                    let expected_str = match type_id {
                        0 => "Int",
                        1 => "Float",
                        2 => "Str",
                        3 => "Bool",
                        4 => "List",
                        5 => "Map",
                        6 => "Function",
                        _ => "Any",
                    };
                    return Err(IfaError::TypeError {
                        expected: expected_str.into(),
                        got: value.type_name().to_string(),
                    });
                }
            }
        }
        Ok(())
    }

    fn dispatch_call(&mut self, bytecode: &Bytecode) -> IfaResult<()> {
        let arg_count = self.read_u8(bytecode)? as usize;
        let stack_len = self.ctx.stack.len();
        if stack_len < arg_count + 1 {
            return Err(IfaError::Runtime("Stack underflow in dispatch_call".into()));
        }
        let func = self.ctx.stack.swap_remove(stack_len - arg_count - 1);
        self.call_value(func, arg_count, false, Some(bytecode))
    }

    fn dispatch_tail_call(&mut self, bytecode: &Bytecode) -> IfaResult<()> {
        let arg_count = self.read_u8(bytecode)? as usize;
        let stack_len = self.ctx.stack.len();
        if stack_len < arg_count + 1 {
            return Err(IfaError::Runtime(
                "Stack underflow in dispatch_tail_call".into(),
            ));
        }
        let func = self.ctx.stack.swap_remove(stack_len - arg_count - 1);
        self.call_value(func, arg_count, true, Some(bytecode))
    }

    fn dispatch_return(&mut self) -> IfaResult<()> {
        if let Some(finally_ip) = self.ctx.recovery_stack.last().and_then(|f| f.finally_ip) {
            let return_value = self.pop().unwrap_or(IfaValue::null());
            self.ctx.recovery_stack.pop();
            self.pending_finally = Some(FinallyResumption::Return { return_value });
            self.ctx.ip = finally_ip;
            return Ok(());
        }

        if let Some(frame) = self.ctx.frames.pop() {
            let return_value = self.pop().unwrap_or(IfaValue::null());
            if self.ctx.stack.len() > frame.base_ptr {
                self.ctx.stack.truncate(frame.base_ptr);
            }
            if frame.async_return {
                self.push(IfaValue::future_ready(return_value))?;
            } else {
                self.push(return_value)?;
            }
            self.ctx.ip = frame.return_addr;
        } else {
            self.ctx.halted = true;
        }

        Ok(())
    }

    fn dispatch_parallel_for(&mut self, bytecode: &Bytecode) -> IfaResult<()> {
        let closure_val = self.pop()?;
        let iterable_val = self.pop()?;

        let items_vec = match iterable_val {
            IfaValue::List(l) => l.clone(),
            _ => {
                return Err(IfaError::TypeError {
                    expected: "List".into(),
                    got: iterable_val.type_name().into(),
                });
            }
        };

        if !matches!(closure_val, IfaValue::Fn(_) | IfaValue::Closure(_)) {
            return Err(IfaError::TypeError {
                expected: "Closure".into(),
                got: closure_val.type_name().into(),
            });
        }

        #[cfg(feature = "parallel")]
        {
            let mut worker_vm = IfaVM::new();
            worker_vm.globals = self.globals.clone();
            let mut results_vec = Vec::with_capacity(items_vec.len());
            for item in items_vec.iter() {
                let val = worker_vm.spawn_task(closure_val.clone(), vec![item.clone()])?;
                let final_val = if matches!(val, IfaValue::Future(_) | IfaValue::NativeFuture(_)) {
                    worker_vm.await_future(&val, bytecode)?
                } else {
                    val
                };
                results_vec.push(final_val);
            }
            self.push(IfaValue::List(ifa_types::gc::IfaGc::new(results_vec)))?;
        }

        #[cfg(not(feature = "parallel"))]
        {
            let mut results = Vec::with_capacity(items_vec.len());
            for item in items_vec.as_ref().iter() {
                let val = self.spawn_task(closure_val.clone(), vec![item.clone()])?;
                let res = if matches!(val, IfaValue::Future(_) | IfaValue::NativeFuture(_)) {
                    self.await_future(&val, bytecode)?
                } else {
                    val
                };
                results.push(res);
            }
            self.push(IfaValue::list(results))?;
        }

        Ok(())
    }

    fn dispatch_call_method(&mut self, bytecode: &Bytecode) -> IfaResult<()> {
        let method_idx = self.read_u16(bytecode)?;
        let arg_count = self.read_u8(bytecode)?;

        let mut args = Vec::with_capacity(arg_count as usize);
        for _ in 0..arg_count {
            args.push(self.pop()?);
        }
        args.reverse();

        let object = self.pop()?;

        let method_name = bytecode
            .strings
            .get(method_idx as usize)
            .cloned()
            .ok_or_else(|| {
                IfaError::Custom(format!("Invalid method name index: {}", method_idx))
            })?;

        if let IfaValue::Str(s) = &object
            && let Some(domain_id) = parse_odu_mod_marker(s)
        {
            let result = self.call_registry(domain_id, &method_name, args, bytecode)?;
            self.push(result)?;
            return Ok(());
        }

        match object {
            IfaValue::Map(map) => {
                let key = ifa_types::CompactString::new(method_name.as_str());
                if let Some(func) = map.get(&key) {
                    let arg_count = args.len();
                    for arg in args {
                        self.push(arg)?;
                    }
                    self.call_value(func.clone(), arg_count, false, Some(bytecode))?;
                } else {
                    return Err(IfaError::Custom(format!(
                        "Map has no method '{}'",
                        method_name
                    )));
                }
            }
            IfaValue::List(mut l) => {
                if method_name == "fikun" || method_name == "append" || method_name == "push" {
                    let val = args.first().ok_or_else(|| IfaError::ArityMismatch {
                        expected: 1,
                        got: args.len(),
                    })?;
                    let vec = ifa_types::gc::IfaGc::make_mut(&mut l);
                    vec.push(val.clone());
                    self.push(IfaValue::null())?;
                    return Ok(());
                } else if method_name == "yi_pada"
                    || method_name == "yipada"
                    || method_name == "map"
                    || method_name == "maapu"
                {
                    let closure = args.first().ok_or_else(|| IfaError::ArityMismatch {
                        expected: 1,
                        got: args.len(),
                    })?;
                    let mut results = Vec::with_capacity(l.len());
                    for item in l.iter() {
                        let task_val = self.spawn_task(closure.clone(), vec![item.clone()])?;
                        let mapped = if let IfaValue::Future(_cell) = &task_val {
                            self.await_future(&task_val, bytecode)?
                        } else {
                            task_val
                        };
                        results.push(mapped);
                    }
                    self.push(IfaValue::List(ifa_types::gc::IfaGc::new(results)))?;
                    return Ok(());
                } else if method_name == "to" || method_name == "sort" {
                    let mut sorted = l.to_vec();
                    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                    self.push(IfaValue::List(ifa_types::gc::IfaGc::new(sorted)))?;
                    return Ok(());
                } else if method_name == "gbogbo" || method_name == "all" {
                    let closure = args.first().ok_or_else(|| IfaError::ArityMismatch {
                        expected: 1,
                        got: args.len(),
                    })?;
                    for item in l.iter() {
                        let task_val = self.spawn_task(closure.clone(), vec![item.clone()])?;
                        let keep = if let IfaValue::Future(_cell) = &task_val {
                            self.await_future(&task_val, bytecode)?
                        } else {
                            task_val
                        };
                        if !keep.is_truthy() {
                            self.push(IfaValue::bool(false))?;
                            return Ok(());
                        }
                    }
                    self.push(IfaValue::bool(true))?;
                    return Ok(());
                } else {
                    return Err(IfaError::Custom(format!(
                        "List has no method '{}'",
                        method_name
                    )));
                }
            }
            obj => {
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

        Ok(())
    }

    // =========================================================================
    // ARITHMETIC OPCODE HANDLER
    // =========================================================================

    // =========================================================================
    // COMPARISON OPCODE HANDLER
    // =========================================================================

    // =========================================================================
    // EXCEPTION OPCODE HANDLER
    // =========================================================================

    // =========================================================================
    // BYTECODE READING HELPERS
    // =========================================================================

    fn read_u8(&mut self, bytecode: &Bytecode) -> IfaResult<u8> {
        if self.ctx.ip >= bytecode.code.len() {
            return Err(IfaError::Custom("Unexpected end of bytecode".to_string()));
        }
        let value = bytecode.code[self.ctx.ip];
        self.ctx.ip += 1;
        Ok(value)
    }

    fn read_u16(&mut self, bytecode: &Bytecode) -> IfaResult<u16> {
        let end = self.ctx.ip + 2;
        if end > bytecode.code.len() {
            return Err(IfaError::Custom("Unexpected end of bytecode".to_string()));
        }
        let value = u16::from_le_bytes(bytecode.code[self.ctx.ip..end].try_into().unwrap());
        self.ctx.ip = end;
        Ok(value)
    }

    fn read_u32(&mut self, bytecode: &Bytecode) -> IfaResult<u32> {
        let end = self.ctx.ip + 4;
        if end > bytecode.code.len() {
            return Err(IfaError::Custom("Unexpected end of bytecode".to_string()));
        }
        let value = u32::from_le_bytes(bytecode.code[self.ctx.ip..end].try_into().unwrap());
        self.ctx.ip = end;
        Ok(value)
    }

    fn read_i64(&mut self, bytecode: &Bytecode) -> IfaResult<i64> {
        let end = self.ctx.ip + 8;
        if end > bytecode.code.len() {
            return Err(IfaError::Custom("Unexpected end of bytecode".to_string()));
        }
        let value = i64::from_le_bytes(bytecode.code[self.ctx.ip..end].try_into().unwrap());
        self.ctx.ip = end;
        Ok(value)
    }

    fn read_u64(&mut self, bytecode: &Bytecode) -> IfaResult<u64> {
        let end = self.ctx.ip + 8;
        if end > bytecode.code.len() {
            return Err(IfaError::Custom("Unexpected end of bytecode".to_string()));
        }
        let value = u64::from_le_bytes(bytecode.code[self.ctx.ip..end].try_into().unwrap());
        self.ctx.ip = end;
        Ok(value)
    }

    fn read_f64(&mut self, bytecode: &Bytecode) -> IfaResult<f64> {
        let end = self.ctx.ip + 8;
        if end > bytecode.code.len() {
            return Err(IfaError::Custom("Unexpected end of bytecode".to_string()));
        }
        let value = f64::from_le_bytes(bytecode.code[self.ctx.ip..end].try_into().unwrap());
        self.ctx.ip = end;
        Ok(value)
    }
}

#[cfg(feature = "compiler")]
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
                effects: _,
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

        assert_eq!(vm.ctx.stack.len(), 1);
        assert_eq!(vm.ctx.stack[0], IfaValue::Int(5));

        // 2. Snapshot
        let snap = vm.snapshot(&bc).expect("Failed to create snapshot");

        // 3. Resume in a fresh VM
        let mut vm2 = IfaVM::resume(&snap, &bc).expect("Failed to resume snapshot");

        // 4. Continue execution
        let final_res = vm2.resume_execution(&bc).unwrap();

        assert_eq!(final_res, IfaValue::Int(8)); // 5 + 3 = 8
        assert_eq!(vm2.ctx.stack.len(), 0);
    }

    #[test]
    fn test_snapshot_resume_preserves_globals() {
        let mut vm = IfaVM::new();
        vm.set_global("answer", IfaValue::Int(41));

        let mut bc = Bytecode::new("test_snapshot_resume_preserves_globals");
        bc.strings.push("answer".to_string());
        bc.code = vec![
            OpCode::Yield as u8,
            OpCode::LoadGlobal as u8,
            0,
            0,
            OpCode::PushInt as u8,
            1,
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

        let yielded = vm.execute(&bc);
        assert!(matches!(yielded, Err(IfaError::Yielded)));

        let snap = vm.snapshot(&bc).expect("snapshot should succeed");
        let mut restored = IfaVM::resume(&snap, &bc).expect("resume should succeed");

        let got = restored
            .resume_execution(&bc)
            .expect("resume_execution should succeed");
        assert_eq!(got, IfaValue::Int(42));
    }

    #[test]
    fn test_len_uses_unicode_code_points() {
        let mut vm = IfaVM::new();
        let mut bc = Bytecode::new("test_len_uses_unicode_code_points");
        bc.strings.push("🔥a".to_string());
        bc.code = vec![
            OpCode::PushStr as u8,
            0,
            0,
            OpCode::Len as u8,
            OpCode::Halt as u8,
        ];

        let got = vm.execute(&bc).expect("vm should succeed");
        assert_eq!(got, IfaValue::Int(2));
    }

    #[test]
    fn test_sandboxed_vm_sets_stricter_limits_and_fuel() {
        let vm = IfaVM::sandboxed(4096);
        assert_eq!(vm.stack_limit, Some(1024));
        assert_eq!(vm.frame_limit, Some(128));
        assert_eq!(vm.fuel, Some(4096));
    }

    #[test]
    fn test_fuel_limit_exhausts_execution_budget() {
        let mut vm = IfaVM::sandboxed(0);
        let mut bc = Bytecode::new("test_fuel_limit_exhausts_execution_budget");
        bc.code = vec![OpCode::PushNull as u8, OpCode::Halt as u8];

        let err = vm
            .execute(&bc)
            .expect_err("fuel exhaustion should stop execution");
        assert!(err.to_string().contains("Execution budget exhausted"));
    }

    #[test]
    fn test_boxed_primitive_opcode_path_for_bitwise_and_casts() {
        let mut vm = IfaVM::new();
        let mut bc = Bytecode::new("test_boxed_primitive_opcode_path_for_bitwise_and_casts");
        bc.code = vec![
            OpCode::PushInt as u8,
            6,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            OpCode::PushInt as u8,
            3,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            OpCode::And as u8,
            OpCode::PushTrue as u8,
            OpCode::ToInt as u8,
            OpCode::Add as u8,
            OpCode::Halt as u8,
        ];

        let got = vm.execute(&bc).expect("vm should succeed");
        assert_eq!(got, IfaValue::Int(3));
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
        ok_vm.set_global("okv", IfaValue::ire(IfaValue::Int(41)));
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
        err_vm.set_global("failv", IfaValue::ibi(IfaValue::str("boom")));
        let mut err_bytecode = Bytecode::new("test_propagate_error_throws_err");
        err_bytecode.strings.push("failv".to_string());
        err_bytecode.code = vec![
            OpCode::TryBegin as u8,
            10,
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
