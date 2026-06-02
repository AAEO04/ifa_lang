# Memory Management & Concurrency Architecture

**Audit date:** 2026-05-29  
**Codebase:** `crates/` tree

---

## 4. Memory Management

### 4.1 Opon — Region/Arena-Based Memory

The Opon system at `ifa-vm/src/opon.rs` (499 lines) is the **primary memory model** and the only allocator for the Ifá VM.

#### 4.1.1 Four Region Sizes

```rust
pub enum OponSize {
    Kekere,    // 256 slots (~4KB)
    Arinrin,   // 4096 slots (~64KB) — default
    Nla,       // 65536 slots (~1MB)
    Ailopin,   // usize::MAX (dynamic, Vec grows), but hard-capped at 1,048,576
}
```

Selected per-program or per-epoch via `#opon kekere;` / `#opon nla;` directive (`grammar.pest:53-59`). Yoruba names map to fixed sizes: `kekere` = tiny, `arinrin` = medium, `nla` = large, `ailopin` = unlimited.

#### 4.1.2 EboEpoch — Scoped Allocation Regions

**The EboEpoch system provides a form of bump-allocation with region-based deallocation:**

- `begin_epoch(name)` at `opon.rs:322`: Records current `memory.len()` as the epoch start.
- `allocate(value)` at `opon.rs:286`: Pushes to `memory: Vec<IfaValue>`, tracks high-water mark.
- `end_epoch()` at `opon.rs:335`: Truncates `self.memory` back to `epoch.start_addr`.

**No per-object free.** Every allocation in an epoch is freed simultaneously when the epoch ends. This is a **region/arena allocator** in the style of Apache's APR or Zig's arenas.

#### 4.1.3 Flight Recorder

A circular buffer (256 events, configurable) recording `(spirit, action, value)` triplets at `opon.rs:359-372`. Used for debugging and the Babalawo's diagnostic trace.

#### 4.1.4 Classification

| Criteria | Opon |
|----------|------|
| **Pattern** | Region/Arena (bump allocation, bulk deallocation) |
| **Allocation** | `Vec::push` — linear bump |
| **Deallocation** | `Vec::truncate` — O(n) bulk truncate |
| **Fragmentation** | Zero — regions are contiguous and freed whole |
| **Determinism** | Fully deterministic — fixed-size regions, no GC pauses |
| **Thread safety** | Single-threaded (per-VM `Opon` not `Sync`) |
| **Memory waste** | Pre-allocates maximum region size (except `Ailopin`) |

### 4.2 Reference Counting

**Every heap-allocated `IfaValue` variant uses `Arc<T>`:**

| Variant | Lines | Heap Type |
|---------|-------|-----------|
| `Str` | 46 | `CompactString` (inline ≤21B, else `Arc<str>`) |
| `List` | 47 | `Arc<Vec<IfaValue>>` |
| `Map` | 48 | `Arc<HashMap<CompactString, IfaValue>>` |
| `Fn` | 51 | `Arc<BytecodeFnData>` |
| `Closure` | 63 | `Arc<ClosureData>` (env: `Arc<Vec<UpvalueCell>>`) |
| `Future` | 66 | `FutureCell = Arc<Mutex<FutureState>>` |
| `Actor` | 71 | `Arc<dyn Any + Send + Sync>` |
| `Resource` | 80 | `Arc<ResourceToken>` |

**Key design decision**: `Arc` (atomic refcounting) is used everywhere, not `Rc` (non-atomic). This enables `Send + Sync` on `IfaValue` so it can cross thread boundaries in the actor system, but imposes an atomic increment/decrement on every `Clone`/`Drop`.

**`CompactString`** at `compact_str.rs:8-15`: Strings ≤21 bytes stored inline (no heap allocation). Larger strings use `Arc<str>` — zero-copy sharing across threads.

**Identity checks**: `Arc::ptr_eq` at `value_union.rs:361,370,385` for detecting same-object references without deep equality.

#### 4.2.1 Classification

| Criteria | `Arc` refcounting |
|----------|-------------------|
| **Throughput** | Atomic RMW per clone/drop — moderate contention on multi-core |
| **Latency** | Deterministic O(1) per increment/decrement — no pause |
| **Cycles** | Not handled — strong refs only; no `Weak` in data model |
| **Thread safety** | `Send + Sync` — safe across threads |
| **Memory overhead** | 16 bytes per `Arc` control block + type tag per variant |

### 4.3 Garbage Collection

**There is NO garbage collector.** The design explicitly avoids GC (`opon.rs:140`: "deterministic memory management without garbage collection").

The only function named `gc` is at `ajose.rs:305` — it prunes dead `Weak` references from the reactive signal graph; it is not a memory GC.

**Why no GC**: Consistent with the performance philosophy (memory > latency > throughput). GC would add unpredictable pause times and memory overhead.

### 4.4 Manual Memory Management

**Not available at the Ifá level.** There is no `malloc`/`free` equivalent, no `alloc`/`dealloc` keywords, no `unsafe` pointer dereference (the `Ptr`/`Ref`/`RefMut` type hints exist in the AST at `ast.rs:336-341` but are aspirational with no corresponding VM operations).

