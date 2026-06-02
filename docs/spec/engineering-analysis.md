# Engineering Analysis — 12 Cross-Cutting Concerns

*Ifá-Lang. Sourced from crates/ as of build. Each section: current state → risks → recommendations.*

---

## 1. Incremental Compilation

### Current State

The VM has a per-file content-hash cache (`vm.rs:587-591`, `vm.rs:780-803`):

```
fn hash_source(source: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    hasher.finish()
}
```

`import_module()` checks `module_cache[module_key].hash == source_hash` before recompiling. If the hash matches, the cached `Bytecode` is cloned.

```rust
if let Some(cached) = self.module.module_cache.get(&cache_key) {
    if cached.hash == source_hash {
        cached.bytecode.clone()
    } else { /* recompile */ }
}
```

The `ImportGuard` detects circular imports at runtime (`module_resolver.rs:148`).

### What Is Missing

| Feature | Status | Evidence |
|---------|--------|----------|
| File-level content-hash cache | ✅ Present | `vm.rs:780-803` |
| Dependency graph tracking | ❌ Absent | No `dependent -> dependency` reverse map |
| Transitive invalidation | ❌ Absent | Only direct source-hash compared |
| Serialized compilation artifacts | ❌ Absent | No `.ifab` cache per module |
| Parallel compilation | ❌ Absent | Single-threaded compile loop |
| Watch mode rebuild | ❌ Absent | No file watcher |

### Risks

1. **Stale caches**: If module `A` imports `B`, and `B` changes, `A`'s cached bytecode is not invalidated because there is no dependency edge tracking.
2. **Cold start**: Every `ifa run` re-parses and re-compiles all dependencies. No persistent artifact cache to disk.
3. **No compile graph**: Without a topological order, large projects with many modules will compile serially.

### Recommendation

Build a dependency DAG. Each `CachedModule` should store its `dependencies: Vec<String>`, and on import of `B`, recursively invalidate all transitive dependents. Then add `--watch` mode using `inotify`/`kqueue` that recompiles only dirty subgraphs.

---

## 2. Dependency Hell / Semantic Versioning

### Current State

**Oja package manager** (`ifa-cli/src/oja.rs`) has:

- `SemVer` struct with `major.minor.patch` (`oja.rs:23-27`)
- `VersionConstraint` with variants `Exact`, `Caret` (`^`), `Tilde` (`~`), `Any` (`oja.rs:44-49`)
- `Caret` allows `major.minor.x` where `x ≥ patch` (and `major.minor+1.0` if major = 0)
- `Tilde` allows only patch bumps within same minor
- `mvs_select()` — Minimum Version Selection algorithm (`oja.rs:119-130`)

**Manifest format** (`OjaManifest`): `[package]`, `[project]`, `[dependencies]`, `[dev-dependencies]`, `[workspace]`.

### What Is Missing

| Feature | Status | Evidence |
|---------|--------|----------|
| SemVer parsing | ✅ Basic | `oja.rs:317-319` |
| Caret/Tilde resolution | ✅ Basic | `oja.rs:83-109` |
| MVS selection | ✅ Basic | `oja.rs:119-130` |
| Lockfile | ❌ Absent | No `oja.lock` |
| Yoga/conflict resolution | ❌ Absent | No diamond-dep resolver |
| Workspace member resolution | ❌ Absent | No workspace path resolution |
| Yank/deprecation | ❌ Partial | Prints "yanked" warning only |
| Registry auth | ❌ Absent | No private registry support |
| Checksum verification | ❌ Absent | No SHA/sig verification of packages |

Specific bugs in `satisfies()`:
- Caret for `0.x` only allows `0.x.y` → `0.x+1.0` (not `0.x+1.y`) — missing. Actually looking at the code: for major=0 and minor=0, it allows `<=0.0.(patch+...)` (no minor bump). For major=0, minor>0, it allows `< 0.(minor+1).0`. This is standard.
- The `min_version()` method only returns the base, not the actual minimum satisfying, which can lead to MVS picking a version that doesn't satisfy all constraints (only filters by `satisfies` individually).

