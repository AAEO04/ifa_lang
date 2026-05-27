# Ifá-Lang Unified Implementation Plan
## All Source Documents → One Execution Schedule

> **Source documents merged:** `security.md`, `memory hardening.md`, `cargo hygiene.md`, `vm pla.md`, `crate split.md`, `dead module wiring.md`, `plan.md`, `egbe.md`, `future.md`, `performance_engineering.md`, `missing_features_implementation.md`, `rust_route_ast_transpiler_impact.md`, `infra pkan.md`, `patch plan`, `dsa_audit.md`, `linus_audit.md`

> **Conflicts resolved:** 14 (see Appendix A items 1–6, Appendix H items 7–14)

> **Rule:** Each phase produces a compiling, passing codebase. No phase depends on work from a later phase. Every phase is one PR.

---

## The Thirteen Streams

All source documents decompose into 13 independent work streams. Within each stream, tasks are strictly ordered. Across streams, dependencies are explicitly marked.

```
Stream A: Security Hardening          ← Blocks nothing. Start immediately.
Stream B: Memory & Unsafe Hardening   ← Blocks nothing. Start immediately.
Stream C: Cargo & Build Hygiene       ← Blocks Stream D (crate split).
Stream D: Crate Split                 ← Needs Stream C Phase 1 done first.           [✅ DONE]
Stream E: VM Refactoring              ← Blocks Stream I (performance).
Stream F: Dead Module Wiring          ← Needs Stream A Fix 4 (bincode) first.
Stream G: Language Features           ← [PARTIAL] Blocks nothing. Start after Stream E Phase 1.
Stream H: Concurrency (Ẹgbẹ́)         ← [✅ DONE] Needs Streams E, F, G Tier 1 all done.
Stream I: Performance (Engine Block)  ← Needs E5. Blocks Stream K.
Stream J: True Parallelism            ← [PARTIAL] Needs H2. Uses Go/Erlang actor model.
Stream K: Missing Language Primitives ← Needs I1, I2 merged first. (THE LINUS RULE)
Stream L: Build & Compile Speed       ← Independent. Start anytime.
Stream M: ifa-infra Crate Extraction  ← Needs F1–F7 wired AND A6 merged first.
```

> [!IMPORTANT]
> **📍 CURRENT EXECUTION CHECKPOINT — 2026-05-18 Audit**
>
> **Completed PRs:** PR-01 (A1–A4) ✅ | PR-05 (A6) ✅ | PR-08 (G1, G2) ✅ | PR-13 (F1, F3-F5, F7) ✅ | PR-14 (H2) ✅ | PR-15 (H3) ✅ | PR-17 (H1) ✅ | PR-19 (D1–D3) ✅ | PR-20 (D4–D6) ✅ | PR-27 (E5+I1) ✅ | PR-28 revised (I3) ✅ | PR-40 (M1+M2) ✅ | PR-41 (M3) ✅
>
> **Next PR:** PR-14 Stage 2 follow-up if reactive/shared UI work is active.
>
> **Critical blockers in order:**
> 1. **PR-14 (H0 Stage 2 follow-up):** Parked until reactive/shared UI work is active
>
> **H0 follow-up:** `PR-14` Stage 1 is complete; Stage 2 stays parked until reactive/shared UI work is active.


---

## Stream A — Security Hardening

*Source: `security.md`*

### A1. Stack Depth Guard [CRITICAL — 15 min]

**Files:** `vm.rs:219-220`, `interpreter/core.rs:143`

```diff
# vm.rs — IfaVM::new() and IfaVM::with_opon()
-            stack_limit: None,
-            frame_limit: None,
+            stack_limit: Some(4096),
+            frame_limit: Some(512),
```

```diff
# interpreter/core.rs
-            call_depth_limit: None,
+            call_depth_limit: Some(512),
```

Wire the guard in the `Call` opcode handler (before `frames.push`):
```rust
if let Some(limit) = self.frame_limit {
    if self.frames.len() >= limit {
        return Err(IfaError::Runtime(
            format!("Stack overflow: call depth exceeded {} frames", limit)
        ));
    }
}
```

Wire in `push()` helper:
```rust
fn push(&mut self, val: IfaValue) -> IfaResult<()> {
    if let Some(limit) = self.stack_limit {
        if self.stack.len() >= limit {
            return Err(IfaError::Runtime(
                format!("Stack overflow: operand stack exceeded {} values", limit)
            ));
        }
    }
    self.stack.push(val);
    Ok(())
}
```

### A2. Fuel Limit [HIGH — 30 min]

**Files:** `vm.rs`, `vm_iroke.rs:20-22`

Add `fuel: Option<u64>` field to `IfaVM`. Activate the fuel check:
```diff
# vm_iroke.rs
-    if vm.ticks & 1023 == 0 {
-        // checks_interrupts(vm)?;
-    }
+    if vm.ticks & 1023 == 0 {
+        if let Some(ref mut remaining) = vm.fuel {
+            if *remaining == 0 {
+                return Err(IfaError::Runtime("Execution budget exhausted (fuel = 0)".into()));
+            }
+            *remaining = remaining.saturating_sub(1024);
+        }
+    }
```

Add `IfaVM::sandboxed(fuel: u64)` constructor with stricter limits.

### A3. Ikin Pool Cap [HIGH — 10 min]

**File:** `vm_ikin.rs:43, :99`

Cap `intern()` at 65,536 strings. Cap `load_from_bytecode` to reject bytecode with more than 65,536 strings.

### A4. Bincode Size Limits [HIGH — 20 min]

**Files:** `vm.rs:318`, `infra/storage.rs:374`

> [!IMPORTANT]
> This fix is a **prerequisite for Stream F Fix 5** (storage wiring). Do not wire storage before this is applied.

```rust
// vm.rs — resume()
const MAX_SNAPSHOT_BYTES: u64 = 64 * 1024 * 1024;
let opts = bincode::DefaultOptions::new()
    .with_limit(MAX_SNAPSHOT_BYTES)
    .with_fixint_encoding()
    .allow_trailing_bytes();
let (saved_hash, vm): (u64, IfaVM) = opts.deserialize(snapshot)
    .map_err(|e| IfaError::Custom(format!("Corrupted snapshot: {}", e)))?;
```

```rust
// storage.rs:374
const MAX_STORED_BYTES: u64 = 16 * 1024 * 1024;
let opts = bincode::DefaultOptions::new().with_limit(MAX_STORED_BYTES);
let value = opts.deserialize(&buffer)?;
```

### A5. FFI Library-Load Gating [HIGH — 45 min]

**File:** `ffi.rs`

Add `allow_native_libs: bool` and `allowed_lib_roots: Vec<PathBuf>` to `FfiSecurity`. Gate `libloading::Library::new` behind capability check. Validate path against allowed roots.

**Status:** completed in `crates/ifa-std/src/ffi.rs`. Native loads now require `allow_native_libs`, are root-gated, and still canonicalize before load.

### A6. Registry Isolation [HIGH — 2 hrs]

**Files:** `registry.rs`, `vm.rs`, `native.rs`

Each `IfaVM` owns a `local_registry: ResourceRegistry`. Thread through `VmContext`:
```rust
pub struct VmContext<'a> {
    pub vm: &'a mut IfaVM,
    pub bytecode: &'a Bytecode,
    pub registry: &'a ResourceRegistry,  // local, not global
}
```

Mark global `REGISTRY` as `#[doc(hidden)]` + deprecation note.

> [!WARNING]
> **Conflict resolved:** `dead module wiring.md` uses `REGISTRY` (global) throughout its dispatch examples. Once A6 is applied, all dispatch shims in Stream F must use `ctx.registry` instead of `REGISTRY`. The dispatch function signatures change to accept `&ResourceRegistry`.

### A7. Embedded VM Parity Tests [MEDIUM — 1 day]

**File:** `ifa-embedded/tests/semantic_parity.rs` (new)

Run identical bytecode through both `IfaVM` and `EmbeddedVm`, assert identical results.

**Status:** completed in `crates/ifa-embedded/tests/semantic_parity.rs`.

### A8. Symlink Canonicalize [MEDIUM — 1 hr]

Canonicalize paths with `std::fs::canonicalize()` before capability checks.

**Status:** completed in `crates/ifa-std/src/ffi.rs` and `crates/ifa-sandbox/src/sandbox.rs`.

### A9. Reconcile Èṣù and Òfún (Capability System Split) [HIGH — 90 min]

**Files:** `ifa-std/src/handlers/ofun.rs`, `vm.rs`

Currently, `Ofun` handles both the generation and enforcement of capabilities. Philosophically, Òfún handles creation/birth, while Èṣù handles enforcement at the crossroads. We must split the capability architecture:
1. **Òfún (The Factory):** Retains `Ofun::create_world(cli_args)` at VM startup to generate the initial `CapabilitySet`.
2. **Èṣù (The Proxy):** Create an `Esu` middleware struct. All I/O calls (`Odi`, `Otura`) must route through `Esu::enforce_crossroads(requested_cap, &world_state)` before executing.
3. **Sacrifice Capabilities (`ju`):** Dropping a capability is an Ẹbọ (sacrifice) routed through `Esu`, updating the world state to revoke access permanently.

**Status:** completed in `crates/ifa-std/src/esu.rs`, `crates/ifa-std/src/odi.rs`, `crates/ifa-std/src/otura.rs`, and `crates/ifa-std/src/vm_registry.rs`.

---

## Stream B — Memory & Unsafe Hardening

*Source: `memory hardening.md`*

### B1. Crate-Level `unsafe` Attributes [ZERO RISK]

Add `#![deny(unsafe_op_in_unsafe_fn)]` to `ifa-core/src/lib.rs`, `ifa-std/src/lib.rs`, `ifa-sandbox/src/lib.rs`.

Add `#![forbid(unsafe_code)]` to crates with no unsafe: `ifa-types`, `ifa-babalawo`, `ifa-fmt` (verify first with `rg "unsafe"`).

**Status:** completed in `crates/ifa-types/src/lib.rs`, `crates/ifa-babalawo/src/lib.rs`, and `crates/ifa-fmt/src/lib.rs`.

### B2. Document `ebo.rs` ManuallyDrop Invariants

Add `// SAFETY:` comments to all 3 unsafe blocks in `ebo.rs` (dismiss, sacrifice, Drop::drop). No behavior change.

**Status:** completed in `crates/ifa-vm/src/ebo.rs`.

### B3. Harden `ffi.rs` Unsafe Surface

- Change `BoundFunction.ptr` from `*mut c_void` to `NonNull<c_void>`.
- Add null check in `bind()` via `NonNull::new().ok_or(...)`.
- Document every `unsafe {}` block in `ffi.rs` with `// SAFETY:` comments.
- Document the raw C-string return path lifetime assumption.

**Status:** completed in `crates/ifa-std/src/ffi.rs`.

### B4. Fuzz FFI Argument Dispatch

Extract `validate_ffi_args()` from `call_native_libffi`. Create fuzz target `ifa-std/fuzz/fuzz_targets/ffi_args.rs`.

**Status:** completed in `crates/ifa-std/fuzz/`.

