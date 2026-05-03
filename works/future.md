# Ẹgbẹ́ Concurrency — Implementation Plan

> Grounded in: `osa.rs`, `cpu.rs`, `gpu.rs`, `iwa.rs`, `opon.rs`, `shared.rs`, `checks.rs`
>
> The infrastructure exists. The actor semantics do not. This plan bridges that gap in strict dependency order.

---

## Reality Check: What Exists vs What's Missing

### ✅ Already Built

| Component | File | Notes |
|---|---|---|
| Tokio spawn, channels, oneshot | `osa.rs` | `Osa::sa()`, `oju_ona()`, `oju_ona_kan()` |
| Rayon par_map, par_reduce, matmul | `cpu.rs` | Full `CpuContext` + `TaskGraph` DAG |
| WGPU matmul, relu, slab allocator | `gpu.rs` | `GpuContext`, `SlabMemoryPool`, `MemoryPool` |
| Borrow ledger (`&T`/`&mut T`) | `iwa.rs` | `IwaEngine`, scope-depth tracking |
| Epoch allocator | `opon.rs` | `begin_epoch`/`end_epoch`, per-actor `Opon` |
| `IfaShared` thread-safe value type | `shared.rs` | `Arc<DashMap>`, `Serialize/Deserialize` |
| `in_async_function` flag | `checks.rs:44` | Exists, no rules attached to it |

### 🔴 Missing

| Component | Needed For |
|---|---|
| `moved_vars: HashSet<String>` in `LintContext` | Move semantics, actor send safety |
| `USE_AFTER_MOVE` error in Babalawo | Proving sender loses ownership |
| Actor struct + mailbox queue | `Ẹgbẹ́` runtime |
| `Osa.ebo()` VM handler | Actual actor send |
| `EBO_MAILBOX_FULL` error variant | Backpressure |
| `&mut` across `daro` suspension check | Async safety rule |
| `iwori.yipo.ori` opcode + Babalawo gate | Parallel-for |
| GPU `iso` proof before `ailewu` dispatch | GPU ownership safety |

---

## Hard Rules (Non-Negotiable)

These came from the user's review comments and govern every phase:

1. **Babalawo stays simple.** No alias graphs. No region analysis. Scope-depth + move-set is the ceiling.
2. **`daro` suspension + `&mut` = hard error.** No warnings. No suggestions. Compile stops.
3. **Cross-actor await is not async.** It is request-reply messaging. Do not pretend otherwise in the type system.
4. **`iwori.yipo.ori` captures only `&T`.** Babalawo rejects `&mut` or `IfaShared` mutation in the parallel body.
5. **Do not write a scheduler.** Wrap tokio multi-thread. Measure first.
6. **Bounded mailboxes from day one.** `Osa.ebo()` returns `Result`, not `()`.
7. **Remote `ebo` is explicit and loud.** Local = zero-copy. Remote = serialize. Never hide the cost.

---

## Phase 1 — Move Tracking in Babalawo

**Goal:** Prove that when `Osa.ebo(x)` is called, `x` is dead in the caller. No alias, no read, no second send.

**Why first:** Everything else depends on ownership transfer being statically proven. Phases 2–5 are meaningless without this.

### 1.1 — Add `moved_vars` to `LintContext`

**File:** `crates/ifa-babalawo/src/checks.rs`

Add to `LintContext`:
```rust
/// Variables that have been moved (ownership transferred, e.g., via Osa.ebo())
pub moved_vars: HashSet<String>,
```

Add to `LintContext::new()`:
```rust
moved_vars: HashSet::new(),
```

### 1.2 — Mark variable as moved on `Osa.ebo()` call

**File:** `crates/ifa-babalawo/src/checks.rs`

In `check_statement`, within the `Statement::Instruction` branch, detect the pattern `call.domain == "Osa" && call.method == "ebo"`:

```rust
// Detect Osa.ebo(x) — first argument is the moved value
if call.domain_matches("osa") && call.method == "ebo" {
    if let Some(Expression::Identifier(moved_name)) = call.args.first() {
        // Record the move
        ctx.moved_vars.insert(moved_name.clone());
        // Release any outstanding borrow on the moved variable
        ctx.iwa_engine.release_borrow(moved_name);
    }
}
```

### 1.3 — Emit `USE_AFTER_MOVE` on subsequent access

**File:** `crates/ifa-babalawo/src/checks.rs`

In `check_expression`, within the `Expression::Identifier(name)` branch, add before the undefined-variable check:

```rust
if ctx.moved_vars.contains(name) {
    baba.error(
        "USE_AFTER_MOVE",
        &format!(
            "Variable '{}' has been moved (sent via Osa.ebo) and cannot be used again. \
             Bind a new value or clone before sending.",
            name
        ),
        file,
        span.line,
        span.column,
    );
    return; // No further checks — variable is gone
}
```