### Risks

1. **No lockfile** means `ifa build` today is non-reproducible across machines.
2. **No diamond dependency resolution** — if A needs B v1 and C needs B v2, the resolver may pick incorrectly and silently.
3. **No checksum verification** — man-in-the-middle on the registry could serve malicious packages.

### Recommendation

Immediate: add `oja.lock` with pinned SemVer + content hashes per resolved dependency. Medium: implement a proper PubGrub or SAT-based resolver for diamond dependencies.

---

## 3. Compile-Time Performance

### Current State

The compilation pipeline is:

```
Source text → pest PEG parser → AST (Program) → Compiler → Bytecode
```

No intermediate representation (IR). No optimization passes. No monomorphization.

From `ifa-compiler/src/lib.rs`:

- Single `compile()` function that iterates statements once (`line 115+`)
- String deduplication via `HashMap<String, u16>` for the string table
- No inlining, no constant folding, no dead code elimination

From `ifa-parser/src/parser.rs` (indirectly via `pest`):

- PEG parsing is known O(n³) worst-case, though linear in practice for well-structured grammars
- The grammar file is a single pest grammar

### What Is Missing

| Feature | Status |
|---------|--------|
| IR (intermediate representation) | ❌ Source → bytecode directly |
| Optimization passes | ❌ None |
| Lazy/modular compilation | ❌ Full parse every time |
| Incremental parsing | ❌ Full text re-parsed |
| Parallel compilation | ❌ No rayon/threading |

### Risks

1. **PEG backtracking cost**: Large files with ambiguous grammar constructs cause exponential backtracking. Pest mitigates this with unlimited-length lookahead, but complex expressions still backtrack.
2. **Compile-on-every-run**: No persistent compilation daemon (no `ifa check --watch`).
3. **No optimization means slow bytecode**: The VM spends time interpreting patterns that a compiler could fold.

### Recommendation

Add a lightweight IR layer (`IfaIR`) between AST and bytecode. This enables:
- Constant folding and propagation
- Dead code elimination
- Effect analysis at compile time (needed for Babalawo)
- Future JIT compilation

Then parallelize module compilation with `rayon` (already a feature flag in the workspace).

---

## 4. Async Runtime Complexity

### Current State

Two async systems coexist:

**1. VM-level cooperative tasks** (`vm.rs:166-168`, `task_queue: VecDeque<Task>`):
- `spawn_task()` creates a `Future` value with `Arc<Mutex<FutureState>>` 
- States: `Pending` / `Ready(IfaValue)`
- `await_future()` drives the task to completion (busy-loop)
- `OpCode::Yield` for cooperative yielding

**2. Actor system** (`actor.rs`):
- Each actor is a tokio `spawn_blocking` task (`actor.rs:28-35`)
- Dedicated `tokio::runtime::Runtime` via `OnceLock<Runtime>` (`actor.rs:25`)
- `spawn_actor_task()` checks `Handle::try_current()` first, falls back to dedicated runtime
- Communication: `mpsc::sync_channel` with capacity 64

### Architecture Issue

There is a fundamental tension:

```
VM Future (Arc<Mutex<FutureState>>) — cooperative, single-threaded polling
    ↓
Actor Task (tokio::spawn_blocking)  — OS thread per actor, blocking recv()
```

`spawn_actor` creates an actor with `spawn_blocking` (designed for IO-bound work), but the actor loop is a pure CPU interpreter loop. This means:
- Each actor blocks an OS thread from tokio's blocking thread pool
- The dedicated runtime (`get_actor_runtime()`) uses a multi-thread scheduler for blocking tasks
- Inside `await_future`, the VM busy-loops driving a cooperative task — inside a blocking task this starves the tokio scheduler

### Risks

