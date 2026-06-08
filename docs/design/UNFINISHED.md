# Unfinished Features — Verified Against Actual Code

> **⚠️ UPDATE (June 2026):** This document was originally written against an earlier version of the codebase. Several sections below describe features that are now **implemented**. Each section has been annotated with its current status. The following sections still describe genuinely unfinished work: §2.3 (Purity Enforcement), §3.2 (Dispatcher Split), §4.2 (Babalawo Decoupling), §5.1 (Domain Registration), §6.1 (Build Cache). See the codebase at `crates/` for the authoritative state.

Items from the unified implementation plan that were confirmed as genuinely **not done** (or only partially done) during the multi-agent audit of `crates/` source code. Each entry includes the current state, target behavior, and a concrete implementation plan.

---

## 1. Language Surface (Grammar / AST / Compiler)

---

### 1.1 `**` Exponentiation — ✅ IMPLEMENTED

**Current state:** `OpCode::Pow = 0x26` exists in the bytecode enum (`ifa-bytecode/src/lib.rs:125`). The VM `step()` in `ifa-vm/src/vm.rs:2540-2579` has a full handler: Int (`checked_pow`), Float (`powf`), Int/Float mixed, and Float/Int mixed.

**Status:** Fully implemented. Use `**` in source code.

---

### 1.2 `ayanfe` / `const` Declarations — ✅ IMPLEMENTED

**Current state:** `Statement::Const { name, value, visibility, span }` exists at `crates/ifa-types/src/ast.rs:106-112`. Grammar `const_stmt` and `const_kw` rules exist in `crates/ifa-parser/src/grammar.pest:84-85`. Parser handler exists at `crates/ifa-parser/src/parser.rs:90`. Lexer token at `crates/ifa-parser/src/lexer.rs:139`.

**Status:** Fully implemented. Use `const NAME = value;` or `ayanfe NAME = value;` in source code.

---

### 1.3 Alias / Rename Syntax — ✅ IMPLEMENTED

**Current state:** `Statement::Alias { name, target, visibility, span }` exists at `crates/ifa-types/src/ast.rs:114-115`. Grammar rule `alias_stmt = { alias_kw ~ ident ~ "=" ~ expression ~ ";" }` exists in `crates/ifa-parser/src/grammar.pest:155`. Parser handler at `crates/ifa-parser/src/parser.rs:125`.

**Status:** Fully implemented. Use `alias NewName = Target;` in source code.

---

### 1.4 Set Type `Set<T>` — ⏳ PARTIALLY IMPLEMENTED

**Current state:** `IfaValue::Set(Arc<HashSet<IfaValue>>)` exists at `crates/ifa-types/src/value_union.rs:48`. All four opcodes (`BuildSet=0x7A`, `SetAdd=0x7B`, `SetHas=0x7C`, `SetRemove=0x7D`) exist at `crates/ifa-bytecode/src/lib.rs:221-227`. All four VM handlers exist in `crates/ifa-vm/src/vm.rs:1850-1903`. Set equality comparison implemented at `value_union.rs:386-394`. Set literal syntax `Set { 1, 2, 3 }` exists in grammar at `crates/ifa-parser/src/grammar.pest:272-274` and is parsed into `Expression::Set(Vec<Expression>)` — compiles to `OpCode::BuildSet`.

**Still missing:** Set backed by `Arc<HashSet>` rather than `IfaGc<HashSet>` (not GC-traced). Set cannot be frozen for actor boundary transfer.

---

### 1.5 Default Parameter Values — ✅ IMPLEMENTED

**Current state:** `Param` struct at `crates/ifa-types/src/ast.rs:313-317` has `pub default_value: Option<Expression>`. Grammar rule `param = { ident ~ (":" ~ type_name)? ~ ("=" ~ expression)? }` at `crates/ifa-parser/src/grammar.pest:118`.

**Status:** Fully implemented. Use `fn name(param: Type = default_value)` syntax.

---

### 1.6 Ìpa Side-Effect Tags (`pelu Ipa`) — ✅ IMPLEMENTED

**Current state:** Effect system implemented via `Effect` enum (`crates/ifa-types/src/ast.rs:10-23`: Pure, Async, Network, FileIO, State, Impure). `EseDef` stores `effects: Vec<Effect>` at `ast.rs:132-140`. Grammar `effects_decl` rule supports `pelu Ipa` syntax at `crates/ifa-parser/src/grammar.pest:113-116`. Parser handles it at `parser.rs:530-538`. Babalawo `EffectChecker` exists in `crates/ifa-babalawo/src/effects.rs`.

