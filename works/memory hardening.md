# Memory & Unsafe Hardening — Implementation Plan

> **Scope:** Real unsafe surface only. Grounded in actual code.
> **Rule:** Fix what is actually broken. Do not redesign what is already safe.

---

## Ground Truth: What Is Actually Unsafe

The submitted review described a memory model that does not exist (`Vec<u8>`, `mmio_base`, raw pointer arithmetic in Opon). The actual unsafe surface is:

| # | Location | Lines | Issue | Severity |
|---|---|---|---|---|
| 1 | `ebo.rs:40,49,59` | `ManuallyDrop::drop/take` | Correct usage, zero safety comments | Low |
| 2 | `ffi.rs:412–416` | `libloading::Library::new` | Correct, but no `// SAFETY:` doc | Low |
| 3 | `ffi.rs:573–595` | `libloading::Symbol` + `*symbol.deref() as *mut c_void` | Raw fn pointer cast stored in `BoundFunction.ptr: *mut c_void` | **High** |
| 4 | `ffi.rs:785–815` | `call_native_libffi` entire body | `libffi::high::call` + raw C-string return with undocumented lifetime | **High** |
| 5 | `odi.rs:83` | `mmap` | Memory-mapped file — no `// SAFETY:` | Medium |
| 6 | `sandbox/omnibox.rs:98` | Platform syscall | Needs doc | Low |
| 7 | `stacks/crypto.rs:386` | Crypto buf operation | Needs doc | Low |

### What Is NOT a Problem (Do Not Touch)
- `opon.rs` — zero `unsafe` blocks. All bounds-checked via `Vec::get`. **Correct.**
- `ebo.rs` `ManuallyDrop` usage — semantically correct. RAII pattern is sound.
- `ifa-bytecode` — already has `#![forbid(unsafe_code)]` at line 15. **Done.**
- `IfaValue` / `value_union.rs` — explicitly documents "No unsafe unions. pure Rust."
- Stack overflow/underflow — handled by `push()`/`pop()` returning `Result`. **Done.**

---

## Phase 1 — Crate-Level `unsafe` Attributes

**Goal:** Force every unsafe operation in every crate to be explicitly justified.  
**Cost:** Zero runtime impact. Compile-time only.

### 1.1 — Add `#![deny(unsafe_op_in_unsafe_fn)]` to ifa-core and ifa-std

**File: `ifa-core/src/lib.rs`** — add at the top:
```rust
// Every unsafe operation must be inside its own unsafe {} block,
// even if the enclosing function is already unsafe.
// This forces explicit justification at each unsafe site.
#![deny(unsafe_op_in_unsafe_fn)]
```

**File: `ifa-std/src/lib.rs`** — add at the top:
```rust
#![deny(unsafe_op_in_unsafe_fn)]
```

**File: `ifa-sandbox/src/lib.rs`** — add at the top:
```rust
#![deny(unsafe_op_in_unsafe_fn)]
```

### 1.2 — Check which crates can add `#![forbid(unsafe_code)]`

`ifa-bytecode` already has `#![forbid(unsafe_code)]`. Check the remaining crates:

```powershell
# Find crates that contain no unsafe blocks at all:
rg "unsafe" crates/ifa-types/src/ crates/ifa-babalawo/src/ crates/ifa-fmt/src/ --type rust
```

For any crate where this search returns nothing: add `#![forbid(unsafe_code)]`. Expected candidates: `ifa-types`, `ifa-babalawo`, `ifa-fmt`.

### Exit Criteria for Phase 1

```powershell
# Should compile cleanly with no new errors introduced by the attribute:
cargo check -p ifa-core
cargo check -p ifa-std
cargo check -p ifa-sandbox

# These should succeed (no unsafe to deny):
cargo check -p ifa-types
cargo check -p ifa-babalawo
```

---

## Phase 2 — Document `ebo.rs` `ManuallyDrop` Invariants

**Goal:** Every `unsafe` block in `ebo.rs` gets a `// SAFETY:` comment explaining why it is sound.

