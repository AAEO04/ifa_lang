# Formal Operational Semantics — Core Runtime Subsystems

**Status:** `IMPLEMENTED`  
**Ifá-Lang Specification v0.1**  
*Definitive reference: crates/ifa-vm/src/opon.rs, crates/ifa-vm/src/ebo.rs, crates/ifa-vm/src/actor.rs, crates/ifa-babalawo/src/taboo.rs, crates/ifa-types/src/capability.rs, crates/ifa-types/src/value_union.rs, crates/ifa-types/src/shared.rs, crates/ifa-vm/src/vm.rs*

---

## Table of Contents

1. [Opon Memory Model](#1-opon-memory-model)
2. [Ebo Lifecycle](#2-ebo-lifecycle)
3. [Actor Isolation Guarantees](#3-actor-isolation-guarantees)
4. [Taboo Enforcement Rules](#4-taboo-enforcement-rules)
5. [Cross-Cutting Invariants](#5-cross-cutting-invariants)
6. [Glossary](#6-glossary)

---

## 1. Opon Memory Model

### 1.1 Purpose

The Opon (sacred calabash) is the memory container for the Ifá-Lang VM. It provides deterministic region-based memory management without tracing garbage collection. Memory is organized as a contiguous vector of `IfaValue` slots, subdivided by Ebo epochs for scoped bulk deallocation.

### 1.2 Configuration

The Opon is parameterized by an `OponSize` variant that selects the slot capacity:

| Variant   | Slot count | Approx. size | Directive              |
|-----------|-----------|--------------|------------------------|
| `Kekere`  | 256       | ~4 KB        | `#opon kekere`         |
| `Arinrin` | 4096      | ~64 KB       | `#opon arinrin` (default) |
| `Nla`     | 65536     | ~1 MB        | `#opon nla`            |
| `Ailopin` | dynamic   | variable     | `#opon ailopin`        |

Slot count is `OponSize::slot_count()`:
- Fixed sizes return literal slot counts.
- `Ailopin` returns `usize::MAX` and grows the backing `Vec` dynamically, capped by `AILOPIN_HARD_LIMIT = 2^20` (1,048,576 slots) as a host-safety ceiling (`opon.rs:14`).

### 1.3 State

```
Opon {
    memory: Vec<IfaValue>       — slot store, index-addressed
    max_slots: usize             — capacity limit (usize::MAX = unlimited)
    history: Vec<OponEvent>      — circular flight recorder buffer
    cursor: usize                 — current write position in history
    history_capacity: usize       — fixed at 256 (16 × 16)
    epochs: Vec<EboEpoch>        — active epoch stack
    next_epoch_id: usize          — monotonic epoch counter
    high_water: usize             — highest address ever allocated
}

EboEpoch {
    id: usize                     — monotonic identifier
    name: String                  — programmer-supplied label
    start_addr: usize             — memory.len() at epoch creation
    alloc_count: usize            — allocations within this epoch
    active: bool                  — true while epoch is open
}
```

### 1.4 Operations

#### 1.4.1 `allocate(value) → Result<addr, OponError>`

```
pre:
    let addr = self.memory.len()

rule:
    if self.max_slots ≠ usize::MAX ∧ addr ≥ self.max_slots:
        fail MemoryLimitExceeded(limit: max_slots, requested: addr+1)
    else:
        self.memory.push(value)
        self.high_water = max(self.high_water, self.memory.len())
        if self.epochs is not empty:
            self.epochs.last().alloc_count += 1
        return Ok(addr)

post:
    ∀ epoch in self.epochs: epoch.start_addr ≤ addr
```

#### 1.4.2 `try_set(addr, value) → Result`

```
pre:
    if addr ≥ self.memory.len():
        if self.max_slots == usize::MAX ∧ addr ≥ AILOPIN_HARD_LIMIT:
            fail MemoryLimitExceeded(limit: AILOPIN_HARD_LIMIT)
        elif self.max_slots ≠ usize::MAX ∧ addr ≥ self.max_slots:
            fail MemoryLimitExceeded(limit: max_slots)
        else:
            self.memory.resize(addr+1, IfaValue::Null)
    self.memory[addr] = value
    return Ok(∎)
```

#### 1.4.3 `get(addr) → Option<&IfaValue>`

```
self.memory.get(addr)
```

Returns `None` if `addr ≥ self.memory.len()`.

#### 1.4.4 `begin_epoch(name)`

```
let id = self.next_epoch_id
self.next_epoch_id = self.next_epoch_id.saturating_add(1)
let start_addr = self.memory.len()
self.epochs.push(EboEpoch { id, name, start_addr, alloc_count: 0, active: true })
```

No memory is moved or copied; the epoch bounds are recorded as a watermark.

#### 1.4.5 `end_epoch() → Result`

```
pre:
    let epoch = self.epochs.pop()   — fails with InvalidAddress if empty

rule:
    epoch.active = false
    self.memory.truncate(epoch.start_addr)
    return Ok(∎)

post:
    self.memory.len() == epoch.start_addr
    — every slot allocated inside the epoch is released
```

#### 1.4.6 `memory_used() → usize`

Count of slots containing non-`Null` values:
```
self.memory.iter().filter(|v| !v.is_null()).count()
```

### 1.5 Invariants

1. **Monotonic addresses**: `allocate` always returns `self.memory.len()` before the push. Addresses are temporally ordered: if epoch A opens before epoch B, then all addresses in A are lower than all addresses in B.

2. **Nested epoch release**: Epochs form a stack (LIFO). If epoch A (outer) opens at address X, then epoch B (inner) opens at address Y ≥ X, then `end_epoch()` on B truncates to Y, and `end_epoch()` on A truncates to X. The test at `opon.rs:479-498` confirms: after inner ends, outer's allocations survive.

3. **No dangling references**: The VM never retains raw pointers into Opon memory. All cross-references use `Arc`-based `IfaValue` variants (`Arc<Vec>`, `Arc<HashMap>`, `Arc<Mutex>`) which live on the heap outside the Opon. Truncating `self.memory` is safe because the Opon only owns the `Vec<IfaValue>` slots directly.

4. **Flight recorder capacity**: `history.len()` never exceeds `history_capacity` (256). After saturation, oldest events are overwritten in circular-buffer fashion (`opon.rs:366-371`).

5. **Host-safety ceiling**: Even `Ailopin` mode enforces `AILOPIN_HARD_LIMIT = 2^20` (`opon.rs:257-263`). This prevents unbounded memory growth from a single VM instance.

---

## 2. Ebo Lifecycle

### 2.1 Purpose

Ebo (sacrifice) provides deterministic RAII resource management. Two mechanisms exist:

1. **Opon Epochs** (`EpochBegin`/`EpochEnd` opcodes): bulk memory region cleanup at the VM level.
2. **Ebo guards** (`ebo.rs`): zero-cost Rust-level RAII for host resources (file handles, connections, allocations).

### 2.2 Opon-Level Ebo (Bytecode)

Compiled from `ebo { ... }` blocks in Ifá source (`compiler/src/lib.rs:969-983`):

```
Statement::Ebo { offering, body } →
    compile_expression(offering)
    OpCode::EpochBegin
    begin_scope()
    for s in body: compile_statement(s)
    end_scope()       — pops locals, emits Pop for each
    OpCode::EpochEnd
```

The `offering` expression is pushed onto the value stack before `EpochBegin`. The VM pushes the epoch name (popped from stack), begins the epoch, executes the body, then ends the epoch.

At the VM level (`vm.rs`):

```
EpochBegin: pop name → opon.begin_epoch(name)
EpochEnd:   opon.end_epoch()
```

### 2.3 Rust-Level Ebo (Host RAII)

Two generic Rust types in `ebo.rs`:

#### `Ebo<F: FnOnce()>` — zero-cost RAII guard

```
Ebo::new(name, cleanup_fn)    — stores FnOnce, runs on drop
Ebo::dismiss(self)             — forgets guard, cleanup never runs
Ebo::sacrifice(self)           — runs cleanup immediately then forgets
```

Drop impl: calls `cleanup` exactly once via `ManuallyDrop::take`. After `dismiss` or `sacrifice`, the guard is `mem::forget`-ed, so Drop never fires.

#### `EboScope<T, F: FnOnce(&mut T)>` — scoped resource

```
EboScope::new(value, cleanup_fn)  — wraps T, runs cleanup(&mut T) on drop
EboScope::into_inner(self)        — runs cleanup, returns T
EboScope::leak(self)              — returns T without cleanup
```

Deref/DerefMut provide transparent access to the inner `T`.

### 2.4 Invariants

1. **Exactly-once execution**: The cleanup closure runs exactly once — either via `Drop`, `sacrifice`, or `into_inner`. The use of `ManuallyDrop` + `mem::forget` prevents double-free.

2. **Opon epoch stacking**: Epochs nest strictly. Every `EpochBegin` is paired with exactly one `EpochEnd`. The VM does not provide a way to skip or duplicate epoch boundaries.

3. **Defer complement**: The `defer!` macro (`ebo.rs:160-165`) creates an `Ebo::new` guard, providing Go-style defer semantics. Deferred cleanup runs at scope exit in LIFO order (stack of `Ebo` guards on the Rust stack).

4. **Resource ownership transfer**: When an `EboScope` is moved across an actor boundary, the caller must either `into_inner()` (which runs cleanup) or `leak()` (which suppresses it). The guard itself does not implement `Send` unless the wrapped `T` is `Send`.

---

## 3. Actor Isolation Guarantees

### 3.1 Purpose

The H2 actor system provides process-level isolation: each actor runs in a fully independent `IfaVM` instance on its own OS thread, communicating exclusively through typed message channels. The system guarantees **no shared mutable state** across actor boundaries.

### 3.2 Architecture

```
Parent VM                          Actor VM (dedicated OS thread)
┌─────────────────────┐            ┌──────────────────────┐
│  globals             │            │  globals (empty)     │
│  opon (private)      │            │  opon (private)      │
│  registry            │            │  registry (fresh)    │
│  resource_registry   │   channel │  resource_registry   │
│  actor_table (shared)│←──────────→│  actor_id = Some(n)  │
└─────────────────────┘  Send/Recv └──────────────────────┘
```

### 3.3 Spawn Protocol

```
spawn_actor(init_fn, bytecode, table, registry, resource_registry) → IfaValue::Actor

1. id = NEXT_ACTOR_ID.fetch_add(1, Relaxed)
2. (tx, rx) = mpsc::sync_channel(ACTOR_INBOX_CAPACITY = 64)
3. handle = ActorHandle { id, tx: Arc::new(tx), resource_registry }
4. table.insert(handle.clone())
5. spawn_actor_task(move || actor_loop(id, init_fn, rx, &bytecode, &table, registry, ...))
6. Return IfaValue::Actor { id, handle: Arc::new(handle) }
```

`spawn_actor_task` uses `tokio::runtime::Handle::spawn_blocking` (or a dedicated multi-threaded tokio runtime on first call). On WASM targets, the function panics with "not supported on WASM" (`actor.rs:53`).

### 3.4 Message Send Protocol

```
actor_send(actor_value, payload, sender_registry) → Result

1. Downcast handle from IfaValue::Actor
2. freeze:  payload.freeze() → IfaShared
     — deep-copies all value tree nodes
     — fails on closures, futures, functions (non-Send types)
3. thaw:    IfaShared.thaw() → IfaValue
     — converts back to thread-local representation
4. transfer_resources:
     — traverse value tree for ResourceToken nodes
     — for each token:
         sender_registry.take(token) → Resource
         recipient_registry.insert_raw(token, resource, actor_id)
5. handle.tx.try_send(ActorMsg::Value(safe_value))
     — fails with Full (back-pressure) or Disconnected (actor exited)
```

### 3.5 Actor Loop

```
actor_loop(id, handler, rx, bytecode, table, registry, resource_registry):

1. Set thread-local actor ID (ActorIdGuard)
2. vm = IfaVM::new()
   vm.actor_id = Some(id)
   vm.registry = registry
   vm.resource_registry = resource_registry
3. while let Ok(msg) = rx.recv():
     match msg:
       Shutdown → break
       Value(value) →
         args = [value]
         match vm.spawn_task(handler, args):
           Future(cell) → vm.await_future(&cell, bytecode)
           _ → continue
4. table.remove(id)
```

### 3.6 Freeze/Thaw Semantics

#### Freeze: `IfaValue::freeze() → Result<IfaShared>`

| Input variant   | Output variant      | Notes                          |
|----------------|---------------------|--------------------------------|
| `Int(i)`       | `IfaShared::Int(i)` | copyless                       |
| `Float(f)`     | `IfaShared::Float(f)`| copyless                      |
| `Bool(b)`      | `IfaShared::Bool(b)` | copyless                      |
| `Null`         | `IfaShared::Null`   | copyless                       |
| `Str(s)`       | `IfaShared::Str(s.as_str().into())` | `Arc<str>` allocation |
| `List(l)`      | `IfaShared::List(...)` | deep copy each element recursively |
| `Map(m)`       | `IfaShared::Map(...)` | deep copy each (k,v) pair recursively |
| `Resource(t)`  | `IfaShared::Resource(token)` | copyless                  |
| `Fn/Closure/Future/Actor/Upvalue/Return/Break/Continue` | **Error** | non-Send types rejected |

#### Thaw: `IfaShared::thaw() → IfaValue`

Reverses the freeze mapping. Strings become `CompactString` via `Arc::as_ref().into()`. Collections are deep-copied recursively. `IfaShared::Fn` thaws to `IfaValue::Null` (function values cannot cross the boundary).

### 3.7 Invariants

1. **No shared globals**: Each actor VM has a fresh `GlobalState`. The parent's globals are never visible.

2. **No shared Opon**: Each actor VM has a fresh `Opon`. The parent's memory region is never accessible.

3. **No shared registry**: Each actor VM receives its own `Box<dyn OduRegistry>`. Registration does not cross boundaries.

4. **Send barrier**: `freeze()` rejects any value that contains a non-`Send` variant (`Fn`, `Closure`, `Future`, `Upvalue`, `Actor`, `Return`, `Break`, `Continue`). This is the compiler-enforced "Hut cannot go to Market without a ritual" rule.

5. **Deep-copy semantics**: After freeze+thaw, the recipient's value is structurally equal but shares no `Arc` pointers with the sender's original. Mutations to one do not affect the other.

6. **Resource ownership is exclusive**: `transfer_resources` removes each `ResourceToken` from the sender's `ResourceRegistry` and inserts it into the recipient's. A resource is owned by exactly one actor at any time.

7. **Bounded channels**: `ACTOR_INBOX_CAPACITY = 64` (`actor.rs:173`). `try_send` returns `Full` when the channel is saturated, providing back-pressure. The channel is synchronous (`sync_channel`), so the sender blocks the sender's thread when the buffer is full (the `try_send` variant returns immediately with an error instead).

8. **Cooperative shutdown**: The actor loop processes in-flight messages before exiting on `Shutdown`. The loop exits cleanly, and `table.remove(id)` is always reached (barring a panic, which is caught by the tokio task).

9. **Thread-local actor ID**: `ActorIdGuard` sets a thread-local actor ID on entry and clears it on drop (including panics). This ensures VM-internal APIs can distinguish actor vs. non-actor contexts (`actor.rs:57-70`).

---

## 4. Taboo Enforcement Rules

### 4.1 Purpose

The Eewo (taboo) enforcer validates architectural constraints at static analysis time. It prevents forbidden dependency patterns between Odu domains and ensures thread-safety for cross-boundary value flows. Architectural constraints become compiler-enforceable law rather than developer discipline.

### 4.2 Taboo Rule Syntax

```
Taboo {
    source_domain: String    — the caller domain (lowercased)
    source_context: String   — the caller context label (e.g., "UI", "Backend")
    target_domain: String    — the callee domain (lowercased)
    target_context: String   — the callee context label
    is_wildcard: bool        — if true, blocks ALL calls from source_domain
}
```

### 4.3 Rule Registration

Four registration methods:

| Method | Semantics |
|--------|-----------|
| `add_taboo(src_domain, src_ctx, tgt_domain, tgt_ctx, false)` | Specific: blocks caller in src_domain+src_ctx from calling tgt_domain+tgt_ctx |
| `add_wildcard_taboo(domain)` | Blocks any call *to* `domain` from any caller |
| `set_context(name)` | Sets the active context label for subsequent checks |
| `add_taboo("src", "", "", "", false)` | Blocks ALL calls from src_domain to any target |

Domains are lowercased for case-insensitive matching. Empty strings in context or domain match universally (wildcard within the rule).

### 4.4 Call Check Semantics

```
check_call(caller_domain, callee_domain, line, col) → bool (allowed)

for each taboo in self.taboos:
    if taboo.is_wildcard:
        if callee == taboo.source_domain:       // lowercase comparison
            record violation → return false
    else:
        source_match  = (caller == taboo.source_domain ∨ taboo.source_domain.is_empty())
        context_match = (current_context == taboo.source_context ∨ taboo.source_context.is_empty())
        target_match  = (callee == taboo.target_domain ∨ taboo.target_domain.is_empty())
        
        if source_match ∧ context_match ∧ target_match:
            record violation → return false

return true   // allowed
```

### 4.5 Thread-Safety Check

```
check_thread_safety(value_type, target_context) → bool (safe)

let forbidden_types   = ["IfaValue", "Rc", "RefCell", "GcPtr", "Hut"]
let shared_contexts   = ["Osa", "Thread", "Spawn", "Market"]

let is_forbidden  = any forbidden_type in value_type (substring match)
let is_shared     = any shared_context in target_context (case-insensitive substring)

if is_forbidden ∧ is_shared:
    record violation "The Hut (Local) → The Market (Shared)"
    return false
else:
    return true
```

### 4.6 Violation Output

Format (`taboo.rs:179-217`):
```
ÈÈWỌ̀ VIOLATIONS (Taboo Broken):

  Line 12: Called forbidden domain 'otura'
    -> Taboo: ose.* is not allowed

  Line 15: 'odi' called 'otura'
    -> Context 'Backend' cannot access 'otura'

Proverb: "Ẹni tó bá fọwọ́ kan èèwọ̀, yóò rí àṣèdá"
(Whoever touches a taboo will see the consequences)
```

### 4.7 Invariants

1. **Taboos are immutable after registration**: The `TabooEnforcer` records violations but does not modify registered rules during checking.

2. **Violation accumulation**: `check_call` and `check_thread_safety` append to the internal `violations` vector. The enforcer does not clear violations between checks.

3. **Wildcards take priority**: A wildcard taboo (e.g., `add_wildcard_taboo("odi")`) is checked first in the iteration order. If both a specific and a wildcard match, the wildcard's violation is recorded.

4. **Empty-string universal matching**: In a non-wildcard taboo, an empty `source_domain` matches any caller. An empty `source_context` matches any context. An empty `target_domain` matches any callee. This allows rules like "from any context in ose, do not call odi" (`add_taboo("ose", "UI", "odi", "")`) or "from ose, do not call anything" (`add_taboo("ose", "", "", "")`).

5. **Context is an explicit directive**: The programmer must call `set_context()` for context-sensitive rules to activate. The default context is empty, which by (4) matches any context — so rules with explicit source contexts are *not* enforced until a context is set.

6. **Thread-safety rules are advisory**: `check_thread_safety` operates on *type name strings*, not on actual Rust type information. It is a naming-convention validator, not a compile-time trait-bound check. The actual Send/Sync enforcement happens at runtime via `freeze()` failure (`value_union.rs:414-418`).

---

## 5. Cross-Cutting Invariants

### 5.1 Memory Isolation

| Boundary | Mechanism | Enforcement |
|----------|-----------|-------------|
| Between Opon epochs | `truncate(start_addr)` on epoch end | Opon API + compiler EpochBegin/EpochEnd pairing |
| Between VM instances | Separate `Vec<IfaValue>`, `GlobalState`, `ResourceRegistry` | Spawn creates fresh VM; no cross-VM pointers |
| Between threads | Freeze+Thaw deep copy | `freeze()` rejects non-Send types; `transfer_resources` moves ownership |

### 5.2 Resource Ownership

1. Every `ResourceToken` is owned by exactly one `ResourceRegistry` at any time.
2. Transfer across actor boundaries is an explicit `take` from sender + `insert_raw` into recipient.
3. No `Clone` for `ResourceToken` — copies are prevented by `Arc<ResourceToken>` indirection and the fact that `freeze()` is the only serialization path for actor messages.

### 5.3 Exception Safety

1. `attempt_recovery` (`vm.rs:1201-1252`) unwinds the stack and call frames to the recovery point.
2. Stack truncation triggers `Drop` on Ebo guards, running all RAII cleanup.
3. After catching, if a `finally_ip` exists, a sentinel `RecoveryFrame{can_catch: false}` remains to ensure the finally block runs on any exit path (return/throw/error from the catch body).

### 5.4 Actor Thread Lifecycle

```
spawn → actor_loop starts on new OS thread
         │
         ├─ recv(Shutdown) → break → table.remove(id) → thread exits
         │
         ├─ recv(Value) → spawn_task → await_future → continue loop
         │
         └─ channel disconnect → recv returns Err → loop exits → table.remove(id) → thread exits
```

The tokio `spawn_blocking` task ensures the OS thread is returned to the pool on exit.

---

## 6. Glossary

| Term | Definition |
|------|-----------|
| **Opon** | Sacred divination tray; the memory container (linear slot array with region epochs) |
| **Ebo** | Sacrifice; RAII scope with deterministic cleanup |
| **EboEpoch** | A scoped allocation region within the Opon |
| **H2 Actor** | Fully isolated IfaVM on a dedicated OS thread, message-passing via bounded channels |
| **Freeze** | Deep-copy conversion from `IfaValue` (thread-local) to `IfaShared` (Send+Sync) |
| **Thaw** | Conversion from `IfaShared` back to `IfaValue` |
| **Ofun** | Capability definition (filesystem, network, execute, bridge, etc.) |
| **CapabilitySet** | Granted + sacrificed capabilities with inheritance |
| **Eewo (Taboo)** | Architectural constraint enforced statically by the Babalawo |
| **TabooEnforcer** | Validates call graphs and thread-safety against registered taboo rules |
| **RecoveryFrame** | Saved VM state for try/catch unwinding (stack depth, call depth, catch/finally IPs) |
| **Ikin** | Sacred palm nuts; runtime constant pool |
| **Opele** | Divination chain; audit trail (flight recorder events) |

---

*Specification version 0.1. Corresponds to crates as of build. All formal notations use postcondition style: `pre / rule / post` for state transitions. Invariants are prefixed with numbered assertions.*