**Note:** Uses `Vec<Effect>` enum rather than a single `has_effect: bool` flag, allowing granular effect tracking.

---

### 1.7 AssertType Opcode — ✅ IMPLEMENTED

**Current state:** `OpCode::AssertType = 0xA6` exists at `crates/ifa-bytecode/src/lib.rs:269`. Full VM handler at `crates/ifa-vm/src/vm.rs:2757-2781` with type ID mapping (0=Int, 1=Float, 2=Str, 3=Bool, 4=List, 5=Map, 6=Fn/Closure, 255=Any).

**Status:** Fully implemented. Use `assert_type(value, Type)` in source code.

---

## 2. Static Analysis (Babalawo)

---

### 2.1 Match Exhaustiveness Checking — ⏳ PARTIALLY IMPLEMENTED

**Current state:** Babalawo's `check_expression` at `crates/ifa-babalawo/src/checks.rs:842-850` warns for missing wildcard: `"Match block may not be exhaustive. Consider adding a '_' wildcard arm."`.

**Still missing:** Full exhaustive pattern analysis (verifying all integer/string variants are covered).

---

### 2.2 Resource Leak Detection — ✅ IMPLEMENTED

**Current state:** `check_unclosed_resources()` at `crates/ifa-babalawo/src/checks.rs:1602-1607` emits `"Resource '...' opened but never closed"`. `IwaEngine` at `crates/ifa-babalawo/src/iwa.rs:255` has `close_resource()` method and `ResourceDebt` tracking at `iwa.rs:267-287` with `unclosed_resources()` accessor. Called from main analysis at `checks.rs:251`.

**Status:** Fully implemented. Babalawo detects and warns about unclosed resources.

---

### 2.3 Purity Enforcement

**Current state:** No purity tracking. The `fun` keyword exists but Babalawo never verifies that a (non-`pelu Ipa`) function body is actually pure. Plan item "G8" is absent.

**Target:** Babalawo flags impure operations inside functions that don't have `pelu Ipa`.

**Implementation:**
1. In `LintContext`, add `pure_scope: bool` — set to `true` when entering a function without `pelu Ipa`
2. Define impure operations: `print`, `Ofun` calls (`dbg`, `read`), `Oyeku` (random), `Otura` (network), `Ogbe` (file I/O), `Ebo` statements, `defer` (if body is impure)
3. When `pure_scope && impure_operation` is detected, emit `LintError`: `"impure operation in pure function"`
4. Domain methods are pure by default; mark specific methods as impure in `ODU_METHODS`

**Files touched:** `crates/ifa-babalawo/src/checks.rs`, `crates/ifa-types/src/odu_metadata.rs`

---

### 2.4 `abo` / `strict` Mode — ✅ IMPLEMENTED

**Current state:** `Statement::Abo` variant exists at `crates/ifa-types/src/ast.rs:174`. Grammar rules `abo_stmt` and `abo_kw` at `crates/ifa-parser/src/grammar.pest:162-163`. Parser handler at `crates/ifa-parser/src/parser.rs:359`.

**Status:** Fully implemented. Use `abo;` at the top of a file.

---

## 3. VM Internals

---

### 3.1 Unified Call Dispatch — ✅ IMPLEMENTED

**Current state:** A unified `fn call_value(...)` dispatch method exists at `crates/ifa-vm/src/vm.rs:993`. All 5 call sites (`vm.rs:684`, `1128`, `2799`, `2809`, `2931`) use `self.call_value(...)` instead of duplicated dispatch logic.

**Status:** Fully implemented.

---

### 3.2 Opcode Dispatcher Split

**Current state:** `step()` is ~1118 lines (lines 1214–2331) with most opcode handlers inline. Only 5 dispatch helpers have been extracted: `dispatch_call`, `dispatch_tail_call`, `dispatch_return`, `dispatch_parallel_for`, `dispatch_call_method`. Plan item "I2" is partially done.

**Target:** Each opcode (or family) has a `#[inline]` helper method. `step()` is a thin match dispatcher.

**Implementation:**
1. For each of the ~80 opcodes, extract inline match-arm code into methods:
   ```rust
   fn dispatch_push_int(&mut self, value: i64) { self.stack.push(IfaValue::Int(value)); }
   fn dispatch_push_str(&mut self, id: usize) -> IfaResult<()> { /* ... */ }
   fn dispatch_add(&mut self) -> IfaResult<()> { /* pop 2, push sum */ }
   fn dispatch_sub(&mut self) -> IfaResult<()> { /* pop 2, push diff */ }
   // ... 50-60 more
   ```
