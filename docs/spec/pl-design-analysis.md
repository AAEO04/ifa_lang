# PL Design Analysis — Ifá-Lang Architecture Deep Dive

*Sourced from crates/. Each section: current state → gaps → recommendation.*

---

## 1. Generics / Parametric Polymorphism

### Current State

Ifá-Lang has **no generics**. The `TypeHint` enum (`ifa-types/src/ast.rs:285-343`) is entirely monomorphic:

```rust
pub enum TypeHint {
    Int, Float, Str, Bool,
    List, Map, Any,
    Function { params: Vec<TypeHint>, ret: Box<TypeHint>, effects: Vec<Effect> },
    Custom(String),                 // user-defined type name, no parameters
    I8..U64, F32, F64,             // sized primitives
    Ptr(Box<TypeHint>),             // only pointer that can nest
    Ref(Box<TypeHint>), RefMut(Box<TypeHint>),
    Array { element: Box<TypeHint>, size: usize },
    Void,
}
```

Key observations:
- `Custom(String)` is the sole mechanism for user-defined types — unparameterized.
- `List` and `Map` have **no element type parameter** (`List<T>` does not exist).
- `Array` has a fixed `size: usize` but its element type can be any single `TypeHint`.
- `Function` has parameter/return types but cannot be generic over them.
- The Babalawo's `infer_expression_type` (`inference.rs`) only resolves concrete types.

### Gap

Every List is `List(Arc<Vec<IfaValue>>)` — heterogenous at runtime. This means:
- No compile-time element type checking.
- Lists of Ints and Lists of Strings are the same type.
- Map keys are always `CompactString`; Map values are always `IfaValue`.

```rust
// value_union.rs:47-48
List(Arc<Vec<IfaValue>>),
Map(Arc<HashMap<CompactString, IfaValue>>),
```

### Recommendation

Add `TypeHint::Generic { name: String, bounds: Vec<Constraint> }` and a `TypeHint::Applied { base: Box<TypeHint>, args: Vec<TypeHint> }` for parameterization. Start with `List<T>`, `Map<K, V>`, `Result<T, E>`. The Babalawo can monomorphize at analysis time since there's no separate compilation. Keep the runtime representation untyped (`Vec<IfaValue>`) and only add type parameters at the syntax/analysis level.

---

## 2. Algebraic Data Types (ADTs)

### Current State

**`Match` / `yan` statement** (`ast.rs:188-193`):
```rust
Match {
    condition: Expression,
    arms: Vec<MatchArm>,
}
```

**Match arms** (`ast.rs:232-251`):
```rust
pub struct MatchArm {
    pub pattern: MatchPattern,
    pub body: Vec<Statement>,
}

pub enum MatchPattern {
    Literal(Expression),     // e.g., 200, "hello"
    Range { start, end },    // e.g., 90..99
    Wildcard,                // _
}
```

**`Result` type** (`value_union.rs:126-130`):
```rust
pub enum ResultPayload {
    Ok(IfaValue),
    Err(IfaValue),
}
```

This is the sum type for error handling.

### Gap

Ifá-Lang has **no user-defined enum/sum types**. The only sum type is `ResultPayload` (built-in). There is no:
- `enum Color { Red, Green, Blue }`
- Constructor syntax
- Exhaustiveness checking on match arms
- Destructuring bind (match `Some(x)` …)

The `MatchPattern` only supports literals, ranges, and wildcards — it cannot pattern-match on compound values or user-defined constructors.

### Recommendation

Add a `Statement::SumType` declaration for user-defined enums:
```
ese Anfaani<T, E> {
    Dada(T),
    Ti(E),
}
```
Then extend `MatchPattern` with constructor patterns:
```
yan result {
    Dada(v) => Ogbe.soro(v),
    Ti(e)  => Okanran.gba(e),
}
```
Add exhaustiveness checking to the Babalawo (the `Scope`/`VarInfo` system in `scope.rs` can track defined variants). This is the single highest-value addition to the type system.

---

