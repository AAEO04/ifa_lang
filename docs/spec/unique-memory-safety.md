# Ifa-Lang Unique Memory Safety Techniques

**Status:** `REFERENCE`  
**Scope:** source-verified design review and roadmap  
**Primary code:** `ifa-types::gc`, `ifa-vm::opon`, `ifa-vm::actor`, `ifa-babalawo::movement`, `ifa-babalawo::iwa`

This document describes the memory-safety techniques that make Ifa-Lang distinct as implemented today, plus the next design steps that fit the current architecture. It intentionally separates implemented behavior from proposed extensions.

## Safety Thesis

Ifa-Lang does not rely on a single memory-management technique. Its current model is a layered system:

| Layer | Technique | Enforced by |
|-------|-----------|-------------|
| Heap lifetime | Reference counting plus cycle collection | `IfaGc<T>` in `crates/ifa-types/src/gc.rs` |
| Scoped slot memory | Opon epochs with bulk truncation | `Opon` in `crates/ifa-vm/src/opon.rs` |
| Actor isolation | Fresh `IfaVM` per actor plus serialized message boundaries | `crates/ifa-vm/src/actor.rs` |
| Ownership intent | Explicit `yanda` move expressions with runtime `MoveLocal` opcode | parser/compiler plus Babalawo `MoveTracker` |
| Borrow/lifecycle linting | lexical borrow debts and resource debts (wired into all block types) | `IwaEngine` in `crates/ifa-babalawo/src/iwa.rs` + `checks.rs` |
| State history diagnostic | 32-step circular buffer with lifecycle traceback | `StateHistoryBuffer` in `crates/ifa-babalawo/src/history.rs` |
| Capability safety | capability sets for privileged effects | `ifa-sandbox` and `ifa-types::capability` |

The result is a hybrid model: dynamic memory remains flexible, but actor boundaries, resource lifetimes, and explicit ownership transfers are statically audited.

## Implemented Techniques

### 1. IfaGc: RC plus Cycle Collection

Heap-backed `IfaValue` variants that can form cycles use `IfaGc<T>`. Each allocation has a `CycleHeader` with:

- `strong`: the normal reference count.
- `tracing_rc`: the temporary count used during cycle detection.
- `color`: Bacon-Rajan-style cycle collection color state.
- `buffered`: whether the allocation is in the thread-local suspect buffer.

`IfaGc::clone` increments `strong`. `Drop` decrements `strong`; when an object is not immediately freed but may be part of a cycle, it is marked suspect. The VM periodically calls `collect_cycles()` during bytecode execution.

This gives Ifa-Lang deterministic fast-path reclamation for acyclic values and eventual reclamation for cyclic lists, maps, closures, and upvalues.

Important caveat: `IfaGc` has conditional `Send` and `Sync` implementations when the payload is `Send + Sync`. That means per-actor heap isolation is a runtime architecture rule, not a hard type-level invariant of `IfaGc` itself.

### 2. Opon: Epoch-Scoped Slot Memory

`Opon` is the VM slot store. It is backed by `Vec<IfaValue>` and supports configured capacities through `#opon`:

| Directive | Intent |
|-----------|--------|
| `#opon kekere` | constrained embedded-style slot budget |
| `#opon arinrin` | default slot budget |
| `#opon nla` | large workload slot budget |
| `#opon ailopin` | dynamically growing mode with a host safety ceiling |

`Opon::allocate(value)` appends a value and returns its slot address. `Opon::begin_epoch(name)` records the current slot length. `Opon::end_epoch()` truncates the vector back to the epoch start.

This is region-style memory management: scoped allocations are released in bulk when the epoch ends. The current implementation is not purely arena-only because `try_set(addr, value)` can resize and overwrite arbitrary slots. Treat Opon as an epoch-aware slot store, not a strict bump-only allocator.

### 3. Ebo Epochs: Deterministic Region Cleanup

The compiler emits `EpochBegin` and `EpochEnd` bytecode for ebo blocks. The VM maps those opcodes to `Opon::begin_epoch` and `Opon::end_epoch`.

