# `vm.rs` Refactoring — Implementation Plan

> **Scope:** `crates/ifa-core/src/vm.rs` (2,861 lines, 109KB)
> **Rule:** No new abstractions that don't already exist in embryo. No `VmEngine` trait. No parallel VM structs. Mechanical decomposition only.

---

## Ground Truth: What the Code Actually Contains

Before any refactoring, the confirmed problems with exact locations:

### Problem 1 — `Fn`/`Closure` dispatch is copy-pasted 5 times

The pattern of:
```rust
match func {
    IfaValue::Fn(data) => { arity_check; push_frame; push args; set ip }
    IfaValue::Closure(closure) => { arity_check; push_frame with env; push args; set ip }
    _ => TypeError
}
```

...appears independently in:

| Location | Lines (approx) |
|---|---|
| `OpCode::Call` | 1363–1438 |
| `OpCode::TailCall` | 1454–1513 |
| `OpCode::CallMethod` → Map branch | 1612–1664 |
| `call_value_task()` | 738–779 |
| `invoke_module_function()` | 484–524 |

Five copies. Every async/arity/closure fix requires touching all five.

### Problem 2 — 12-variable save/restore is copy-pasted twice

`invoke_module_function` (line 467–541) and `execute_module` (line 672–703) each independently save and restore:
```
ip, halted, stack_len, frames_len, recovery_len, stack_limit, frame_limit, opon_size, ikin, globals
```
And then **manually truncate the stack, frames, and recovery_stack** on cleanup. This is a bug factory — add any field to `IfaVM` and you will forget to save/restore it in one of them.

### Problem 3 — `IfaVM` struct carries 17 fields across two unrelated concerns

The module subsystem (`imported`, `import_guard`, `resolver`, `module_cache`, `module_exports`, `module_bytecode`, `module_globals`) has no business living on the execution struct. It is a separate concern.

### Problem 4 — Hot variables are not hoisted in the main loop

`resume_execution` (line 707) accesses `self.ip`, `self.halted`, `self.stack`, `self.frames` through `&mut self` on every instruction. The compiler cannot split borrows. Hoisting `ip` into a local variable before the loop is a direct throughput win.

### Problem 5 — `Ikin` bulk-load skips `string_map` population

In `load_from_bytecode` (vm_ikin.rs line 96–109), the `string_map` is **intentionally not updated** during bulk loads. This means `intern()` at runtime cannot deduplicate against loaded constants. Low severity now. Will bite later.

---

## Refactoring Phases (Strict Dependency Order)

```
Phase 1: call_value() helper          ← No dependencies. Pure extraction.
    ↓
Phase 2: ModuleLoader struct          ← No dependencies. Pure extraction.
    ↓
Phase 3: ExecutionContext struct       ← Needs Phase 2 (fewer fields to move)
    ↓
Phase 4: Hot variable hoisting        ← Needs Phase 3 (clean state separation)
    ↓
Phase 5: Ikin dedup fix               ← Independent. Do anytime.
```

---

## Phase 1 — Extract `call_value()`

**Goal:** Eliminate the 5 copies of `Fn`/`Closure` dispatch into one function.

**Files changed:** `crates/ifa-core/src/vm.rs` only.

### The new function

Add to `impl IfaVM`, **before** `step()`:

```rust
/// Dispatch a call to a function or closure value.
///
/// Pushes a new call frame, pushes arguments, and sets ip.
/// For async functions, spawns a task instead and pushes the Future.
///
/// Used by: Call, TailCall, CallMethod, call_value_task, invoke_module_function.
fn call_value(
    &mut self,
    func: IfaValue,
    args: Vec<IfaValue>,
    return_addr: usize,
) -> IfaResult<CallOutcome> {
    match func {
        IfaValue::Fn(data) => {
            if args.len() != data.arity as usize {
                return Err(IfaError::ArityMismatch {
                    expected: data.arity as usize,
                    got: args.len(),
                });
            }
            if data.is_async {
                let future = self.spawn_task(IfaValue::Fn(data), args)?;
                return Ok(CallOutcome::AsyncSpawned(future));
            }
            self.push_frame(CallFrame::new(return_addr, self.stack.len(), None, false))?;
            for arg in args {
                self.push(arg)?;
            }
            Ok(CallOutcome::Jumped(data.start_ip))
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
                let future = self.spawn_task(IfaValue::Closure(closure), args)?;
                return Ok(CallOutcome::AsyncSpawned(future));
            }
            let start_ip = data.start_ip;
            self.push_frame(CallFrame::new(
                return_addr,
                self.stack.len(),
                Some(closure.env.clone()),
                false,
            ))?;
            for arg in args {
                self.push(arg)?;
            }
            Ok(CallOutcome::Jumped(start_ip))
        }
        other => Err(IfaError::TypeError {
            expected: "Function".into(),
            got: other.type_name().into(),
        }),
    }
}
```