The `Odi` domain (`odi.si` / `odi.pa` for file open/close) provides resource lifecycle management but not raw memory management.

### 4.5 Memory Architecture Design Evaluation

```
Hierarchy of memory strategies in Ifá-Lang:

  Opon (Region) ← Primary: deterministic, scoped, no fragmentation
    ↓
  Arc (Refcounting) ← Values escaping region scope
    ↓
  No GC ← Explicit design choice
```

**Strengths**:
- Opon epochs provide cheap, deterministic allocation/deallocation for short-lived scopes (function bodies, request handling).
- No GC pauses — latency is predictable.
- Flight recorder enables post-mortem memory debugging.

**Weaknesses**:
- `Arc` on every heap value is heavyweight — single-threaded workloads pay atomic costs unnecessarily.
- No `Weak` references in data model — all lists/maps hold strong refs (potential reference cycles that leak).
- Opon's `Vec::truncate` is O(n) for epoch deallocation (the `Vec` doesn't actually free memory — it just adjusts `len`, leaving capacity allocated).
- No generational awareness — all allocations are in one flat region.
- No integration with the OS memory map (no `mmap`, no huge pages).

**Comparison against design goals**:

| Goal | Alignment |
|------|-----------|
| Memory usage (ranked #1) | Opon pre-allocates maximum capacity (wasteful for small programs) |
| Startup time (#2) | Bump allocation is fast; no GC init |
| Latency (#3) | No GC pauses; `Arc` atomic ops add small per-operation cost |
| Throughput (#4) | `Arc` contention scales poorly; Opon bump is fast |

---

## 5. Concurrency Model

### 5.1 Threads (OS-level)

**Actor system** (`ifa-vm/src/actor.rs`, 412 lines):

Each actor is an **OS thread** running a fully isolated `IfaVM`:

- **Spawn**: `spawn_actor()` at `actor.rs:179` — creates VM, assigns unique atomic u64 ID, spawns OS thread via `tokio::spawn_blocking`.
- **Communication**: `mpsc::sync_channel<ActorMsg>` with capacity 64 — bounded back-pressure.
- **Isolation**: Actor VMs share **nothing** — no globals, no Opon, no registry.

```
┌─────────────┐     mpsc::sync_channel(64)     ┌─────────────┐
│   VM #1     │ ──────────────────────────────→ │ Actor VM #2 │
│  (caller)   │   ActorMsg { IfaValue }         │  (handler)   │
└─────────────┘                                 └─────────────┘
                                                      │
                                              ┌───────┴───────┐
                                              │   ActorTable   │
                                              │ (process-wide) │
                                              └───────────────┘
```

**`ActorHandle`** at `actor.rs:94-101`: Cheaply cloneable (`Arc<SyncSender<ActorMsg>>`), `Send + Sync`.

**`ActorMsg`** at `actor.rs:75-81`:
```rust
pub enum ActorMsg {
    Value(IfaValue),
    Shutdown,
}
```
Marked `unsafe impl Send` at `actor.rs:86` — since all `IfaValue` variants use `Arc`, the impl is safe, but there is no static guard if a non-`Send` variant is added.

**Resource transfer** at `actor.rs:226-258`: Before sending, resource tokens owned by the sender are moved to the recipient's registry. Enforced by Babalawo (resources are linear — cannot be used after sending).

### 5.2 Async/Await

**Cooperative task scheduling** within a single VM:

- **`spawn_task(func, args)`** at `vm.rs:1177`: Creates a `FutureCell` in pending state, pushes a `Task` (captured `ExecutionContext` clone) onto `task_queue: VecDeque<Task>`.
- **`poll_one_task(bytecode)`** at `vm.rs:1097`: Pops front of queue, runs one "slice" of the task (executes until `yield` or completion).
- **`await_future(cell, bytecode)`** at `vm.rs:1115`: Polls repeatedly — checks `FutureState`, if pending calls `poll_one_task`.

```
VM task_queue:
  ┌──────┬──────┬──────┬──────┐
  │ Task │ Task │ Task │ ...  │ → pop_front → execute slice → push_back (if incomplete)
  └──────┴──────┴──────┴──────┘
```

**`Yield (0x56)`**: Cooperative yield point at `vm.rs:1803` — returns `IfaError::Yielded` to suspend until the task is rescheduled.

**`Await (0x58)`** at `vm.rs:1545`: Pops `FutureCell`, calls `await_future` to spinloop-poll until ready.

**`async_return` flag** at `CallFrame::async_return`: Marks frames that should wrap their return value in a `Future` rather than pushing directly.

**Architecture**:
- Single-threaded event loop (tasks share the same VM).
- Cooperative (no preemption — tasks must `yield` or complete).
- Futures are `Arc<Mutex<FutureState>>` — polling checks `FutureState::Pending` / `Ready`.
- No I/O reactor — the task queue is the only scheduling mechanism.

### 5.3 Data Parallelism

**`OpCode::ParallelFor (0x5C)`** — `iwori.yipo.ori(list, closure)`:

- **Compiler** (`compiler/src/lib.rs:1724`): Checks `call.domain == Iwori && call.method == "yipo.ori"`, emits `ParallelFor`.
- **VM** (`vm.rs:2677`): Pops closure + list, uses **Rayon** for parallel iteration:
  ```
  #[cfg(feature = "parallel")]
  items.par_iter().map_init(|| { new IfaVM() }, |worker_vm, item| {
      worker_vm.spawn_task(closure.clone(), vec![item.clone()])
  })
  ```
- Each Rayon worker gets a **fresh `IfaVM`** sharing `globals`.
- Sequential fallback when `parallel` feature is disabled.

Feature-gated at `Cargo.toml`: `parallel = ["rayon"]`, enabled by default.

### 5.4 Actor System (Message-Passing)

Already covered in §5.1 — the actor model is the primary concurrency primitive.

**Key design points**:
- **OS threads per actor** (not green threads / goroutines).
- **Bounded channels** (sync_channel 64) for back-pressure.
- **Resource transfer** for ownership-based message passing.
- **`freeze`/`thaw`** for crossing thread boundaries (`IfaValue → IfaShared → IfaValue`).

### 5.5 CSP (Communicating Sequential Processes)

**No separate CSP implementation.** The actor system uses `std::sync::mpsc::sync_channel` which follows the CSP paradigm (channel-based communication between sequential processes), but there is:

- No `select` statement for multiplexing across channels.
- No buffered/unbuffered channel distinction (all actors use capacity 64).
- No channel closing semantics (only `Shutdown` message).

### 5.6 What's Missing

| Mechanism | Status | Gap |
|-----------|--------|-----|
| **Green threads / Fibers** | ❌ | Actors are OS threads (heavy); async tasks are cooperative but single-threaded. No M:N scheduling. |
| **STM** | ❌ | No software transactional memory. |
| **SIMD** | ❌ | No explicit SIMD types or auto-vectorization. |
| **Deterministic execution** | ⚠️ Partial | Opon epochs are deterministic; `parallel_for` with Rayon is not. No seed control for RNG determinism. |
| **Thread-safe mutation** | ❌ | No `Arc<RwLock<T>>` exposed at language level — actors provide isolation but no shared-state concurrency. |
| **Atomics** | ❌ | No `AtomicInt`, `compare_and_swap`, `memory_order` at language level. |
| **Barriers / Latches** | ❌ | No `Barrier`, `CountDownLatch`, `WaitGroup`. |
| **Mutex / Lock** | ❌ | No language-level mutex — isolation is by actor boundary only. |

### 5.7 Architecture Diagram

```
Ifá-Lang Concurrency Model
═══════════════════════════

SINGLE-THREADED              MULTI-THREADED
─────────────────            ────────────────

IfaVM (main thread)           ActorTable
│                              │
├── task_queue                ├── Actor VM #1 (OS thread)
│   ├── Task A                │   └── isolated IfaVM
│   ├── Task B                │
│   └── Task C (yielded)      ├── Actor VM #2 (OS thread)
│                              │   └── isolated IfaVM
├── poll_one_task() ←─────────┤
│   (cooperative)              └── Actor VM #3 (OS thread)
│                                  └── isolated IfaVM
├── dispatch_parallel_for()
│   └── rayon::par_iter()
│       ├── Worker #1 (thread pool)
│       ├── Worker #2
│       └── Worker #3
│
└── await_future()
    └── poll tasks until ready

Communication:
  Actor X ──sync_channel(64)──→ Actor Y
                                              
  freeze/thaw for cross-VM data transfer
  Resource tokens transferred on send
```

### 5.8 Design Evaluation

**Strengths**:
- Actor isolation prevents data races by construction.
- `freeze`/`thaw` provides safe, explicit cross-thread communication.
- Resource transfer enforces linearity (no resource leaks across actors).
- Bounded channels prevent unbounded memory growth under back-pressure.
- Parallel `for` leverages Rayon for data parallelism with thread-local VM isolation.

**Weaknesses**:
- OS-thread-per-actor is heavy (8MB stack per thread, context-switch overhead). 1,000 actors = 8GB virtual memory.
- No M:N scheduling — cannot have 10,000 lightweight actors like Erlang/Go.
- Cooperative async means a single long-running task blocks all other tasks.
- No async I/O reactor — `await` is task-scheduling only, not I/O multiplexing.
- `unsafe impl Send for ActorMsg` at `actor.rs:86` has no compile-time protection.
- Parallel for shares globals without synchronization (potential race on global mutation).

**Comparison against design goals**:

| Goal | Alignment |
|------|-----------|
| Memory (#1) | OS threads per actor is memory-heavy (contradicts priority) |
| Latency (#3) | No preemption → cooperative yields keep latency predictable |
| Throughput (#4) | Actor isolation eliminates locking overhead; no shared-memory throughput |
| Determinism | Opon yes, parallel for no — inconsistent |