---

## Stream C — Cargo & Build Hygiene

*Source: `cargo hygiene.md`*

### C1. Workspace Dependency Fixes [ZERO RISK]

- Add `colored`, `once_cell`, `dashmap`, `rayon = "1.10"` to workspace root `[workspace.dependencies]`.
- Fix `ifa-types` and `ifa-bytecode` to use `version.workspace = true`.
- Fix `sysinfo` duplicate in `ifa-cli`.
- Fix `rayon` version mismatch (`1.8` vs `1.10`).

### C2. `ifa-std` Feature Flag Surgery

- Decouple `wasm` feature from `ndarray` (break `rsa_math` dependency for WASM target).
- Remove `auto-initialize` from `pyo3` dependency.
- Define `ci` feature: `["dashmap", "parallel"]` for minimal CI testing.

### C3. Feature-Gate Parser/Lexer in `ifa-core`

Add `compiler` feature to `ifa-core`. Make `logos`, `pest`, `pest_derive` optional behind it. Gate `parser.rs`, `lexer.rs`, `compiler.rs` modules.

> [!NOTE]
> **Conflict resolved:** `crate split.md` proposes extracting `ifa-parser` as a separate crate. `cargo hygiene.md` Phase 5 says "Deferred — feature-gating achieves the same isolation at zero structural cost." **Decision:** Do C3 (feature-gating) first. If the crate split in Stream D still proceeds, the feature gate makes extraction easier, not harder. They are not contradictory.

### C4. Wire `ifa-babalawo` Off `ifa-core`

Replace `ifa-babalawo`'s dependency on `ifa-core` with direct dependency on `ifa-types`. Update import paths from `ifa_core::ast::*` to `ifa_types::ast::*`.

### C5. Strip `OmniBox` from `oja fetch`

Remove AOT WASM compilation from the `ifa oja fetch` command in `crates/ifa-cli/src/oja.rs`. A package manager has no business invoking a compiler backend during dependency resolution. Compilation (`.wasm` → `.cwasm`) belongs in the build step, not fetch time.

---

## Stream D — Crate Split

*Source: `crate split.md`*

> [!IMPORTANT]
> **Depends on:** Stream C Phase 1 (workspace deps must be clean before restructuring).

### D1. Extract `ifa-parser`

Move `grammar.pest`, `parser.rs`, `lexer.rs` from `ifa-core` to new `crates/ifa-parser`. Add deps: `pest`, `pest_derive`, `ifa-types`.

### D2. Extract `ifa-compiler`

Move `compiler.rs` to new `crates/ifa-compiler`. Add deps: `ifa-types`, `ifa-parser`, `ifa-bytecode`.

### D3. Extract `ifa-transpiler`

Move `transpiler/` to new `crates/ifa-transpiler`. Add deps: `ifa-parser`, `ifa-types`.

### D4. Isolate `ifa-interpreter`

Move `interpreter/` to new `crates/ifa-interpreter`. Keep it building. This is architectural isolation, not deletion.

### D5. Rename `ifa-core` → `ifa-vm`

After extraction, `ifa-core` contains only: `vm.rs`, `opon.rs`, `vm_ikin.rs`, `vm_iroke.rs`, `ebo.rs`, `native.rs`. Rename to `ifa-vm`. Search-replace `ifa_core` → `ifa_vm` workspace-wide.

### D6. Workspace Cleanup

Update root `Cargo.toml` members list. `cargo clean && cargo build --workspace`.

---

## Stream E — VM Refactoring

*Source: `vm pla.md`*

### E1. Extract `call_value()` Helper

Collapse 5 copies of `Fn`/`Closure` dispatch into one `call_value()` function with a `CallOutcome` enum. Leave `TailCall` untouched (correctly different).

### E2. Extract `ModuleLoader` Struct

Move 7 module-system fields off `IfaVM` into `ModuleLoader`. Create `ExecutionSave` struct to kill the duplicated 12-variable save/restore blocks.

### E3. Extract `ExecutionContext` Struct

Group `stack`, `frames`, `ip`, `halted`, `recovery_stack` into `ExecutionContext`. Makes `swap_task_state` a single `std::mem::swap`.

### E4. Hot Variable Hoisting

Hoist `ip` into a local variable in `resume_execution()`. Write-back before step, read-back after. Expect 5–15% throughput gain.

### E5. Fix `Ikin` Bulk-Load Deduplication

Populate `string_map` during `load_from_bytecode`. Ensures `intern()` deduplicates against loaded constants.

### E6. Constant Divination Specialization *(Mojo steal: `@parameter` compile-time dispatch)*

**After E5 — compiler-only, VM-transparent.**

When `ifa-babalawo` resolves the `OduDomain` of a call site at parse time (it already does in `checks.rs`), the compiler can emit a **domain-locked `CallOdu`** with the domain byte hard-baked, eliminating the runtime `match domain_id` branch in `vm_registry.rs`. For functions explicitly typed `ese f(x: Ogbe.File)`, the compiler emits `CallOdu 0x00` inline — no string resolution, no registry lookup overhead.

Required changes:
- Add `resolved_domain: Option<u8>` to the `OduCall` AST node in `ifa-types/src/ast.rs`.
- In `ifa-compiler`, when a call site's domain is statically known, set `resolved_domain` and emit a specialized `CallOdu <domain_id>` instead of a dynamic dispatch sequence.
- `ifa-babalawo` already validates `OduDomain` — no new analysis required; just plumb the result through.
- **No `vm.rs` changes needed.** The VM already handles `CallOdu` correctly.

Expected gain: eliminates one `HashMap` lookup per Odù call in the hot path.

---

## Stream F — Dead Module Wiring

*Source: `dead module wiring.md`, `vm_registry.rs` audit*

> [!IMPORTANT]
> **Audit Finding:** The `vm_registry.rs` `import()` block defines 30 domains. Only 16 are handled in the `call()` dispatcher. There are **14 dead domains** that silently fail at runtime, not 7 as originally planned.
>
> **Prerequisite:** Stream A Fix 4 (bincode limits) must be applied before Fix 5 (storage wiring).
> **Prerequisite:** Stream A Fix 6 (registry isolation) changes the dispatch shim signatures. If A6 is done first, all dispatch shims use `ctx.registry` instead of global `REGISTRY`.

### Canonical Pattern

Every fix follows:
1. Write `dispatch(method: &str, args: Vec<IfaValue>) -> IfaResult<IfaValue>`.
2. Convert Rust types to `ResourceToken` via registry (for stateful types).
3. Wire into `vm_registry.rs` `call()` match arm.

Add `extract_token()` utility to `vm_registry.rs` once.

### F1. `kernel.rs` (Domain 29) — 2 hours

Synchronous, stateless. Wire `num_cores`, `total_memory`, `available_memory`, `uptime`.

### F2. `crypto.rs` (Domain 23) — 4 hours

Synchronous, stateless. Hex-encode all byte returns. Wire `sha256`, `hmac_sha256`, `password_hash`, `password_verify`, `base64_encode`, `base64_decode`, `uuid`, `random_bytes`.

### F3. `irosu.rs` Audio (Domain 4) — 3 hours

Wire `siro`/`play` audio path. Register `AudioHandle` in registry. Wire `duro_orin`/`stop`, `ohun`/`volume`.

### F4. `cpu.rs` (Domain 18) — 1 day

Wire `configure`. **Drop `par_map` as VM-callable** — generic Rust closures cannot accept `IfaValue::Closure`.

### F5. `storage.rs` (Domain 20) — 2 days

**After A4 (bincode fix).** Bridge async via `SysRuntime::block_on`. Register `OduStore` in registry. Wire `open`, `set`, `get`, `delete`, `compact`.

### F6. `otura.rs` Networking (Domain 12) — 2 days

Bridge async via `SysRuntime::block_on`. Register `TcpListener`/`TcpStream` in registry. Wire `gba`/`get`, `ran`/`post`, `de`/`listen`, `soro`/`connect`.

### F7. `gpu.rs` (Domain 19) — 1 week

**Async executor: `SysRuntime::block_on`** (not pollster — unified with storage and otura).

Delete the conditional `new_blocking()` that relies on `pollster`. Register `GpuContext` in registry. Wire `init`, `dispatch_pipeline`, `read_buffer`.

### F8. `stacks/ml.rs` — Library/package follow-up [DEFERRED]

This is no longer VM domain wiring. `ml` should be treated as a library/package surface, with any remaining extraction work handled outside `vm_registry.rs`.

### F9. `stacks/gamedev.rs` — Library/package follow-up [DEFERRED]

This is no longer VM domain wiring. `gamedev` follows the same package-oriented path as `ml`.

### F10. `stacks/iot.rs` — Library/package follow-up [DEFERRED]

This is no longer VM domain wiring. `iot` remains platform-dependent, but the work now belongs to the library/package layer.

### F11–F13. Remaining Unwired VM Domains (Batch D) — Unscheduled

The following domains exist in `import()` but still have no dispatch logic:
- Domain 13: `Irete`
- Domain 16: `Coop`
- Domain 17: `Opele`
- Domain 21: `Backend`
- Domain 22: `Frontend`
- Domain 28: `Fidio`

### Final `vm_registry.rs` State

```rust
fn call(&self, domain_id: u8, method_name: &str, args: Vec<IfaValue>, ctx: &mut VmContext) -> IfaResult<IfaValue> {
    match domain_id {
        0  => dispatch_ogbe(method_name, args),
        1  => dispatch_oyeku(method_name, args),
        2  => dispatch_iwori(method_name, args),
        3  => self.dispatch_odi(method_name, args),
        4  => self.dispatch_irosu(method_name, args),
        5  => dispatch_owonrin(method_name, args),
        6  => dispatch_obara(method_name, args),
        7  => dispatch_okanran(method_name, args),
        8  => dispatch_ogunda(method_name, args),
        9  => dispatch_osa(method_name, args, ctx),
        10 => dispatch_ika(method_name, args),
        11 => dispatch_oturupon(method_name, args),
        12 => dispatch_otura(method_name, args),
        14 => crate::ose::Ose::dispatch(method_name, args),
        15 => dispatch_ofun(method_name, args),
        18 => crate::infra::cpu::dispatch(method_name, args),
        19 => crate::infra::gpu::dispatch(method_name, args),
        20 => crate::infra::storage::dispatch(method_name, args),
        23 => crate::stacks::crypto::dispatch(method_name, args),
        29 => crate::infra::kernel::dispatch(method_name, args),
        _  => Err(IfaError::Custom(format!("Unknown Odù domain ID: {}", domain_id))),
    }
}
```

---

## Stream G — Language Features

*Source: `plan.md`*

### Tier 1 — Phase 2 (Unblocked Now)