This gives Ifa-Lang deterministic cleanup for temporary VM slot state. The safety value is strongest when values are used inside the epoch and not exposed as raw slot addresses.

Current gap: values do not carry a static epoch token, so Babalawo cannot yet prove that a slot-derived reference cannot escape the epoch. Runtime generation tags are the smallest useful next step.

### 4. Yanda: Explicit Ownership Transfer Intent

The parser recognizes `yanda expr` as `Expression::MoveExpr`. Babalawo uses `MoveTracker` to mark moved identifiers and reports use-after-move. It also rejects moving a value while it is borrowed and rejects moving pointer-like/reference-like values.

At actor boundaries, Babalawo requires explicit moves for non-scalar payload variables. This makes ownership transfer visible in source code:

```ifa
ayanmo payload = [1, 2, 3];
Osa.ran(worker, yanda payload);

// Babalawo error: use after move
Obara.so(payload);
```

At the bytecode level, `MoveExpr` compiles to the `MoveLocal` opcode (0x1F) for identifier moves, which replaces the source slot with `Null`. For general expressions, the inner expression is evaluated without slot invalidation. Actor send serializes payloads before delivery — `yanda` is implemented as static ownership discipline plus runtime slot clearing, not as universal physical zero-copy transfer.

### 5. Actor Heaps: Isolation by VM Boundary

Each actor runs a fresh `IfaVM` and receives messages through a bounded channel. Actor payloads are serialized before crossing the boundary and deserialized inside the receiving actor loop.

This design avoids shared mutable VM state between actors. It favors correctness and isolation over zero-copy throughput.

Resources inside actor messages are handled specially: `transfer_resources` walks the payload and transfers contained `ResourceToken` ownership between registries. This is stronger than ordinary deep copy for external resources because it changes the authoritative owner.

### 6. IwaEngine: Borrow and Resource Debt Tracking

`IwaEngine` tracks:

- resource debts: opened resources that need matching close actions;
- immutable borrow debts;
- mutable borrow debts;
- scope depth for lexical release.

Its borrow model is intentionally simpler than Rust's:

- immutable borrows are allowed unless a mutable borrow exists;
- mutable borrows are allowed only when no other borrow exists;
- scope exit releases borrows from that scope;
- `yanda` cannot move a currently borrowed identifier.

The borrow checker is wired into `checks.rs`:
- `enter_scope()` / `exit_scope()` helpers wrap calls to `IwaEngine` for all block types (if, while, for, match, ebo, defer, ailewu, try, catch, finally).
- `borrow()` is called for `UnaryOperator::AddressOf` identifiers and for loop iterable validation.
- `borrow_mut()` is called for `UnaryOperator::AddressOfMut` identifiers.
- `exit_scope()` records "Scope Exit: Borrow Released" lifecycle events in the state history buffer before releasing.
- `release_borrow()` is available but not called from `checks.rs` — borrows are released implicitly via scope exit.

`LintContext` includes a `StateHistoryBuffer` (32-step circular buffer) that records lifecycle transitions: `Declared`, `Initialized`, `Mutated`, `Moved`, `Read`, `Borrowed (Immutable)`, `Borrowed (Mutable)`, `Scope Exit`. When a borrow or move violation is detected, a trace log is appended to the diagnostic pointing to the transition sites.

This is a Babalawo analysis mechanism, not a general-purpose runtime borrow-reference system.

## What Makes The Model Unique

Ifa-Lang's memory safety is not "Rust but with Yoruba keywords" and not "GC plus actors." The unusual part is the composition:

| Ifa-Lang mechanism | Closest known family | Difference |
|--------------------|----------------------|------------|
| `IfaGc<T>` | Python/Nim-style RC plus cycle collection | implemented as a VM value primitive with thread-local suspect buffers |
| Opon epochs | Zig/Odin arenas, region systems | integrated with bytecode opcodes and `#opon` capacity declarations |
| `yanda` | affine/linear ownership transfer | used as explicit actor-boundary intent and checked by Babalawo |
| actor delivery | Erlang/Pony-style isolation | currently chooses serialized isolation rather than shared heap messaging |
| resource token transfer | capability systems | transfers external resource ownership across actor registries |
| Babalawo | static analyzer/linter/type checker hybrid | combines move, borrow, effect, taboo, and capability checks |