## 3. Ownership & Borrowing

### Current State

**Babalawo's `IwaEngine`** (`iwa.rs`) implements a simplified borrow checker:

```rust
pub struct BorrowDebt {
    var_name: String,
    is_mutable: bool,        // true for &mut, false for &
    line, column: usize,
    scope_depth: usize,
}
```

Rules enforced (`iwa.rs:130-190`):
- `borrow(&T)`: fails if a mutable borrow exists on the same variable.
- `borrow_mut(&mut T)`: fails if **any** borrow (immutable or mutable) exists on the same variable.
- Scope-based release: `exit_scope()` drops all borrows at or below the current depth.
- Release: `release_borrow(name)` manually clears a borrow.

**Move Tracker** (`movement.rs`) enforces linearity:
- `record_move(name)` marks a variable as moved (dead).
- `check_use(name)` returns error for `Moved`, warning for `MaybeMoved` (branch-dependent).
- Copy-eligible types: `Int, Float, Bool, Nil` — everything else is move-by-default.
- `merge_branches`: if a variable is moved on either branch of an `if`, it becomes `MaybeMoved`.

**Type-level annotations** (`ast.rs:329-335`):
```rust
Ptr(Box<TypeHint>),    // *T — unsafe pointer (requires ailewu)
Ref(Box<TypeHint>),     // &T — immutable reference
RefMut(Box<TypeHint>),  // &mut T — mutable reference
```

### Gap

The borrow checker is **not connected to the type system**. `Ref(T)` and `RefMut(T)` are syntactic hints — there is no:
- Lifetime tracking (lexical or non-lexical).
- Connection between `IwaEngine.borrow()` calls and code analysis.
- Compilation of references at the bytecode level.
- Rust-style `&` and `&mut` operators that actually produce borrow-checked values.

The existing `IwaEngine` code is **dead infrastructure** — it has tests but is not integrated into the analysis pipeline (`checks.rs` never calls `borrow()` or `borrow_mut()`).

### Recommendation

Three-tier approach:
1. **Tier 1 (now)**: Connect `IwaEngine` to `checks.rs`. When the code contains `&x` or `&mut x`, record the borrow. When the borrowed variable is used, check for conflicts. This gets lexical borrow checking.
2. **Tier 2**: Add `Statement::RefBinding { name, target, mutable }` to the AST so `if let x = &mut y { }` works syntactically.
3. **Tier 3**: Non-lexical lifetimes (NLL) — track the *last use* of a borrow rather than its lexical scope.

---

## 4. Dependent Types

### Current State

**No support.** The closest thing is `Array { element: Box<TypeHint>, size: usize }` (`ast.rs:337-340`), where `size` is a compile-time constant — a fixed-size array, which is a weak form of dependent type (the type depends on a natural number literal).

`OponSize` variants (`opon.rs:63-76`) are also value-dependent at the runtime level: `Kekere = 256`, `Arinrin = 4096`, `Nla = 65536`. But these are enum variants, not type-level naturals.

### Gap

Full dependent types (`Vec<n>` where `n` is a runtime value, or `Str<s>` where `s` is a string literal) are not in scope. This is correct — dependent types are a research language feature (Idris, Agda, Coq). Ifá-Lang's pragmatic choices are appropriate here.

### Recommendation

Do not add full dependent types. The `Array(T, n)` fixed-size type is sufficient for the embedded/ffi use case. If length-indexed types are needed later, add them as a special case (e.g., `Array(Str, 256)` for fixed buffers in embedded contexts).

---

## 5. Linear Types

### Current State

**Move Tracker** (`movement.rs`) is a lightweight linear type system:

| Feature | Status |
|---------|--------|
| Variable consumption (move) | ✅ `record_move()` / `check_use()` |
| Copy-eligible types | ✅ `is_copy_eligible()` — Int/Float/Bool/Nil |
| Use-after-move error | ✅ `UseAfterMove` |
| Branch-conditional move | ✅ `MaybeMoved` warning |
| Revival via reassignment | ✅ `revive()` |
| Explicit `move(x)` expression | 🔜 Reserved but not yet parsed (`ast.rs:482` `MoveExpr(Box<Expression>)`) |
| Destructuring move | ❌ |