### The outcome enum

Add at the top of `vm.rs` (module level):

```rust
/// Result of a call dispatch — used internally by call_value().
enum CallOutcome {
    /// Function was invoked synchronously. IP is now set to this address.
    Jumped(usize),
    /// Async function was spawned. Caller should push this future.
    AsyncSpawned(IfaValue),
}
```

### Replacing the 5 call sites

**`OpCode::Call` (line ~1363):**
```rust
OpCode::Call => {
    let arg_count = self.read_u8(bytecode)? as usize;
    let mut args: Vec<IfaValue> = (0..arg_count).map(|_| self.pop()).collect::<IfaResult<_>>()?;
    args.reverse();
    let func = self.pop()?;

    // Handle string-encoded markers before generic dispatch
    if let IfaValue::Str(ref s) = func {
        if let Some((domain_id, method)) = parse_odu_fn_marker(s) {
            let result = self.call_registry(domain_id, &method, args, bytecode)?;
            self.push(result)?;
            return Ok(());
        }
        if let Some((module_key, fn_name)) = parse_module_fn_marker(s) {
            let result = self.invoke_module_function(&module_key, &fn_name, args)?;
            self.push(result)?;
            return Ok(());
        }
    }

    match self.call_value(func, args, self.ip)? {
        CallOutcome::Jumped(ip) => self.ip = ip,
        CallOutcome::AsyncSpawned(future) => self.push(future)?,
    }
}
```

**`call_value_task` (line ~737):** Replace the match block entirely with `self.call_value(func, args, self.ip)?` and handle the outcome.

**`invoke_module_function` (line ~484):** Replace the match on `func` with `call_value()`.

**`CallMethod` Map branch (line ~1612):** Replace the duplicated `Fn`/`Closure` match with `call_value()`.

**`TailCall` (line ~1441):** This one is different — tail call reuses the existing frame. Do NOT collapse it into `call_value()`. Leave TailCall as-is. It is correctly different.

### Exit Criteria for Phase 1

- All existing tests pass unchanged.
- `CallOutcome` is `#[allow(dead_code)]` free — all variants are used.
- `call_value_task`, `Call`, `invoke_module_function`, and `CallMethod` Map branch all delegate to `call_value()`.
- `TailCall` is untouched.
- `grep -n "IfaValue::Fn(data) =>" vm.rs` returns exactly 2 hits: `call_value()` itself and `TailCall`.

---

## Phase 2 — Extract `ModuleLoader`

**Goal:** Move the 7 module-system fields off `IfaVM` into a dedicated `ModuleLoader` struct. Eliminate the duplicated 12-variable save/restore blocks.

**Files changed:** `crates/ifa-core/src/vm.rs` only.

### The new struct

```rust
/// Module loading and caching subsystem.
/// Separated from VM execution state to keep IfaVM fields manageable.
#[derive(Default)]
struct ModuleLoader {
    imported: std::collections::HashSet<String>,
    import_guard: crate::module_resolver::ImportGuard,
    resolver: crate::module_resolver::ModuleResolver,
    module_cache: std::collections::HashMap<String, CachedModule>,
    module_exports: std::collections::HashMap<String, IfaValue>,
    module_bytecode: std::collections::HashMap<String, Bytecode>,
    module_globals: std::collections::HashMap<String, std::collections::HashMap<String, IfaValue>>,
}
```

### Update `IfaVM`

Remove the 7 fields. Add:
```rust
pub struct IfaVM {
    // ... existing fields ...
    #[serde(skip)]
    modules: ModuleLoader,
}
```

### The `ExecutionSave` struct — kills the save/restore duplication