1. **Blocking thread exhaustion**: `spawn_blocking` has a default max of ~512 threads. Each actor consumes one permanently.
2. **Priority inversion**: The VM's cooperative `Yield` is meaningless inside `spawn_blocking` because tokio cannot preempt it.
3. **Runtime double-initialization**: `get_actor_runtime()` creates a full multi-thread tokio runtime on first call, but `spawn_actor_task` also tries `Handle::try_current()` — if the caller is already in a tokio context, the blocking task runs on the caller's runtime instead.
4. **No async I/O in VM**: The `daro`/`await` keyword only drives futures, it doesn't integrate with tokio's reactor. You cannot `await` a network request without blocking a thread.

### Recommendation

Refactor actors to use `tokio::task::spawn_blocking` only for CPU-bound interpreter work, with a separate `tokio::spawn` for message handling. Better yet: make the VM itself `async fn step()` so that `Yield` truly yields to tokio's scheduler, eliminating the need for `spawn_blocking` at all. This requires `IfaValue::Future` to integrate with `tokio::sync::oneshot` instead of `Arc<Mutex<FutureState>>`.

---

## 5. Soundness Holes

### 5.1 `Arc<Mutex>` as Universal Heap

`IfaValue` heap variants (`value_union.rs:38-94`):

```rust
List(Arc<Vec<IfaValue>>)        — Arc, not Arc<Mutex>
Map(Arc<HashMap<...>>)          — Arc, not Arc<Mutex>
Upvalue(Arc<Mutex<IfaValue>>)   — Arc<Mutex>
Future(Arc<Mutex<FutureState>>) — Arc<Mutex>
Actor{handle: Arc<dyn Any+Send+Sync>}
```

**Problem**: `List` and `Map` use plain `Arc`, which means they are immutable once created. Mutation of lists (via `SetIndex`, `Append`) requires creating a new `Arc`. This is fine for correctness but means:
- `Append` replaces the entire `Arc<Vec>` with a new allocation
- No concurrent mutation, but also no interior mutability for lists

`Upvalue` and `Future` use `Arc<Mutex>`, which is correct but:
- Lock contention on every read/write of captured variables
- Potential for deadlock (though unlikely in single-threaded bytecode)

### 5.2 `unsafe impl Send` for `ActorMsg`

```rust
// actor.rs:86
unsafe impl Send for ActorMsg {}
```

The comment claims safety because `IfaValue` contains only `Arc`-wrapped data. This is *probably* sound for the current set of variants, but:
- It is not enforced by the type system
- Adding a new non-`Send` variant to `IfaValue` (e.g., `Rc<T>`) would silently make `ActorMsg` unsound
- No unit test asserts `IfaValue: Send`

### 5.3 Freeze/Thaw Runtime Failures

`freeze()` returns `Err` for closures, functions, actors (`value_union.rs:414-418`). This means **soundness is enforced at runtime, not at compile time**. A program that sends a closure to an actor compiles successfully and fails at runtime.

### 5.4 Null as First-Class

`IfaValue::Null` with `??` null-coalescing (`ast.rs:532`). No `Option<T>` type. This means:
- Every expression can return `Null`
- No static tracking of nullability
- Null propagation is unchecked

### 5.5 No Type Safety in OduRegistry Dispatch

```
CallOdu: domain.method(args) → runtime dispatch through OduRegistry
```

The return type is `IfaResult<IfaValue>` — there is no compile-time checking that the returned type matches the expected type at the call site.

### 5.6 Unsafe Blocks in Ebo

```rust
// ebo.rs:42-44
unsafe {
    ManuallyDrop::drop(&mut self.cleanup);
}
std::mem::forget(self);
```

This is sound (ManuallyDrop + forget avoids double-free), but the safety invariant relies on the programmer remembering that `dismiss`/`sacrifice` must `forget(self)`. A future refactor that removes `forget` would cause double-free.

### Risk Summary

| Hole | Severity | Detection |
|------|----------|----------|
| `unsafe impl Send` on `ActorMsg` | Medium | Code review; no CI check |
| Freeze failures at runtime | Medium | Test coverage |
| Null as first-class | Low | Common in dynamic languages |
| OduRegistry untyped dispatch | Low | Common in dynamic dispatch |
| Ebo `ManuallyDrop` + `forget` | Low | Well-understood pattern |
| List/Map immutability via Arc | None | Correct but allocation-heavy |