### Gap

The move tracker is a **Babalawo-only analysis**. It has no effect on:
- Code generation (the compiler does not elide copies after moves).
- Runtime behavior (the VM does not track moved variables).
- Actor boundaries (moves are enforced by `freeze()` failure at runtime, not at analysis time).

### Recommendation

Connect the move tracker to code generation. When a variable is moved:
1. The compiler emits a `Pop` or a `Null` assignment to the old slot (preventing accidental reuse at the bytecode level).
2. The Babalawo marks the variable as `Moved` and issues an error on subsequent use.
3. The actor boundary check (`freeze()` call) is paired with a `record_move()` call so the Babalawo catches violations before runtime.

---

## 6. Effect Systems

### Current State

**`Effect` enum** (`ast.rs:9-23`):
```rust
pub enum Effect {
    Pure,
    Async,     // Osa domain
    Network,   // Otura domain
    FileIO,    // Odi domain
    State,     // mutable state
    Impure,    // FFI/unsafe
}
```

**`EffectChecker`** (`babalawo/src/effects.rs`):
- `enter_function(effects)`: declares the effects a function can perform.
- `check_call(callee_effects)`: validates that:
  - A `Pure` function cannot call any function with non-Pure effects.
  - A function must declare all effects that its callees exhibit.
- `domain_effects(domain)`: maps Odu domains to their known effects.
  - `Osa → Async`, `Otura → Network`, `Odi → FileIO`, `Ofun/Sys/Coop → Impure`.

**Function declarations** (`ast.rs:103-111`):
```rust
EseDef {
    name, visibility,
    params,
    body,
    effects: Vec<Effect>,    // ← declared effects
}
```

### Gap

| Feature | Status | Evidence |
|---------|--------|----------|
| Effect declarations on functions | ✅ | `ast.rs:109` |
| Domain → effect mapping | ✅ | `effects.rs:53-60` |
| Pure constraint enforcement | ✅ | `effects.rs:28-36` |
| Missing effect detection | ✅ | `effects.rs:40-49` |
| Effect polymorphism | ❌ | No `<E: Effect>` on generic functions |
| Effect inference | ❌ | Must be manually declared |
| Effect subtyping | ❌ | No effect lattice (Pure <: Async ⊂ ...) |
| Effect erasure at compile time | ❌ | Effects are not compiled into bytecode |
| Async effect ↔ Await connection | ❌ | No static check that async functions are awaited |
| Taboo ↔ Effect integration | ❌ | Taboo checks domain calls; effect checks side effects — separate systems |

### Recommendation

The effect system is the **most important missing piece** for formal correctness (as identified in the prior analysis). Priority actions:

1. **Effect lattice**: Define a partial order `Pure <: Async <: Impure`, `Pure <: FileIO <: Impure`, etc. so a function declared `effects(Async)` can call `Pure` functions without complaint.
2. **Effect inference**: The Babalawo should infer effects when they are not declared (like Rust's `#[must_use]` but inverted). Common case: `ese foo() { Ogbe.soro("hi"); }` → infer `Pure` automatically.
3. **Encode effects in bytecode**: Add an `EffectSet` operand on `Call`/`CallOdu` opcodes. The VM can fast-fail if a `Pure` function tries to call a non-Pure function.
4. **Merge Taboo + Effect**: Domain taboos (`babalawo/taboo.rs`) should be expressible as effect constraints: `ese fetch() -> Data effects(Network)` encodes both the type and the architectural constraint.

---

## 7. Nullability Model

### Current State

Ifá-Lang has **null as a first-class value**:

```rust
// value_union.rs:39
Null,
```

- The `??` null-coalescing operator (`ast.rs:532`): `lhs ?? rhs` evaluates to `rhs` if `lhs` is `Null`.
- No `Option<T>` type.
- No non-null guarantee at the type level.
- `IfaValue::null()` constructor returns `IfaValue::Null`.
- `is_null()` checks for `Null`.
- Every expression can produce `Null`.

### Gap

The absence of `Option<T>` means:
- Every function return type implicitly includes `Null`.
- The type `Int` can actually be `Null`.
- Pattern matching cannot distinguish `Some(x)` from `Null`.
- FFI boundaries have no way to express nullable pointers vs. non-nullable.

### Recommendation

Two-phase approach:

**Phase 1 (non-breaking)**: Add `Option<T>` as a special built-in wrapper type alongside `Null`.

```
TypeHint::Option(Box<TypeHint>)
```

At runtime, `Option<Int>` is represented as `Null | Int(Int)`. The `??` operator is sugar for `yan opt { Null => rhs, _ => opt }`.

**Phase 2 (long-term)**: Make the type system *non-null by default*. `Int` means a real integer. `Nullable<Int>` (or `Int?`) means `Null | Int`. The Babalawo warns when a non-nullable value could be null. This is a major change and should wait until the effect system and ADTs are stable.

---

## 8. Memory Management

### Current State

Ifá-Lang uses a **hybrid approach** with clear tradeoffs:

| Strategy | Where | Details |
|----------|-------|---------|
| **Region/Arena** | Opon memory | `Vec<IfaValue>` with epoch-based bulk deallocation |
| **Reference Counting** | Heap values | `Arc<Vec>`, `Arc<HashMap>`, `Arc<Mutex>`, `Arc<str>` |
| **RAII** | Host resources | `Ebo<F>` and `EboScope<T,F>` guards in Rust |
| **No GC** | — | Explicitly rejected — no tracing, no mark-sweep, no generational GC |

**Opon** (`opon.rs`):
- `begin_epoch(name)` → allocations tracked
- `end_epoch()` → `self.memory.truncate(start_addr)` — bulk free
- Sized variants: 256 slots (4KB) to dynamic (1M slot hard limit)
- No individual deallocation within an epoch — only bulk release

**Arc sharing** (`value_union.rs`):
- `List(Arc<Vec<IfaValue>>)` — immutable after creation
- `Map(Arc<HashMap<...>>)` — immutable after creation
- `Upvalue(Arc<Mutex<IfaValue>>)` — mutable via lock
- `Future(Arc<Mutex<FutureState>>)` — mutable via lock

**Allocation cost per type**:

| Operation | Cost |
|-----------|------|
| Push Int on stack | O(1), inline in Opon slot |
| Create List of 1000 items | O(n) alloc + O(n) Arc ref increments |
| Clone a List | O(1) — `Arc::clone()` (ptr bump only) |
| Append to List | O(n) — allocates new `Arc<Vec>` with +1 element |
| Actor send of large List | O(n) — deep copy via freeze+thaw |
| Epoch end | O(n_freed) — `truncate()` |

### Gap

| Feature | Status |
|---------|--------|
| Stack allocation | ✅ Opon slots |
| Scoped epochs | ✅ |
| Arc refcounting | ✅ |
| RAII for resources | ✅ |
| Ownership transfer (move) | Partial — only at Babalawo analysis level |
| Custom allocators | ❌ (Opon always uses `Vec`) |
| Memory pools for fixed-size types | ❌ |
| Cycle detection for Arc | ❌ (Arc cycles are memory leaks) |
| GC for long-lived data | ❌ (by design) |

### Risk: Arc Cycle Leaks

Any Rust program that creates `List(Arc<Vec<...>>)` containing an `IfaValue` that transitively references the same `Arc` creates a cycle. Ifá-Lang has no cycle collector, so these are permanent leaks. This is acceptable for short-lived programs but dangerous for long-running actor systems.

### Recommendation

Stay the course — **no GC**. The region + Arc model is correct for Ifá-Lang's niche. Three targeted improvements:

1. **Weak references**: Add `IfaValue::WeakList(Arc<std::sync::Weak<Vec<IfaValue>>>)` and `IfaValue::WeakMap(...)` for cache-like structures where cycles are a risk.
2. **Arena allocator for Opon**: Replace `Vec<IfaValue>` with a slab/arena allocator that can handle deallocation of individual slots within an epoch (for long-lived epochs like actor loops).
3. **Cross-epoch references**: Track references from Opon to `Arc` heap values so that when an epoch ends, associated heap memory is also released (not just the Opon slot).

---

## 9. Concurrency Model

### 9.1 Overview

Ifá-Lang embeds **four concurrency models** simultaneously:

| Model | Mechanism | Where | Use Case |
|-------|-----------|-------|----------|
| **OS Threads** | `spawn_actor` → OS thread via tokio `spawn_blocking` | `actor.rs:338` | Isolated actor instances |
| **Async/Await** | `FutureState { Pending, Ready }` + `task_queue` | `vm.rs:166-168` | Cooperative multitasking |
| **Actor System (H2)** | `IfaVM` per actor + `mpsc::sync_channel(64)` | `actor.rs` | Message-passing isolation |
| **Data Parallelism** | `OpCode::ParallelFor` | `bytecode/src/lib.rs:174` | Parallel iteration |

### 9.2 OS Threads

Each actor runs on a dedicated thread via tokio's `spawn_blocking` (`actor.rs:44-48`):

```rust
if let Ok(handle) = tokio::runtime::Handle::try_current() {
    handle.spawn_blocking(f);
} else {
    get_actor_runtime().spawn_blocking(f);
}
```

- Dedicated `tokio::runtime::Runtime` created in `OnceLock` (`actor.rs:25-35`).
- Multi-thread scheduler with thread name `"ifa-actor-pool"`.
- WASM target: panics with "not supported" (`actor.rs:53`).

### 9.3 Async/Await

**VM-level futures** (`value_union.rs:116-124`):
```rust
pub enum FutureState {
    Pending,
    Ready(IfaValue),
}
pub type FutureCell = Arc<Mutex<FutureState>>;
```

- `spawn_task(func, args)` creates a `Future` value, pushes it to `task_queue`.
- `await_future(cell, bytecode)` busy-polls: repeats `resume_execution` until the future is `Ready`.
- `OpCode::Yield` during execution causes `resume_execution` to return early with `IfaError::Yielded` — the task is not preempted, it yields cooperatively.

**Key problem**: `await_future` is a **busy-loop** in the current thread. It does not yield to the OS scheduler or to tokio. This means:
- One actor cannot run multiple futures concurrently.
- The actor thread stays 100% CPU while "awaiting."
- No integration with tokio's reactor for I/O-based futures.

### 9.4 Actor System (H2)

**Isolation guarantees** (from `actor.rs:1-13`):
- Each actor is a **fresh `IfaVM`** — new globals, new Opon, new registry.
- Communication: `sync_channel(64)` — bounded, back-pressure.
- Message delivery: freeze + thaw (deep copy) + resource transfer.
- Shutdown: cooperative via `ActorMsg::Shutdown`.

**`ActorHandle`** (`actor.rs:94-102`):
```rust
pub struct ActorHandle {
    pub id: u64,
    tx: Arc<SyncSender<ActorMsg>>,
    pub resource_registry: Arc<ResourceRegistry>,
}
```

**Send protocol** (`actor_send`, `actor.rs:261-318`):
```
1. freeze(value) → IfaShared  (deep copy; fails on non-Send)
2. thaw(shared) → IfaValue    (convert back)
3. transfer_resources(...)    (move ResourceTokens)
4. tx.try_send(ActorMsg::Value(thawed))
```

### 9.5 Data Parallelism

`OpCode::ParallelFor` (0x5C) exists in the opcode table (`bytecode/src/lib.rs:174`) with stack effect `(2, 1)` — pops iterable + closure, pushes list of results. The compiler emits it for `ParallelFor` statements.

### 9.6 CSP (Communicating Sequential Processes)

Ifá-Lang does **not** implement CSP directly, but the actor system's channel-based message passing (`sync_channel`) is CSP-adjacent. The `Osa` domain provides:
- `oju_ona` (mpsc channel)
- `oju_ona_kan` (oneshot channel)
- `titipe` (Mutex)
- `kaka` (RwLock)

These are tokio wrappers, not CSP primitives.

### 9.7 STM (Software Transactional Memory)

**Not present.** No `stm` crate usage, no transactional memory constructs.

### 9.8 Deterministic Concurrency

**Not addressed.** The actor system's `HashMap<u64, ActorHandle>` iteration is non-deterministic (randomized hash). Message ordering between actors is not guaranteed (depends on OS thread scheduling).

### Gap Summary

| Feature | Status | Evidence |
|---------|--------|----------|
| OS threads (per actor) | ✅ | `spawn_actor_task` → `spawn_blocking` |
| Async/await (`daro`/`reti`) | ✅ | `FutureState`, `task_queue` |
| Actor isolation | ✅ | Fresh VM per actor |
| Bounded channels | ✅ | `sync_channel(64)` |
| Cooperative shutdown | ✅ | `ActorMsg::Shutdown` |
| Data parallelism | ✅ | `OpCode::ParallelFor` |
| Green threads / fibers | ❌ | Actors are OS threads, not M:N |
| Async I/O integration | ❌ | `await` is a busy-loop, not reactor-based |
| Work-stealing scheduler | ❌ | Each actor has its own thread |
| STM | ❌ | |
| Deterministic message delivery | ❌ | |
| Structured concurrency | ❌ | No cancellation scope, no supervision tree |

### Recommendation

1. **Fix the async actor loop**: Replace the busy-loop `await_future` with a true `tokio::select!` that yields to tokio's scheduler when no task is ready. This lets a single actor thread multiplex many futures efficiently.

2. **Add structured concurrency**: A `scope { }` block that:
   - Spawns child actors/futures that are automatically joined on scope exit.
   - Propagates cancellation: if the scope exits early, all children are cancelled.
   - This prevents orphaned actors.

3. **Deterministic actor IDs**: Replace `AtomicU64` with a deterministic ID scheme based on the actor's creation location (module + line number + parent ID). This is optional but helps with debugging and reproducibility.

4. **Green threads (long-term)**: If actor count grows beyond thousands, consider M:N scheduling where many Ifá-Lang "actors" are multiplexed onto a pool of OS threads via tokio tasks instead of `spawn_blocking`. Each actor becomes a lightweight task that yields on message receive.

---

## Comparison Matrix

| Feature | Ifá-Lang | Rust | Go | Erlang | Haskell |
|---------|----------|------|----|--------|---------|
| **Generics** | None | ✅ Trait-based | ❌ (interface{}) | ❌ (dynamic) | ✅ Type classes |
| **ADTs** | Partial (Result, Match) | ✅ Enum | ❌ | ❌ | ✅ |
| **Ownership** | Static analysis only (IwaEngine) | ✅ Borrow checker | ❌ (GC) | ❌ (GC) | ❌ (GC) |
| **Linear types** | MoveTracker (analysis only) | ✅ Drop semantics | ❌ | ❌ | ✅ |
| **Effect system** | Basic (6 effects, function decl) | ❌ (no effects) | ❌ | ❌ | ✅ Monads |
| **Nullability** | Null + `??` | ✅ Option<T> | ✅ nil | ❌ (everything is a term) | ❌ (Maybe) |
| **Memory** | Region+Arc+RAII | Ownership+RAII | GC+escape | GC | GC |
| **Concurrency** | Actors+Async+Threads | Async+Threads | Goroutines | Actors | Async+STM |

---

*Specification v0.1. All claims reference specific code locations in crates/. The strongest asymmetry: IwaEngine's borrow checker and MoveTracker exist but are not wired into the compilation pipeline. The effect system is functional but disconnected from the bytecode. These are integration gaps, not architecture gaps.*