**File: `ifa-core/src/ebo.rs`**

### 2.1 — `dismiss()` (line 40)
```rust
// Before
pub fn dismiss(mut self) {
    unsafe {
        ManuallyDrop::drop(&mut self.cleanup);
    }
    std::mem::forget(self);
}

// After
pub fn dismiss(mut self) {
    // SAFETY: We are the sole owner of `cleanup` (ManuallyDrop prevents the
    // compiler from auto-dropping it). We call `drop` here to release the
    // Option<F> without running the cleanup, then `forget(self)` to prevent
    // a double-drop in the Drop impl. This is the canonical dismiss pattern.
    unsafe {
        ManuallyDrop::drop(&mut self.cleanup);
    }
    std::mem::forget(self);
}
```

### 2.2 — `sacrifice()` (line 49)
```rust
// Before
pub fn sacrifice(mut self) {
    if let Some(cleanup) = unsafe { ManuallyDrop::take(&mut self.cleanup) } {
        cleanup();
    }
    std::mem::forget(self);
}

// After
pub fn sacrifice(mut self) {
    // SAFETY: `ManuallyDrop::take` is called exactly once — here.
    // The Drop impl will not run the cleanup again because `forget(self)`
    // prevents the Drop impl from executing at all.
    // Invariant: `cleanup` is always initialized at construction and
    // taken at most once (either here in `sacrifice`, or in `Drop::drop`).
    if let Some(cleanup) = unsafe { ManuallyDrop::take(&mut self.cleanup) } {
        cleanup();
    }
    std::mem::forget(self);
}
```

### 2.3 — `Drop::drop` (line 59)
```rust
impl<F: FnOnce()> Drop for Ebo<F> {
    fn drop(&mut self) {
        // SAFETY: `ManuallyDrop::take` is called at most once across
        // `sacrifice`, `dismiss`, and `Drop::drop`. Only one of these
        // paths executes per Ebo lifetime:
        // - `sacrifice` → takes cleanup, then forgets self (Drop not called)
        // - `dismiss` → drops cleanup uninitialised, then forgets self (Drop not called)
        // - `Drop::drop` → only reached if neither of the above ran
        if let Some(cleanup) = unsafe { ManuallyDrop::take(&mut self.cleanup) } {
            cleanup();
        }
    }
}
```

### Exit Criteria for Phase 2

- Every `unsafe` block in `ebo.rs` has a `// SAFETY:` comment.
- All existing `ebo.rs` tests pass.
- `cargo doc -p ifa-core` renders the safety invariants visibly.

---

## Phase 3 — Harden `ffi.rs` Unsafe Surface

This is the only genuinely high-risk unsafe code in the codebase. Three specific problems:

### 3.1 — `BoundFunction.ptr` should be `NonNull<c_void>`, not `*mut c_void`

**File: `ifa-std/src/ffi.rs`**

```rust
// Before (line 150)
pub struct BoundFunction {
    pub name: String,
    pub ptr: *mut c_void,
    pub sig: FfiSignature,
}

// After
use std::ptr::NonNull;

pub struct BoundFunction {
    pub name: String,
    /// Non-null pointer to the loaded C function.
    /// SAFETY invariant: ptr is always a valid function pointer obtained
    /// via libloading::Symbol, which guarantees the pointed-to address
    /// is non-null and valid for the lifetime of the parent Library.
    pub ptr: NonNull<c_void>,
    pub sig: FfiSignature,
}
```

Update `bind()` (line 578):
```rust
// Before
let ptr = *symbol.deref() as *const () as *mut c_void;

// After
// SAFETY: libloading guarantees the symbol address is non-null and valid
// for as long as the Library is alive. The Library lives in self.backends.
let raw_ptr = *symbol.deref() as *const () as *mut c_void;
let ptr = NonNull::new(raw_ptr)
    .ok_or(FfiError::FunctionNotFound(format!("{} resolved to null", func)))?;
```

### 3.2 — Document `Library::new` in `load_native_verified()` (line 412)

