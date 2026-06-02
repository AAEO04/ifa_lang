# The Opon Slab Allocator (Zero-Copy Architecture Phase 3)

**Status:** `DRAFT`  
**Supersedes:** Phase 2 Ìwòrì Arc/Rc toggle (if this is implemented, Phase 2 becomes unnecessary)  
**Crate:** `ifa-types` (`src/value_union.rs`), `ifa-vm` (`src/vm.rs`, `src/actor.rs`)

## Motivation

While the Phase 1 "Logical Move" semantics correctly enforce a zero-copy transfer of `IfaValue` objects across actor boundaries using the `yanda` keyword, the underlying memory model remains built on Rust's `Arc<Mutex>` and `Arc<RwLock>` reference counting. 

This means that even though we skip deep copying data during an `Osa.ran` (actor send) operation, dropping the transferred variables eventually results in an atomic `lock xadd` instruction to decrement the reference count. In heavily concurrent, highly-scaled actor topologies, this cache-line bouncing limits linear scalability.

To achieve theoretical maximum performance (similar to Erlang or BEAM), Ifá-Lang proposes Phase 3: **The Opon Slab Allocator**.

## Architecture: Opon (The Divination Tray)

In this architecture, we completely tear out the `Arc` allocation wrappers from `IfaValue`. 

Every Actor (an isolated VM thread) is initialized with an `Opon` — a pre-allocated contiguous block of memory (a Slab allocator).

### 1. Variables as Indices

Instead of being heap-allocated pointers, complex variables (Lists, Maps, Strings, Closures) become simple integers.

```rust
pub enum IfaValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    // Instead of Arc<Vec<IfaValue>>
    List(u32), // Index into the Actor's Opon Slab
    Map(u32),  // Index into the Actor's Opon Slab
    Str(u32),  // Index into the Actor's Opon Slab
}
```

Because an `IfaValue` never exceeds a primitive scalar size (effectively the size of a CPU register), moving an `IfaValue` inside the VM is lightning fast.

### 2. Zero-Copy `yanda` Transfers

When an actor executes `Osa.ran(worker, yanda payload);`:

1.  `Babalawo` (static analysis) ensures the sender's scope immediately loses all read/write capabilities to `payload`.
2.  The Runtime intercepts the `yanda` operation.
3.  Because `payload` is just a `u32` index, the VM treats it as a cross-actor reference. The data physically remains in the sender's `Opon` slab (or a global lock-free message pool for large allocations), but ownership is logically transferred.
4.  The receiver gets the `u32` index and reads directly from the designated memory region.
5.  When the receiver drops the variable, it sends an asynchronous, batched "free this index" message back to the sender's allocator (or the global pool). No TLB shootdowns or OS syscalls are triggered.

### 3. Ikin (Global Constants)

Global constants (string literals, fixed numbers, bytecode function templates) do not belong in an actor's mutable `Opon`. They belong in the global `Ikin` (sacred palm nuts). 

The `Ikin` is a read-only memory region initialized at VM startup. If an `IfaValue` holds a string that was defined at compile-time, it holds an index with a special high-bit set (e.g., `0x8000_0000 | index`) indicating the value should be fetched from the read-only `Ikin` rather than the mutable `Opon`.

## Challenges & Risks

Implementing this requires a fundamental rewrite of the `ifa-types` crate and the `ifa-vm` execution loop.

*   **Garbage Collection**: Because we lose Rust's automatic `Drop` reference counting for `IfaValue`, the `Opon` needs its own garbage collection mechanism. Given the isolated actor model, a **generational semi-space copying collector** per-actor is ideal. This avoids stop-the-world pauses by only pausing individual actors for microseconds.
*   **Borrowing (`Iwa`)**: Borrowing becomes a matter of passing around transient `u32` indices with strict lifetime bounds enforced by `Babalawo`.

---

## Relationship to Other Specs

- **Supersedes [iwori_arc_rc_duality.md](iwori_arc_rc_duality.md)**: If Phase 3 is implemented, there is no `Arc` or `Rc` to toggle. Phase 2 becomes unnecessary.
- **Orthogonal to [osa_mn_fiber_scheduler.md](osa_mn_fiber_scheduler.md)**: Each fiber's `Box<IfaVM>` would contain its own Opon slab. The scheduler is unaffected.
- **Depends on [move_semantics.md](move_semantics.md)**: `yanda` move semantics become a `u32` index handoff instead of an `Arc` pointer clone.
- **Depends on [opon-ebo-actor-taboo-spec.md](opon-ebo-actor-taboo-spec.md)**: The current Opon region/epoch system (§1) would be replaced by the slab allocator. Ebo cleanup (§2) would trigger slab region deallocation instead of `Vec::truncate`.

This document serves as the specification for when the core team decides to execute the full Phase 3 zero-copy memory model.