```rust
/// Transient execution state saved and restored around sub-executions
/// (module loading, cross-module function calls).
struct ExecutionSave {
    ip: usize,
    halted: bool,
    stack_len: usize,
    frames_len: usize,
    recovery_len: usize,
    stack_limit: Option<usize>,
    frame_limit: Option<usize>,
    opon_size: crate::bytecode::OponSize,
    ikin: Ikin,
    globals: std::collections::HashMap<String, IfaValue>,
}

impl ExecutionSave {
    fn capture(vm: &IfaVM) -> Self {
        ExecutionSave {
            ip: vm.ip,
            halted: vm.halted,
            stack_len: vm.stack.len(),
            frames_len: vm.frames.len(),
            recovery_len: vm.recovery_stack.len(),
            stack_limit: vm.stack_limit,
            frame_limit: vm.frame_limit,
            opon_size: vm.opon_size,
            ikin: vm.ikin.clone(),
            globals: vm.globals.clone(),
        }
    }

    fn restore(self, vm: &mut IfaVM) {
        vm.ip = self.ip;
        vm.halted = self.halted;
        vm.stack.truncate(self.stack_len);
        vm.frames.truncate(self.frames_len);
        vm.recovery_stack.truncate(self.recovery_len);
        vm.stack_limit = self.stack_limit;
        vm.frame_limit = self.frame_limit;
        vm.opon_size = self.opon_size;
        vm.ikin = self.ikin;
        vm.globals = self.globals;
    }
}
```

### Replace `invoke_module_function`

```rust
fn invoke_module_function(...) -> IfaResult<IfaValue> {
    let save = ExecutionSave::capture(self);

    let result = (|| {
        // ... setup ...
        self.resume_execution(&bytecode)
    })();

    let updated_globals = self.globals.clone();
    save.restore(self);
    self.modules.module_globals.insert(module_key.to_string(), updated_globals);

    result
}
```

### Replace `execute_module`

```rust
fn execute_module(&mut self, bytecode: &Bytecode) -> IfaResult<()> {
    let save = ExecutionSave::capture(self);

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

    save.restore(self);
    result
}
```

### Update all field accesses

Do a `sed`-equivalent pass: `self.imported` → `self.modules.imported`, etc. for all 7 fields.

### Exit Criteria for Phase 2

- `IfaVM` has exactly 10 fields (was 17).
- `invoke_module_function` and `execute_module` each have exactly one `ExecutionSave::capture` and one `save.restore()` call.
- All module-system tests pass.
- `grep -n "saved_ip\|saved_halted\|saved_ikin\|saved_limits" vm.rs` returns 0 hits.

---

## Phase 3 — Extract `ExecutionContext`

**Goal:** Give `swap_task_state` (line 729) a proper type to swap. Make cooperative scheduling legible.

**Note:** This is a smaller, more targeted change than the summary's `VmEngine` proposal. We are not creating a polymorphic VM family. We are giving the task-scheduler's swap operation a real type.

### The new struct

```rust
/// The transient per-task execution state.
/// Swapped in/out by the cooperative scheduler for task switching.
#[derive(Default, Clone)]
struct ExecutionContext {
    stack: Vec<IfaValue>,
    frames: Vec<CallFrame>,
    ip: usize,
    halted: bool,
    recovery_stack: Vec<RecoveryFrame>,
}
```

### Update `IfaVM`

Replace the 5 individual fields (`stack`, `frames`, `ip`, `halted`, `recovery_stack`) with:
```rust
ctx: ExecutionContext,
```

Access becomes `self.ctx.stack`, `self.ctx.ip`, etc.

### `swap_task_state` becomes trivial

```rust
fn swap_task_state(&mut self, task: &mut Task) {
    std::mem::swap(&mut self.ctx, &mut task.ctx);
}
```

Where `Task.state` is renamed to `Task.ctx: ExecutionContext`.

### Update `ExecutionSave` from Phase 2

`ExecutionSave::capture` / `restore` now copies `self.ctx` as a unit for the fields it owns.

### Exit Criteria for Phase 3

- `swap_task_state` is 1 line.
- `IfaVM` has no top-level `stack`, `frames`, `ip`, `halted`, or `recovery_stack` fields — all accessed via `self.ctx.*`.
- All existing snapshot/resume tests pass (they serialize `IfaVM`, so `ExecutionContext` must be `Serialize`/`Deserialize`).

> **Serde note:** `ExecutionContext` must derive `Serialize, Deserialize`. The existing `serde(skip)` on `task_queue` is unaffected.