| # | Feature | Time | Files |
|---|---|---|---|
| G1 | `%=` ModAssign | [✅ DONE] | `grammar.pest`, `ast.rs`, parser, compiler, interpreter |
| G2 | `??` null coalescing (and negative index) | [✅ DONE] | `vm.rs` GetIndex, `interpreter/core.rs` |
| G3 | static array shapes (and `**` exponentiation) | [PARTIAL] | `grammar.pest`, `ast.rs`, parser, compiler, interpreter, `vm.rs` (fix mixed types) |
| G4 | `??` null coalescing | 3 hrs | `grammar.pest`, `ast.rs`, parser, compiler (Dup+PushNull+Eq+JumpIfFalse), interpreter |
| G5 | `ayanfe` const VM enforcement | 4 hrs | `ifa-bytecode` (new `MarkConst` opcode), `vm.rs` (CallFrame.const_slots), compiler |
| G5.5 | `alias` compile-time substitution *(Mojo steal)* | 2 hrs | `ifa-compiler` (inline `PushInt`/`PushStr` at reference sites), E5 `Ikin` dedup required first |

### Tier 2 — Phase 2 (After Conformance Gate)

| # | Feature | Time | Files |
|---|---|---|---|
| G6 | B2 match exhaustiveness (wildcard-only) | 2 hrs | `ifa-babalawo/src/checks.rs` |
| G7 | B1 resource leak detection (heuristic) | 1 day | `ifa-babalawo/src/checks.rs` |
| G8 | B3 purity violation (`#[pure]` attribute) | 1 day | `ast.rs` (add `is_pure` to EseDef), `ifa-babalawo/src/checks.rs` |
| G8.5 | `abo ese` strict function declarations *(Mojo steal: `fn` vs `def`)* | 1 day | `grammar.pest`, `ast.rs` (add `is_strict` to `EseDef`), `ifa-babalawo` (enforce typed params + no implicit coercion), compiler (prefix `MarkConst` opcodes for params) |
| G8.6 | `__ebo__` lifecycle destructor hook *(Mojo steal: `__del__`)* | 1 day | `ast.rs` (detect `ese __ebo__` in `OduDef`), `ifa-babalawo` (enforce: no raise, no alloc, no spawn inside `__ebo__`; G7 leak detection calls it), compiler (emit `__ebo__` call before `EboEpoch::end`) |

### Tier 3 — Phase 5 (After Phase 2 Ships)

| # | Feature | Time | Blocker |
|---|---|---|---|
| G9 | `iru` sum types (ADTs) | 1–2 weeks | New `IfaValue::Variant`, 3 new opcodes |
| G10 | `wo` structural fold | 1 week | Depends on G9 |
| G11 | `pade` structural generate | 3 days | Depends on G9 |
| G11.5 | Static array shape validation *(Mojo steal: `Tensor[DType, *shape]`)* | 1 week | Complete `TypeHint::Array { element_type, size }` in Babalawo; reject out-of-bounds literal index at compile time; compiler emits `BuildList N` with static count. Depends on G9 (`iru`) for typed element enforcement. |
| G11.6 | `ogbon` protocol declarations *(Mojo steal: Traits)* | 2 weeks | New `OgbonDef` AST node; Babalawo verifies `OduDef` conformance (required `ese` methods exist); foundation for making Odù domain typing safe (e.g. `Closeable` enforces `.close()` call). Depends on G9 + G8.6. |

---

## Stream H — Ẹgbẹ́ Concurrency

*Sources: `egbe.md`, `future.md`*

> [!WARNING]
> **Conflict resolved:** `egbe.md` proposes `bincode`-serialized message passing. `future.md` proposes `tokio::sync::mpsc` with `IfaValue` passed directly. **Decision:** Direct transfer for local actors. Serialize only for remote/cross-process (deferred). Erlang serializes because of per-process GC. Ifá-Lang has no per-process GC — ownership transfer via `mpsc` is safe and zero-copy.

> [!WARNING]
> **Conflict resolved (updated):** `egbe.md` says `IfaValue` will NOT become `Send`. The H0 migration now has `IfaFn` and `Object` on the `Arc` path, and `value_union.rs:80` already has `pub type UpvalueCell = Arc<Mutex<IfaValue>>`.
>
> **Decision:** Two-stage migration. Stage 1 (`Rc→Arc`, plus `Object` on `Mutex`) achieves `IfaValue: Send` and unlocks actors. Stage 2 (`Mutex`/`RwLock` policy for the shared object path) is only needed later for `IfaValue: Sync` and reactive UI (`Ose` domain actors).

### H0/H2. `IfaValue: Send` — Two-Stage Migration [✅ DONE]

**Stage 1 — Achieves `IfaValue: Send` (complete; unlocks actors, Osa.sa, Osa.oju_ona::<IfaValue>)**

```rust
// value.rs:25
// Before
pub type IfaFn = Rc<dyn Fn(Vec<IfaValue>) -> IfaValue>;
// After
pub type IfaFn = Arc<dyn Fn(Vec<IfaValue>) -> IfaValue + Send + Sync>;

// value.rs:45
// Before
Object(Rc<RefCell<HashMap<Arc<str>, IfaValue>>>),
// After
Object(Arc<Mutex<HashMap<Arc<str>, IfaValue>>>),

// value_union.rs:80
// Before
pub type UpvalueCell = Rc<RefCell<IfaValue>>;
// After
pub type UpvalueCell = Arc<Mutex<IfaValue>>;
```

**Stage 2 — Achieves `IfaValue: Sync` (unlocks `Signal<IfaValue>`, reactive bindings)**

```rust
// value.rs:45
Object(Arc<RwLock<HashMap<Arc<str>, IfaValue>>>),

// value_union.rs:80
pub type UpvalueCell = Arc<Mutex<IfaValue>>;
```

> [!NOTE]
> `freeze()` / `IfaShared` is NOT made redundant by this change. `freeze()` is for *shared concurrent access* (multiple actors reading the same data). Actor `mpsc` send is for *ownership transfer* (sender loses the value). Both remain meaningful. `freeze()` is now the explicit, documented choice when you want sharing; actor send is when you want movement.

### H1. Move Tracking in Babalawo [✅ DONE]

**Can start Week 2 in parallel with Stream G** — touches only `ifa-babalawo/src/checks.rs`, no conflict.

Add `moved_vars: HashSet<String>` to `LintContext`. Mark variables as moved on `Osa.ebo()`. Emit `USE_AFTER_MOVE` on subsequent access. Clear on `exit_function()`.

### H1.5. Borrow Annotations (`borrowed` / `inout` / `owned`) *(Mojo steal)*

**Babalawo-only — VM-transparent. Can run in parallel with H1.**

Add optional parameter convention keywords to `ese` (and `abo ese` from G8.5):

```ifa
ese swap(inout a: Int, inout b: Int) { ... }
ese log(borrowed msg: Str) { ... }
ese consume(owned data: List) { ... }
```

- `borrowed` → Babalawo enforces no mutation of the parameter within the function body.
- `owned` → Babalawo triggers the H1 `moved_vars` mechanism at the call site. Caller cannot use the variable after the call.
- `inout` → Babalawo enforces that the variable at the call site is defined and mutable, and treats it as mutated after the call.

These annotations are **erased after analysis** — the compiler emits identical bytecode as unannotated parameters. Zero VM changes. Zero `no_std` risk.

Required changes: `grammar.pest` (new tokens), `ast.rs` (add `ParamConvention` enum to `Param`), `ifa-babalawo/src/checks.rs` (enforce per-convention rules).

### H2. Actor Runtime + Per-Actor Opon [✅ DONE]

**Prerequisite: H0 Stage 1 must be merged first.**

New file: `ifa-core/src/actor.rs`. Define `EboMessage`, `ActorHandle`, `EboError`. Implement `spawn_actor` with bounded `tokio::sync::mpsc` mailbox (default depth 64).

**Per-actor Opon:** Each actor gets its own `Opon::new(size)` on spawn. Size is determined by a `#opon` directive scoped to the `egbe` block:

```ifa
#opon kekere
egbe CounterActor {
    ayanmo count = 0;
    gba(msg) { count = count + 1; }
}
```

Required changes:
- Add `opon_size: OponSize` field to `EgbeDef` AST node (`ifa-types/src/ast.rs`)
- Compiler encodes `OponSize` into the egbe bytecode header
- `spawn_actor` reads size and calls `Opon::new(size)`

**Epoch-per-message pattern** (deterministic memory reclaim, no GC):
```rust
while let Ok(msg) = rx.recv() {
    opon.begin_epoch("message_handler");
    vm.push(msg.payload);
    vm.execute_handler();
    opon.end_epoch().unwrap(); // drops all message temporaries
}
```
Actor persistent state lives in an outer epoch opened at init and closed at actor death.

**New Babalawo rule:** Actors must not declare `#opon ailopin`. 100 actors × 1M slots × 48 bytes/slot = 4.8GB potential. Hard error:
```
EGBE_OPON_AILOPIN: Actor memory cannot be 'ailopin'. Actors must have bounded
memory. Use '#opon nla' as the largest permitted actor size.
```

**Opon is `!Send` today** because it contains `Vec<IfaValue>` and `IfaValue: !Send`. After H0 Stage 1, `Opon: Send` is automatic — no changes to `opon.rs` required.

### H3. `daro` Async Enforcement

Enforce that `&mut` borrows do not cross `daro` suspension points. Hard error `MUTABLE_BORROW_ACROSS_DARO`.

### H4. `iwori.yipo.ori` Parallel-For Gate

New `ParallelFor` opcode. Babalawo gate: reject `&mut` or `IfaShared` mutation in parallel body. VM dispatch to `rayon::par_iter`.

### H5. GPU Dispatch Safety

GPU calls require `ailewu` block. Babalawo enforces: no outstanding borrows on dispatched data. GPU send = move (same `moved_vars` mechanism as H1).

---

## Master Execution Schedule

```
Week 1 ─────────────────────────────────────────────────────────
  Stream A: A1 (15m) → A2 (30m) → A3 (10m) → A4 (20m)    [CRITICAL PATH]
  Stream B: B1 (30m) → B2 (1hr)                            [parallel]
  Stream C: C1 (2hr)                                        [parallel]

Week 2 ─────────────────────────────────────────────────────────
  Stream A: A5 (45m) → A6 (2hr) → A8 (1hr)
  Stream B: B3 (4hr) → B4 (1day)
  Stream C: C2 (4hr)
  Stream G: G1 (30m) → G2 (1hr) → G3 (2hr) → G4 (3hr)    [parallel]
  Stream H: H1 (2hr) → H1.5 (4hr) ← Babalawo only         [parallel]
            H1.5 = borrow annotations (borrowed/inout/owned) [Mojo steal]

Week 3 ─────────────────────────────────────────────────────────
  Stream C: C3 (4hr) → C4 (2hr)
  Stream E: E1 (4hr) → E2 (1day)
  Stream F: F1 (2hr) → F2 (4hr) → F3 (3hr)                [parallel]
  Stream G: G5 (4hr) → G5.5 (2hr) ← alias substitution    [Mojo steal]
  Stream H: H0 Stage 1 — IfaValue: Send (Rc→Arc)           [after E1]
            [touches ifa-types/src/value.rs + value_union.rs]

Week 4 ─────────────────────────────────────────────────────────
  Stream D: D1 (4hr) → D2 (4hr) → D3 (2hr)
  Stream E: E3 (4hr) → E4 (2hr) → E5 (1hr) → E6 (4hr)    [Mojo steal]
            E6 = Constant Divination specialization (compiler-only)
  Stream F: F4 (1day) → F5 (2day)                          [after A4]
  Stream H: H2 — actor runtime + per-actor Opon            [after H0]

Week 5–6 ───────────────────────────────────────────────────────
  Stream A: A7 (1day)
  Stream D: D4 (4hr) → D5 (4hr) → D6 (2hr)
  Stream F: F6 (2day) → F7 (1week)
  Stream G: G6 (2hr) → G7 (1day) → G8 (1day)
            G8.5 (1day) → G8.6 (1day)                      [Mojo steals]
            G8.5 = abo ese strict functions
            G8.6 = __ebo__ lifecycle hooks
  Stream H: H3 (daro enforcement) → H4 (parallel-for gate) [parallel with F6]

Month 2+ ───────────────────────────────────────────────────────
  Stream G: G9–G11 (iru/wo/pade)
            G11.5 (1week) → G11.6 (2weeks)                 [Mojo steals]
            G11.5 = static array shape validation
            G11.6 = ogbon protocol declarations
  Stream H: H0 Stage 2 (RefCell→Mutex, IfaValue: Sync)
            H5 (GPU dispatch safety)
  NOTE: H0 Stage 2 is only needed before reactive Signal<IfaValue> UI work.
        Do not block H2–H4 on Stage 2.

```