2. Group related opcodes into shared handlers:
   ```rust
   fn dispatch_binary_int_op(&mut self, op: IntOp) -> IfaResult<()> {
       let b = self.pop_int()?;
       let a = self.pop_int()?;
       self.stack.push(IfaValue::Int(match op {
           IntOp::Add => a + b,
           IntOp::Sub => a - b,
           IntOp::Mul => a * b,
           IntOp::Div => a / b,
           // ...
       }));
       Ok(())
   }
   ```
3. Result: `step()` becomes ~20 lines of `match` → `self.dispatch_*(...)`
4. Test: byte-for-byte identical execution for all existing test suites

**Files touched:** `crates/ifa-vm/src/vm.rs` (major refactor, single file)

---

## 4. Architecture Refactoring

---

### 4.1 Feature-gate Parser Dependencies

**Current state:** [DONE] Parser dependencies (`logos`, `pest`) are now fully optional, gated behind a `compiler` feature in `ifa-parser/Cargo.toml`.

**Target:** Parser dependencies are optional, gated behind a `compiler` feature. Programs using only `ifa-types` or the bytecode crate don't link pest/logos.

**Implementation:**
[COMPLETED]
1. In `crates/ifa-parser/Cargo.toml`:
   ```toml
   [features]
   default = ["compiler"]
   compiler = ["dep:logos", "dep:pest", "dep:pest_derive"]
   ```
   pest_consume = { version = "...", optional = true }
   ```
2. Gate parser implementation body behind `#[cfg(feature = "compiler")]`
3. When `compiler` is disabled, export stub types only (AST, token types) — the `parse()` function returns `Err("parser not available")`
4. Update `crates/ifa-vm/Cargo.toml` to add a `compiler` feature:
   ```toml
   [features]
   compiler = ["ifa-parser/compiler"]
   ```
5. Update downstream dependents that need parsing to enable the feature

**Files touched:** `crates/ifa-parser/Cargo.toml`, `crates/ifa-parser/src/lib.rs`, `crates/ifa-vm/Cargo.toml`, other crates' Cargo.toml as needed

---

### 4.2 Decouple Babalawo from ifa-std

**Current state:** `ifa-babalawo/Cargo.toml` lists `ifa-std` as a dependency. This pulls all domain implementations (network, crypto, hardware, etc.) into babalawo's dependency tree, even though babalawo only needs metadata about domains (names, methods, type signatures). Plan item "C4" is absent.

**Target:** Babalawo depends on `ifa-types` only. Domain metadata lives in `ifa-types/src/odu_metadata.rs`.

**Implementation:**
1. Audit all `use ifa_std::*` imports in `crates/ifa-babalawo/src/`. Common patterns:
   - Domain name lookups → move to `ifa-types/src/odu_metadata.rs`
   - Method type signatures → already in `ODU_METHODS`
   - Capability checks → use `CapabilitySet` from `ifa-types`
2. Move any missing metadata from `ifa-std/src/vm_registry.rs` into `ifa-types/src/odu_metadata.rs`
3. Replace `ifa-std` dep with `ifa-types` in `ifa-babalawo/Cargo.toml`
4. Update import paths throughout babalawo
5. Test: `cargo check -p ifa-babalawo --no-default-features` succeeds without ifa-std
6. Test: `cargo test -p ifa-babalawo` still passes

**Files touched:** `crates/ifa-babalawo/Cargo.toml`, various `crates/ifa-babalawo/src/*.rs`, `crates/ifa-types/src/odu_metadata.rs`, `crates/ifa-std/src/vm_registry.rs`

---

### 4.3 ModuleLoader Struct — ❌ NOT IMPLEMENTED

**Current state:** No `loader.rs` exists in any crate under `crates/`. No `ModuleLoader` struct found anywhere in the codebase. Module loading is handled inline in the CLI and VM without caching or `ImportGuard` cycle detection.

**Status:** Still absent. The previous claim of completion was incorrect.

---

### 4.4 ExecutionContext Extraction — ✅ IMPLEMENTED

**Current state:** `pub struct ExecutionContext` exists at `crates/ifa-vm/src/vm.rs:209` with fields: `stack`, `frames`, `ip`, `halted`, `recovery_stack`, `loop_stack`.

