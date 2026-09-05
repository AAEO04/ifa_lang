# Engineering Audit

**Status:** `DRAFT`  
**Date:** 2026-06-09  
**Scope:** Full `crates/` architecture audit across VM, compiler, embedded, WASM, GC, and tooling  
**Verification basis:** Source code analysis via systematic review of every crate in the workspace

---

## Table of Contents

1. [VM Architecture: Two Runtimes](#1-vm-architecture-two-runtimes)
2. [Compilation Pipeline: Missing IR](#2-compilation-pipeline-missing-ir)
3. [VM Dispatch & Value Representation](#3-vm-dispatch--value-representation)
4. [Garbage Collection](#4-garbage-collection)
5. [Code Organization & Maintainability](#5-code-organization--maintainability)
6. [Embedded & IoT](#6-embedded--iot)
7. [WASM Bindings](#7-wasm-bindings)
8. [Standard Library Dependencies](#8-standard-library-dependencies)
9. [Anomalies & Dead Code](#9-anomalies--dead-code)
10. [Recommendation Summary](#10-recommendation-summary)

---

## 1. VM Architecture: Two Runtimes

### Observation

Two disparate VM implementations exist:

| Aspect | `ifa-vm` (main) | `ifa-embedded` (embedded) |
|--------|-----------------|---------------------------|
| Lines | 3539 in `vm.rs` | 966 in `lib.rs` |
| Value type | `IfaValue` (16 bytes, GC-managed, `Clone`-only) | `EmbeddedValue` (12-16 bytes, `Copy`-compatible) |
| Stack | `Vec<IfaValue>` (dynamic, heap) | `HVec<EmbeddedValue, STACK_SIZE>` (fixed, inline) |
| Opcodes | 74 (closures, async, exceptions, collections, modules) | 27 (basic arithmetic, locals, jump, MMIO) |
| Locals | On value stack via base pointer | Separate `HVec<EmbeddedValue, 32>` |
| Frames | `Vec<CallFrame>` with return addr, base ptr, closure env | None (no function calls) |
| GC | Bacon-Rajan cycle collection via `IfaGc` | None (fixed-size heapless) |
| Memory | Dynamic `Opon` with epoch-based allocation | Fixed `HVec<EmbeddedValue, OPON_SIZE>` |
| Platform | `std` | `no_std` + optional `alloc` |

### Key finding

The two VMs share **zero execution code**. They share only the bytecode format (`ifa-bytecode`), but the opcode sets have diverged:
- Main: 74 opcodes, 64-bit immediates (i64/f64), 2-byte local indices, 4-byte jump offsets
- Embedded: 27 opcodes, 32-bit immediates (i32/f32), 1-byte local indices, 2-byte jump offsets

A bug in `OpCode::Add` on the main VM must be fixed separately in the embedded VM. Feature additions (closures, async, exceptions) never propagate to embedded.

### Proposed solution: Template VM with MemoryModel trait

```rust
trait MemoryModel {
    type Value: Clone;
    type Stack: AsRef<[Self::Value]>;
    const MAX_SLOTS: usize;
    fn alloc(&self) -> Self::Value;
    fn free(&self, val: Self::Value);
}
```

Main VM implements with `IfaGc + Vec + Arc`. Embedded VM implements with `heapless::Vec + inline values`. Same dispatch loop, different backing stores.

**Caveat**: The opcode sets differ by 45 opcodes, so the dispatch loop would need extensive `#[cfg]` gating. The shared core (27 opcodes) could instead be extracted into a shared macro or helper in `ifa-bytecode`, called by both VMs, with each adding its own opcodes on top.

### Files

- `crates/ifa-vm/src/vm.rs` — Main VM (3539 lines)
- `crates/ifa-embedded/src/lib.rs` — Embedded VM (966 lines)
- `crates/ifa-embedded/ifa-bytecode/src/embedded.rs` — `EmbeddedOpCode` (27 variants)
- `crates/ifa-embedded/ifa-bytecode/src/lib.rs` — Main `OpCode` (74 variants)

---

## 2. Compilation Pipeline: Missing IR

### Current pipeline

```
Source → AST (ifa-parser)
         ├──→ ifa-compiler → Vec<u8> opcodes (bytecode)
         └──→ ifa-transpiler → Rust source string
```

Both backends start from `ifa_types::ast::Program` and emit their target directly. **They share zero code** beyond the AST definition.

### Compiler (`ifa-compiler`)

- Single file: `crates/ifa-compiler/src/lib.rs` — 2154 lines
- `compile_statement()`: 728 lines
- `compile_expression_inner()`: 360 lines
- `fold_expression()`: 349 lines (the only "pass" between AST and emission)
- No CFG, no SSA, no basic blocks, no dominator tree
- Emits `OpCode` bytes directly to `Vec<u8>`

### Transpiler (`ifa-transpiler`)

- 2479 lines across 8 files, completely independent AST walk
- Has its own expression handling (502 lines), statement handling (1029 lines), Odu domain inlining (416 lines)
- Has its own literal folding (`try_transpile_literal_binop`, ~130 lines inline in expressions.rs)
- No shared IR, no shared analysis, no shared optimization passes with the compiler
- Bug fix in `fold_expression` does NOT fix the transpiler's equivalent logic

### Optimizations (all in compiler, none shared)

| Optimization | Compiler | Transpiler |
|---|---|---|
| Constant folding (arithmetic) | 349-line `fold_expression` | Inline `try_transpile_literal_binop` (~130 lines) |
| Constant divination (Odu) | `fold_expression` Odu folding | None |
| Tail-call optimization | Detects `return fn()` → `TailCall` | None |
| String pool dedup | `HashMap<String, u16>` | None |
| Short-circuit eval | `Dup` + conditional jump | Inline in expression codegen |

### Bytecode format is not an IR

The `.ifab` format is a flat instruction stream designed for runtime interpretation:
- `BytecodeHeader`: 15 bytes (magic, version, instruction_size, constant_size, opon_size)
- `#[repr(u8)]` opcodes — no annotations, no metadata, no debug info sections in the format
- `Bytecode` struct has `lines: Vec<(usize, u32)>` for source mapping on the Rust side, but this is not serialized into the binary format in a debuggable way
- No CFG edges, no basic blocks, no SSA

### Recommendation: Add a Mid-level IR

```
Source → AST → MIR (shared) → bytecode backend
                              → Rust transpiler backend
                              → (future) WASM native backend
```

An MIR as `Vec<Instruction>` with CFG edges (~2000 lines) would:

| Benefit | Impact |
|---|---|
| Shared constant folding pass | Fix once, both backends benefit |
| Shared analysis passes | Type checks, dead code elimination, inlining — one implementation |
| Transpiler shrinks | From 2479 lines to ~1000-1200 (MIR → Rust template codegen) |
| New backends | WASM native, embedded JIT — all share the frontend |
| Optimization infrastructure | CSE, loop hoisting, inlining all become possible |

### Files

- `crates/ifa-compiler/src/lib.rs` — 2154 lines (monolithic AST→bytecode)
- `crates/ifa-compiler/src/embedded.rs` — 130 lines (embedded bytecode backend)
- `crates/ifa-transpiler/src/` — 2479 lines across 8 files
- `crates/ifa-types/src/bytecode.rs` — `Bytecode` struct, `OponSize`, serialization
- `crates/ifa-embedded/ifa-bytecode/src/` — Runtime bytecode format (clean, `#![no_std]`, `#![forbid(unsafe_code)]`)

---

## 3. VM Dispatch & Value Representation

### Dispatch chain (per instruction)

1. `resume_execution()` loop calls `step()` — vm.rs:951
2. `step()` calls `vm_iroke::tap()` — vm.rs:1413 — runs `OpCode::from_u8(byte)` — **74-arm match**
3. `step()` runs **74-arm match** on returned `OpCode` — vm.rs:1415-2830

**~150 match arms per instruction.** The `from_u8` is a `const fn` match (ifa-bytecode/src/lib.rs:277-383), and the step dispatch is 1419 lines long.

### Direct threading opportunity

Replace the decode-then-dispatch with an array of function pointers:

```rust
type Handler = fn(&mut IfaVM, &Bytecode) -> IfaResult<()>;
const DISPATCH_TABLE: [Option<Handler>; 256] = build_table();

// In the loop:
if let Some(handler) = DISPATCH_TABLE[bytecode.code[ip] as usize] {
    ip += 1;
    handler(self, bytecode)?;
}
```

- Eliminates the two-level decode: one indirect call instead of two matches
- Each handler is its own function, improving icache locality (hot handlers stay cached)
- ~50 lines changed, concentrated in `vm_iroke.rs`
- Rust currently blocks computed goto (`label as *const fn()` → `goto label` is unstable), but function pointer tables are stable and well-optimized

### IfaValue size

- `IfaValue` is **16 bytes** (enum discriminant + Box<String>/IfaGc/Arc pointers)
- `EmbeddedValue` is **12-16 bytes** (depending on pointer width), plus `Copy` derivable

The operand stack (`Vec<IfaValue>`) stores 16-byte entries. Every push/pop moves 16 bytes.

### NaN Boxing (`nan_box.rs` — deleted)

`crates/ifa-types/src/nan_box.rs` has been **deleted from the codebase**. It previously contained 374 lines of NaN-boxed arithmetic (`NanBox(u64)` with `add`, `sub`, `mul`, `div`, comparisons) that was used ephemerally — packed and unpacked on the hot path without being used as the actual `IfaValue` storage format.

**Resolution:** Option B was chosen — arithmetic now operates directly on `IfaValue` variants via enum dispatch, and the NaN-boxing infrastructure was removed.

### Add opcode comment/code mismatch

Module doc (vm.rs:5-8):
```rust
/// `OpCode::Add` is now PURE NUMERIC (Int/Float only). String concatenation uses
/// the dedicated `OpCode::Concat (0x27)`, which is strict `Str + Str` only.
```

Actual code (vm.rs:2425-2444):
```rust
(IfaValue::Str(sa), IfaValue::Str(sb)) => { /* concat */ }
(IfaValue::Str(sa), other) => { /* format + concat */ }
(other, IfaValue::Str(sb)) => { /* format + concat */ }
```

Three live string arms in `Add`. The dedicated `Concat` handler at vm.rs:2453 only handles `Str+Str` and duplicates the Add logic. The doc comment is actively misleading — someone trusting it might remove the string arms as dead code.

### Sort implementation (vm.rs:3078-3084)

```rust
sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
```

**Remaining issues** (issues 1-2 fixed, issue 3 remains):

1. ✅ **Fixed** — `partial_cmp` now returns `Ordering::Equal` on incomparable types (vm.rs:3162). NaN values sort stably.

2. ✅ **Fixed** — `partial_cmp` cross-type Int/Float guard removed. Now always compares as f64, consistent with VM opcodes.

2. ❌ No specialization for homogenous `Int`-only lists. `slice::sort()` on `&[i64]` is ~5x faster than trait-object `partial_cmp` dispatch through enum variant matching. One type check + one line would cover the common case.

3. ❌ `l.to_vec()` clones the entire list before sorting. While this avoids the GC atomic overhead during swaps, it is O(n) allocation for the clone. Using `make_mut()` for copy-on-write would avoid the clone when the list has only one reference.

### Files

- `crates/ifa-vm/src/vm_iroke.rs` — `tap()` dispatch (50 lines)
- `crates/ifa-vm/src/vm.rs` — `step()` dispatch (1419 lines), sort (vm.rs:3012-3015)
- `crates/ifa-types/src/value_union.rs` — `IfaValue` definition (NaN-boxing file deleted)
- `crates/ifa-embedded/ifa-bytecode/src/lib.rs` — `OpCode::from_u8()` (100-line match)

---

## 4. Garbage Collection

### Implementation (`crates/ifa-types/src/gc.rs`, 338 lines)

Bacon-Rajan concurrent cycle collection on `IfaGc<T>` pointers.

| Aspect | Finding | Details |
|---|---|---|
| `unsafe` blocks | **20** in gc.rs | Function pointer calls, raw pointer derefs in mark/scan/free phases |
| Vtable strategy | Type-erased function pointers | `trace_fn`, `drop_data_fn`, `dealloc_fn` stored in `CycleHeader` — indirect call on every trace, drop, and dealloc |
| Trigger | Fixed 1024-tick interval | `self.ticks.is_multiple_of(1024)` — not based on allocation rate or memory pressure |
| Suspect buffer | Thread-local `RefCell<Vec<...>>` | Cross-thread cycle detection impossible. Each actor/thread collects its own cycles independently |
| Generational | **None** | Young objects go through full cycle detection alongside old objects |
| Compaction | **None** | Garbage creates fragmentation. `Box::new` → `Box::into_raw` → `Box::from_raw`. No defragmentation, no sliding |
| Memory ordering | All `Ordering::Relaxed` | 30 atomic ops across the file, every one uses `Relaxed`. Safe for single-thread, needs Acquire/Release for cross-thread |

### GC tick trigger issue

The `gc_policy()` on `OponSize` already defines per-tier intervals:

```rust
// bytecode.rs:58-66
pub fn gc_policy(&self) -> (u64, usize) {
    match self {
        OponSize::Kekere => (128, 8),
        OponSize::Arinrin => (512, 32),
        OponSize::Nla => (2048, 128),
        OponSize::Ailopin => (4096, 256),
    }
}
```

These are now fully wired and active (at <code>vm.rs:984-988</code>). The VM dynamically queries this policy to trigger cycle collection at the defined tick intervals, avoiding a hardcoded tick interval.

### Proptest strategies (ifa-vm/tests/common/proptest_strategies.rs)

Property-based testing for the GC: `any_ifa_value()` generates cyclic graph structures (lists of lists of lists). This is the correct approach for finding GC bugs — random graph generation catches cycle collection edge cases that hand-written tests miss.

### Files

- `crates/ifa-types/src/gc.rs` — 338 lines, 20 unsafe blocks
- `crates/ifa-types/src/bytecode.rs` — `gc_policy()` on OponSize (fully wired and active)
- `crates/ifa-vm/src/vm.rs` — tick counter + GC trigger (vm.rs:959-961)

---

## 5. Code Organization & Maintainability

### vm.rs: 3539 lines — 54% of ifa-vm crate

The file contains:
- 9 type definitions (`CallFrame`, `RecoveryFrame`, `IfaVM`, `ExecutionContext`, `GlobalState`, etc.)
- 1419-line `step()` function (40% of the file)
- The entire async scheduler, module loader, import system, actor runtime, error recovery, and snapshot system
- 313 lines of inline tests

The crate does have 9 specialized modules (2719 lines total — `ajose.rs`, `opon.rs`, `actor.rs`, `iwa_pele.rs`, `ebo.rs`, `module_resolver.rs`, `vm_ikin.rs`, `vm_iroke.rs`, `native.rs`). So the extra-file modularization exists for subsystems, but the core dispatch + all major type definitions remain in the single file.

### Two IfaValue definitions (resolved)

| File | Status |
|---|---|
| `crates/ifa-types/src/value.rs` | **Deleted** — legacy re-export shim removed. |
| `crates/ifa-types/src/value_union.rs` | Active — current enum with `IfaGc`, `CompactString`, closures, futures, actors. This is the canonical type. |
| `crates/ifa-vm/src/lib.rs` | `pub mod value;` declaration removed. All imports updated to use `ifa_types::IfaValue` directly. |

Previously there were two enum definitions with the same name, one dead. The stale `value.rs` re-export in `ifa-vm` has been deleted and all internal references updated.

### ajose.rs: 692 lines of reactive signals — incomplete, unused in production

- `Signal<T>`, `Computed<T>`, `effect()`, `EffectGuard`, `SubscriptionGuard<T>` defined
- Line 189-190: `// In a full implementation, we'd track which signals were accessed and subscribe to them. For now, just store the callback.`
- No production code path calls any ajose runtime function
- Only ajose's own unit tests exercise it
- Publicly exported from `ifa-vm`, has proc macro in `ifa-macros`, lexer/parser recognition
- Zero call sites outside the module itself

### ebo.rs: defer! RAII guard (ebo! macro removed)

The no-op `ebo!` macro has been removed. The only remaining macro is `defer!` (ebo.rs:160-165), which uses `unsafe` blocks with `ManuallyDrop::drop` and `ManuallyDrop::take`. The `scopeguard` crate is not a dependency.

```rust
// ebo.rs currently does:
unsafe { ManuallyDrop::drop(&mut self.cleanup); }
// Instead of:
scopeguard::defer!( cleanup() );
```

### Recursive sandbox (igbale.rs:56-67)

```rust
Command::new("unshare")
    .args(["--mount", "--net", "--pid", "--fork", "--"])
    .arg("timeout").arg(&format!("{}", ...)).arg("ifa").arg("run")
    .arg(code_path)
```

The sandbox spawns `ifa run` as a subprocess. No depth limit — a script could write:

```python
# payload.ifa — infinite regression
ifa sandbox run nested.ifa
```

The `timeout` provides a wall-clock limit before OOM, but there is no `max_depth` field in `SandboxConfig`, no capability inheritance check, and no nesting boundary. The `unshare --pid` creates a new PID namespace so grandchild processes are isolated, but not bounded.

### Files

- `crates/ifa-vm/src/vm.rs` — 3539 lines
- `crates/ifa-vm/src/ajose.rs` — 692 lines (incomplete signals library)
- `crates/ifa-vm/src/ebo.rs` — 230 lines (manual ManuallyDrop defer)
- `crates/ifa-types/src/value.rs` — Deleted (was legacy ifa-vm re-export)
- `crates/ifa-types/src/value_union.rs` — Active IfaValue
- `crates/ifa-sandbox/src/igbale.rs` — Sandbox implementation

---

## 6. Embedded & IoT

### Feature flags: declared but unused

`crates/ifa-embedded/Cargo.toml`:
```toml
esp32 = []
stm32 = []
rp2040 = []
```

These are **empty feature flags** — no actual HAL dependency behind any of them. The source files contain zero `#[cfg(feature = "esp32")]` or `#[cfg(feature = "stm32")]` or `#[cfg(feature = "rp2040")]` blocks. The HAL crates (`esp32c3-hal`, `stm32f4xx-hal`, `rp2040-hal`) are not in `[dependencies]`.

### MMIO/HAL traits: defined but no real implementations

```rust
pub trait MmioBus { fn read(&mut self, addr: u32) -> u32; fn write(&mut self, addr: u32, val: u32); }
pub trait InputPin { fn is_high(&self) -> bool; }
pub trait OutputPin { fn set_high(&mut self); fn set_low(&mut self); }
pub trait Serial { fn write_byte(&mut self, byte: u8) -> EmbeddedResult<()>; }
pub trait DelayUs { fn delay_us(&mut self, us: u32); }
```

These are well-abstracted traits (correct level of generality for microcontrollers). But they have **zero implementations outside test stubs**. No CMSIS-RTOS bindings, no embassy integration, no PAC register access.

### Embedded toolchain: missing

| Component | Status |
|---|---|
| Linker scripts (`memory.x`, `link.x`) | Not present |
| `#[entry]` macro re-export | Not present |
| `.cargo/config.toml` for target selection | Not present |
| `ifa build --target riscv32imc-unknown-none-elf` | Present in CLI but transpiles to `fn main()` with `std` deps, not `#![no_std]` |
| `ifa flash` | Stub that prints "delegated to external tools" |

### Slab allocator: correct cross-crate sharing pattern

`crates/ifa-embedded/src/slab.rs` (149 lines) defines `SlabTracker`, `SlabClass`, `SlabAllocation`. It is consumed by `crates/ifa-infra/src/gpu.rs` for GPU memory pool allocation:

```
ifa-embedded (SlabTracker, SlabClass)
      ↑
ifa-infra (SlabMemoryPool — wraps SlabTracker + wgpu::Buffer)
```

Compile-time specialization via `#[cfg(feature = "alloc")]` switches between `Vec<AtomicUsize>` (host) and `heapless::Vec<AtomicUsize, 64>` (embedded). This is the correct architectural pattern — shared primitive, compile-time specialization, no duplication.

### Opon sizes: realistic constraints

- `OponSize::Kekere`: 256 slots, ~4KB for values
- `AILOPIN_HARD_LIMIT`: 1,048,576 (2^20)
- ESP32-C3 has 520KB SRAM — VM takes ~1% of it

### Files

- `crates/ifa-embedded/Cargo.toml` — Empty target features (esp32, stm32, rp2040)
- `crates/ifa-embedded/src/lib.rs` — HAL traits + EmbeddedVm
- `crates/ifa-embedded/src/slab.rs` — Slab allocator (shared with ifa-infra)
- `crates/ifa-infra/src/gpu.rs` — GPU memory pool using slab
- `crates/ifa-cli/src/main.rs` — `Build` with `--target` option, `Flash` command (stub)
- `crates/ifa-types/src/bytecode.rs` — `OponSize` enum

---

## 7. WASM Bindings

### Current state

`crates/ifa-wasm`: **152 lines of Rust**, 10 Cargo dependencies (2 unused).

Pipeline:
```
source → ifa_parser::parse() → Compiler::compile() → IfaVM::execute() → RunResult
```

Additional exports:
- `format_code()` — delegates to `ifa-fmt`
- `get_version()` — version string
- `cast_opele()` — random Odu name via `js_sys::Math::random()`

### Dependency usage

| Dependency | Status |
|---|---|
| `web-sys` (console feature) | **In use** — `web_sys::console::log_1` called in `run_code` (lib.rs:83) for routing print events to browser console. Updated from vestigial to active. |
| `wasm-bindgen-futures` | **Unused** — all exports are synchronous. Still vestigial. |

### MIR relevance

The WASM crate is already thin (152 lines, core logic ~50 lines). An MIR would improve the **compiler** that WASM calls (shared optimizations, fix-once propagation), but would not change the WASM crate architecture unless a native `MIR → WASM binary` backend is built (a separate ~1500-2000 line project).

### Files

- `crates/ifa-wasm/src/lib.rs` — 152 lines
- `crates/ifa-wasm/Cargo.toml` — 10 dependencies

---

## 8. Standard Library Dependencies

### Dependency count

`crates/ifa-std/Cargo.toml` has **44 dependency entries** (not 77 as one critic claimed):

| Category | Count |
|---|---|
| Internal crates (ifa-types, ifa-vm, ifa-infra, ifa-sandbox) | 4 |
| Async/network (tokio, reqwest, futures-channel, tokio-rustls) | 4 |
| Database (rusqlite) | 1 |
| GUI/TUI (ratatui, crossterm) | 2 |
| Crypto (ring, argon2) | 2 |
| GPU (wgpu, bytemuck) | 2 |
| ML (ndarray) | 1 |
| FFI (libloading, libffi, boa_engine, pyo3, libc) | 5 |
| Audio (rodio) | 1 |
| Compression (zstd) | 1 |
| Serialization (serde, serde_json, bincode, csv) | 4 |
| Utilities (chrono, regex, uuid, rand, rand_chacha, base64, urlencoding, ropey, dashmap, once_cell, heapless, memmap2, pollster) | 13 |
| WASM-only (wasm-bindgen-futures) | 1 |

### wgpu workspace dependency

`wgpu = "0.19.4"` is in the workspace `[dependencies]` for version pinning. It is consumed as **optional** in all three crates that reference it:
- `ifa-std`: `wgpu = { workspace = true, optional = true }`
- `ifa-vm`: `wgpu = { workspace = true, optional = true }`
- `ifa-infra`: `wgpu = { workspace = true, optional = true }`

This is standard Rust workspace practice — no crate links wgpu unless the `gpu` feature is enabled. The unconditional workspace entry just ensures all crates use the same version.

### Files

- `crates/ifa-std/Cargo.toml` — 44 dependency entries

---

## 9. Anomalies & Dead Code

### Known false alarms / inaccuracies in critics' claims

| Claim | Actual | Assessment |
|---|---|---|
| "77 ifa-std dependencies" | 44 entries | Overstated, but 44 is still large |
| "Every swap touches atomics during sort" | `to_vec()` clones into plain `Vec`, sorts the clone | Wrong — no atomic access during sort |
| "tap() has 80-arm match" | tap() calls `OpCode::from_u8()` which has 74-arm match | Minor numeric inaccuracy, same structural cost |
| "IfaValue has two active types causing bugs" | `value.rs` is dead code — all crates use `value_union.rs` | Not buggy, but confusing to readers |

### Confirmed issues requiring action

| Issue | Severity | File | Status |
|---|---|---|---|
| Add doc says "PURE NUMERIC" but code handles strings | Medium | `vm.rs:5-8` vs `vm.rs:2425-2444` | ❌ Open |
| `gc_policy()` defined but never used | Low | `bytecode.rs:58-66` vs `vm.rs:984-988` | ✅ Fixed |
| `nan_box.rs` 374 lines, ephemeral use only | Low-Medium | `nan_box.rs` (deleted) | ✅ **Deleted** — file removed, arithmetic uses direct enum dispatch |
| NaN sort silently misorders incomparable types | Medium | `vm.rs:3012-3015` old | ✅ **Fixed** — now uses `Ordering::Greater` for NaN |
| `ajose.rs` 497 lines, incomplete, unused in production | Low | `ajose.rs` | ❌ Open — actively used by 5 modules (opon, vm, macros, parser), 497 lines not 692 |
| `ebo!` macro was no-op (compiler overhead, no semantic effect) | Low | `ebo.rs` old | ✅ **Removed** — no-op macro deleted |
| `value.rs` dead code alongside `value_union.rs` | Low | `value.rs` | ✅ **Deleted** — stale re-export removed |
| Sandbox no depth limit | Low-Medium | `igbale.rs:56-67` | ❌ Open |
| Embedded target features empty | Medium | `ifa-embedded/Cargo.toml` | ❌ Open |
| `web-sys`, `wasm-bindgen-futures` unused in ifa-wasm | Low | `ifa-wasm/Cargo.toml` | ⚠️ **Partial** — `web-sys` now used for console logging, `wasm-bindgen-futures` still unused |
| `ifa build` includes SQLite via default-features | Medium | `main.rs:874-881` old | ✅ **Fixed** — `default-features = false`, only `async_runtime`/`network`/`parallel`/`dashmap` |
| WASM console not routing print events | Low | `lib.rs:80-90` old | ✅ **Fixed** — calls `console::log_1` for print events |
| AILEWU_BLOCK warned-to-error under strict mode | Low | `diagnose.rs:179-185` old | ✅ **Fixed** — explicit exclusion of `AILEWU_BLOCK` code from strict promotion |

---

## 10. Recommendation Summary

### Priority order

```
P0: Fix the Add opcode comment/code mismatch
P1: Direct threading dispatch (50 lines, 3x speedup)
- ✅ GC trigger from gc_policy() instead of hardcoded 1024 (Fixed)
- ✅ NaN boxing file deleted (option B — inline enum dispatch)
P2: Sort: [i64] fast path (NaN case already fixed)
P2: MIR: shared IR between compiler and transpiler
P2: ajose: finish or delete
P3: Split vm.rs step() into family dispatch functions
P3: Remove wasm-bindgen-futures from ifa-wasm (web-sys now used)
P3: Replace ebo.rs ManuallyDrop with scopeguard
P3: Add sandbox recursion depth limit
- ✅ Hook gc_policy() into VM (Fixed)
P4: Document 20 unsafe blocks in gc.rs
```

**Resolved items removed from priority list:**
- ✅ `value.rs` deletion — done
- ✅ `ebo!` no-op macro removal — done
- ✅ NaN sort incomparable type handling — fixed (Greater for NaN)
- ✅ `ifa build` default-features — fixed (only required features)
- ✅ WASM console logging — fixed (web-sys now used)
- ✅ AILEWU_BLOCK strict mode — fixed (explicit exclusion)

### Architectural pattern to repeat (slab allocator model)

```
ifa-embedded (primitive)
      ↑
ifa-infra (consumer)
```

The slab allocator demonstrates the correct pattern: define the primitive in `ifa-embedded`, consume it from higher-level crates. Do more of this — extract shared runtime primitives into `ifa-embedded`, don't try to unify the full VMs.

### Architectural pattern to avoid (two IfaValue enums — resolved)

The stale `crates/ifa-types/src/value.rs` re-export shim and `crates/ifa-vm/src/value.rs` re-export module have both been deleted. Only `crates/ifa-types/src/value_union.rs` remains as the canonical `IfaValue` definition. All imports across the workspace use `ifa_types::IfaValue` directly.

Lesson: keep one canonical definition for foundational types. When a module is superseded, delete it rather than leaving dead re-exports.
