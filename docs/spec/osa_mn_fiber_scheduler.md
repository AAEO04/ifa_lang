# Ọ̀sá: M:N Stackful Fiber Scheduler

**Status:** `DRAFT`  
**Supersedes:** OS-thread-per-actor model (`opon-ebo-actor-taboo-spec.md` §3.3, `actor.rs:38-55`)  
**Crate:** `ifa-vm` (`src/actor.rs`, `src/vm.rs`)

---

## 1. Purpose

The current actor system spawns one OS thread per actor via `tokio::spawn_blocking` (`actor.rs:45-48`). Each OS thread consumes ~8MB of virtual address space for its stack. At 1,000 actors, this is 8GB of virtual memory. At 10,000, the kernel's context-switch overhead dominates.

This spec replaces the 1:1 OS-thread model with an M:N stackful fiber scheduler, where M user-space fibers (actors) are multiplexed across N OS worker threads.

---

## 2. Architecture

### 2.1 Components

```
┌─────────────────────────────────────────────────────┐
│                    Osa Scheduler                     │
│                                                     │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐             │
│  │Worker #0│  │Worker #1│  │Worker #2│  ... (N)     │
│  │(OS thrd)│  │(OS thrd)│  │(OS thrd)│              │
│  └────┬────┘  └────┬────┘  └────┬────┘              │
│       │            │            │                    │
│  ┌────▼────────────▼────────────▼────┐              │
│  │         Global Run Queue          │              │
│  │  [Fiber] [Fiber] [Fiber] [Fiber]  │              │
│  └───────────────────────────────────┘              │
│                                                     │
│  ┌───────────────────────────────────┐              │
│  │         I/O Reactor (epoll)       │              │
│  │  Monitors fds, timers, signals    │              │
│  └───────────────────────────────────┘              │
└─────────────────────────────────────────────────────┘
```

### 2.2 Fiber

A Fiber is a user-space execution context containing a saved register set and a small stack.

```rust
pub struct Fiber {
    /// Unique fiber ID (monotonic u64, same semantics as current actor ID)
    id: u64,
    /// Saved CPU register state (stack pointer, instruction pointer, callee-saved)
    context: FiberContext,
    /// The fiber's private stack allocation
    stack: FiberStack,
    /// The fiber's isolated IfaVM instance
    vm: Box<IfaVM>,
    /// Current scheduling state
    state: FiberState,
    /// Inbox channel (same mpsc::sync_channel as current ActorMsg)
    inbox: mpsc::Receiver<ActorMsg>,
    /// Sender half (stored in ActorHandle, shared via Arc)
    tx: Arc<mpsc::SyncSender<ActorMsg>>,
}

pub enum FiberState {
    /// Ready to run — in the global run queue
    Ready,
    /// Currently executing on a worker thread
    Running,
    /// Waiting for I/O completion
    IoBlocked { fd: RawFd, interest: Interest },
    /// Waiting for inbox message (no messages available)
    ReceiveBlocked,
    /// Completed — will be cleaned up
    Dead,
}
```

### 2.3 Fiber Stack

```rust
pub struct FiberStack {
    /// mmap-allocated stack memory
    base: *mut u8,
    /// Current stack size (starts at initial, grows to max)
    size: usize,
    /// Guard page at the bottom for stack overflow detection
    guard_page: *mut u8,
}

const FIBER_STACK_INITIAL: usize = 8 * 1024;     // 8 KB (vs 8 MB for OS thread)
const FIBER_STACK_MAX: usize = 1024 * 1024;       // 1 MB maximum growth
const FIBER_GUARD_PAGE_SIZE: usize = 4096;         // 4 KB guard page
```

Initial stack is 8KB. Growth is triggered by guard page fault (SIGSEGV handler on Unix, VEH on Windows). The guard page is remapped as the stack grows, up to `FIBER_STACK_MAX`.

### 2.4 Worker Threads

```rust
pub struct Scheduler {
    /// OS worker threads (N = num_cpus by default)
    workers: Vec<JoinHandle<()>>,
    /// Shared run queue (lock-free MPMC queue)
    run_queue: Arc<SegQueue<FiberId>>,
    /// Fiber storage (all fibers indexed by ID)
    fibers: Arc<DashMap<u64, Fiber>>,
    /// I/O reactor thread
    reactor: ReactorHandle,
    /// Worker count
    num_workers: usize,
}
```

**Worker count:** `N = std::thread::available_parallelism()` by default. Configurable via `#osa workers 4;` directive.

**Scheduling algorithm:** Work-stealing with a global fallback queue. Each worker has a local deque. When empty, it steals from other workers' deques. When all local deques are empty, it pops from the global queue.

### 2.5 Context Switch

Context switching between fibers is a pure user-space register save/restore. No kernel transition.

```
switch_fiber(current, next):
    1. Save current's registers (rsp, rbp, rbx, r12-r15) to current.context
    2. Load next's registers from next.context
    3. Jump to next's saved instruction pointer
```

On x86_64, this is approximately 20 instructions (~10ns). An OS thread context switch costs ~1-5μs (100-500x slower).

---

## 3. spawn_actor Migration

### 3.1 Current (1:1)