---

## Appendix A — Conflicts Resolved Between Documents

| # | Conflict | Documents | Resolution |
|---|---|---|---|
| 1 | `egbe.md` says serialize all messages via bincode. `future.md` says pass `IfaValue` directly via tokio mpsc. | `egbe.md` vs `future.md` | **Direct transfer for local actors.** Serialization is unnecessary overhead without per-process GC. Serialize only for remote/cross-process (deferred). |
| 2 | `egbe.md` says "IfaValue will NOT become Send+Sync". `future.md` requires `mpsc::Sender<EboMessage>` which requires `IfaValue: Send`. | `egbe.md` vs `future.md` | **IfaValue must become Send. Two sites still need Stage 1 changes, and UpvalueCell is already on the Stage 2 shape.** (1) `IfaFn`: `Rc<dyn Fn>` → `Arc<dyn Fn + Send + Sync>`. (2) `Object`: `Rc<RefCell<...>>` → `Arc<RefCell<...>>` (Stage 1) then `Arc<RwLock<...>>` (Stage 2). `UpvalueCell` is already `Arc<Mutex<IfaValue>>` in the current tree. Stage 1 is sufficient for actors. Stage 2 is needed only for `Signal<IfaValue>` reactive bindings. `freeze()`/`IfaShared` remains: it is for *sharing* (concurrent read), not for moving. Actor send is for *ownership transfer*. Distinct and both necessary. |
| 3 | `cargo hygiene.md` Phase 5 defers `ifa-parser` extraction. `crate split.md` Phase 1 extracts `ifa-parser`. | `cargo hygiene.md` vs `crate split.md` | **Do both.** Feature-gate first (Stream C3), then extract (Stream D1). The feature gate makes extraction mechanical. |
| 4 | `dead module wiring.md` dispatch shims use global `REGISTRY`. `security.md` Fix 6 isolates the registry per-VM. | `dead module wiring.md` vs `security.md` | **Apply A6 first.** All dispatch shims must accept `&ResourceRegistry` from `VmContext`, not reference the global. |
| 5 | `crate split.md` says rename `ifa-core` → `ifa-vm`. `vm pla.md` refactors within `ifa-core`. | `crate split.md` vs `vm pla.md` | **Refactor first (Stream E), rename after (Stream D5).** Refactoring within a stable name is safer. The rename is mechanical find-replace. |
| 6 | `plan.md` `iru`/`wo` features say "Phase 5". `future.md` parallel-for says "Phase 4". Both use "Phase" numbering that conflicts. | `plan.md` vs `future.md` | **Unified into Stream G (Tier 3) and Stream H.** No more conflicting phase numbers. |

---

## Appendix B — File Change Density Map

| File | Streams Touching It |
|---|---|
| `vm.rs` | A1, A2, A6, E1–E4, G2–G5 |
| `vm_ikin.rs` | A3, E5 |
| `vm_iroke.rs` | A2 |
| `vm_registry.rs` | F1–F7, **E6** (Mojo steal: domain-locked CallOdu) |
| `ffi.rs` | A5, B3, B4 |
| `ebo.rs` | B2 |
| `storage.rs` | A4, F5 |
| `ifa-core/src/lib.rs` | B1, C3 |
| `ifa-std/src/lib.rs` | B1 |
| `ifa-std/Cargo.toml` | C1, C2 |
| `ifa-babalawo/Cargo.toml` | C1, C4 |
| `ifa-babalawo/src/checks.rs` | G6–G8, **G8.5, G8.6**, H1, **H1.5**, H3–H5 |
| `grammar.pest` | G1, G3, G4, G5, **G8.5** (`abo`), **H1.5** (borrow keywords) |
| `ast.rs` | G1, G3, G4, G5, G8, G9, **E6** (`resolved_domain`), **G8.5** (`is_strict`), **G8.6** (`__ebo__`), **G11.6** (`OgbonDef`), **H1.5** (`ParamConvention`), **H2** (EgbeDef.opon_size) |
| `native.rs` | A6 |
| `registry.rs` | A6 |
| `interpreter/core.rs` | A1, G1–G5 |
| `ifa-types/src/value.rs` | **H0** Stage 1 + Stage 2 (Object variant) |
| `ifa-types/src/value_union.rs` | **H0** Stage 2 only (UpvalueCell already migrated) |
| `ifa-core/src/opon.rs` | **None** — no changes needed, inherits Send after H0 |
| `ifa-core/src/actor.rs` | **H2** (new file) |

> [!CAUTION]
> `vm.rs` is touched by 3 streams (A, E, G). **Do not parallelize changes to `vm.rs`.** Execute A first, then E, then G.

> [!CAUTION]
> `value.rs` and `value_union.rs` are touched by H0 which has two stages. **Stage 1 and Stage 2 must be separate PRs.** Stage 1 merges in Week 3. Stage 2 is Month 2+. Do not combine — Stage 2 changes `Object` from `RefCell` to `RwLock`, which changes borrow semantics and may break interpreter code that calls `o.borrow()` expecting infallibility.

## Appendix C — Opon/Actor Interaction Summary

| Concern | Action | File |
|---|---|---|
| `Opon: !Send` today | Automatically fixed after H0 Stage 1 | `opon.rs` — no changes |
| Per-actor Opon size | Add `opon_size: OponSize` to `EgbeDef` | `ifa-types/src/ast.rs` |
| Compiler encodes size | Read `#opon` directive inside `egbe` block | `ifa-core/src/compiler.rs` |
| Runtime uses size | `spawn_actor` calls `Opon::new(opon_size)` | `ifa-core/src/actor.rs` |
| Epoch per message | `begin_epoch` on recv, `end_epoch` after handler | `ifa-core/src/actor.rs` |
| Block `ailopin` in actors | New Babalawo error `EGBE_OPON_AILOPIN` | `ifa-babalawo/src/checks.rs` |
| Observability | Future: `Osa.opon_high_water(actor_ref)` | `ifa-std/src/osa.rs` (deferred) |

---

## Appendix D — PR Queue (One PR Per Safe Merge Unit)

The weekly schedule is planning guidance. The merge queue below is the actual execution unit. If a week slips, the PR order does not change.

| PR # | Scope | Includes | Must Be True Before Merge |
|---|---|---|---|
| PR-01 | Security floor | A1, A2, A3, A4 | Stack/fuel/intern/snapshot limits covered by tests |
| PR-02 | Unsafe policy | B1, B2 | New crate attributes compile clean; `ebo.rs` unsafe blocks documented |
| PR-03 | Workspace hygiene | C1 | `cargo check --workspace` clean with unified deps |
| PR-04 | FFI gate | A5, A8, B3 | Native lib loading capability-gated; path canonicalized; no raw null pointers |
| PR-05 | Registry isolation | A6 | No new VM path reads global `REGISTRY` directly |
| PR-06 | Parser/compiler feature gates | C2, C3, C4 | Default feature graph sane; `ifa-babalawo` no longer depends on `ifa-core` |
| PR-07 | VM dispatch cleanup | E1, E2 | `call_value()` extracted; save/restore duplication removed |
| PR-08 | Tier 1 language batch A | G1, G2 | Assignment/index semantics tested in VM and interpreter |
| PR-09 | Tier 1 language batch B | G3, G4, G5 | New operators + const enforcement tested end-to-end |
| PR-10 | FFI fuzzing | B4 | Fuzz target builds and runs locally in smoke mode |
| PR-11 | Embedded parity | A7 | Embedded VM and main VM agree on shared bytecode corpus |
| PR-12 | VM state refactor | E3, E4, E5 | Execution-context extraction preserves behavior and improves locality |
| PR-13 | Dead wiring batch A | F1, F2, F3 | Kernel/crypto/audio domains wired with registry-backed handles |
| PR-14 | `IfaValue: Send` Stage 1 | H0 Stage 1 | Actor precondition complete; `IfaFn`, `Object`, and `UpvalueCell` are on the sendable path |
| PR-15 | Actor linting | H1 | `USE_AFTER_MOVE` lands before runtime actor send |
| PR-16 | Dead wiring batch B | F4, F5 | CPU/storage dispatch stable; storage path respects bounded deserialize |
| PR-17 | Actor runtime | H2 | Per-actor Opon, bounded mailbox, request-reply path working |
| PR-18 | Async/parallel analysis | H3, H4 | `daro` mutable-borrow rule and parallel-for gate enforced |
| PR-19 | Crate split batch A | D1, D2, D3 | Parser/compiler/transpiler crates extracted and building |
| PR-20 | Crate split batch B | D4, D5, D6 | Interpreter isolated, `ifa-core` renamed, workspace green |
| PR-21 | Dead wiring batch C | F6, F7 | Networking/GPU wired through shared runtime bridge |
| PR-22 | Tier 2 language analysis | G6, G7, G8 | Babalawo-only checks land after runtime churn settles |
| PR-23 | Sync migration | H0 Stage 2, H5 | `IfaValue: Sync` only when reactive/UI work is ready |
| PR-24 | Mojo steals batch A | G5.5, E6 | `alias` substitution requires E5 Ikin dedup; E6 requires F-series domain IDs stable |
| PR-25 | Mojo steals batch B | G8.5, G8.6, H1.5 | Strict functions and borrow annotations are Babalawo-only; `__ebo__` requires G7 (leak detection) merged first |
| PR-26+ | Deferred epics + Mojo Tier 3 | F8, F9, F10, G9-G11, G11.5, G11.6 | Static arrays and `ogbon` protocols require `iru` (G9) and `__ebo__` (G8.6) both merged |

### Why This PR Grouping

