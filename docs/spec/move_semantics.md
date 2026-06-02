# Zero-Copy Move Semantics (The `yanda` / `move` keyword)

**Status:** `IMPLEMENTED`  
**Crate:** `ifa-babalawo` (`src/movement.rs`, `src/checks.rs`), `ifa-vm` (`src/actor.rs:273-275`)

Ifá-Lang introduces mathematically-proven zero-copy ownership transfer for concurrent programming using the `yanda` (surrender) or `move` keyword.

## The Core Concept: Sacrifice (Ẹbọ)

In highly parallel Actor systems, passing complex data (like large Maps or Lists) between threads typically requires either:
1. **Garbage Collection (GC)**, which introduces unpredictable pauses and overhead.
2. **Deep Copying**, which creates performance bottlenecks (O(N) memory allocation per message).

Ifá-Lang rejects both. Instead, it uses **Linear Type Semantics** enforced by the `Babalawo` static analyzer. When you pass data across an Actor boundary (the `Osa` domain), you must explicitly surrender ownership of that data. The VM can then safely pass the exact memory pointer to the receiving thread instantly (O(1) time complexity) without risking a data race.

## Syntax

The syntax for an explicit ownership transfer uses the prefix keyword `yanda` (Yoruba: to surrender/release) or its English alias `move`:

```ifa
ayanmo payload = { "data": [1, 2, 3, 4, 5] };

// Transfer ownership of `payload` to the `worker_actor`.
// The actor receives the exact memory block, zero-copy.
Osa.ran(worker_actor, yanda payload);

// Compile Error: USE_AFTER_MOVE
// Babalawo knows you have surrendered `payload` and will forbid further access.
Obara.so(payload); 
```

## Actor Boundaries (`Osa` Domain)

The `Osa` domain governs concurrency and multi-threading. **Any non-scalar variable passed to an `Osa` boundary must be explicitly moved.**

If you attempt to pass a non-scalar variable (like a List or Map) to an actor without explicit surrender, `Babalawo` will fail the compilation:

```ifa
ayanmo list = [1, 2, 3];

// ERROR: EXPLICIT_MOVE_REQUIRED
// Cannot pass non-scalar variable to actor boundary. Use 'yanda' (or 'move').
Osa.ran(actor, list); 
```

To resolve this, you must explicitly surrender the variable using `yanda`:

```ifa
Osa.ran(actor, yanda list); // OK: O(1) Zero-copy transfer
```

### Scalar Variables

Scalar types (Int, Float, Bool, Nil) are exempt from explicit move requirements because they are trivial to copy (they fit in a CPU register). You can pass them directly:

```ifa
ayanmo count = 42;

// OK: Scalars are automatically copied
Osa.ran(actor, count); 
Obara.so(count); // Still accessible!
```

## Borrowing Conflicts

You cannot surrender a variable if it is currently borrowed by an `Ìwà` reference. `Babalawo` will emit a `MOVE_WHILE_BORROWED` error:

```ifa
ayanmo list = [1, 2];
ayanmo ref = &list; // Borrowed

// ERROR: MOVE_WHILE_BORROWED
// Cannot move 'list' while it is borrowed
Osa.ran(actor, yanda list); 
```