### Recommendations

1. Add `static_assert!` or a `#[test]` that checks `IfaValue: Send + Sync`.
2. Add a `freeze` capability annotation to the Babalawo type system so that sending a non-freezable value to an actor is a compile-time error.
3. Consider adding `Option<T>` as a first-class type alongside `Null`, where `Option<Int> ≠ Int`.

---

## 6. Cross-Platform ABI

### Current State

**`.ifab` binary format** (`ifa-bytecode/src/format.rs:9-80`):

```
[4 bytes magic "IFA\0"]
[2 bytes version LE u16]
[4 bytes instruction_size LE u32]
[4 bytes constant_size LE u32]
[1 byte opon_size u8]
= 15 bytes header
```

- Little-endian encoding throughout
- Version 3 (compatible with v2 per `validate()` line 46)
- No platform-specific definitions
- `no_std` + `forbid(unsafe_code)` in `ifa-bytecode`

**Serialization** (`vm.rs:458-481`): VM state snapshot uses `bincode::serialize` with the bytecode hash for integrity checking. Snapshot has a 64 MB limit (`MAX_SNAPSHOT_BYTES`).

### What Is Missing

| Feature | Status |
|---------|--------|
| Fixed binary header | ✅ 15 bytes, LE |
| Version compatibility | ✅ v2 and v3 accepted |
| no_std | ✅ Core crate |
| Pointer size portability | ❌ `usize` in string indices may differ on 32-bit targets |
| Endian portability | ❌ Assumes LE host |
| Float binary representation | ❌ No NaN boxing in serialized form (Snapshots serialize full IfaValue) |
| Alignment guarantees | ❌ No explicit padding in binary layout |
| Section/segment structure | ❌ Flat binary, no section table |

### Risks

1. **`usize` width**: The bytecode format stores string indices as `u16` (2 bytes), which is fine. But `InstructionCache` in `vm_ikin.rs` uses `u32` for string map values — safe on both 32-bit and 64-bit. The snapshot format uses `bincode` which handles `usize` platform-dependently — **snapshots are not portable between 32-bit and 64-bit hosts**.
2. **Endianness**: `.ifab` files are hardcoded to little-endian. Running on a big-endian host (e.g., certain embedded targets) would require swapping. The `ESP32` is LE, `STM32` is LE, `RP2040` is LE — so this is de-facto safe for current targets, but not architecturally clean.
3. **NaN boxing**: The NaN-boxing path (`nan_box.rs`) embeds pointer tags in the NaN space of IEEE 754 doubles. This is inherently endian-sensitive and also assumes `f64` is 8 bytes (true everywhere modern).

### Recommendation

Add `#![cfg(target_endian = "little")]` assertions or endian-swapping in the bytecode loader. Document that `.ifab` is LE-only. For `bincode` snapshots, add a platform fingerprint (endianness + pointer width) to the snapshot header so cross-platform loading errors are caught early with a clear message.

---

## 7. Deterministic Builds

### Current State

| Component | Deterministic? | Evidence |
|-----------|---------------|----------|
| Bytecode compilation | Partial | Source-hash-based cache, but no canonical ordering guarantees |
| String dedup | No | `HashMap<String, u16>` insertion order is non-deterministic (randomized hash) |
| Global name resolution | No | `HashMap`-backed `global_names_index` |
| Module resolution | Yes | `ModuleResolver` iterates `search_paths` in insertion order |
| `.ifab` serialization | Unknown | `bincode` with `IfaValueSurrogate` — depends on `HashMap` iteration for `IfaValue::Map` |
| Package resolution | No | No lockfile, MVS depends on registry query order |

**Root cause**: Rust's `HashMap` uses a randomized `SipHash` by default. Any data structure that depends on `HashMap` iteration order produces non-deterministic output across runs.

### Specific Non-Determinism Sites