```rust
// actor.rs:199 — current implementation
spawn_actor_task(move || {
    actor_loop(id, init_fn, rx, &bytecode, &table, registry, resource_registry);
});
// spawn_actor_task calls tokio::spawn_blocking — creates an OS thread
```

### 3.2 Proposed (M:N)

```rust
pub fn spawn_actor(...) -> IfaResult<IfaValue> {
    let id = next_actor_id();
    let (tx, rx) = mpsc::sync_channel::<ActorMsg>(ACTOR_INBOX_CAPACITY);

    let fiber = Fiber::new(id, rx, Arc::new(tx.clone()));
    // Initialize the fiber's VM — identical isolation guarantees
    fiber.vm.actor_id = Some(id);
    fiber.vm.registry = registry;
    fiber.vm.resource_registry = resource_registry;

    // Place fiber in the scheduler's run queue — no OS thread created
    SCHEDULER.spawn(fiber);

    let handle = ActorHandle { id, tx: Arc::new(tx), resource_registry };
    table.insert(handle.clone());
    Ok(IfaValue::Actor(Arc::new(ifa_types::ActorData { id, handle: Arc::new(handle) as _ })))
}
```

### 3.3 API Compatibility

The public API (`Osa.ran`, `Osa.ise`, `ActorHandle::send`, `ActorHandle::shutdown`) remains **identical**. The change is purely internal to `spawn_actor_task` and `actor_loop`.

---

## 4. I/O Reactor Integration

### 4.1 Problem

If a fiber makes a blocking syscall (e.g., `read()`, `connect()`), the entire OS worker thread blocks, freezing all other fibers on that worker.

### 4.2 Solution

All blocking I/O operations in the standard library (`Odi`, `Otura`) are replaced with non-blocking equivalents:

| Domain | Current | Proposed |
|--------|---------|----------|
| `Odi.si` (file open) | `std::fs::File::open` (blocking) | `io_uring` / `epoll` + fiber yield |
| `Odi.ka` (file read) | `std::io::Read::read` (blocking) | Non-blocking read + fiber yield |
| `Otura.get` (HTTP) | `ureq::get` (blocking) | Non-blocking socket + fiber yield |
| `Otura.so` (connect) | `TcpStream::connect` (blocking) | Non-blocking connect + reactor wait |

### 4.3 Yield Protocol

When a fiber encounters an I/O operation:

```
1. Convert fd to non-blocking mode
2. Attempt the operation
3. If EAGAIN/EWOULDBLOCK:
    a. Register fd + interest (Read/Write) with the I/O reactor
    b. Set fiber state to IoBlocked { fd, interest }
    c. Remove fiber from run queue
    d. Context-switch to next ready fiber
4. When reactor detects fd readiness:
    a. Set fiber state to Ready
    b. Push fiber back onto run queue
5. When fiber resumes, retry the operation (now ready)
```

---

## 5. Fiber Panic Handling

When a fiber panics:

1. The panic is caught by `std::panic::catch_unwind` at the fiber entry point.
2. The fiber's `IfaVM` is dropped (running all `Ebo` RAII cleanup).
3. The fiber is removed from the `ActorTable`.
4. The fiber's inbox channel is dropped (senders receive `Disconnected` on next `try_send`).
5. The worker thread is **not** affected — it simply picks up the next fiber.

---

## 6. Invariants

1. **Isolation preserved**: Each fiber contains its own `Box<IfaVM>`. No shared globals, no shared Opon, no shared registry. Identical to the current OS-thread model.

2. **Message semantics preserved**: `actor_send` still uses `value.clone()` (or `yanda` move). The channel type (`mpsc::sync_channel`) is unchanged.

3. **No preemption**: Fibers are cooperatively scheduled. A fiber yields at I/O boundaries, `await` points, and explicit `Yield` opcodes. A CPU-bound fiber that never yields will monopolize its worker thread. (Future: add periodic yield injection at loop back-edges.)

4. **Guard page safety**: Stack overflow in a fiber triggers SIGSEGV on the guard page, which the signal handler converts to an `IfaError::StackOverflow` rather than crashing the process.

5. **Graceful degradation**: On platforms where `mmap` is unavailable (WASM, some embedded), `spawn_actor` falls back to the current `tokio::spawn_blocking` model. The `#[cfg(target_arch = "wasm32")]` panic at `actor.rs:53` remains.

---

## 7. Relationship to Other Specs

- **Supersedes** `opon-ebo-actor-taboo-spec.md` §3.3 (Spawn Protocol) and §3.5 (Actor Loop). All other sections of that spec (Opon, Ebo, Taboo, Freeze/Thaw) are unaffected.
- **Orthogonal to** [iwori_arc_rc_duality.md](iwori_arc_rc_duality.md): The Rc/Arc toggle and fiber scheduling are independent concerns. Single-threaded Rc mode would use a single worker thread with M fibers.
- **Orthogonal to** [opon_slab_allocator.md](opon_slab_allocator.md): Slab allocation is a per-VM memory concern. Each fiber's `Box<IfaVM>` would contain its own slab.
- **Depends on** [effect_system.md](effect_system.md): The `effects(Async)` annotation ensures that only functions declaring async effects can trigger fiber yields.
