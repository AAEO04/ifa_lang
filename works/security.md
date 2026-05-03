# Security Issue Fixes
## Grounded in Verified File Locations

---

## Fix 4 — Stack Depth Guard (Critical, ~15 minutes)

**Root cause:** `stack_limit` and `frame_limit` in the VM, and `call_depth_limit` in the interpreter, are initialized to `None` and never set.

### VM (`crates/ifa-core/src/vm.rs:219-220`)

```diff
-            stack_limit: None,
-            frame_limit: None,
+            stack_limit: Some(4096),   // 4096 values on the operand stack
+            frame_limit: Some(512),    // 512 call frames = 512 recursive calls
```

Apply the same change in `IfaVM::with_opon()` at lines 264-265.

Then wire it in the `Call`/`TailCall` opcode handler. Find where `self.frames.push(...)` occurs and add before it:

```rust
// vm.rs — inside the Call opcode handler, before frames.push()
if let Some(limit) = self.frame_limit {
    if self.frames.len() >= limit {
        return Err(IfaError::Runtime(
            format!("Stack overflow: call depth exceeded {} frames", limit)
        ));
    }
}
```

And for the operand stack, find `self.push(...)` helper method and add:

```rust
// vm.rs — in the push() helper
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

### Interpreter (`crates/ifa-core/src/interpreter/core.rs:143`)

```diff
-            call_depth_limit: None,
+            call_depth_limit: Some(512),
```

The check at line 1560-1562 is already wired. This one line activates it.

---

## Fix 5a — Fuel Limit (High, ~30 minutes)

**Root cause:** `vm_iroke.rs:20-22` has a `checks_interrupts` call commented out. The `ticks` counter runs but triggers nothing.

### Step 1 — Add fuel field to `IfaVM` (`vm.rs`)

```rust
// In the IfaVM struct, add:
/// Remaining execution budget. None = unlimited.
fuel: Option<u64>,
```

Initialize in `IfaVM::new()`:
```rust
fuel: None, // Set to Some(N) for sandboxed execution
```

### Step 2 — Activate the fuel check in `vm_iroke.rs:20-22`

```diff
-    if vm.ticks & 1023 == 0 {
-        // checks_interrupts(vm)?; // Placeholder for future checking
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

### Step 3 — Expose a sandbox constructor

```rust
// vm.rs — new method
pub fn sandboxed(fuel: u64) -> Self {
    let mut vm = Self::new();
    vm.fuel = Some(fuel);
    vm.frame_limit = Some(256);   // Stricter limit for sandboxed execution
    vm.stack_limit = Some(2048);
    vm
}
```

---

## Fix 5b — Ikin Pool Cap (High, ~10 minutes)

**Root cause:** `vm_ikin.rs:43` — `intern()` has no size limit.

```diff
 pub fn intern(&mut self, s: &str) -> u32 {
+    const MAX_STRINGS: usize = 65536;
+    if self.strings.len() >= MAX_STRINGS {
+        // In sandboxed use, this should be a hard error.
+        // For now, return the last valid ID to avoid OOM.
+        // TODO: propagate IfaResult to callers that use intern().
+        return (self.strings.len() - 1) as u32;
+    }
     if let Some(&id) = self.string_map.get(s) {
```

> **Note:** The real fix is making `intern()` return `IfaResult<u32>` and propagating the error. That touches all call sites. The cap above is the minimal safe patch — it prevents OOM while tracking that callers need to be updated.

Also cap `load_from_bytecode` at `vm_ikin.rs:99`:

```diff
 self.strings.reserve(bytecode.strings.len());

+if bytecode.strings.len() > 65536 {
+    return Err(IfaError::Runtime(
+        format!("Bytecode has {} strings; maximum is 65536", bytecode.strings.len())
+    ));
+}
 for s in &bytecode.strings {
```

---

## Fix 6 — Global REGISTRY Isolation (High, ~2 hours)

**Root cause:** `registry.rs:65` — `REGISTRY` is a `static`, shared across all `IfaVM` instances. Cross-actor token forgery is possible.

**The fix:** Stop using the global. Each `IfaVM` owns its own `ResourceRegistry`.

### Step 1 — Add registry to `IfaVM`

```rust
// vm.rs — in IfaVM struct
#[serde(skip)]
local_registry: ifa_std::handlers::registry::ResourceRegistry,
```

Initialize in `new()`:
```rust
local_registry: ifa_std::handlers::registry::ResourceRegistry::new(),
```

### Step 2 — Thread the local registry through domain calls

Wherever the domain handlers currently call `REGISTRY.register(...)` / `REGISTRY.get(...)` / `REGISTRY.close(...)`, they must receive the local registry instead. The `VmContext` struct (used to pass context to domain handlers) should hold a mutable reference to the local registry:

```rust
// In native.rs VmContext:
pub struct VmContext<'a> {
    pub globals: &'a mut HashMap<String, IfaValue>,
    pub registry: &'a mut ResourceRegistry,  // ← was the global, now local
}
```

### Step 3 — Keep the global REGISTRY only for `ifa run` (non-sandboxed)

Mark `REGISTRY` as `#[doc(hidden)]` and add a deprecation note. Sandboxed VMs must never touch it.

---

## Fix 7 — Bincode Size Limits (High, ~20 minutes)

**Root cause:** `vm.rs:318` and `infra/storage.rs:374` use `bincode::deserialize` with no size cap.

### `vm.rs:315-328` — Snapshot resume

```diff
 pub fn resume(snapshot: &[u8], bytecode: &Bytecode) -> IfaResult<Self> {
+    const MAX_SNAPSHOT_BYTES: u64 = 64 * 1024 * 1024; // 64MB hard ceiling
+    let opts = bincode::DefaultOptions::new()
+        .with_limit(MAX_SNAPSHOT_BYTES)
+        .with_fixint_encoding()
+        .allow_trailing_bytes();
-    let (saved_hash, vm): (u64, IfaVM) = bincode::deserialize(snapshot)
+    let (saved_hash, vm): (u64, IfaVM) = opts.deserialize(snapshot)
         .map_err(|e| IfaError::Custom(format!("Corrupted snapshot: {}", e)))?;
```

### `infra/storage.rs:374` — Storage deserialization

```diff
+    const MAX_STORED_BYTES: u64 = 16 * 1024 * 1024; // 16MB per stored value
+    let opts = bincode::DefaultOptions::new().with_limit(MAX_STORED_BYTES);
-    let value = bincode::deserialize(&buffer)?;
+    let value = opts.deserialize(&buffer)?;
```

---

## Fix 3 — FFI Library-Load Gating (High, ~45 minutes)

**Root cause:** The FFI env-function block (`ffi.rs:228`) only gates individual function names within the `os` module. A call to load a new native library via `Backend::Native(libloading::Library)` is not capability-gated.

### Add a library-load capability check

In `ffi.rs`, find where `libloading::Library::new(path)` is called. Add before it:

```rust
// ffi.rs — in load_native_library() or wherever Library::new() is invoked
if !self.allow_native_libs {
    return Err(FfiError::SecurityViolation(
        "Loading native libraries requires the 'ffi' capability (--allow-ffi)".into()
    ));
}

// Verify the library path is within the allowed roots, not an absolute
// path pointing outside the workspace:
let path = std::path::Path::new(&lib_path);
if path.is_absolute() && !self.allowed_lib_roots.iter().any(|root| path.starts_with(root)) {
    return Err(FfiError::SecurityViolation(
        format!("Native library '{}' is outside allowed roots", lib_path)
    ));
}
```

Add `allow_native_libs: bool` and `allowed_lib_roots: Vec<PathBuf>` to the `FfiSecurity` struct. Default both to `false`/empty. Set them only when `--allow-ffi` is passed to the CLI.

---

## Fix 2 — Embedded VM Divergence (Medium, ongoing)

**Root cause:** `ifa-embedded/src/lib.rs` has its own dispatch loop that shares bytecode format with `ifa-core/vm.rs` but not the semantic implementation.

**The fix is not to "merge" the files.** The fix is a **shared conformance test** that runs against both VMs.

Add to `ifa-embedded/tests/`:

```rust
// crates/ifa-embedded/tests/semantic_parity.rs
// Runs the same Tier 1 conformance programs through the embedded VM
// and asserts they produce identical results to ifa-core's IfaVM.
#[test]
fn embedded_arithmetic_matches_core() {
    let source = "pada 2 + 3 * 4;";
    let program = parse(source).unwrap();
    let bytecode = Compiler::new("test").compile(&program).unwrap();

    let core_result = IfaVM::new().execute(&bytecode).unwrap();
    let embedded_result = EmbeddedVm::new().execute(&bytecode.code).unwrap();

    assert_eq!(core_result, embedded_result);
}
```

Run this for every Tier 1 conformance `.ifa` file. Any divergence is a bug in the embedded VM, not the spec.

---

## Fix 8 — Symlink Bypass on `--allow-*` (Medium, verify first)

**Status:** Unverified from code. Standard fix pattern:

Wherever path capability checks occur (likely `ifa-sandbox` or `ifa-cli`), canonicalize the path before checking:

```rust
// Before checking if path is within an allowed root:
let canonical = std::fs::canonicalize(&requested_path)
    .map_err(|_| PermissionDenied("Cannot resolve path".into()))?;

if !allowed_roots.iter().any(|root| canonical.starts_with(root)) {
    return Err(PermissionDenied(format!("Path '{}' is outside allowed roots", canonical.display())));
}
```

`canonicalize()` resolves symlinks. A symlink pointing outside the allowed root will resolve to its real target, which will fail the prefix check.

---

## Priority Order

| Fix | File | Time | Severity |
|---|---|---|---|
| **4** Stack depth guard | `vm.rs:219`, `core.rs:143` | 15 min | **CRITICAL — do first** |
| **5a** Fuel limit | `vm_iroke.rs:20` | 30 min | High |
| **5b** Ikin pool cap | `vm_ikin.rs:43`, `:99` | 10 min | High |
| **7** Bincode limits | `vm.rs:318`, `storage.rs:374` | 20 min | High |
| **3** FFI library-load gate | `ffi.rs` | 45 min | High |
| **6** Registry isolation | `registry.rs`, `vm.rs` | 2 hrs | High |
| **2** Embedded parity tests | `ifa-embedded/tests/` | 1 day | Medium |
| **8** Symlink canonicalize | `ifa-sandbox` or `ifa-cli` | 1 hr | Medium |