| Site | Effect |
|------|--------|
| `vm_ikin.rs:28` `string_map: HashMap<Arc<str>, u32>` | String interning order changes between runs |
| `vm.rs:140` `global_names_index: HashMap<String, usize>` | Global slot assignment order |
| `compiler/src/lib.rs` string table dedup | String constant indexes change |
| `IfaValueSurrogate::List` | List contents are ordered (Vec), but Map entries are not |

### Risks

1. **Reproducible builds impossible** — `ifa build` produces different `.ifab` each run.
2. **Cache misses** — non-deterministic hashes in module cache may cause false mismatches (unlikely but possible with hash collision).
3. **Debugging difficulty** — stack traces and instruction addresses change between runs.

### Recommendation

Replace `HashMap` with `IndexMap` (from `indexmap` crate) in all compiler paths where iteration order affects output. Add a `#[cfg(test)]` test that compiles the same source twice and asserts byte equality of output. For caching, use a content-defined hash (BLAKE3) instead of `DefaultHasher`.

---

## 8. Unicode

### Current State

**`CompactString`** (`ifa-types/src/compact_str.rs`):

- Two representations: inline (up to 15 bytes + 1 byte `char_len`) and heap (`Arc<str>` for longer)
- `char_len()` method returns character (code point) count
- For inline strings: precomputed `char_len` at construction time
- For heap strings: `arc.chars().count()` — O(n) scan each call

**`unicode_string_len`** (`value_union.rs:142-147`):

```rust
pub fn unicode_string_len(s: &str) -> usize {
    if s.is_ascii() { return s.len(); }
    s.chars().count()
}
```

This counts Unicode **scalar values** (code points), not **grapheme clusters**. The test confirms:
```rust
assert_eq!(IfaValue::unicode_string_len("e\u{301}"), 2);
// "é" as e + combining accent = 2 code points, 1 grapheme cluster
```

### What Is Missing

| Feature | Status |
|---------|--------|
| Code point counting | ✅ |
| Grapheme cluster segmentation | ❌ |
| Unicode normalization (NFC/NFD) | ❌ |
| Case folding for comparison | ❌ (uses `==` on CompactString) |
| Unicode-aware identifier validation | ❌ (pest grammar may accept non-ASCII) |
| Custom `Eq` for grapheme-aware comparison | ❌ |

### Risks

1. **Incorrect string length**: `unicode_string_len("e\u{301}") == 2` but a user expects `1` (one visible character). This matters for UI layout, text truncation, and display.
2. **Non-normalized comparison**: `"é"` (precomposed U+00E9) ≠ `"e\u{301}"` (decomposed) even though they render identically. This causes lookup failures in maps.
3. **O(n) scan for heap strings**: Every `char_len()` call on a heap `CompactString` scans the entire string. Large strings (e.g., file contents) trigger O(n) per operation.

### Recommendation

Add a `unicode-normalization` dependency to normalize all `CompactString` values to NFC on construction. Cache grapheme cluster count (not just code point count) if the grapheme feature is needed. For heap strings, cache the length in a second `AtomicU32` field.

---

## 9. Floating Point Semantics

### Current State

`IfaValue` stores `f64` directly (`value_union.rs:43`).

**Equality comparison** (`value_union.rs:358`):
```rust
(IfaValue::Float(a), IfaValue::Float(b)) => (a - b).abs() < f64::EPSILON,
```

**Partial ordering** (`value_union.rs:434-458`):
```rust
(IfaValue::Float(a), IfaValue::Float(b)) => a.partial_cmp(b),
```

**Int ↔ Float comparisons**:
```rust
(IfaValue::Int(a), IfaValue::Float(b)) => {
    let a_f64 = *a as f64;
    if a_f64 as i64 == *a { a_f64.partial_cmp(b) }
    else { None }
}
```

**NaN boxing** (`nan_box.rs`): NaN space of IEEE 754 is repurposed for tagged pointers.

### Risks

1. **`f64::EPSILON` comparison is wrong** (`value_union.rs:358`). `EPSILON` is the spacing between 1.0 and the next representable `f64`. For numbers much larger than 1.0, `(a - b).abs() < f64::EPSILON` will never be true even for identically computed values. Standard approach: use `ulp`-based comparison or a relative+absolute tolerance.