1. Security limits land before any new wiring increases attack surface.
2. Registry isolation lands before dead-module dispatch work, so the shims are written once.
3. Tier 1 language features are split into two PRs because `vm.rs` is already hot from security and refactoring work.
4. `IfaValue: Send` lands before actor runtime, but `IfaValue: Sync` is postponed until there is a real consumer.
5. Crate split is deliberately late because it is mostly structural churn with high merge-conflict risk and low user-visible value.

---

## Appendix E — Verification Gates Per PR

Every PR must satisfy three gates: local compile, targeted tests, and one regression probe for the touched subsystem. Do not merge on compile alone.

| Area | Minimum Verification |
|---|---|
| Security limits | `cargo test -p ifa-vm` plus new recursion/fuel/intern/snapshot regression tests |
| Unsafe policy / FFI | `cargo check --workspace`; FFI unit tests; fuzz target smoke build |
| Cargo/workspace | `cargo check --workspace --all-features` and `--no-default-features` where supported |
| VM refactor | Existing VM tests plus bytecode fixture replay before/after refactor |
| Language features | Parser tests, compiler golden tests, VM execution tests, interpreter parity tests |
| Dead module wiring | Dispatch-level unit tests plus one integration script per wired domain |
| Actor runtime | Multi-actor integration test: spawn, send, mailbox-full, dead-actor, request-reply |
| Crate split | `cargo build --workspace`, `cargo test --workspace`, docs/examples still resolve imports |

### Required Regression Probes

| Probe | Why |
|---|---|
| Deep recursion program | Proves A1 did not regress into unbounded frame growth |
| Tight infinite loop with fuel | Proves A2 triggers predictably |
| Snapshot with oversized payload | Proves A4 rejects hostile inputs |
| Native library load outside allowed roots | Proves A5/A8 deny escaped paths |
| Stateful dispatch round-trip | Proves A6/F-series registry token flow works |
| Closure send attempt to actor | Proves H1/H2 ownership model is enforced |
| `daro` with live `&mut` borrow | Proves H3 rejects unsound suspension |
| Parallel body mutating shared state | Proves H4 gate catches illegal captures |

### Merge Stop Conditions

Do not merge if any of the following is true:

1. A PR changes `vm.rs` and crate structure simultaneously.
2. A PR introduces new `unsafe` without a `// SAFETY:` justification.
3. A PR adds a new async bridge that does not use the same runtime strategy as storage/otura/gpu.
4. A PR makes `IfaValue: Sync` before actor send semantics have shipped and settled.
5. A PR wires a stateful domain through the global registry after A6 has merged.

---

## Appendix F — Sequencing Rules For Humans

These are the operational rules that keep parallel work from colliding.

### Rule 1: `vm.rs` Has a Single Owner At Any Time

`vm.rs` is the highest-conflict file in the repo. Ownership passes in this order only:

`A-series security` -> `E-series refactor` -> `G-series language` -> `H-series actor hooks`

No exceptions. If two PRs need `vm.rs`, the later PR rebases after the earlier one merges.

### Rule 2: Babalawo Can Parallelize Safely

`ifa-babalawo/src/checks.rs` can advance in parallel with most runtime work if the changes are split by concern:

- H1 can land while G Tier 1 is in flight.
- G6-G8 can land after Tier 1 runtime semantics are stable.
- H3/H4 should rebase on top of H1 to reuse move-tracking machinery.

### Rule 3: Structural Churn Waits For Semantic Churn To Settle

Do not start D1-D6 while these are still moving:

- `ifa-core` module boundaries
- parser/compiler public APIs
- imports in `ifa-babalawo`
- any rename involving `ifa_core`

If the crate split starts too early, every live branch pays the rename tax.

### Rule 4: Async Bridges Must Converge On One Runtime Story

Storage, networking, GPU, and actors must not invent four executor patterns.

- Preferred bridge: `SysRuntime::block_on`
- Actor internal scheduling: `tokio`
- Explicitly rejected for new work: ad hoc `pollster` paths, bespoke thread-per-call shims, hidden background runtimes

### Rule 5: Deferred Means Deferred

The following stay out until the merge queue reaches them:

- F8 `stacks/ml.rs`
- F9 `stacks/gamedev.rs`
- F10 `stacks/iot.rs`
- G9-G11 `iru` / `wo` / `pade`
- H0 Stage 2 and H5 unless reactive/shared UI work is actively blocked on them

Starting deferred work early will increase branch divergence without reducing current delivery risk.

---

## Appendix G — First Three PRs To Open Now

If work starts immediately, open these in order:

1. `PR-01 Security floor` — smallest critical-path change set with the highest risk reduction.
2. `PR-02 Unsafe policy` — independent of runtime semantics and unlikely to conflict.
3. `PR-03 Workspace hygiene` — unlocks later crate work and reduces incidental build noise.

That gives the codebase a safer runtime envelope, a cleaner dependency graph, and a lower-conflict base before the larger refactors begin.

---

## Stream I — Performance Engineering (The Engine Block)

*Sources: `performance_engineering.md`, `dsa_audit.md`, `linus_audit.md`*

> [!CAUTION]
> **Linus Review Verdict:** These are not optimizations. They are *prerequisite corrections* to the data layout. The VM cannot be taken seriously until I1–I3 ship. Do not start Stream K (features) until I1 is merged.

### I0. NaN Boxing (The Odù Signature) [CRITICAL — 400 lines]

**Files:** `ifa-types/src/value_union.rs` (complete rewrite)

Pack `IfaValue` into 8 bytes using IEEE 754 NaN signaling space. 56 → 8 bytes per stack value. 7× less memory traffic. 

> [!WARNING]
> **Contradiction Resolved:** NaN boxing makes `IfaValue` an opaque `u64`. This conflicts with H0 Stage 1 (`Rc→Arc` migration) because primitives no longer use reference counting at all. **Decision: NaN Boxing wins.** We do not pay the `Arc` clone penalty for integers and booleans. Primitives are pure 8-byte copies. Only heap objects (Strings, Lists) will be wrapped in `Arc` for actor message passing.

#### Architectural Constraints (The Philosophy of Safe `unsafe`)

NaN Boxing in Rust is incredibly dangerous (memory leaks, double frees, loss of pattern matching). We mitigate this by strictly enforcing Ifá philosophical boundaries around the `unsafe` code:

1. **The Babalawo API (Data Hiding):** The VM (`vm.rs`) must **never** perform a bit-shift or read the raw `u64`. All bit-twiddling must be locked inside `ifa-types/src/value_union.rs`. The VM may only use safe getters (`val.is_int()`, `val.as_int()`).
2. **Ìwà vs Odù (Tags vs Payloads):** Bit-masks must be explicitly defined as constants (`IWA_INT`, `IWA_PTR_STR`). Never use raw hex values in logic. The top 13 bits are the Ìwà (Type Tag), the bottom 51 bits are the Odù (Payload/Pointer).
3. **The Ẹbọ Ritual (The `Drop` Trait):** The custom `Drop` trait is the only place memory is freed. It must rigorously check the Ìwà tag before performing the sacrifice (`Arc::from_raw`). **Linus Mandate:** The `Drop` trait must pass exhaustive Miri (Rust memory sanitizer) validation before the VM is allowed to use the new `IfaValue`.
4. **Ọpọ́n Ifá (The Boundary & Embedded Targets):** We cannot enforce 64-bit NaN boxing on 32-bit microcontrollers (`ifa-embedded`) or WebAssembly (`ifa-wasm`) without destroying performance due to register pressure and missing FPU hardware. 
   **Solution (The Dual Architecture):** We will use conditional compilation. 
   - `#[cfg(target_pointer_width = "64")]` uses the high-performance `u64` NaN Box.
   - `#[cfg(target_pointer_width = "32")]` falls back to a safe Rust `enum`.
   Because the VM uses the safe Babalawo API (Rule 1), it will compile and run perfectly on both architectures without knowing the underlying memory layout changed.

### I1. Global Variable HashMap → Vec [CRITICAL — 70 lines]

**Files:** `vm.rs` (L98 `globals` field), `vm.rs` LoadGlobal/StoreGlobal handlers

The compiler already emits a `u16` string index for globals. The VM reads the index, looks up the string in Ikin, then hashes that string into a `HashMap`. This is insane.

```rust
// Before (vm.rs L98):
globals: std::collections::HashMap<String, IfaValue>,

// After:
globals: Vec<IfaValue>,
```

LoadGlobal/StoreGlobal handlers use the `u16` directly as a Vec index. Speedup: 10–40× on global access.

**Prerequisite:** E5 (Ikin dedup) must be done so global indices are stable.

### I2. Opcode Dispatcher Split [HIGH — 200 lines]

**Files:** `vm.rs` step() method (~1,700 lines)

Extract each opcode handler into `#[inline]` private methods. The current monolithic `match` blows L1 instruction cache. Hot opcodes (Push, LoadLocal, Add, Jump) must fit in cache; cold opcodes (Import, BuildMap) must not pollute it.

This is mechanical refactoring — no logic change. Aligns with E1–E3 refactoring work.

### I3. O(1) String Length [HIGH — 40 lines]

**Files:** `ifa-vm/src/vm_ikin.rs` (intern pool cache), `vm.rs` Len handler

Cache Unicode scalar lengths in the VM's intern pool using a pointer-keyed `HashMap<usize, usize>`. The `Len` opcode reads the cached value for interned strings instead of calling `s.chars().count()` on every use.

Preserves Yorùbá-correct character counting. Eliminates O(n) in hot paths without changing the `Str` variant layout.

### I4. Small String Optimization [MEDIUM — 120 lines]

**Files:** `ifa-types/src/value_union.rs`

Inline strings ≤ 22 bytes into the value struct. No heap allocation for short strings. Most Yorùbá keywords and variable names fit. Eliminates ~80% of string heap allocations.

### I5. Small Integer Pool [MEDIUM — 30 lines]

**Files:** `ifa-types/src/value_union.rs`

Pre-allocate `IfaValue::Int` for 0..255. Arithmetic loops stop creating/discarding millions of identical values. Stolen from CPython and JVM.

**Status:** completed in `crates/ifa-types/src/value_union.rs`.

### I7. Lazy Registry Initialization [LOW — 20 lines]

**Files:** `ifa-std/src/vm_registry.rs`

Use `OnceCell` for domain instances. Scripts that don't use I/O skip Irosu/Odi construction. ~1ms startup savings.

**Status:** completed in `crates/ifa-std/src/vm_registry.rs`.

---

## Stream J — True Parallelism Design

*Sources: `rust_route_ast_transpiler_impact.md`, `infra pkan.md`, `egbe.md`, `future.md`, `performance_engineering.md` §2*

> [!IMPORTANT]
> **Architectural Decision:** Ifá-Lang adopts the **Go/Erlang hybrid model**, not the Rust model. The bytecode VM stays single-threaded per actor. Parallelism comes from running multiple actors on multiple OS threads, not from sharing mutable state across threads.

### J0. Design Principles (Stolen from Go, Erlang, Lua, and Ifá)

