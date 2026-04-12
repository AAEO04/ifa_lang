# Ifa-Lang Known Bugs

## Resolved

## BUG-001 - `nipari` (finally) skipped when `pada` (return) is inside `gba` (catch) block
**Severity:** High  
**Status:** Resolved  
**Found in:** `ifa-core`

### Resolution
The VM recovery flow now preserves a finally-only recovery frame after entering catch, so `return` from the catch body still diverts through `nipari` before the function completes.

---

## BUG-002 - `OpCode::Return` popped the call frame too early
**Severity:** Critical  
**Status:** Resolved  
**Found in:** `ifa-core`

### Resolution
`OpCode::Return` no longer pops the active `CallFrame` before entering `nipari`. The frame is now preserved until the final `FinallyEnd`, so cleanup code still sees valid locals.

---

## BUG-003 - Nested `finally` blocks skipped during `Return`
**Severity:** High  
**Status:** Resolved  
**Found in:** `ifa-core`

### Resolution
`FinallyEnd` now chains pending return propagation through outer recovery frames instead of completing the return after only the innermost `finally`.

---

## BUG-004 - `PropagateError` (`?`) was semantically incomplete
**Severity:** Medium  
**Status:** Resolved  
**Found in:** `ifa-core`

### Resolution
The VM now unwraps `IfaValue::Result(true, value)` and rethrows `IfaValue::Result(false, err)` as `IfaError::UserError`, which makes `?` behave like real propagation instead of a no-op.

---

## BUG-005 - Security: Bytecode execution (`runb`) bypassed Babalawo
**Severity:** Critical  
**Status:** Resolved  
**Found in:** `ifa-cli`

### Resolution
`runb` now refuses to execute unverified bytecode unless a matching `.ifa` source file exists and passes `run_babalawo`.

---

## BUG-006 - `Odi.ko` (file write) false positive lifecycle error
**Severity:** Medium  
**Status:** Resolved  
**Found in:** `ifa-babalawo`

### Resolution
`LIFECYCLE_RULES` now treats `odi.ko` as auto-closing instead of requiring `odi.pa`.

---

## BUG-009 - `TabooEnforcer` could not block all targets from a source
**Severity:** Low  
**Status:** Resolved  
**Found in:** `ifa-babalawo`

### Resolution
Specific taboo matching now treats an empty `target_domain` as a wildcard target, so `source -> *` rules work.

---

## BUG-011 - Babalawo diagnostic format non-compliance
**Severity:** Low  
**Status:** Resolved  
**Found in:** `ifa-babalawo`

### Resolution
Current diagnostic formatting emits the compact `severity[Odu] file:line:col` shape required for IDE integration, and the crate tests pass with that formatter.

---

## BUG-012 - Security: `AjoseBridge::sh` allowed arbitrary host shell execution
**Severity:** Critical  
**Status:** Resolved  
**Found in:** `ifa-core`

### Resolution
`coop.sh` is now disabled. The bridge no longer shells out through `cmd /C` or `sh -c`.

---

## BUG-013 - Transpiler aborted with `panic!` and `unimplemented!`
**Severity:** Medium  
**Status:** Resolved  
**Found in:** `ifa-core`

### Resolution
The affected domain-lowering paths now emit structured `compile_error!` output instead of host-process aborts.

---

## BUG-015 - Closure opcode round-trip and module import execution regressions
**Severity:** Medium  
**Status:** Resolved  
**Found in:** `ifa-core`

### Resolution
The VM now preserves closure environments across top-level tail calls, imported module functions execute against their owning module bytecode instead of the caller chunk, and module reloads invalidate stale export state correctly. The closure opcode round-trip integration test now passes again.

---

## BUG-016 - AST top-level return normalization and `ayanfe` parsing mismatch
**Severity:** Medium  
**Status:** Resolved  
**Found in:** `ifa-core`

### Resolution
The AST interpreter now unwraps top-level `Return(...)` values before reporting results, and the parser/grammar now accept `ayanfe` / `àyànfẹ́` as constant declarations, which makes the AST conformance suite pass again.

---

## BUG-017 - AST `nipari`, unlimited Opon growth, and lossy serialization cleanup
**Severity:** Medium  
**Status:** Resolved  
**Found in:** `ifa-core`, `ifa-types`

### Resolution
The AST interpreter now executes `finally_body`, unlimited `Opon` growth is bounded by a hard host-safety ceiling, and unsupported `IfaValue` serialization now fails explicitly instead of degrading to `Null`.

---

## BUG-007 - `IfaValue::Object` serialization is skipped or degraded
**Severity:** Medium  
**Status:** Resolved  
**Found in:** `ifa-types`

### Description
Unsupported runtime values no longer degrade silently to `Null` during `value_union` serialization. They now fail explicitly. The legacy `value.rs` path still has skipped variants, but the active runtime path no longer lies about round-tripping.

---

## BUG-008 - Arithmetic traits still return `Null` on division by zero
**Severity:** Low  
**Status:** Resolved  
**Found in:** `ifa-types`

### Description
`IfaValue::checked_div` returns proper `DivisionByZero` errors. The flawed `impl Div` trait which silently yielded `Null` has been explicitly deleted, forcing all invocation chains through the safety perimeter.

---

## BUG-010 - OOM risk in `Opon` unlimited mode
**Severity:** Medium  
**Status:** Resolved  
**Found in:** `ifa-core`

### Description
`OponSize::Ailopin` is now bounded by a hard host-safety ceiling instead of allowing arbitrary `Vec` growth to any `usize`.

---

## BUG-014 - Supply-chain advisory: `rustls-pemfile` 1.0.4 is unmaintained
**Status**: Resolved
**Priority**: Medium
**Component**: Dependencies

The dependency graph was locked to `rustls-pemfile` `1.0.4` which had a RustSec advisory `RUSTSEC-2025-0134`. This has been resolved and it is no longer in the dependency graph.

---

## Open

## BUG-018 - Thread-Unsafe Environment Manipulation in FFI Bridge
**Severity:** Critical  
**Status:** Open  
**Found in:** `ifa-std`

### Description
`std::env::set_var` is used in `itumo` (ffi binary summon) to set `PYTHONHOME`. This is globally unsound in multi-threaded programs and leads to segmentation faults if other threads access the environment concurrently.

---

## BUG-019 - Lack of Guest-Level Sandbox Isolation in Python/JS Bridges
**Severity:** Critical  
**Status:** Open  
**Found in:** `ifa-std`

### Description
The Python and JS bridges do not propagate Ifá's capability-based security into the guest interpreters. A whitelisted Python bridge can be used to execute `os.system()` or file I/O that bypasses all Ifá taboos.

---

## BUG-020 - Memory Leak in Native String Return Handling
**Severity:** Medium  
**Status:** Open  
**Found in:** `ifa-std`

### Description
Native FFI calls returning `char*` (strings) are copied into owned Ifa strings, but the original pointer is never freed. This causes progressive memory leaks for libraries that return heap-allocated strings (e.g., `strdup`).

---

## BUG-021 - TOCTOU Race Condition in Library Path Validation
**Severity:** Medium  
**Status:** Open  
**Found in:** `ifa-std`

### Description
There is a window between path validation (checking symlinks/traversal) and actual library loading. An attacker with write access to the library directory could replace a validated library with a malicious one before `libloading` execution.