2. **NaN behavior undefined**: `f64::NAN` exists as a valid `IfaValue::Float(f64::NAN)`, but:
   - `is_equal(NaN, NaN)` returns `false` because `(NaN - NaN).abs() < EPSILON` is false — actually `NaN - NaN = NaN`, so this is `NaN < EPSILON = false`. This is consistent with IEEE 754, but surprising to users.
   - `partial_cmp` returns `None` for NaN — correct per IEEE 754, but the VM might not handle `None` gracefully in `If` conditions.
   - `to_nan_boxed_primitive()` may misinterpret NaN-bit patterns as tagged pointers.

3. **NaN boxing conflict**: The NaN-boxing system (`nan_box.rs`) relies on quiet NaN values. If a user program produces `f64::NAN` via `0.0 / 0.0`, the NaN-boxing path could misinterpret it as a tagged pointer. The current code guards against this (`TAG_FLOAT_NAN = 0x0005` is checked separately), but the interaction is fragile.

### Recommendation

Replace `EPSILON` comparison with a relative-error comparator:
```rust
fn float_eq(a: f64, b: f64) -> bool {
    if a == b { return true; }
    let diff = (a - b).abs();
    diff <= f64::EPSILON || diff <= f64::max(a.abs(), b.abs()) * f64::EPSILON
}
```
Add explicit NaN handling: `FloatEq` should return `false` when either operand is NaN (consistent with IEEE 754, but document it). For NaN boxing, add a check that the entire NaN-boxing path is bypassed when the operand is a true arithmetic NaN (not a tagged pointer).

---

## 10. Undefined Behavior

### Current State

**`#![forbid(unsafe_code)]`** in `ifa-bytecode/src/lib.rs:15` and `ifa-babalawo/src/lib.rs:6`.

**`unsafe` blocks in the workspace**:

| Location | Block | Purpose |
|----------|-------|---------|
| `ebo.rs:42-44` | `ManuallyDrop::drop` + `mem::forget` | Ebo guard dismiss |
| `ebo.rs:53` | `ManuallyDrop::take` | Ebo guard sacrifice |
| `ebo.rs:65` | `ManuallyDrop::take` | Ebo drop |
| `actor.rs:86` | `unsafe impl Send` | ActorMsg thread safety |
| `value_union.rs` | `unsafe impl Send` (implied by Arc) | IfaValue variants |
| Various FFI | Potentially in `ifa-std/src/ffi.rs` | Native library loading |

**Known UB-adjacent patterns**:

1. **`ManuallyDrop::take` in Drop**: `ebo.rs:65` calls `ManuallyDrop::take` inside `Drop::drop`. This is actually safe because `Drop` is called exactly once, and `ManuallyDrop::take` is designed for this use case. However, if `dismiss` or `sacrifice` has already taken the inner value, `self` would have been `mem::forget`-ten, so `Drop::drop` never fires. This relies on `forget` being correct — a safe invariant that is not enforced by the type system.

2. **`unsafe impl Send` for `ActorMsg`**: `actor.rs:86`. The comment says "IfaValue contains only Arc-wrapped heap data and scalars." This is *almost* true, but `Future(Arc<Mutex<FutureState>>)` contains `FutureState` which is an enum — all its variants are Send. This is sound today but fragile.

3. **Type-erased `dyn Any` for ActorHandle**: `value_union.rs:75` — `handle: Arc<dyn Any + Send + Sync>`. The `actor_send` function downcasts with `downcast_ref::<Arc<ActorHandle>>`. If a different `Arc<T>` is stored there, the downcast fails with a runtime error (not UB). This is safe.

4. **NaN boxing bit manipulation**: `nan_box.rs` reads/writes `u64` representations of `f64` values. This is technically implementation-defined behavior in the Rust abstract machine, but in practice:
   - All modern Rust targets use IEEE 754 with the same representation
   - The code uses `f64::to_bits()` / `f64::from_bits()` which are safe and portable
   - No actual UB, but the semantics are platform-dependent