1. **No shared mutable state.** Actors own their data. Period.
2. **Message passing is the only communication.** Like Erlang processes, like Go channels.
3. **The VM loop is single-threaded per actor.** Like Lua coroutines — each actor has its own stack/IP.
4. **Rayon is for data parallelism only.** CPU-bound bulk operations (map/filter/reduce on large arrays) go through Rayon. They do not touch `IfaValue` closures — only primitive data.
5. **The transpiler generates `tokio::spawn` for actors.** Each actor compiles to a Rust async task with its own state.

### J0.5. The Philosophical Justification (The Ẹgbẹ́ vs Shared Memory)

The Western "Shared Memory" model (C++, Rust) forces multiple threads to fight over a single block of memory, requiring Mutex Locks that cause bottlenecks and deadlocks. 

In Ifá philosophy, you do not put four Babalawos around a single Ọpọ́n Ifá (Divination Tray) and ask them to draw marks simultaneously. That creates chaos (Data Races).

Instead, Ifá-Lang scales through the **Ẹgbẹ́ (The Guild)**:
- You give **each Babalawo their own Ọpọ́n Ifá** (Every Actor gets its own VM instance and private Heap).
- Because the tray is private, they never wait for locks. They cast Ikin at the exact same physical nanosecond across multiple CPU cores. **This is true, lock-free parallelism.**
- When they have answers, they **speak** to each other (Message Passing via channels). 

This philosophy directly mirrors the world's most resilient distributed systems architectures (Erlang/OTP) and completely eliminates the concept of a Deadlock from the language.

### J1. Wire CPU Domain (Domain 18) to VM [CRITICAL — 60 lines]

**Files:** `ifa-std/src/vm_registry.rs`

The Rayon-based CPU module (`infra/cpu.rs`) has `par_map`, `par_filter`, `par_reduce`, `par_sort` — all implemented, all unwired.

```rust
// vm_registry.rs — add to call() match:
18 => crate::infra::cpu::dispatch(method_name, args),
```

**Constraint:** These functions operate on `Vec<f64>` or `Vec<i64>`, NOT on `IfaValue` closures. The programmer passes data arrays. Rayon processes them. No `IfaValue` crosses a thread boundary.

```ifa
// Ifá usage:
ayanmo data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
ayanmo squares = Cpu.par_map(data, "square")  // built-in named ops only
```

### J2. Actor-to-OS-Thread Scheduling [AFTER H2]

**Files:** `ifa-vm/src/actor.rs`

After H2 (actor runtime) ships with single-threaded cooperative scheduling, upgrade the scheduler:

1. Create a `tokio::Runtime` with a thread pool sized to CPU cores
2. Each actor's message loop runs as a `tokio::spawn` task
3. The `mpsc` channels from H2 remain unchanged — they already work across threads
4. Each actor owns its own `IfaVM` instance with its own `Opon`

**Why this works:** Actors share nothing. Each has its own VM, its own stack, its own globals Vec. No locks needed inside the VM hot path. The only synchronization point is the `mpsc` channel, which Tokio handles.

### J3. Transpiler Escape Analysis [AFTER J2]

**Files:** `ifa-babalawo/src/checks.rs`, `ifa-types/src/ast.rs`, `ifa-transpiler/src/transpiler/core.rs`

When transpiling to Rust (`ifa build`), Babalawo must flag variables that cross actor boundaries:

1. Add `escapes_thread: bool` to `VarDecl` in `ast.rs`
2. Babalawo walks closures passed to `Osa.ise()` and marks captured variables
3. Transpiler emits `Arc<RwLock<...>>` for escaped variables, plain `let mut` for local ones

### J4. Parallel-For (Data Parallelism for Loops) [PARTIAL]

**Files:** `grammar.pest`, `ast.rs`, compiler, `vm.rs`

```ifa
// Syntax (Yorùbá):
papọ fun item ninu large_list { ... }

// Syntax (English Alias):
parallel for item in large_list { ... }
```

Compiles to `ParallelFor` opcode. VM dispatches loop body to Rayon `par_iter`. Babalawo gate (H4) rejects `&mut` or shared-state mutation in the body.

### J5. Async I/O Bridge [AFTER J2]

**Files:** `ifa-std/src/handlers/odi.rs`, `ifa-std/src/handlers/otura.rs`

Offload blocking I/O to the Rayon/Tokio thread pool. Return `Future` immediately:

```rust
"ka_async" | "read_async" => {
    let path = args[0].to_string();
    let cell = FutureCell::new();
    let c = cell.clone();
    rayon::spawn(move || {
        let r = std::fs::read_to_string(&path)
            .map(IfaValue::str)
            .unwrap_or_else(|e| IfaValue::str(e.to_string()));
        c.resolve(r);
    });
    Ok(IfaValue::Future(cell))
}
```

---

## Stream K — Missing Language Primitives

*Source: `missing_features_implementation.md`*

> [!CAUTION]
> **Linus Review:** Do NOT start K-series until I1 (globals Vec) and I2 (dispatcher split) are merged. Adding features to a broken engine is painting the roof of a house with no foundation.

### K1. `break` / `continue` [P0 — 140 lines]

**Files:** `grammar.pest`, `ast.rs`, `compiler/lib.rs`

Loops are crippled without these. Add `Statement::Break` and `Statement::Continue` to AST. Compiler tracks a `Vec<LoopContext>` with break-jump patch locations and continue-target offsets.

### K2. Lambda / Anonymous Functions [P0 — 100 lines]

**Files:** `grammar.pest`, `ast.rs`, `compiler/lib.rs`

```ifa
ayanmo double = (x) => x * 2
ayanmo items = Ogunda.yan(list, (x) => x > 5)
```

Parser emits `Expression::Lambda { params, body }`. Compiler treats identically to `EseDef` with generated name `__lambda_N`.

### K3. `reduce` / `fold` [P0 — 110 lines]

**Files:** `ifa-std/src/vm_registry.rs` (Ogunda domain)

Completes the map/filter/reduce functional trio. Requires fixing the registry `take()/put-back` ownership model so the VM can call `IfaValue::Fn` from within a registry dispatcher.

### K4. Set Type [P1 — 200 lines]

**Files:** `ifa-types/src/value_union.rs`, `vm.rs`, `compiler/lib.rs`

Add `Hash + Eq` to `IfaValue`. Add `Set(Arc<HashSet<IfaValue>>)` variant. New opcodes: `BuildSet(count)`, `SetAdd`, `SetHas`, `SetRemove`.

### K5. `else if` Chains [P1 — 30 lines]

**Files:** `grammar.pest`

Trivial grammar fix. Currently requires nesting else blocks manually.

### K6. Default Parameter Values [P2 — 80 lines]

**Files:** `grammar.pest`, `ast.rs`, compiler

```ifa
ese greet(name, greeting = "Hello") { ... }
```

### K7. Source Maps [P1 — 110 lines]

**Files:** `ifa-bytecode/src/lib.rs`, `compiler/lib.rs`, `vm.rs`

Add `line_table: Vec<(usize, u32)>` to `Bytecode`. Compiler emits entries at statement boundaries. VM error messages show source line numbers instead of instruction pointers.

### K8. Ìpa (Side-Effect Tags) [HIGH — 150 lines]

**Files:** `grammar.pest`, `ast.rs`, `compiler/lib.rs`, `ifa-babalawo/src/checks.rs`

Allows the compiler to distinguish between pure functions (safe for Rayon parallelism) and functions that mutate the world or have side effects.

```ifa
// Syntax (Yorùbá):
ese write_log(msg) pelu Ipa {
    // Requires Ipa to modify external state
}

// Syntax (English Alias):
fn write_log(msg) effect {
    // Side-effecting function declaration
}
```

1. Update `grammar.pest` to parse optional `pelu Ipa` (or the English alias `effect`) on `EseDef` and `Lambda`.
2. Add `has_effect: bool` to `EseDef` AST node.
3. `Babalawo` enforces that pure functions cannot call functions with `Ìpa`, and cannot mutate captured state.
4. Rayon operations (`Cpu.par_map`) throw compile errors if passed a function with `Ìpa`.

### K9. Orí (Gradual Type Annotations) [P0 — 80 lines]

**Files:** `grammar.pest`, `ast.rs`

Introduces the syntax for Gradual Typing (Structural Contracts).

```ifa
ayanmo age: Int = 25;
ese calculate_tax(amount: Float) -> Float { ... }
```

1. Update `grammar.pest` to support optional `: Type` after identifiers in `VarDecl` and `EseDef` parameters.
2. Update `grammar.pest` to support optional `-> Type` for function return types.
3. Update AST nodes to hold `Option<TypeAnnotation>`.

### K10. Ọ̀rúnmìlà (AST Type Propagator) [P1 — 200 lines]

**Files:** `ifa-babalawo/src/type_checker.rs` (New)

Implements basic forward data-flow analysis to resolve gradual types before the VM compiles the bytecode.

1. Traverses the AST and builds a Lexical Symbol Table tracking `Type` for variables.
2. If `Babalawo` detects a Type Mismatch (e.g. assigning a `Str` to a variable with an `Int` Orí), it throws a compile error.
3. No SMT solvers. Basic Control Flow Graph (CFG) resolution only. If a type is purely dynamic, it assumes `IfaValue` (Any) and defers to runtime.

### K11. Higher-Order Contracts (AssertType) [P1 — 120 lines]

**Files:** `ifa-bytecode/src/lib.rs`, `compiler/lib.rs`, `vm.rs`

Enforces gradual typing at runtime when dynamic data (e.g. from network or untyped functions) crosses into typed boundaries.

1. Add `OpCode::AssertType(u8)` to `ifa-bytecode`. The `u8` corresponds to an internal type enum (Int, Float, Str, etc.).
2. Update the compiler to emit `AssertType` at the start of any gradually-typed function if the caller's type cannot be statically proven by `Ọ̀rúnmìlà`.
3. Update `vm_iroke.rs` (the dispatcher) to execute the check and throw `IfaError::TypeError` immediately if the NaN-Boxed `IfaValue` does not match the required type.

---

## Stream L — Build & Compile Speed

*Sources: `performance_engineering.md` §3 and §5, `infra pkan.md`*

### L1. Incremental Build Cache [P1 — 20 lines]

**Files:** `ifa-cli/src/main.rs` (Build command)

Replace `std::env::temp_dir()` with `.oja/build_cache`. Don't delete `target/`. Subsequent `ifa build` drops from 30s to 2–5s.

### L2. Parallel Babalawo + Compiler [P1 — 10 lines]

**Files:** `ifa-cli/src/main.rs`

```rust
let (baba_result, bytecode) = rayon::join(
    || babalawo::check_program(&ast),
    || compiler::compile(&ast),
);
```

Both only read the AST. Free 15% wall-clock reduction.

### L3. Slim `ifa-types` [P2 — varies]

Move `Display`, `PartialEq`, `From` impls out of `ifa-types/src/value_union.rs` into `ifa-vm`. Cuts `ifa-types` compile time to <1s, unblocking parallel compilation of `ifa-parser` and `ifa-compiler`.

---

## Appendix H — Linus Review: Contradictions Found Across All Documents

> The following contradictions were identified by reviewing the full corpus of `works/`, `patch plan`, `infra pkan.md`, and the existing unified plan simultaneously.