### 1.4 — Reset `moved_vars` on scope exit

Moved variables should not escape their scope. Clear `moved_vars` in `LintContext::exit_function()`:

```rust
pub fn exit_function(&mut self) {
    self.current_function = None;
    self.has_return = false;
    self.in_async_function = false;
    self.moved_vars.clear(); // Moved set is per-function
}
```

### Exit Criteria for Phase 1

- `Osa.ebo(x); fọ x` produces `USE_AFTER_MOVE` error.
- `Osa.ebo(x); Osa.ebo(x)` produces `USE_AFTER_MOVE` on the second send.
- `Osa.ebo(x)` with `x` still borrowed produces `ValueBorrowed` from `IwaEngine` before move is recorded.
- Unit test in `ifa-babalawo` covering all three cases.

---

## Phase 2 — Actor Runtime (`Ẹgbẹ́`)

**Goal:** Make actors real: a spawnable unit with an `Opon`, a tokio task, and a bounded mpsc mailbox.

**Depends on:** Phase 1 (Babalawo must prove sends are safe before runtime trusts them).

### 2.1 — Actor struct

**New file:** `crates/ifa-core/src/actor.rs`

```rust
use crate::opon::{Opon, OponSize};
use crate::value::IfaValue;
use tokio::sync::mpsc;

/// An actor's inbound message
pub struct EboMessage {
    /// The payload (owned — sender has relinquished)
    pub payload: IfaValue,
    /// Optional reply channel (for request-reply pattern)
    pub reply_tx: Option<tokio::sync::oneshot::Sender<IfaValue>>,
}

/// Actor handle — identity only (Pony `tag` equivalent)
/// Holds only the sender side of the mailbox. Cannot read actor state.
#[derive(Clone)]
pub struct ActorHandle {
    pub id: u64,
    pub tx: mpsc::Sender<EboMessage>,
}

impl ActorHandle {
    /// Send a message (fire-and-forget). Returns Err if mailbox is full.
    pub async fn ebo(&self, payload: IfaValue) -> Result<(), EboError> {
        self.tx
            .try_send(EboMessage { payload, reply_tx: None })
            .map_err(|e| match e {
                mpsc::error::TrySendError::Full(_) => EboError::MailboxFull,
                mpsc::error::TrySendError::Closed(_) => EboError::ActorDead,
            })
    }

    /// Send a message and await a reply (request-reply).
    pub async fn ebo_await(&self, payload: IfaValue) -> Result<IfaValue, EboError> {
        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
        self.tx
            .try_send(EboMessage { payload, reply_tx: Some(reply_tx) })
            .map_err(|_| EboError::MailboxFull)?;
        reply_rx.await.map_err(|_| EboError::ActorDead)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EboError {
    MailboxFull,
    ActorDead,
}
```

### 2.2 — Actor spawn

**File:** `crates/ifa-core/src/actor.rs`

```rust
/// Default mailbox depth — configurable via #opon directive
pub const DEFAULT_MAILBOX_DEPTH: usize = 64;

/// Spawn an actor. Returns its handle.
/// The closure receives messages from the mailbox.
pub fn spawn_actor<F, Fut>(
    id: u64,
    mailbox_depth: usize,
    handler: F,
) -> ActorHandle
where
    F: FnOnce(mpsc::Receiver<EboMessage>, Opon) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    let (tx, rx) = mpsc::channel(mailbox_depth);
    let opon = Opon::new(OponSize::Arinrin); // Each actor gets its own Opon

    tokio::spawn(handler(rx, opon));

    ActorHandle { id, tx }
}
```

### 2.3 — Expose `EboError` as `IfaValue`

**File:** `crates/ifa-types/src/error.rs`

Add `EboMailboxFull` and `EboActorDead` variants to `IfaError`. The VM handler for `Osa.ebo()` must surface these as catchable `gbiyanju` errors, not panics.

### 2.4 — VM handler for `Osa.ebo`

**File:** `crates/ifa-core/src/vm.rs` (or the relevant handler file)

Wire the `Osa.ebo` instruction to call `ActorHandle::ebo()`. This requires the VM to have access to a registry of live actor handles, keyed by `u64` ID. The registry is a `DashMap<u64, ActorHandle>` shared across the runtime — this is the only global state permitted.

### Exit Criteria for Phase 2

- `ifa runb` can spawn two actors that exchange messages.
- Sending to a full mailbox produces a catchable `IfaError::EboMailboxFull`.
- Sending to a dead actor produces `IfaError::EboActorDead`.
- Each actor has its own `Opon`; memory from one actor is not visible to another.

---

## Phase 3 — `daro` Async Enforcement

