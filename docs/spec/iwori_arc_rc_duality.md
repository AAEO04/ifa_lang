# Ìwòrì: Arc/Rc Duality — Compile-Time Refcount Toggle

**Status:** `DRAFT`  
**Supersedes:** Nothing (new capability)  
**Superseded by:** Phase 3 Opon Slab Allocator (if implemented, this spec becomes unnecessary)  
**Crate:** `ifa-types` (`src/value_union.rs`), `ifa-vm` (`Cargo.toml`)

---

## 1. Purpose

`IfaValue` heap variants (`List`, `Map`, `Str`, `Fn`, `Closure`, `Future`, `Actor`) currently use `Arc<T>` (atomic reference counting) unconditionally. This guarantees thread safety for the actor system (`Osa.ran`), but imposes an atomic `lock xadd` instruction on every `Clone` and `Drop`, even when the program never spawns a concurrent actor.

This spec introduces a compile-time feature flag that switches `Arc` to `Rc` (non-atomic reference counting) for single-threaded deployments, eliminating all atomic overhead.

---

## 2. Mechanism

### 2.1 The Type Alias

In `ifa-types/src/value_union.rs`, introduce a conditional type alias:

```rust
#[cfg(feature = "parallel")]
pub(crate) type IfaRc<T> = std::sync::Arc<T>;

#[cfg(not(feature = "parallel"))]
pub(crate) type IfaRc<T> = std::rc::Rc<T>;

#[cfg(feature = "parallel")]
pub(crate) type IfaMutex<T> = std::sync::Mutex<T>;

#[cfg(not(feature = "parallel"))]
pub(crate) type IfaMutex<T> = std::cell::RefCell<T>;
```

All `Arc<T>` references in `IfaValue` variants are replaced with `IfaRc<T>`. All `Mutex<T>` usages (e.g., `FutureCell`) are replaced with `IfaMutex<T>`.

### 2.2 Affected Variants

| Variant | Current type | `parallel` ON | `parallel` OFF |
|---------|-------------|---------------|----------------|
| `List` | `Arc<Vec<IfaValue>>` | `Arc<Vec<IfaValue>>` | `Rc<Vec<IfaValue>>` |
| `Map` | `Arc<HashMap<...>>` | `Arc<HashMap<...>>` | `Rc<HashMap<...>>` |
| `Str` | `CompactString` (inline or `Arc<str>`) | No change for inline; `Arc<str>` → `Rc<str>` for heap | Same |
| `Fn` | `Arc<BytecodeFnData>` | `Arc<BytecodeFnData>` | `Rc<BytecodeFnData>` |
| `Closure` | `Arc<ClosureData>` | `Arc<ClosureData>` | `Rc<ClosureData>` |
| `Future` | `Arc<Mutex<FutureState>>` | `Arc<Mutex<FutureState>>` | `Rc<RefCell<FutureState>>` |
| `Actor` | `Arc<dyn Any + Send + Sync>` | No change | **Removed entirely** |

### 2.3 Feature Flag Location

In `ifa-types/Cargo.toml`:

```toml
[features]
default = ["vm", "std", "parallel"]
parallel = []
```

The `parallel` feature is ON by default. Embedded and scripting deployments opt out explicitly.

---

## 3. The Ironclad Sandbox

### 3.1 Problem

`Rc<T>` does not implement `Send` or `Sync`. If any `IfaValue` crosses an OS thread boundary while backed by `Rc`, the program triggers undefined behavior. This must be prevented at compile time, not runtime.

### 3.2 Solution: Conditional Compilation Removal

When `feature = "parallel"` is OFF:

1. **`IfaValue::Actor` variant is removed** via `#[cfg(feature = "parallel")]`. The enum variant physically does not exist. Any code referencing `IfaValue::Actor` fails to compile.

2. **`spawn_actor()` is removed** via `#[cfg(feature = "parallel")]` on the function definition in `ifa-vm/src/actor.rs:179`.

3. **`Osa.ran` and `Osa.ise` standard library bindings** are conditionally excluded from the `OduRegistry` when `parallel` is off. Calling `Osa.ran()` in user code produces a runtime error: `"Actor concurrency requires the 'parallel' feature"`.

4. **`OpCode::ParallelFor`** (Rayon-backed `iwori.yipo.ori`) falls through to the sequential fallback (already implemented at `vm.rs:2725-2731`).

5. **`unsafe impl Send for ActorMsg`** at `actor.rs:86` is gated behind `#[cfg(feature = "parallel")]`.

### 3.3 FFI Boundary

FFI bridges via `Aje` (the polyglot bridge) cannot be statically analyzed for thread spawning. When `parallel` is OFF, FFI calls that attempt to pass an `IfaValue` to a foreign thread will fail at the Rust type level because `Rc` does not implement `Send`. This is enforced by the Rust compiler itself, not by Babalawo.

---

## 4. Invariants

1. **No `Rc` crosses a thread boundary**: Guaranteed by Rust's type system (`Rc: !Send`). If `parallel` is off, no API exists to spawn threads from Ifá code.

2. **Feature flag is binary**: There is no mixed mode. The entire VM is either fully `Arc` (multi-threaded capable) or fully `Rc` (single-threaded only).

3. **No behavioral difference**: All Ifá programs that do not use `Osa` (actors) or `iwori.yipo.ori` (parallel for) produce identical results regardless of the `parallel` flag. Only performance characteristics change.

4. **Default is safe**: `parallel` is ON by default. Users must explicitly opt into single-threaded mode.

---

## 5. Relationship to Other Specs

- **Superseded by [opon_slab_allocator.md](opon_slab_allocator.md)**: If Phase 3 is implemented (replacing `Arc`/`Rc` with `u32` slab indices entirely), this spec becomes moot. There would be no reference counting to toggle.
- **Depends on [move_semantics.md](move_semantics.md)**: The `yanda` move system works identically under both `Arc` and `Rc` because `MoveTracker` operates at the AST level, not the type level.
- **Depends on [effect_system.md](effect_system.md)**: Effect declarations are orthogonal to the refcount backend.