| # | Contradiction | Documents | Resolution |
|---|---|---|---|
| 7 | Performance doc recommends `Arc→Rc` (§4.1, P1). H0 Stage 1 requires `Rc→Arc` for actors. These are opposite directions. | `performance_engineering.md` vs `unified_plan` H0 | **Kill the `Arc→Rc` recommendation.** If actors ship (H2), values must be `Send`. If NaN boxing ships (I6), reference counting is eliminated for primitives anyway. The `Arc→Rc` path is a dead end that will be reverted. |
| 8 | Performance doc lists CPU/Rayon wiring as P0. Unified plan F4 says "Drop `par_map` as VM-callable — generic Rust closures cannot accept `IfaValue::Closure`." | `performance_engineering.md` vs `unified_plan` F4 | **Both are correct but talking past each other.** F4 is right that `par_map` cannot accept Ifá closures. J1 is right that `par_map` should be wired for primitive arrays (`Vec<f64>`). Wire it with built-in named operations, not user closures. |
| 9 | `infra pkan.md` proposes dynamic dispatch `Box<dyn ComputeBackend>` for GPU/CPU unification. Linus review (same file, L293–333) says "don't use dynamic dispatch in the hot path." | `infra pkan.md` §2A vs `infra pkan.md` §297–299 | **Linus is right.** Use compile-time generics + feature flags for backend selection. Dynamic dispatch only at initialization (`best_available()`), never inside a compute kernel. |
| 10 | `patch plan` proposes moving interpreter logic into Babalawo for constant folding. `unified_plan` Stream D isolates `ifa-interpreter` as a separate crate. These pull in opposite directions. | `patch plan` §1–2 vs `unified_plan` D4 | **Both are valid for different timelines.** D4 (isolate interpreter) is correct *now* — it preserves the REPL. The `patch plan` vision (Babalawo as partial evaluator) is correct *later* — after the VM-based REPL is stable. The interpreter crate should NOT be deleted until `ifa repl` works on the bytecode VM. |
| 11 | Performance doc says "REPL: ❌ Not implemented." Actual codebase has a working REPL (`ifa repl`, main.rs L1097–1219). | `performance_engineering.md` §4.1 vs actual code | **Document was wrong. Corrected in `missing_features_implementation.md`.** REPL exists but uses the tree-walking interpreter, not the VM. Migration to VM-based REPL is a future task, not a missing feature. |
| 12 | Performance doc says "Package Manager: ❌ Not implemented." Oja is 1,482 lines with SemVer, lockfiles, and transitive resolution. | `performance_engineering.md` §4.4 vs `oja.rs` | **Document was wrong. Corrected.** Oja is fully implemented. Remaining gaps: git deps, lock-file diffing. |
| 13 | `infra pkan.md` proposes "Raft-lite replication" for OduStore. Linus review (same file) says "You're building a language runtime, not a cloud database." | `infra pkan.md` §C3 vs `infra pkan.md` §310–314 | **Linus is right.** Defer all distributed storage. Ship a rock-solid single-node append-only log first. Replication is a user-space library, not a runtime primitive. |
| 14 | H0 Stage 1 changes `Rc→Arc` for `IfaValue: Send`. This makes every value clone 10× slower (atomic increment). Stream I wants the VM to be *faster*. | `unified_plan` H0 vs Stream I | **This is the fundamental tension.** Resolution: Ship H0 Stage 1 (`Rc→Arc`) because actors are architecturally necessary. Accept the clone cost. Then recover it via NaN boxing (I6) which eliminates ref-counting for primitives (Int, Float, Bool, Null = 95% of clones). The net effect is: clones of heap objects (Str, List, Map) pay `Arc` cost but are rare; clones of primitives pay zero cost via NaN boxing. |
| 15 | Audit (2026-05-17) claimed the WASM sandbox was "not yet built" and the `oja-registry` didn't exist. Both were wrong. | Audit findings vs actual codebase | **Audit was wrong.** `ifa-sandbox::OmniBox` is a production Wasmtime-backed sandbox with pooling allocator, epoch interruption, AOT compilation, WASI P1, and the `Ewo` capability ABI. The `oja-registry` exists at `oja-registry/packages.json` (empty package list, infrastructure live). No deprecation path is needed for stacks migration — Igbale (`.oja/`) and `OmniBox` handle the transition transparently. |

---

## Stream M — `ifa-infra` Crate Extraction

*Source: `infra pkan.md`, audit finding 2026-05-17*

> [!IMPORTANT]
> **Prerequisites (both must be merged before M1 begins):**
> - **A6 (Registry Isolation):** All dispatch shims must accept `&ResourceRegistry` from `VmContext`, not the global. If M1 starts before A6, the extracted handlers will be written once and then require a second rewrite.
> - **F1–F7 (Module Wiring):** The infra handlers (`cpu.rs`, `gpu.rs`, `storage.rs`, `kernel.rs`) must be wired into `vm_registry.rs` before extraction. Extracting unwired dead code adds structural churn with zero user-visible gain.

> [!WARNING]
> **Design constraint:** The `ComputeBackend` trait (if introduced) MUST use compile-time dispatch — generics and feature flags, not `Box<dyn ComputeBackend>`. Dynamic dispatch in a matrix multiply inner loop is an automatic failure. Every infra abstraction is a zero-cost abstraction or it ships as a monolith.

### M1. Create `crates/ifa-infra` [HIGH — 1 day]

**Files:** Move `ifa-std/src/infra/{cpu,gpu,kernel,storage,runtime,shaders,mod}.rs` → `crates/ifa-infra/src/`

Dependencies for the new crate: `ifa-types`, `ifa-bytecode`. No dependency on `ifa-std` or `ifa-vm`.

Feature flags (mandatory):
```toml
[features]
default = ["cpu", "storage"]
cpu     = ["rayon", "tokio"]
gpu     = ["wgpu", "bytemuck"]     # opt-in — do not compile on headless servers by default
cuda    = ["cudarc"]               # opt-in — NVIDIA only, behind thick feature wall
graphics = ["wgpu", "winit"]       # opt-in — never compiled for server targets
```

A server-side AI worker must be able to build `ifa-infra` with `cpu` + `storage` and pull in **zero windowing or graphics dependencies**.

**Status:** completed in `crates/ifa-infra/`.

### M2. Dispatch Shim Update [MEDIUM — 2 hrs]

Update `ifa-std/src/vm_registry.rs` dispatch arms to call through `ifa-infra` public API instead of inlined `ifa-std/src/infra/` paths. `ifa-std` adds `ifa-infra` as a workspace dependency.

**Status:** completed in `crates/ifa-std/src/vm_registry.rs` and the live stack imports.

### M3. `ComputeBackend` Trait [MEDIUM — 4 hrs]

**Only after M1 and M2 are stable.**

Introduce the unified compute abstraction using generics, not trait objects:

```rust
// ifa-infra/src/compute.rs
pub trait ComputeBackend: Send + Sync {
    fn par_map<T, U, F>(&self, data: &[T], f: F) -> Vec<U>
    where T: Send, U: Send, F: Fn(&T) -> U + Send + Sync;
    fn matmul(&self, a: &[f32], b: &[f32], m: usize, n: usize, k: usize) -> Vec<f32>;
    fn reduce_sum(&self, data: &[f32]) -> f32;
    fn device_info(&self) -> DeviceInfo;
}

// Callers pin to a concrete type — zero dynamic dispatch overhead:
pub fn run_on<B: ComputeBackend>(backend: &B, data: &[f32]) -> f32 {
    backend.reduce_sum(data)
}
```

Provide `best_available<B: ComputeBackend>()` only at the *application* level (in `ifa-cli` or `ifa-std`), not inside the hot path. Hardware selection is a startup decision, not a per-call decision.

**Status:** completed in `crates/ifa-infra/src/compute.rs`, `crates/ifa-infra/src/cpu.rs`, and `crates/ifa-infra/src/gpu.rs`.

### M4. UMA-Aware Memory Strategy [LOW — deferred after M3]

Add `is_unified_memory()` detection to `DeviceInfo` (detect shared memory bus, not "Is this a Mac?"). This makes `ifa-infra` world-class on AMD APUs, Steam Deck, and smartphones — not just Apple Silicon.

For discrete GPU targets: implement **asynchronous DMA transfer** so PCIe data movement happens in the background while the CPU prepares the next task. This is the "slow path" that must be fast.

For headless server targets (no GPU): graceful degrade to AVX/AMX CPU intrinsics via `#[cfg]`. The infra layer must compile and run correctly on a 128-core Linux server with no display.

### M5. `GpuOpon` Integration [LOW — deferred, needs H2 first]

After the actor runtime (H2) ships: per-actor GPU memory pools fixed to available VRAM, managed by `ifa-infra` and surfaced to the VM as a sandboxed heap. Not before.

### M6. Architecture: The "Domain Zero" Bottleneck [DESIGN]

The modules in `ifa-std/src/stacks/` (`ml.rs`, `gamedev.rs`, `iot.rs`, `crypto.rs`, etc.) are currently baked into the VM registry. This is an architectural bottleneck that couples language releases to domain API releases.

- **Phase 1 (Now):** Maintain explicit separation between `ifa-infra` (zero-cost hardware primitives) and `stacks/` (high-level domain APIs). Do not let domain logic leak into the infra extraction (M1).
- **Phase 2 (Future):** The `stacks/` must eventually be extracted *completely out* of `ifa-std` and published to the Oja registry (`oja-registry/`) as official versioned packages (e.g., `AAEO04/ifa-ml`). **No separate deprecation path is needed.** The Igbale (`.oja/`) and `OmniBox` (Wasmtime-backed WASM sandbox in `ifa-sandbox`) handle the transition transparently. A user runs `ifa oja install AAEO04/ifa-ml`; Oja fetches the package, `OmniBox` AOT-compiles the `.wasm` module to a `.cwasm` artifact in `.oja/cache/`, and the VM loads it. The domain API surface from the user's perspective does not change.

### What Does NOT Go Into `ifa-infra`

- Distributed databases, Raft, or sharding — build those *on top of* Ifá-Lang, not inside the infra crate.
- Language-specific types (`IfaValue`, `Bytecode`, AST nodes) — those stay in `ifa-types`.
- Any `async` bridge that doesn't use `SysRuntime::block_on` — one executor story, not four.

---

## Appendix I — Updated Stream Dependency Graph

```
Stream A: Security ──────────────────────────────┐
Stream B: Unsafe Hardening ──────────────────────┤
Stream C: Cargo Hygiene ─── → Stream D: Crate Split  [✅ DONE]
Stream E: VM Refactor ───── → Stream I: Performance ──→ Stream K: Features
Stream F: Module Wiring ──────────────────────────────────────────────┐
Stream G: Language Features ────────────────────┤                     │
Stream H: Concurrency ─── → Stream J: True Parallelism                │
Stream L: Build Speed (independent)                                   ↓
                                            Stream M: ifa-infra Extraction
                                            (needs A6 + F1–F7 first)
```