```rust
// Before
unsafe {
    let lib = libloading::Library::new(&validated_path)
        .map_err(|e| FfiError::LibraryNotFound(format!("{}: {}", lib_path, e)))?;
    self.backends.insert(name.to_string(), Backend::Native(lib));
}

// After
// SAFETY: `validated_path` has been:
//   1. Checked for path traversal (..)
//   2. Checked to not be a symlink
//   3. Canonicalized to an absolute path
//   4. Optionally verified against a SHA-256 digest
// We still cannot guarantee the library's internal code is safe,
// which is why this entire operation requires an `ailewu` block in Ifá-Lang.
unsafe {
    let lib = libloading::Library::new(&validated_path)
        .map_err(|e| FfiError::LibraryNotFound(format!("{}: {}", lib_path, e)))?;
    self.backends.insert(name.to_string(), Backend::Native(lib));
}
```

### 3.3 — Document and harden the raw C-string return path (line ~812)

This is the highest-risk site: a raw `*const c_char` returned from a C function with no ownership semantics documented.

```rust
// Before (line ~812)
IfaType::Str => {
    let result: *const std::os::raw::c_char = call(code_ptr, ffi_args.as_slice());
    if result.is_null() {
        Ok(FfiValue::Null)
    // ... rest elided
    }
}

// After — add explicit SAFETY comment and document the lifetime assumption
IfaType::Str => {
    // SAFETY: We call the C function and receive a *const c_char.
    // ASSUMPTION (caller contract): The returned pointer is valid UTF-8,
    // null-terminated, and remains valid for the duration of this unsafe block.
    // We immediately copy the bytes into an owned String, so the C-side
    // lifetime constraint ends here. If the C function returns a pointer
    // to stack memory or a pointer that is freed on the next call,
    // this will be UB. Ifá-Lang Babalawo cannot verify this contract —
    // it is the ailewu caller's responsibility.
    let result: *const std::os::raw::c_char = call(code_ptr, ffi_args.as_slice());
    if result.is_null() {
        Ok(FfiValue::Null)
    } else {
        // SAFETY: We verified non-null above. CStr::from_ptr requires
        // the pointer to be valid and null-terminated — this is the
        // contract the caller accepted by using ailewu + IfaType::Str.
        let s = unsafe {
            std::ffi::CStr::from_ptr(result)
                .to_string_lossy()
                .into_owned()
        };
        Ok(FfiValue::Str(s))
    }
}
```

### 3.4 — Document `bind()` libloading::Symbol unsafe block (line 573)

```rust
Backend::Native(lib_handle) => {
    // SAFETY: `lib_handle` is a valid Library obtained from `load_native_verified`.
    // `get` returns a Symbol whose lifetime is tied to the Library's lifetime.
    // We immediately extract the raw pointer — the Symbol itself is dropped here,
    // but the underlying function pointer remains valid as long as the Library
    // lives in `self.backends`. This is safe because:
    // 1. `self.backends` owns the Library
    // 2. BoundFunction is stored in `self.functions` which has the same lifetime
    // 3. We do not expose BoundFunction outside of IfaFfi
    unsafe {
        // ... existing code ...
    }
}
```

### Exit Criteria for Phase 3

- `cargo clippy -p ifa-std -- -W clippy::undocumented_unsafe_blocks` produces zero warnings in `ffi.rs`.
- `BoundFunction.ptr` is `NonNull<c_void>` — null function pointer can never be stored.
- Every `unsafe {}` block in `ffi.rs` has a `// SAFETY:` comment.

---

## Phase 4 — Fuzz the FFI Argument Dispatch

**Goal:** Confirm that malformed bytecode sequences cannot produce UB through the FFI argument path.

**Why `call_native_libffi` specifically:** It takes `args: &[FfiValue]` and builds libffi argument vectors through multiple indexed arrays (`i32_args`, `i64_args`, etc.) with a separate pass to build `ffi_args`. If the first pass and second pass get out of sync (e.g., due to argument type edge cases), `ffi_args[n]` could reference a stale or wrong index.