The design is best understood as "layered ownership": ordinary code can stay dynamic, but dangerous boundaries require explicit ownership, region, or capability evidence.

## Roadmap: Fit With Current Code

### Priority 1: Harden `yanda` at Runtime (Done)

`yanda` now has runtime support for identifier moves. The `MoveLocal` opcode (0x1F) is defined in `ifa-bytecode`, dispatched in `ifa-vm`, and emitted by `ifa-compiler` for `Expression::MoveExpr(Identifier)` paths. The source slot is replaced with `Null` at runtime. Babalawo remains the primary static use-after-move checker.

Closes the previous gap between static move intent and runtime behavior for the variable-move case. General expression moves (non-identifier) still compile as pass-through evaluation and remain a static-only check.

### Priority 2: Add Generational Opon Handles

Add a generation counter parallel to `Opon::memory`.

Expected shape:

```text
OponHandle {
    slot: usize,
    generation: u64,
}
```

When an epoch truncates slots, bump generation counters for invalidated slots. Any later access through an old handle must compare the expected generation to the current generation and fail on mismatch.

This gives runtime protection against stale slot handles before the language has full region typing.

### Priority 3: Add Babalawo Region Tracking

After runtime handles exist, add static region analysis:

- track active ebo epoch scopes in `LintContext`;
- tag variables allocated inside an epoch with a region ID;
- reject escaping references to region-owned values;
- allow explicit safe escape through copy, freeze, or `yanda` into a valid owner.

This turns Opon epochs from runtime regions into analyzable language regions.

### Priority 4: Add `iso` Isolated Graphs

`iso` should mean a uniquely owned object graph that contains no shared references, channels, futures, borrowed references, or non-owned resources.

Expected rule:

```ifa
ayanmo packet = iso {
    "samples": [1, 2, 3],
    "tag": "frame"
};

Osa.ran(worker, yanda packet);
```

An `iso` value is eligible for future zero-copy actor transfer because Babalawo can prove no aliases remain in the sender.

### Priority 5: Immutable/Frozen Regions

Add a way to declare an epoch or graph immutable after construction. Frozen values can be shared read-only without repeated mutation checks and with fewer refcount churn points in optimized paths.

This should build on `iso`: construct uniquely, freeze, then share.

## Interaction Discipline

The memory-safety features must not become separate mini-systems. If `yanda`, `iso`, Opon epochs, region views, borrow tracking, constraint references, and CHERI all invent their own state, the design will be hard to reason about and easy to break.

The implementation should have one ownership state model that every feature updates.

### Shared State Model

Every value or region that participates in advanced memory safety should be classifiable by a small set of states:

| State | Meaning |
|-------|---------|
| `Owned` | exactly one source-level owner has authority |
| `BorrowedImmutable` | temporary read-only borrows exist |
| `BorrowedMutable` | one temporary mutable borrow exists |
| `Isolated` | the full reachable graph has one owner |
| `OpenMutableRegion` | an isolated region is temporarily opened for mutation |
| `OpenImmutableRegion` | a region is temporarily opened read-only |
| `Frozen` | permanently immutable after construction |
| `SharedReadOnly` | safely shareable because mutation is impossible |
| `Moved` | source owner has been consumed |
| `Expired` | region or slot lifetime has ended |
| `CapabilityBounded` | hardware or runtime capability bounds apply |

These are conceptual states. They do not require a single enum in the VM on day one, but Babalawo and the runtime should behave as if this is the state machine.

### Operations

