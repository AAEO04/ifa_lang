# Ifá-Lang Hardening Patch: Strategic Blueprint

This patch documents the architectural and security remediations required for Ifá-Lang v0.2. It addresses performance bottlenecks in string interpolation and critical vulnerabilities in the polyglot FFI bridge.

---

## 1. String Interpolation (Hot-Path Optimization)

**Issue**: Current implementation overloads `OpCode::Add`, polluting the arithmetic hot path.

### Proposed Changes:
- **`ifa-bytecode`**: Define `OpCode::Concat (0x27)` and `OpCode::ToString (0x28)`.
- **`ifa-core` (VM)**: 
    - Implement `OpCode::Concat` for strict `Str + Str` operations.
    - Implement `OpCode::ToString` for coerced conversion.
    - Revert `OpCode::Add` to pure numeric logic (Int/Float only).
- **`ifa-core` (Compiler)**: Emit `ToString` followed by `Concat` for all interpolated segments.

---

## 2. FFI Bridge Security (The Shield of Ọ̀kànràn)

**Issue**: Thread-unsafe environment manipulation, guest-level sandbox escapes, and memory leaks.

### Remediation Blueprint:
- **Eliminate `set_var` (BUG-018)**: Refactor bridge initialization to pass configuration directly to guest interpreter structs (e.g., `PyConfig` for Python) instead of modifying process-wide environment variables.
- **Guest-Level Audit Hooks (BUG-019)**: Implement a callback system between the guest interpreter (Python/JS) and the Ifá `Ofun` capability matrix. Trap guest requests for `os.system` or file I/O and verify against Ifá permissions.
- **Native Pointer Ownership (BUG-020)**: Implement a `Free` capability in the bridge. Native returns marking ownership must be explicitly deallocated using `libc::free` or the bridge-specific allocator after the Ifá string is copied.
- **Secure Library Verification (BUG-021)**: Implement a hash-locked loading mechanism. Instead of just checking the path, verify the library's content hash against a "Sanctified" manifest to prevent TOCTOU replacement hacks.

---

## 3. Static Oversight (Babalawo)

**New Rule**: `TABOO_UNSAFE_FFI`
- Babalawo must statically flag any `ffi.itumo()` (Summon Bridge) call.
- All bridge summons require explicit authorization in the `Babalawo.wisdom` manifest.
- **The Proverb**: "Hidden mirrors reflect stolen shadows" — FFI should never be invisible to the auditor.

---

## 4. Catastrophic Failure & Segfault Handling

**Philosophy**: "Fail-Stop over Corruption."

### Strategy:
- **The Core**: Rely on Rust's memory safety to eliminate segfaults in the VM logic. bounds-checked accesses and the `Option/Result` pattern ensure that the runtime itself is immune to standard C-style memory corruption.
- **The Bridges**: FFI-induced segfaults are treated as fatal. No process-level signal handlers are implemented; a segfault in a foreign library terminates the Ifá-Lang process immediately. This "hard exit" is intentional to prevent compromised guest code from corrupting host memory.
- **Static Guard**: Babalawo flags high-risk FFI bindings via `TABOO_UNSAFE_FFI` to prevent "Hidden Crashes" before they can manifest at runtime.

---

**Status**: DOCUMENTED / READY FOR IMPLEMENTATION
**Action**: DO NOT EXECUTE - Refer to this blueprint for future hardening cycles.