### 4.1 — Add fuzz target for `call_native_libffi` argument preparation

**New file: `ifa-std/fuzz/fuzz_targets/ffi_args.rs`**

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;
use ifa_std::ffi::{FfiValue, IfaType, FfiSignature};

fuzz_target!(|data: &[u8]| {
    // Construct arbitrary FfiValue sequences and FfiSignature combinations
    // and verify that argument preparation never panics, even with mismatched types.
    // The actual libffi call is NOT made (we test only the arg-building path).
    if data.len() < 4 { return; }
    
    let arg_count = (data[0] as usize) % 8;
    let mut args = Vec::new();
    let mut arg_types = Vec::new();
    
    for i in 0..arg_count {
        let byte = data.get(i + 1).copied().unwrap_or(0);
        let (val, ty) = match byte % 5 {
            0 => (FfiValue::I32(byte as i32), IfaType::I32),
            1 => (FfiValue::I64(byte as i64), IfaType::I64),
            2 => (FfiValue::F64(byte as f64), IfaType::F64),
            3 => (FfiValue::Str("test".into()), IfaType::Str),
            _ => (FfiValue::Null, IfaType::Void),
        };
        args.push(val);
        arg_types.push(ty);
    }
    
    // Test that type-validation rejects mismatches cleanly, not with UB
    let sig = FfiSignature { arg_types, ret_type: IfaType::Void };
    // Call only the validation path, not the actual libffi dispatch
    let _ = validate_ffi_args(&args, &sig);
});
```

This requires extracting the argument validation loop from `call_native_libffi` into a `validate_ffi_args(&[FfiValue], &FfiSignature) -> FfiResult<()>` function that can be called independently.

### Exit Criteria for Phase 4

- `cargo fuzz run ffi_args -- -max_total_time=300` finds no panics or UB.
- The validation function is a distinct, testable unit separate from the libffi dispatch.

---

## Sequencing

```
Phase 1: Crate attributes          ← Zero risk. Purely declarative.
    ↓
Phase 2: ebo.rs SAFETY comments    ← Purely documentary. No behavior change.
    ↓
Phase 3: ffi.rs hardening          ← NonNull change is a small breaking API change
    ↓                                 (BoundFunction.ptr type changes).
Phase 4: Fuzzing                   ← Needs Phase 3 (validation function extraction).
```

---

## What Is Explicitly Not Done

| Idea from the submitted review | Decision | Reason |
|---|---|---|
| Rewrite Opon as `Vec<u8>` + `mmio_base` | **Rejected** | Opon is `Vec<IfaValue>`. No byte-level access exists. Adding raw bytes would introduce the safety problems the review hallucinated. |
| `SlotMap<Key, AllocMeta>` for generational indices | **Rejected** | Opon's epoch model is correct for stack-scoped lifetimes. Generational indices solve a different problem (arbitrary individual pointer invalidation). |
| `#[deny(unsafe_op_in_unsafe_fn)]` everywhere | **Done in Phase 1** | This one was correct. |
| Strict provenance (`addr_of!`, `NonNull`) | **Partial** — `NonNull` applied to `BoundFunction.ptr` in Phase 3. `addr_of!` not needed: no pointer-integer casts in the codebase except the libffi path, which is documented. |
| Miri clean code mandate | **Deferred** | Run `cargo miri test` once. Fix actual miri findings. Do not speculatively rewrite code that may already be miri-clean. |

---

## File Change Map

| Phase | Files Changed |
|---|---|
| 1 | `ifa-core/src/lib.rs`, `ifa-std/src/lib.rs`, `ifa-sandbox/src/lib.rs`, possibly `ifa-types/src/lib.rs`, `ifa-babalawo/src/lib.rs` |
| 2 | `ifa-core/src/ebo.rs` |
| 3 | `ifa-std/src/ffi.rs` |
| 4 | `ifa-std/fuzz/fuzz_targets/ffi_args.rs` (new), `ifa-std/src/ffi.rs` (extract validation fn) |