### No-UB Guarantees

| Property | Status |
|----------|--------|
| No raw pointer dereference in user code | ✅ (VM does internally via safe abstractions) |
| No uninitialized memory | ✅ (Vec::resize with IfaValue::Null) |
| No buffer overflow in Opon | ✅ (bounds checks in get/try_set) |
| No use-after-free in Opon | ✅ (all values are Arc-owned, Opon owns Vec<IfaValue>) |
| No stack overflow VM crash | ✅ (configurable stack_limit, default 4096) |
| No double-free in Ebo | ✅ (ManuallyDrop + forget pattern) |

### Recommendation

Add `#![forbid(unsafe_code)]` to as many crates as possible (currently only `ifa-bytecode` and `ifa-babalawo`). Create an `unsafe` audit document listing every `unsafe` block, its justification, and the invariants that must hold. Add a CI step that denies new `unsafe` without explicit review approval.

---

## 11. Memory Aliasing

### Current State

The language has **no borrow checker**. Memory aliasing is managed through:

1. **Immutable `Arc` sharing**: `List(Arc<Vec<IfaValue>>)` and `Map(Arc<HashMap<...>>)` are read-only after creation. Mutation creates a new `Arc`.
2. **`Arc<Mutex>` interior mutability**: `Upvalue(Arc<Mutex<IfaValue>>)` allows mutation behind a lock.
3. **Deep copy on actor send**: `freeze()` + `thaw()` creates independent copies.
4. **Opon region ownership**: The Opon owns its `Vec<IfaValue>`. All references are by address index, not by pointer.

**Type-level aliasing hints** (`ast.rs:329-335`):
```rust
Ptr(Box<TypeHint>),   // *T — unsafe pointer
Ref(Box<TypeHint>),    // &T — immutable reference
RefMut(Box<TypeHint>), // &mut T — mutable reference
```

These exist as type hints but are **not enforced by the compiler**. They serve as documentation and FFI markers only.

### Aliasing Scenarios

| Scenario | Aliasing? | Safety |
|----------|-----------|--------|
| Two variables referencing same `List` | Yes — `Arc` pointer aliasing | Immutable, safe |
| Upvalue captured by closure | Yes — `Arc<Mutex>` shared | Mutex ensures mutual exclusion |
| Cross-actor shared value | No — deep copied via freeze/thaw | Full isolation |
| Opon slot shared across functions | No — copied out of Opon (clone on read) | Safe |
| List mutation via `SetIndex` | No — creates new `Arc<Vec>` | Allocation-heavy but safe |

### Risks

1. **No alias analysis for optimization**: Without alias tracking, the VM cannot:
   - Reorder memory operations
   - Eliminate redundant loads
   - Vectorize list operations
2. **Unenforced `RefMut`**: A program can declare `&mut T` and then alias it, violating the "no aliased mutable references" invariant.
3. **Arc refcount overhead**: Every capture, every clone, every list access atomically increments/decrements refcounts.

### Recommendation

Add a **move tracking** phase in Babalawo (already partially present in `movement.rs` with `MoveTracker`). Extend it so that:
- `RefMut(T)` variables are tracked linearly (cannot be copied, only moved)
- Cross-function borrows are validated at call sites
- The tracker emits warnings for unnecessary `Arc` clones

For the VM, add an **alias analysis pass** on the bytecode level that identifies when two stack values reference the same `Arc`. This enables load elimination and list operation optimization.

---

## 12. Error Message Quality

### Current State

**Babalawo** (`ifa-babalawo/src/diagnose.rs`) is the error diagnosis system:

**Severity levels** (`diagnose.rs:10-16`):
- `Error` — "Aṣiṣe — must fix"
- `Warning` — "Ìkìlọ̀ — should fix"  
- `Info` — "Ìmọ̀ràn — suggestion"
- `Style` — "Style recommendation"

**Output formats** (`diagnose.rs`):

