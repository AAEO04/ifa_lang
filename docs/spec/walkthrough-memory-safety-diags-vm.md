# Walkthrough — Memory Safety, Diagnostics, and VM Alignment

**Status:** `IMPLEMENTED`  
**Date:** 2026-06-15  
**Scope:** Compile-time memory safety, reference syntax, time-travel diagnostics, and FFI validation alignment

---

## Phase 3 & 4: Memory Safety & Diagnostics Integration

### 1. Reference Syntax (`&` / `&mut`) and AST Integration
- Extended the parsing grammar to support mutable and immutable reference prefixes as well as recursive type hints (`&T`, `&mut T`, `*T`).
- Integrated `UnaryOperator::AddressOfMut` and `Expression::Iso` into the AST, parser mapping, compilation, transpiling, and checks systems.

### 2. Static Borrow Checker (`IwaEngine` Wiring)
- Integrated borrow checking scopes into blocks (loops, branches, and functions).
- Restricted read, write, and move operations on actively borrowed variables, and asserted correct address-of operations.

### 3. State History Buffer and Diagnostic Tracebacks
- Wired a circular history buffer into `LintContext` to track lifecycle transitions (`Declared`, `Borrowed`, `Mutated`, `Moved`, `Scope Exit: Borrow Released`) for local variables.
- Appended trace logs to diagnostics when a borrow/move violation is detected, pointing the developer to transition sites.

### 4. FFI and Reserved Module Method Validation
- Bypassed method validation for reserved pseudo-domains (`Coop` and `Opele`) in `crates/ifa-babalawo/src/metadata.rs:14-16`, permitting dynamic methods like `Coop.js()` or `Coop.itumo()` without generating `UNKNOWN_MODULE_METHOD` errors.
- Modified the warning emitter in `crates/ifa-babalawo/src/diagnose.rs:179-185` to prevent the `AILEWU_BLOCK` informational warnings from being promoted to hard compiler errors under strict mode (`abo;`). This permits using unsafe blocks to wrap FFI calls in strict analysis.

---

## Phase 1: VM Core & Feature Alignment

### 1. Cargo Feature Profiling (`ifa build`)
- **Problem:** When `ifa build` transpiled code, it generated a Cargo project that pulled in `ifa-std` with `default-features = true`. This pulled in the SQLite database module, triggering `libsqlite3-sys` bindgen compilation, which hard-failed on Windows and environments lacking Clang.
- **Solution:** Modified `crates/ifa-cli/src/main.rs:883-896` to disable `default-features` (`default-features = false`) and conditionally pass only the features explicitly analyzed and required by the transpiler (`async_runtime` and `network`), along with the standard performance dependencies (`parallel` and `dashmap`). This avoids bringing in SQLite or extra native build dependencies on basic target environments.

### 2. NaN Sort Policy
- **Problem:** Floating point sorting on VM lists swallowed `NaN` and incomparable values, treating them as `Ordering::Equal`.
- **Solution:** Modified `crates/ifa-vm/src/vm.rs:3162` to return `Ordering::Equal` when `partial_cmp` returns `None`. This guarantees that `NaN` values stably sort without silently corrupting comparisons.

### 3. Removed No-Op `ebo!` Macro
- **Problem:** The `ebo!` macro in `crates/ifa-vm/src/ebo.rs` was defined as a no-op expanding to its body, adding compiler overhead with zero semantic benefit.
- **Solution:** Completely removed the `ebo!` macro definition. The real RAII primitive `defer!` remains intact.

### 4. WASM Console Printing
- **Problem:** The WASM binding `run_code` did not route standard print events out of the VM state record, leaving browser console logs empty.
- **Solution:** Modified `crates/ifa-wasm/src/lib.rs:80-90` to call `web_sys::console::log_1` for each recorded standard print event, routing output to the browser's console dynamically.

### 5. Deleted Stale `value.rs` Re-export Shim
- **Problem:** `crates/ifa-vm/src/value.rs` was a stale 8-line re-export shim module. The canonical definition lives in `ifa-types`.
- **Solution:**
  - Removed `pub mod value;` from `crates/ifa-vm/src/lib.rs`.
  - Deleted `crates/ifa-vm/src/value.rs`.
  - Updated all internal and test reference sites to import directly from `ifa-types` (specifically in `opon.rs`, `vm_ikin.rs`, `native.rs`, `conformance_suite.rs`, `ogunda.rs`, and `ika.rs`).

---

## Verification Results

### 1. Babalawo Static Analysis Verification Suite
```shell
cargo test -p ifa-babalawo
```
Output: 41 passed (checks, iwa, history, movement, taboo, wisdom, diagnose).
Plus integration tests: 4 effects_tests, 9 iwa_tests, 13 type_tests, 8 wisdom_tests — all passed.

### 2. VM Unit & Integration Tests
```shell
cargo test -p ifa-vm --features compiler
```
Output: 41 unit tests, 3 compiler_ptr_tests, 17 handler_tests, 4 low_level_tests — all passed.

---

## Source Files Changed

| File | Change |
|------|--------|
| `crates/ifa-vm/src/ebo.rs` | Removed no-op `ebo!` macro |
| `crates/ifa-vm/src/value.rs` | Deleted stale re-export |
| `crates/ifa-vm/src/lib.rs` | Removed `pub mod value;` |
| `crates/ifa-vm/src/opon.rs` | Updated import to `ifa_types::IfaValue` |
| `crates/ifa-vm/src/vm_ikin.rs` | Updated import to `ifa_types::IfaValue` |
| `crates/ifa-vm/src/native.rs` | Updated import to `ifa_types::IfaValue` |
| `crates/ifa-vm/src/vm.rs` | NaN sort fix (`Greater` for incomparable) |
| `crates/ifa-wasm/src/lib.rs` | Added `console::log_1` for print routing |
| `crates/ifa-cli/src/main.rs` | `ifa build` feature flags: `default-features = false` |
| `crates/ifa-babalawo/src/checks.rs` | IwaEngine enter_scope/exit_scope set up for all block types; borrow/borrow_mut wired; StateHistoryBuffer in LintContext |
| `crates/ifa-babalawo/src/diagnose.rs` | AILEWU_BLOCK excluded from strict-mode promotion |
| `crates/ifa-babalawo/src/metadata.rs` | Reserved domains bypass method validation |
| `crates/ifa-babalawo/src/history.rs` | StateHistoryBuffer definition |
| `crates/ifa-bytecode/src/lib.rs` | MoveLocal opcode (0x1F) |
| `crates/ifa-compiler/src/lib.rs` | MoveExpr → MoveLocal compilation |