**Status:** Fully implemented. Execution state is self-contained in the `ExecutionContext` struct.

---



---

### 5.1 Remaining Domain Registration

**Current state:** [DONE] Missing domains (Irosu audio methods: `siro_duro`, `kigbe_orin`) are now wired up in `vm_registry.rs`. Otura TCP methods appropriately return errors specifying they are unsupported in the VM registry.

**Target:** Complete dispatch tables for all 16 canonical domains and their features.

**Implementation:**
[COMPLETED]

**Target:** Every domain from 0–29 has at least a stub dispatch returning "domain not implemented", with real methods for key domains.

**Domains to wire:**

| ID | Name | Purpose | Priority |
|----|------|---------|----------|

| 15 | Ofun Nje | Security / permissions supplement | Medium |
| 16 | Coop | Cooperative concurrency, actors | Low |
| 17 | Opele | Extended random / divination chain | Low |
| 19 | — | GPU compute shaders | Low |
| 29 | — | Kernel (process, syscall) | Low |

**Also missing:**
- Domain 3 (Irosu): audio methods `siro` (play), `gbigbọn` (vibrate), `ariwo` (volume)
- Domain 12 (Otura): TCP methods `connect`, `listen`, `send`, `recv` (currently HTTP only)

**Implementation:**
1. For each new domain, create a stub module in `crates/ifa-std/src/domains/` (e.g., `eta_ogunda.rs`):
   ```rust
   pub fn dispatch(method: &str, args: Vec<IfaValue>, ctx: &mut VmContext) -> IfaResult<IfaValue> {
       match method {
           "ping" => Ok(IfaValue::Str("eta-ogunda: pong".into())),
           _ => Err(VmError::NotImplemented(format!("Eta-Ogunda: {} not implemented", method))),
       }
   }
   ```
2. Register in `crates/ifa-std/src/vm_registry.rs`:
   - Add `mod eta_ogunda;` etc.
   - Add dispatch table entries at lines 107–151
   - Add name mappings at lines 320–340
3. For Irosu audio: add methods to domain 3 dispatch (line ~500 area):
   ```rust
   "siro" | "play" => { /* delegate to audio backend */ }
   "gbigbọn" | "vibrate" => { /* delegate to haptic backend */ }
   ```
4. For Otura TCP: extend domain 12 dispatch with TCP methods alongside existing HTTP
5. Test: each domain responds to at least `ping`; audio methods exist in dispatch

**Files touched:** `crates/ifa-std/src/vm_registry.rs`, 10+ new files in `crates/ifa-std/src/domains/`, `crates/ifa-std/src/lib.rs`

---

## 6. Tooling

---

### 6.1 Build Cache

**Current state:** `ifa build` creates a temporary directory `ifa_build_{pid}` and recompiles from scratch every time. No persistent cache. Plan item "L1" is absent.

**Target:** `ifa build` caches compiled bytecode in `.oja/build_cache/`, keyed by source file hash. Subsequent builds skip unchanged files.

**Implementation:**
1. Add `build_cache` module in `crates/ifa-cli/src/`:
   ```rust
   pub struct BuildCache {
       cache_dir: PathBuf,
   }
   
   impl BuildCache {
       pub fn new(project_root: &Path) -> Self;
       pub fn lookup(&self, source_path: &Path) -> Option<Vec<u8>>;
       pub fn store(&self, source_path: &Path, bytecode: &[u8]) -> IfaResult<()>;
       pub fn clean(&self, older_than: Duration) -> IfaResult<()>;
   }
   ```
2. Cache key: SHA-256 of source file contents
3. Cache entry: `<hash>.ifab` in `.oja/build_cache/`
4. In `ifa build` command: before compilation, check cache; after compilation, store in cache
5. `ifa build --clean` deletes entries older than 30 days
6. Cache invalidation: if source file modification time is newer than cache entry, recompile

**Files touched:** new file `crates/ifa-cli/src/build_cache.rs`, `crates/ifa-cli/src/main.rs`

---

### 6.2 Parallel Babalawo + Compiler — ✅ IMPLEMENTED

**Current state:** `ifa run` at `crates/ifa-cli/src/main.rs:490-497` runs Babalawo static analysis and bytecode compilation in **parallel threads**. The AST types implement `Send + Sync` for safe concurrent access.

**Status:** Fully implemented. Babalawo and compiler run in parallel in the CLI.
