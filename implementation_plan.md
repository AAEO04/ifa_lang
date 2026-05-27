# Ifá-Lang Memory Safety Fixes

Three confirmed bugs, in execution order. Skipping the ones that need a design
decision (UpvalueCell Weak migration, module LRU) until the trivially-correct
fixes are in.

---

## Fix 1 — FFI Dangling Pointer (UB, ship-blocking)

**Root cause**: [ffi.rs:826](file:///c:/Users/allio/Desktop/ifa_lang/crates/ifa-std/src/ffi.rs#L826)

```rust
ffi_args.push(Arg::new(&str_args[str_idx].as_ptr()));
//                      ^^^^^^^^^^^^^^^^^^^^^^^^^
// as_ptr() → *const c_char (temporary rvalue on the stack)
// & takes its address → dead after the semicolon
// Arg::new stores that dead pointer as *const c_void
// by call() time at line 839, it is dangling — UB
```

The SAFETY comment at line 836 claiming "arguments stored in stable locals" is
incorrect. `str_args: Vec<CString>` IS stable. The pointer VALUES derived from it
are not collected before building `Arg` references.

### Change: [ffi.rs](file:///c:/Users/allio/Desktop/ifa_lang/crates/ifa-std/src/ffi.rs) — `call_native_libffi`

After the first loop that populates `str_args` (lines 782–801), collect all
`*const c_char` pointers into a stable `Vec` **before** the second loop that
builds `Arg` references:

```rust
// After str_args is fully populated — pointers are now stable:
let str_ptrs: Vec<*const std::os::raw::c_char> =
    str_args.iter().map(|s| s.as_ptr()).collect();
```

Then in the `Arg`-building loop at line 826, change:

```diff
- ffi_args.push(Arg::new(&str_args[str_idx].as_ptr()));
+ ffi_args.push(Arg::new(&str_ptrs[str_idx]));
```

`str_ptrs` lives as long as `ffi_args` (same scope, before the `unsafe` block).
`str_args` keeps the `CString` allocations alive. The pointer stored in
`str_ptrs[i]` is stable for the entire call.

Update the SAFETY comment to accurately describe this invariant.

**No new dependencies. Two lines changed. Zero behavioral difference.**

---

## Fix 2 — OwnedStr Return Never Freed (leak per FFI call)

**Root cause**: [ffi.rs:875–883](file:///c:/Users/allio/Desktop/ifa_lang/crates/ifa-std/src/ffi.rs#L875-L883)

```rust
IfaType::OwnedStr => {
    let result: *mut std::os::raw::c_char = call(code_ptr, ffi_args.as_slice());
    if result.is_null() {
        Ok(FfiValue::Null)
    } else {
        let c_str = std::ffi::CStr::from_ptr(result);
        let owned = c_str.to_string_lossy().into_owned();
        Ok(FfiValue::Str(owned))  // ← result pointer leaked here
    }
}
```

### The allocator problem

We cannot blindly call `libc::free`. The `result` pointer was allocated by the
foreign function — it could be `malloc`, `CoTaskMemAlloc`, a custom arena, or a
Rust `Box`. Calling the wrong deallocator is UB.

The correct fix has two parts:

**Part A — Add a `free_fn` field to `FfiSignature`**

```rust
#[derive(Debug, Clone)]
pub struct FfiSignature {
    pub arg_types: Vec<IfaType>,
    pub ret_type: IfaType,
    /// Deallocator for OwnedStr returns. If None, the caller owns the pointer
    /// (e.g. static string) and it must not be freed.
    /// Most C libraries that return malloc'd strings: set to `libc::free`.
    pub owned_str_free: Option<unsafe extern "C" fn(*mut std::os::raw::c_void)>,
}
```

**Part B — Use it in the OwnedStr match arm**

```rust
IfaType::OwnedStr => {
    let result: *mut std::os::raw::c_char = call(code_ptr, ffi_args.as_slice());
    if result.is_null() {
        Ok(FfiValue::Null)
    } else {
        let c_str = std::ffi::CStr::from_ptr(result);
        let owned = c_str.to_string_lossy().into_owned();
        // Free if a deallocator was registered for this function.
        // SAFETY: caller guaranteed free_fn is compatible with how the C
        // library allocated this pointer.
        if let Some(free_fn) = bound.sig.owned_str_free {
            free_fn(result as *mut std::os::raw::c_void);
        }
        Ok(FfiValue::Str(owned))
    }
}
```

**Part C — Update `bind()` to accept an optional free fn**

The `bind()` method at [ffi.rs:607](file:///c:/Users/allio/Desktop/ifa_lang/crates/ifa-std/src/ffi.rs#L607)
builds `FfiSignature`. It needs a new parameter or a builder method. Simplest:

```rust
pub(crate) fn bind_with_free(
    &mut self,
    lib: &str,
    func: &str,
    args: &[&str],
    ret: &str,
    free_fn: Option<unsafe extern "C" fn(*mut c_void)>,
) -> FfiResult<()>
```

The existing `bind()` becomes a forwarding shim with `free_fn: None`.

> [!IMPORTANT]
> For common C libraries (glibc, musl, MSVCRT), the correct `free_fn` is
> `libc::free`. Add `libc` as an **optional** dependency to
> `ifa-std/Cargo.toml` under `[features] libc_free = ["libc"]` so existing
> builds without it are unaffected.

---

## Fix 3 — Signal Subscriber Unbounded Growth + Broken Unsubscribe Design

**Root cause**: [ajose.rs:84–85](file:///c:/Users/allio/Desktop/ifa_lang/crates/ifa-vm/src/ajose.rs#L84-L85)

```rust
pub fn subscribe(&self, callback: impl Fn(&T) + Send + Sync + 'static) {
    self.subscribers.write().unwrap().push(Box::new(callback));
}
// no unsubscribe, no handle, grows forever
```

### The index-based removal problem

The naive `SubscriptionGuard` that stores a Vec index and calls `swap_remove(idx)`
on drop is broken under concurrent unsubscribes: dropping guard A at index 3
`swap_remove`s index 3 (moving the last element there), then guard B which holds
index 4 is now stale — it will remove the wrong subscriber or panic.

### Correct approach: ID-based slot map

**Part A — Change `Subscribers<T>` to a slot map**

```rust
// Replace:
type Subscribers<T> = Arc<RwLock<Vec<Box<dyn Fn(&T) + Send + Sync>>>>;

// With:
type SubscriberId = u64;
type Subscribers<T> = Arc<RwLock<HashMap<SubscriberId, Box<dyn Fn(&T) + Send + Sync>>>>;
```

**Part B — Add a per-Signal atomic counter for IDs**

`Signal<T>` already has `version: Arc<AtomicU64>`. Add:

```rust
next_sub_id: Arc<AtomicU64>,
```

**Part C — Change `subscribe` to return a guard**

```rust
pub fn subscribe(
    &self,
    callback: impl Fn(&T) + Send + Sync + 'static,
) -> SubscriptionGuard<T> {
    let id = self.next_sub_id.fetch_add(1, Ordering::Relaxed);
    self.subscribers.write().unwrap().insert(id, Box::new(callback));
    SubscriptionGuard {
        subscribers: Arc::clone(&self.subscribers),
        id,
    }
}
```

**Part D — `SubscriptionGuard` with correct Drop**

```rust
pub struct SubscriptionGuard<T> {
    subscribers: Arc<RwLock<HashMap<u64, Box<dyn Fn(&T) + Send + Sync>>>>,
    id: u64,
}

impl<T> Drop for SubscriptionGuard<T> {
    fn drop(&mut self) {
        // HashMap::remove by key — correct regardless of other concurrent drops
        self.subscribers.write().unwrap().remove(&self.id);
    }
}
```

**Part E — Update `notify` to iterate the map**

```rust
fn notify(&self) {
    let value = self.value.read().unwrap();
    for sub in self.subscribers.read().unwrap().values() {
        sub(&value);
    }
}
```

> [!NOTE]
> The `bind!` macro at [ajose.rs:294–308](file:///c:/Users/allio/Desktop/ifa_lang/crates/ifa-vm/src/ajose.rs#L294-L308)
> calls `source.subscribe(...)` and discards the return value — the guard is
> immediately dropped, unsubscribing instantly. Macro callers must bind the
> guard to a variable. Update the macro to return the guard.

---

## Fix 4 — UpvalueCell Arc Cycle via Weak Migration (requires design care)

**Root cause**: [value_union.rs:106,113](file:///c:/Users/allio/Desktop/ifa_lang/crates/ifa-types/src/value_union.rs#L106-L113) +
[vm.rs:1296–1298](file:///c:/Users/allio/Desktop/ifa_lang/crates/ifa-vm/src/vm.rs#L1296-L1298)

The cycle:
```
UpvalueCell (Arc<Mutex<IfaValue>>)
  └→ IfaValue::Closure(Arc<ClosureData>)
       └→ env: Arc<Vec<UpvalueCell>>
            └→ same UpvalueCell  ← refcount never hits 0
```

Triggered by `StoreUpvalue` at [vm.rs:1280–1298](file:///c:/Users/allio/Desktop/ifa_lang/crates/ifa-vm/src/vm.rs#L1280-L1298) and `StoreLocal`
at [vm.rs:1321–1336](file:///c:/Users/allio/Desktop/ifa_lang/crates/ifa-vm/src/vm.rs#L1321-L1336) when a `Closure` is written into a cell it captured.

> [!WARNING]
> Do NOT add a cycle check in `StoreLocal`/`StoreUpvalue` that returns
> `Err` — this breaks legitimate code patterns and only catches the direct
> self-referential case, not transitive cycles.

### Structural fix: `WeakUpvalueCell` in `ClosureData.env`

**Part A — New type in `value_union.rs`**

```rust
/// Strong cell: the stack slot owns the value.
pub type UpvalueCell = Arc<Mutex<IfaValue>>;

/// Weak reference from a closure's captured env back to a stack cell.
/// Prevents UpvalueCell ↔ Closure cycles. Upgraded on access; if the
/// cell is dead (stack frame popped and no other owner), returns IfaValue::Null.
#[cfg(feature = "vm")]
pub type WeakUpvalueCell = std::sync::Weak<Mutex<IfaValue>>;
```

**Part B — Change `ClosureData.env`**

```diff
pub struct ClosureData {
    pub fn_data: Arc<BytecodeFnData>,
-   pub env: Arc<Vec<UpvalueCell>>,
+   pub env: Arc<Vec<WeakUpvalueCell>>,
}
```

**Part C — `MakeClosure` at [vm.rs:1403–1469](file:///c:/Users/allio/Desktop/ifa_lang/crates/ifa-vm/src/vm.rs#L1403-L1469)**

When building `env` in `MakeClosure`, downgrade each captured cell:

```diff
-   env.push(cell);
+   env.push(Arc::downgrade(&cell));
```

The `UpvalueCell` (`Arc<Mutex<IfaValue>>`) itself stays on the stack slot
(`self.ctx.stack[slot_index] = IfaValue::Upvalue(cell.clone())`). The closure
holds only a `Weak`. No cycle possible.

**Part D — `LoadUpvalue` at [vm.rs:1259–1278](file:///c:/Users/allio/Desktop/ifa_lang/crates/ifa-vm/src/vm.rs#L1259-L1278)**

Upgrade the Weak on each access:

```rust
OpCode::LoadUpvalue => {
    let slot = self.read_u16(bytecode)? as usize;
    let env = self.ctx.frames.last()
        .and_then(|f| f.closure_env.clone())
        .ok_or_else(|| IfaError::Runtime("No closure environment".into()))?;

    let weak = env.get(slot).cloned()
        .ok_or_else(|| IfaError::UndefinedVariable(format!("<upvalue:{}>", slot)))?;

    // Upgrade: if the originating stack frame is gone, the cell is dead.
    let cell = weak.upgrade()
        .ok_or_else(|| IfaError::Runtime(
            format!("Upvalue <{}> accessed after its stack frame was popped", slot)
        ))?;

    let value = cell.try_lock()
        .map_err(|_| IfaError::Runtime("Upvalue lock failed".into()))?
        .clone();
    self.push(value)?;
}
```

**Part E — `StoreUpvalue` at [vm.rs:1280–1298](file:///c:/Users/allio/Desktop/ifa_lang/crates/ifa-vm/src/vm.rs#L1280-L1298)**

Same upgrade pattern before the write.

**Part F — `CallFrame.closure_env` type change**

```diff
- pub closure_env: Option<Arc<Vec<UpvalueCell>>>,
+ pub closure_env: Option<Arc<Vec<WeakUpvalueCell>>>,
```

Update all construction sites: [vm.rs:36](file:///c:/Users/allio/Desktop/ifa_lang/crates/ifa-vm/src/vm.rs#L36),
[vm.rs:45](file:///c:/Users/allio/Desktop/ifa_lang/crates/ifa-vm/src/vm.rs#L45),
[vm.rs:1418](file:///c:/Users/allio/Desktop/ifa_lang/crates/ifa-vm/src/vm.rs#L1418),
[vm.rs:2456](file:///c:/Users/allio/Desktop/ifa_lang/crates/ifa-vm/src/vm.rs#L2456).

> [!CAUTION]
> `CallFrame` derives `Serialize`/`Deserialize` for VM snapshots.
> `Weak<T>` is not serializable. `closure_env` must be `#[serde(skip)]` — which
> it already isn't, so existing snapshots that include frame data will break.
> This needs a snapshot version bump and a migration note in the changelog.

---

## Explicitly Out of Scope (this PR)

| Issue | Reason deferred |
|-------|----------------|
| Module cache LRU | Fake random-eviction LRU is worse than no LRU. Needs an actual `lru::LruCache` swap or a principled size limit with a proper eviction policy. Separate PR. |
| Ìtọ̀jú cycle collector | The locking story (Traceable traversal while holding UpvalueCell mutexes) is unsolved. Needs a design revision that accounts for the concurrent type system before any code is written. |
| `GlobalState` unbounded Vec | Not a realistic leak for any workload shorter than a running server. Address alongside module cache. |
| `IfaFfi` backend unload | `Backend::Native(lib)` already drops `libloading::Library` (calls `dlclose`) when removed from the HashMap. The only real issue is no API surface to remove; add `remove_backend` in a separate PR. |

---

## Implementation Order

```
Fix 1 (ffi.rs:826)         — 1 day, no deps, pure correctness
Fix 2 (OwnedStr free_fn)   — 1 day, no deps, add libc optional dep
Fix 3 (SubscriptionGuard)  — 1–2 days, ajose.rs only, update bind! macro
Fix 4 (Weak migration)     — 2–3 days, touches value_union + vm, snapshot compat
```

---

## Verification Plan

### Fix 1 — FFI dangling pointer
- `cargo test -p ifa-std --features native_ffi` (FFI conformance suite)
- Add a unit test that calls a C function taking a `const char*` via `OwnedStr` and asserts the string arrives correctly.
- Run under Miri: `cargo +nightly miri test -p ifa-std` — Miri will catch the dangling reference in the old code.

### Fix 2 — OwnedStr free
- Write a test C shim that returns `strdup("hello")` as `OwnedStr`, registers `libc::free` as the `free_fn`, calls via FFI, and then verifies (via Valgrind or LSAN) no heap leak.
- Verify that `free_fn: None` (static string return) does not crash.

### Fix 3 — SubscriptionGuard
- Existing `test_signal_subscribe` at [ajose.rs:553](file:///c:/Users/allio/Desktop/ifa_lang/crates/ifa-vm/src/ajose.rs#L553) must continue to pass with the guard held in scope.
- Add test: subscribe 3 callbacks, drop the second guard, fire signal, assert only callbacks 1 and 3 ran.
- Add test: subscribe and immediately drop (returns `_`); fire signal; assert callback did not run.

### Fix 4 — Weak upvalue
- Existing closure conformance tests in `ifa-vm/tests/` must pass unchanged.
- Add a regression test for the specific cycle: `let x = 0; let f = fn() { x }; x = f` — must not leak (verify via a custom allocator count or `Arc::strong_count` assertion after VM teardown).
- Snapshot round-trip test: snapshot before fix 4, attempt resume after — must fail with a clear version mismatch error, not a silent corruption.
- `cargo +nightly miri test -p ifa-vm` — Miri will report any use-after-free from a failed Weak upgrade.