**Critical path for performance:**
```
E5 (Ikin dedup) [✅] → I1 (globals Vec) [✅] → I2 (dispatcher split) → I3 (string len) → K-series
```

**Critical path for parallelism:**
```
H0 (IfaValue: Send) → H2 (actor runtime) → J1 (CPU wiring) → J2 (actor threading) → J3 (escape analysis)
```

**Critical path for ifa-infra extraction:**
```
A4 (bincode) [✅] → A6 (registry isolation) → F1–F7 (wiring) → M1 (extract crate) → M2 (shim update) → M3 (ComputeBackend trait)
```

---

## Appendix J — Unified PR Queue Update (PR-27 through PR-41)

| PR # | Scope | Includes | Must Be True Before Merge |
|---|---|---|---|
| PR-27 | Engine block fix | E5, I1 | ✅ **DONE** — Globals use Vec; E5 dedup complete; all tests green |
| PR-28 | String length cache | I3 | O(1) string len via pointer-keyed VM intern cache; no `Str` layout change |
| PR-29 | CPU parallelism | J1 | Domain 18 wired; `Cpu.par_map` works on primitive arrays |
| PR-30 | Loop primitives | K1, K2 | `break`/`continue` work; lambda syntax parses and compiles |
| PR-31 | Functional trio | K3 | `reduce`/`fold` works end-to-end with named functions |
| PR-32 | Source maps | K7 | Runtime errors show source line numbers |
| PR-33 | Build cache | L1, L2 | Incremental builds use `.oja/build_cache`; Babalawo runs parallel |
| PR-34 | Actor threading | J2 | Actors run on multiple OS threads via tokio |
| PR-35 | Set type | K4 | `Hash + Eq` on IfaValue; Set operations work |
| PR-36 | Effect tracking | K8 | `pelu Ipa` parses; pure functions blocked from making I/O calls |
| PR-37 | Gradual Types | K9 | `: Type` and `-> Type` syntax parses successfully |
| PR-38 | Type Inference | K10 | `Babalawo` catches static type mismatches |
| PR-39 | Runtime Contracts | K11 | `AssertType` opcode emitted and verified by VM |
| PR-40 | `ifa-infra` extraction | M1, M2 | ✅ Done — `crates/ifa-infra` builds; feature flags enforced; zero new graphics deps on `cpu+storage` build |
| PR-41 | ComputeBackend trait | M3 | ✅ Done — compile-time generic dispatch only; no `Box<dyn>`; UMA detection added |

---

## Appendix K — Architecture & The Linus Review

### Part 1: The Core Architectural Path
Given Ifá-Lang's core philosophy—**Ìjìnlẹ̀** (Deep Essence), **Ọpọ́n** (Strict Boundaries/Sandboxing), and the **Ikin/Iroke** divide (Wisdom vs. Execution)—here is the exact architectural path the language should take.

**1. The FFI Boundary: The "Batched Ọpọ́n" (Shared Memory)**
*The Philosophy:* Ikin (Preparation) and Iroke (The Strike). Ifá-Lang hates hidden costs. A developer shouldn't write a loop that secretly crosses a heavy C-FFI boundary 10,000 times, destroying performance. 
*The Implementation:* Adopt the WebGL / TaskGraph Model. Instead of making fine-grained native function calls, the VM provides an `OponBuffer` (a shared memory block). The developer uses pure Ifá code to calculate and write thousands of commands into this buffer (Ikin — the quiet preparation). Then, they make one single call to hand the buffer to the native hardware or WASM module (Iroke — the decisive strike). This guarantees zero-cost data transfer while forcing the developer into high-performance, batched architectures by default.

**2. The Standard Library: The "Rust Extreme Core"**
*The Philosophy:* Ẹbọ (Sacrifice for Growth). To allow the ecosystem to grow rapidly, the core VM must sacrifice its desire to control everything. 
*The Implementation:* Adopt the Rust Model. The `ifa-std` crate should be stripped down to its absolute bare metal essence: `ifa-infra` (CPU, Storage, basic Networking). The `stacks/` (`ml`, `gamedev`, `crypto`) must be violently evicted from the VM registry and moved to the Ọjà Registry as official, versioned packages (e.g., `AAEO04/ifa-ml`). The core VM remains a tiny, indestructible diamond. The domains become fluid, fast-updating packages that the community can actually contribute to without needing to recompile the entire language.

**3. Native Security: The "WASM-Only Ọpọ́n"**
*The Philosophy:* Ọpọ́n (The Sacred Boundary). An Ọpọ́n is an absolute containment field. If you allow native `.so` or `.dll` files via a simple `--allow-ffi` flag (like Deno), you are admitting that your Ọpọ́n is just an illusion that can be bypassed by anyone who asks nicely. 
*The Implementation:* Adopt the Strict WebAssembly Model. Third-party Oja packages are forbidden from loading raw `.so`/`.dll` files. Period. If a developer wants to write a custom physics engine in C++ or Rust, they must compile it to `.wasm`. When the VM executes that WASM plugin, it runs inside a secondary, nested Ọpọ́n. If the C++ code has a use-after-free bug and segfaults, it only destroys its own isolated memory. The main Ifá-Lang VM catches the trap, gracefully logs an error, and continues running. The sacred boundary is never breached.

**The Verdict for Ifá-Lang:**
If you want to build a language that actually lives up to the Ifá philosophy, you don't compromise.
- **No hidden FFI costs:** Force batched memory transfers.
- **No monolithic bloat:** Keep the core VM tiny and push domains to Ọjà.
- **No native `.so` backdoors:** All third-party native code must be WebAssembly.

---

### Part 2: Linus Review — Memory-Mapped Structs (`Ọpọ́nView`)

While the concept of an `Ọpọ́nView` (where the VM natively understands C-struct layouts to write directly to contiguous memory) is the correct path for the "Batched Ọpọ́n" shared memory model, the implementation details must address three brutal hardware realities:

**1. Alignment and Padding**
Hardware hates unaligned memory. C compilers insert invisible padding bytes into structs to align fields on 4-byte or 8-byte boundaries. If the Ifá-Lang VM just sequentially packs bytes together and hands that pointer to a native library, the native code will read garbage or crash with a bus error on ARM. The VM must explicitly calculate and enforce C-ABI alignment rules, not just byte lengths.

**2. The Dangling Pointer Problem (Ownership)**
You cannot just "hand a raw pointer" across a boundary. If the Ifá-Lang script resizes the buffer, the Rust `Vec<u8>` reallocates, freeing the old memory. If the GPU or WASM module is still reading it, you get a segfault. To share memory across a boundary without copying, you MUST **Pin** the memory. The Ọpọ́n must guarantee the buffer cannot be moved, resized, or dropped as long as the native module holds a reference.

**3. The Access Tax**
This does not eliminate overhead; it just moves it. When an Ifá script reads `p.x` from an `Ọpọ́nView`, the VM must:
1. Do a bounds check.
2. Read raw bytes.
3. Box those bytes into an `IfaValue::Float` enum so the VM's standard opcodes can process it.
Struct views are fast for native code to read in batches, but *slow* for the VM to manipulate property-by-property. Developers should only use `Ọpọ́nView` buffers for data bound for the GPU/WASM, not for general-purpose script variables.

---

## Current Checkpoint — 2026-05-18 Full Audit

### What Is Done (Verified Against Codebase)

| PR | Tasks | Status |
|---|---|---|
| PR-01 | A1 stack guard, A2 fuel limit, A3 Ikin cap, A4 bincode limit | ✅ Done |
| PR-02 | B1 unsafe policy, B2 ebo.rs safety comments | ✅ Done |
| PR-04 | A5 FFI gate, A8 symlink canonicalize, B3 ffi.rs NonNull | ✅ Done |
| PR-19 | D1 ifa-parser, D2 ifa-compiler, D3 ifa-transpiler extracted | ✅ Done |
| PR-20 | D4 ifa-interpreter isolated, D5 ifa-core→ifa-vm rename, D6 workspace clean | ✅ Done |
| PR-27 | E5 string pool dedup (compiler), I1 globals Vec + O(1) name index | ✅ Done |
| PR-10 | B4 FFI fuzzing | ✅ Done |
| PR-11 | A7 embedded parity | ✅ Done |

### Active Blockers (Priority Order)

1. **PR-14 — H0 Stage 2 follow-up:** `IfaFn`, `Object`, and `UpvalueCell` are on the sendable path; the remaining `Sync`/reactive cleanup is a follow-up, not the current blocker.

### Not Started (Nearest Wins First)

- **PR-14:** H0 Stage 2 follow-up

### Next Immediate Actions (In Order)

1. PR-14 follow-up: Stage 2 `Sync` cleanup for reactive/shared UI
2. After PR-14 Stage 2: PR-13 (F1+F2+F3) → PR-40 (M1+M2 infra extraction)

### The Linus Rule (Still Active)

> Do not open PR-30 (K-series features) until the engine block work is complete and the remaining registry isolation work is merged. K-series stays closed until the runtime foundations are done.

### Full NaN Boxing Prerequisite Chain

```
PR-14 (H0 Stage 1 complete; Stage 2 parked)
    ↓
Expand NaN tag space (need ≥13 tags, currently 8)
    ↓
IfaValue(u64): Drop + Clone + safe accessors
    ↓
Miri validation (mandatory)
    ↓
Migrate ~300 match arms across all crates
    ↓
Manual Serialize/Deserialize
    ↓
32-bit/wasm #[cfg] fallback
    ↓
Full NaN boxing complete (I0 Phase B)
```

Estimated work after PR-14 Stage 2: 3–4 focused days.


### Previous Checkpoint (2026-05-03) Status

- Stream D crate split: **Stabilized.** Workspace compiles.
- `ifa-sandbox` security tests: **Green.**
- `ifa-std` JSON round-trip: **Fixed.**

### What Changed Since Last Checkpoint

- Full DSA audit completed (`dsa_audit.md`, `dsa_programmer_view.md`)
- Missing features inventory created (`missing_features_implementation.md`)
- Performance engineering analysis completed (`performance_engineering.md`)
- True parallelism impact analysis completed (`rust_route_ast_transpiler_impact.md`)
- Oja (package manager) and REPL confirmed as already implemented — documents corrected
- **4 new streams added to this plan:** I (Performance), J (Parallelism), K (Features), L (Build Speed)
- **8 new contradictions identified and resolved** (Appendix H, items 7–14)

### Active Blockers

1. `ifa-types` invariant cleanup (legacy bytecode string-pool round-trip)
2. `IfaValue` layout assertion/documentation mismatch

### Next Immediate Actions (In Order)

1. Clear `ifa-types` blocker → `cargo test -p ifa-types --lib`
2. Complete Stream A (A1 stack limits, A2 fuel wiring)
3. Complete E5 (Ikin dedup) — **prerequisite for I1**
4. **Ship PR-29** (J1 CPU wiring) — unlocks data parallelism
5. Ship PR-28 revised (I3 string len cache only)

### The Linus Rule

> Do not open PR-30 (K-series features) until the engine block work is complete and the VM dispatcher is structurally split. Do not add a single new language feature to a VM that hashes strings on every global access and pushes 56 bytes per integer.
