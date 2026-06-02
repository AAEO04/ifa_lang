# Ifá-Lang Transpiler Completion Plan

This plan aims to implement the final remaining features in the AOT transpiler (`ifa-transpiler`), specifically targeting pointer manipulation, module imports, and the remaining standard library domains (Obara, Ika, Oturupon).

## User Review Required

> [!WARNING]
> **Pointer Semantics in Safe Rust**
> The Ifá-Lang AOT transpiler currently outputs safe Rust. True raw pointer arithmetic (`*mut T` and `*const T`) requires `unsafe` blocks and risks memory unsafety (leaks, UAF) if we directly write dynamically typed `IfaValue` enums to raw memory addresses. 
> 
> **Proposal:** Instead of actual raw memory pointers, we introduce a new variant to the generated `IfaValue` enum: `Ptr(std::sync::Arc<std::sync::Mutex<IfaValue>>)` or `Ptr(Box<IfaValue>)`. This would safely map Ifá pointer semantics to Rust's managed heap, avoiding raw memory unsafety while still fulfilling the `*p = v` and `*p += 1` semantics.

> [!IMPORTANT]
> **Module Imports**
> Implementing module imports at transpilation time means the AOT transpiler will either:
> 1. Read the imported `.ifa` file, parse its AST, and inline the output directly into the generated `main.rs`.
> 2. Transpile the imported `.ifa` file into a separate `mod {name};` Rust file inside the generated build directory.
> 
> **Proposal:** I will go with **Approach 1 (Inlining)**. It is significantly more reliable for the single-binary output of the AOT builder and avoids complex file I/O tracking during the `rustc` stage.

## Open Questions

> [!CAUTION]
> 1. For pointer implementation, do you strictly require raw memory addresses (like `IfaValue::Ptr(usize)` modeling a simulated byte array), or is the managed heap approach (`Arc<Mutex<IfaValue>>`) acceptable for safety?
> 2. For module imports, is AST inlining acceptable?

## Proposed Changes

### 1. Pointer and Dereference Stubs

#### [MODIFY] [statements.rs](file:///C:/Users/allio/Desktop/ifa_lang/crates/ifa-transpiler/src/transpiler/statements.rs)
- **`IfaValue` Definition:** Add `Ptr(std::sync::Arc<std::sync::Mutex<Box<IfaValue>>>)` to the transpiler's generated enum.
- **Dereference Assignment (`*p = v`):** Update `Statement::Assign` to recognize `AssignTarget::Dereference`. It will unwrap the pointer variant and assign the new value inside the lock.
- **Dereference Update (`*p += 1`):** Update `Statement::Update` to handle `AssignTarget::Dereference` by locking the pointer and applying the `UpdateOp`.

### 2. Module Imports

#### [MODIFY] [statements.rs](file:///C:/Users/allio/Desktop/ifa_lang/crates/ifa-transpiler/src/transpiler/statements.rs)
- **`Statement::Import`:** Remove the `compile_error!()` stub.
- **Logic:** Add an inline transpilation step. When `Import(path)` is encountered, the transpiler will attempt to read the file, run the `ifa-parser`, and transpile the statements directly into the current scope (or wrap them in a pseudo-namespace if necessary).

### 3. Missing Odù Domains (Obara, Ika, Oturupon)

#### [MODIFY] [domains.rs](file:///C:/Users/allio/Desktop/ifa_lang/crates/ifa-transpiler/src/transpiler/domains.rs)
- **Obara (Network/IO):** Implement basic mappings for HTTP requests or standard I/O (e.g. `Obara.gbegbe` or equivalent). 
- **Ika (Cryptography):** Map Ika methods to basic Rust hashing/crypto functions if available (e.g., `std::collections::hash_map::DefaultHasher` for basic hashing, or SHA256 if `ifa-std` provides it).
- **Oturupon (Math/ML):** Map Oturupon methods (e.g., `Oturupon.sin`, `Oturupon.cos`, `Oturupon.sqrt`) directly to Rust's `f64::sin()`, `f64::cos()`, and `f64::sqrt()` on `IfaValue::Float`.

## Verification Plan

### Automated Tests
- Run `cargo test -p ifa-cli -- --nocapture` to ensure the transpiler builds successfully.
- Ensure the `conformance_tier1.rs` execution does not fail on any new tests.

### Manual Verification
- Write a dummy `pointer.ifa` script that uses `*p = v` and `*p += 1` and compile it via `ifa build pointer.ifa`. Verify the output Rust code.
- Write a dummy `imports.ifa` script that uses module imports and verify the AST inlining works.