| Operation | Required input | Output state | Enforcement |
|-----------|----------------|--------------|-------------|
| `yanda x` | `Owned` or `Isolated`, no active borrows | `Moved` for source, `Owned` for destination | Babalawo plus runtime `take` |
| `open iso as mutable` | `Isolated` | `OpenMutableRegion`, then `Isolated` on scope exit | Babalawo region scope |
| `open iso as immutable` | `Isolated` or `Frozen` | `OpenImmutableRegion`, then previous state | Babalawo region scope |
| `freeze x` | `Owned` or `Isolated`, no mutable borrow | `Frozen` / `SharedReadOnly` | Babalawo plus runtime immutability bit |
| Opon `end_epoch` | active epoch | slots become `Expired` | runtime truncate plus generations |
| Opon handle read | non-expired matching generation | unchanged | runtime generation check |
| constraint ref create | owner alive | owner tracks outstanding refs | runtime counter |
| owner destroy | no outstanding constraint refs | `Expired` | debug/runtime assertion |
| CHERI derive | valid bounded authority | narrower `CapabilityBounded` authority | hardware plus feature gate |

### Invalid Combinations

The implementation should reject these cases early:

- `yanda` while `BorrowedImmutable` or `BorrowedMutable`.
- `freeze` while `BorrowedMutable`.
- `open mutable` while any immutable view is active.
- storing a borrow or region view in a heap object unless the type system can prove it does not escape.
- sending non-`iso` mutable graphs through a future zero-copy actor path.
- treating a CHERI capability or Opon handle converted to an integer as reusable authority.
- reading an Opon handle after its generation has expired.

### Correct Composition Examples

An isolated graph can be opened, mutated, closed, then moved:

```ifa
ayanmo packet = iso {
    "samples": [1, 2, 3]
};

open packet as mutable p {
    p.samples.push(4);
}

Osa.ran(worker, yanda packet);
```

An Opon handle is safe only while its epoch and generation are valid:

```ifa
ebo frame {
    ayanmo h = Opon.alloc([1, 2, 3]);
    Obara.so(Opon.read(h));
}

// Runtime error once generational handles exist:
// h points into an expired epoch.
Opon.read(h);
```

A frozen graph can be shared but not mutated:

```ifa
ayanmo config = freeze {
    "host": "localhost",
    "port": 8080
};

Osa.ran(worker, config);
Osa.ran(logger, config);
```

### Implementation Rule

The pipeline should be:

```text
source syntax
  -> Babalawo ownership/region proof
  -> bytecode/runtime state transition
  -> Opon generation or constraint check
  -> optional CHERI hardware enforcement
```

Babalawo should prove the common safe cases. The VM should still fail closed when static analysis is bypassed, incomplete, or running in a mode that accepts dynamic code.

### Engineering Review

The useful idea is not "add every memory-safety technique." That is how a language gets a complicated safety story and still misses bugs.

The useful idea is one boring invariant:

> At every boundary, Ifa-Lang must know who has authority, what region that authority covers, whether mutation is allowed, and when that authority expires.

Everything else is an implementation detail. `yanda` changes who has authority. `iso` proves the authority covers a whole graph. Opon generations decide whether authority has expired. Region views decide whether mutation is allowed. CHERI can enforce the physical bounds of that authority in hardware.

If a proposed feature cannot be expressed as one of those authority transitions, it should not be added to the core memory-safety model.

## Non-Goals

The following should not be first-line memory-safety work for the current codebase:

- replacing `IfaValue` with a fully unique-reference model;
- adding a free-list allocator to Opon before stale-handle protection exists;
- implementing full Rust-style non-lexical lifetimes;
- adopting CHERI-style hardware capabilities as a language feature;
- replacing the bytecode VM with graph reduction or interaction nets.

Those are different architectures. Ifa-Lang's near-term advantage is the combination of Babalawo checks, Opon epochs, `IfaGc`, and actor isolation.

## Design Rule

When adding memory-safety features, prefer this order:

1. make ownership visible in source;
2. make stale access fail at runtime;
3. make Babalawo prove the common safe cases;
4. optimize transfer and sharing only after the invariant is enforceable.

That sequence fits the current VM and avoids promising physical zero-copy before the type and runtime layers can enforce it.