---

## Phase 4 — Hot Variable Hoisting in `resume_execution`

**Goal:** Let the compiler reason about `ip` as a local variable inside the main loop. Remove `self.ctx.ip` from the hot path.

**Files changed:** `crates/ifa-core/src/vm.rs` only — `resume_execution()` and `step()`.

### The change

```rust
pub fn resume_execution(&mut self, bytecode: &Bytecode) -> IfaResult<IfaValue> {
    self.ctx.halted = false;
    let mut ip = self.ctx.ip; // hoist

    while !self.ctx.halted && ip < bytecode.code.len() {
        self.ctx.ip = ip; // write back before each step (needed for error reporting)
        match self.step(bytecode) {
            Ok(()) => { ip = self.ctx.ip; } // read back after step
            Err(IfaError::Yielded) => {
                self.ctx.ip = ip;
                return Err(IfaError::Yielded);
            }
            Err(e) => {
                self.ctx.ip = ip;
                if self.attempt_recovery(&e)? {
                    ip = self.ctx.ip;
                    continue;
                }
                return Err(e);
            }
        }
    }

    Ok(self.ctx.stack.pop().unwrap_or(IfaValue::null()))
}
```

> **Note:** `step()` itself calls `vm_iroke::tap()` which increments `vm.ip`. After Phase 3, this means it increments `self.ctx.ip`. The write-back/read-back pattern above keeps `ip` local in sync. This is a standard bytecode VM optimization.

### Exit Criteria for Phase 4

- All conformance tests pass.
- Benchmark: run a tight loop program (1 million iterations) before and after. Expect 5–15% throughput improvement.

---

## Phase 5 — Fix `Ikin` Bulk-Load Deduplication (Independent)

**Goal:** Ensure `intern()` at runtime can deduplicate against constants loaded from bytecode.

**Files changed:** `crates/ifa-core/src/vm_ikin.rs` only.

### The fix

In `load_from_bytecode` (line 96–109), populate `string_map` during the bulk load:

```rust
pub fn load_from_bytecode(&mut self, bytecode: &crate::bytecode::Bytecode) {
    self.strings.clear();
    self.string_map.clear();
    self.strings.reserve(bytecode.strings.len());

    for (i, s) in bytecode.strings.iter().enumerate() {
        let arc: Arc<str> = s.as_str().into();
        self.strings.push(arc.clone());
        self.string_map.insert(arc, i as u32); // ← was intentionally omitted
    }
}
```

Remove the comment explaining why the map is not populated — it no longer applies.

### Exit Criteria for Phase 5

- `intern("some_string_that_was_in_bytecode")` returns the existing index, not a new one.
- `ikin.strings.len()` does not grow when interning a string already in the pool.

---

## Sequencing and Risk

| Phase | Risk | Rollback |
|---|---|---|
| 1 — `call_value()` | Low. Pure extraction. | Revert the 4 call sites. |
| 2 — `ModuleLoader` | Medium. Many field renames. | Field renames are mechanical. |
| 3 — `ExecutionContext` | Medium. Serde must be preserved. | Verify snapshot tests first. |
| 4 — Hot variable hoisting | Low. Correctness preserved by tests. | One-line revert of resume_execution. |
| 5 — Ikin dedup | Low. Tiny change, isolated file. | One-line revert. |

**Do not combine phases in a single PR.** Each phase produces a provably correct intermediate state. Combined PRs make bisection impossible when a test breaks.

---

## What Is Explicitly Not Done

| Idea | Why Not |
|---|---|
| `trait VmEngine` | There is one VM. Traits are for multiple implementations. |
| `struct IkinVm` / `struct IrokeVm` | These don't exist. Don't invent them. |
| `Arc<Ikin>` for thread sharing | No active use case. Would complicate serde shim. |
| Splitting `step()` into separate files | `step()` is one match. File-splitting doesn't reduce coupling. |
| Removing `vm_iroke::tap()` | It's 42 lines and correct. Leave it. |

---

## File Change Map

| Phase | Files Changed |
|---|---|
| 1 | `ifa-core/src/vm.rs` |
| 2 | `ifa-core/src/vm.rs` |
| 3 | `ifa-core/src/vm.rs` |
| 4 | `ifa-core/src/vm.rs` |
| 5 | `ifa-core/src/vm_ikin.rs` |