| Format | Method | Use case |
|--------|--------|----------|
| Default | `format()` | Terminal: `error[Ogbè] file.ifa:10:5` + message + wisdom |
| Source-annotated | `format_with_source(source)` | Full context with code listing, carets (`^^^`), line numbers, ANSI colors, notes, wisdom |
| Compact | `format_compact()` | One-liner for IDE integration: `file:10:5: error: message` |
| JSON | `format_json()` | IDE integration: `{"severity":"error","code":"UNDEFINED_VARIABLE","line":10,...}` |

**Structure per diagnostic**:
```rust
Diagnostic {
    severity: Severity,
    error: LintError{code, message, file, line, column, span, context, notes},
    odu: "OGBE",       // mapped from error code
    wisdom: Option<String>,  // Yoruba proverb
}
```

**Wisdom mapping** (`wisdom.rs`): Each error code maps to an Odu, and each Odu has a wisdom proverb. For example:
- `UNDEFINED_VARIABLE` → Ògbè → "The beginning of wisdom is knowing what you do not know"
- Disabled in `fast()` mode for performance

**LSP Integration** (`ifa-cli/src/lsp.rs`): `publish_diagnostics` sends `Vec<Diagnostic>` to the client.

### What Is Missing

| Feature | Status |
|---------|--------|
| Source-annotated output | ✅ |
| Multi-line context | ✅ |
| Caret pointing | ✅ |
| Severity levels | ✅ |
| Error codes | ✅ |
| Odu/wisdom integration | ✅ |
| JSON output | ✅ |
| Compact output | ✅ |
| Primary/secondary labels | ❌ (future Rust-style `help:` labels) |
| Multi-span errors | ❌ (each error has one span) |
| Suggestions/snippets | ❌ (e.g., "did you mean `obara`?") |
| Color customization | ❌ (always ANSI) |
| Error code reference | ❌ (no `--explain` command) |
| Fix applicability | ❌ (no `#[allow]` or auto-fix) |

### Specific Quality Issues

1. **Wisdom can be distracting**: Proverb per error is unique, but when there are 20 errors, the user sees 20 proverbs. The `fast()` mode disables wisdom, but there's no per-diagnostic filtering.

2. **No multi-span support**: Consider `if ayanmo x = f(x)` — the error should point to BOTH `x` (used) and `x` (defined-in-statement). Today only one span is reported.

3. **No suggested fix**: "Variable `x` not defined" is correct but unhelpful. "Variable `x` not defined. Did you mean `y`? It is defined at line 42" is better. The Babalawo has the symbol table (`LintContext`) but doesn't generate suggestions.

4. **No `--explain`**: Rust's `--explain E0308` is a powerful learning tool. Ifá-Lang error codes like `UNDEFINED_VARIABLE` could have long-form explanations.

### Recommendation

Add a `--explain <ERROR_CODE>` command that prints the full wisdom text, Odu context, and examples. For the source formatter, add multi-span support (each `Diagnostic` gets `primary_span` + `Vec<secondary_span>`) and a suggestion field. For the Babalawo, add a "did you mean?" check using Levenshtein distance against the scope's defined variables.

---

## Summary Risk Matrix

| Topic | Risk Level | Impact | Effort to Fix |
|-------|-----------|--------|---------------|
| Incremental Compilation | High | Developer productivity | Medium |
| Dependency Hell | High | Build reproducibility | Medium |
| Compile-Time Performance | Medium | Developer productivity | Low (add IR) |
| Async Runtime Complexity | High | Correctness at scale | High |
| Soundness Holes | Medium | Long-term correctness | Medium |
| Cross-Platform ABI | Low | Portability | Low |
| Deterministic Builds | Medium | Reproducibility | Low |
| Unicode | Low | Correctness | Low |
| Floating Point Semantics | Medium | Numerical correctness | Low |
| Undefined Behavior | Low | Safety guarantee | Low |
| Memory Aliasing | Medium | Optimization | Medium |
| Error Message Quality | Low | Developer experience | Low |

*Specification v0.1. All claims reference specific code locations in the crates/* directory.
For questions of priority, the source code is the definitive reference.*