**Goal:** Make `in_async_function` do real work. Enforce that `&mut` borrows do not cross `daro` suspension points.

**Depends on:** Phase 1 (borrow ledger must be in good shape).

> "The `in_async_function` flag exists — now make the rules brutal."

### 3.1 — Track suspension points

`daro x` (an await expression) is a suspension point. When Babalawo encounters `await` inside a `daro` function, it must snapshot the current `borrow_ledger` from `IwaEngine`.

**File:** `crates/ifa-babalawo/src/checks.rs`

In `check_expression`, detect `Expression::Await { .. }` (or the Ifá-Lang equivalent):

```rust
if ctx.in_async_function {
    // Check for active mutable borrows at this suspension point
    for borrow in ctx.iwa_engine.active_borrows() {
        if borrow.is_mutable {
            baba.error(
                "MUTABLE_BORROW_ACROSS_DARO",
                &format!(
                    "Variable '{}' is mutably borrowed across a 'daro' suspension point (line {}). \
                     Release the borrow before suspending.",
                    borrow.var_name, borrow.line
                ),
                file,
                span.line,
                span.column,
            );
        }
    }
}
```

### 3.2 — Cross-actor `daro` is request-reply, not shared future

Babalawo must warn (not error, initially) when a `daro` expression appears to `await` a value from a different actor's scope. The diagnostic:

```
CROSS_ACTOR_AWAIT: 'daro' on a cross-actor message is request-reply. 
The sender blocks until the receiving actor responds. 
This is not shared-memory async — actor isolation still holds.
```

### 3.3 — Parser support for `daro`

Verify `grammar.pest` and `parser.rs` handle `daro` as a keyword that marks both function definitions (`ese daro`) and expressions (`daro some_call()`). If not present, add the grammar rule and AST node.

### Exit Criteria for Phase 3

- `ese daro someFunc() { let x = &mut y; daro heavyOp(); fọ x; }` → `MUTABLE_BORROW_ACROSS_DARO` error.
- `ese daro someFunc() { daro heavyOp(); }` with no borrows → compiles cleanly.
- Unit test in `ifa-babalawo` for both cases.

---

## Phase 4 — `iwori.yipo.ori` Parallel-For Gate

**Goal:** Expose `CpuContext::par_map` to Ifá-Lang programs via a safe, Babalawo-gated `iwori.yipo.ori` syntax.

**Depends on:** Phase 1 (move tracking), Phase 2 (actor context established).

> "`iwori.yipo.ori` captures only `&T`. Babalawo rejects `&mut` or `IfaShared` mutation inside the parallel body."

### 4.1 — Bytecode opcode

**File:** `crates/ifa-bytecode/src/format.rs` (or equivalent)

Add:
```rust
ParallelFor {
    iter_reg: u8,     // Register holding the list/array to iterate
    body_fn: u32,     // Bytecode address of the body function
    result_reg: u8,   // Register to store collected results
},
```

### 4.2 — Babalawo gate

**File:** `crates/ifa-babalawo/src/checks.rs`

When Babalawo encounters a `Statement::ParallelFor { var, iterable, body }`:

1. Enter a new lint scope marked as `parallel_body: true`.
2. Walk the body. For every `&mut` borrow or mutation of a captured variable → hard error `PARALLEL_MUT_CAPTURE`.
3. For every access to `IfaShared::Object` with a write → hard error `PARALLEL_SHARED_MUTATION`.
4. The loop variable `var` is treated as `&T` (immutable) within the body.

```rust
"PARALLEL_MUT_CAPTURE" =>
  "iwori.yipo.ori body captures '{}' mutably. Parallel bodies must be pure 
   (read-only captures only). Use a sequential loop or collect into a new list."

"PARALLEL_SHARED_MUTATION" =>
  "iwori.yipo.ori body mutates IfaShared state. This reintroduces data races. 
   Pass results out via the return value, not side effects."
```

### 4.3 — VM dispatch

**File:** `crates/ifa-core/src/vm.rs`

The `ParallelFor` opcode dispatches to `CpuContext::par_map`. Since `CpuContext::par_map` already exists and is `rayon`-backed, the VM implementation is a thin wrapper:

```rust
Opcode::ParallelFor { iter_reg, body_fn, result_reg } => {
    let list = self.registers[iter_reg].as_list()?;
    // body_fn is a pure function — Babalawo already proved no mut captures
    let results = rayon::iter::IntoParallelRefIterator::par_iter(&list)
        .map(|item| self.call_pure_fn(body_fn, item))
        .collect::<Result<Vec<_>, _>>()?;
    self.registers[result_reg] = IfaValue::List(results);
}
```

### Exit Criteria for Phase 4

- `iwori yipo.ori (x ninu list) { fọ x * 2; }` compiles and runs in parallel.
- `iwori yipo.ori (x ninu list) { y = y + 1; }` (capturing `y` mutably) → `PARALLEL_MUT_CAPTURE`.
- Benchmark: parallel matmul via `iwori.yipo.ori` over rows is faster than sequential.

---

## Phase 5 — GPU Dispatch Safety

**Goal:** Connect the existing `GpuContext` (wgpu, `SlabMemoryPool`) to Ifá-Lang programs safely. Ensure data sent to GPU is unaliased before dispatch.

**Depends on:** Phase 1 (move tracking — GPU send is a move, not a borrow).

> "GPU offload is an FFI boundary (`ailewu` block) calling into CUDA/Vulkan/WGSL."
> This is already the correct model. `GpuContext` exists. The gate is Babalawo proving the data is `iso`.

### 5.1 — GPU send = move (same as actor send)

When `ifa-std/infra/gpu.rs` functions are called from an `ailewu` block:

- The `IfaValue` passed to the GPU is **moved** (same `moved_vars` mechanism from Phase 1).
- Babalawo enforces: no outstanding `&T` or `&mut T` on the value being dispatched.
- After dispatch, the value is gone from the Ifá-Lang side until `GpuContext::read_buffer` brings it back.

### 5.2 — Babalawo: GPU calls require `ailewu`

**File:** `crates/ifa-babalawo/src/checks.rs`

The `check_unsafe_ffi_call` function already exists. Extend it to recognize GPU domain calls:

```rust
fn is_gpu_call(call: &OduCall) -> bool {
    matches!(call.domain_str(), "infra" | "gpu") 
    && matches!(call.method.as_str(), "matmul" | "relu" | "vec_add" | "dispatch" | ...)
}

if is_gpu_call(call) && !ctx.in_ailewu {
    baba.error(
        "GPU_OUTSIDE_AILEWU",
        &format!(
            "GPU dispatch '{}' must be inside an 'ailewu' block. \
             GPU memory is unmanaged — Babalawo cannot track it outside unsafe context.",
            call.method
        ),
        file, span.line, span.column,
    );
}
```

### 5.3 — CPU fallback path

`CpuContext::matmul` already exists as a rayon-parallel CPU fallback. The VM should prefer GPU when `GpuContext` is initialized, fall back to `CpuContext` transparently. No Babalawo changes needed — both paths go through the same `ailewu` gate.

### Exit Criteria for Phase 5

- GPU calls outside `ailewu` → `GPU_OUTSIDE_AILEWU` error.
- GPU calls inside `ailewu` with outstanding borrow on dispatched data → borrow error from `IwaEngine`.
- GPU calls inside `ailewu` with clean ownership → compiles and dispatches.

---

## Sequencing Summary

```
Phase 1: Move tracking (Babalawo)         ← No dependencies. Start here.
    ↓
Phase 2: Actor runtime (Ẹgbẹ́ + Osa.ebo)  ← Needs Phase 1 (proven sends)
    ↓
Phase 3: daro async enforcement            ← Needs Phase 1 (borrow ledger)
    ↓
Phase 4: Parallel-for gate                 ← Needs Phase 1, Phase 2 (context)
    ↓
Phase 5: GPU dispatch safety               ← Needs Phase 1 (move tracking)
```

Phases 3, 4, and 5 can proceed in parallel after Phase 1 and 2 are done.

---

## What Is Explicitly Deferred

Per the user's review comments and the ROADMAP's deferred list:

| Feature | Reason Deferred |
|---|---|
| Work-stealing scheduler | Use tokio. Measure first. Build only if tokio is the bottleneck. |
| Distributed `Osa.ebo(x, node:)` | `IfaShared` is serializable but remote failure modes add significant design surface. |
| Real-time guarantees (`#opon wakati`) | Requires OS scheduler contract. `ifa-embedded` concern. |
| Actor topology deadlock analysis | Valid future Babalawo pass. Not a Phase 1–5 blocker. |
| Pony `trn` / `iso → val` transition type | Useful but adds complexity. Revisit after Phase 2 ships. |
| Linear types (`must use exactly once`) | Use `#[must_use]` attribute instead. Cheaper, targeted. |

---

## File Change Map

| Phase | Files Changed |
|---|---|
| 1 | `ifa-babalawo/src/checks.rs` |
| 2 | `ifa-core/src/actor.rs` (new), `ifa-core/src/vm.rs`, `ifa-types/src/error.rs` |
| 3 | `ifa-babalawo/src/checks.rs`, `ifa-core/src/grammar.pest`, `ifa-core/src/parser.rs` |
| 4 | `ifa-babalawo/src/checks.rs`, `ifa-bytecode/src/format.rs`, `ifa-core/src/vm.rs` |
| 5 | `ifa-babalawo/src/checks.rs` |

