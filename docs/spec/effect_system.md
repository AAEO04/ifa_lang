# Effect System Specification

**Status:** `IMPLEMENTED`  
**Version:** 0.1  
**Crate:** `ifa-babalawo` (`src/effects.rs`), `ifa-types` (`src/ast.rs`)

---

## 1. Purpose

The Effect System tracks and enforces side-effect boundaries at compile time. Every function in Ifá-Lang either declares its effects explicitly via the `effects(...)` annotation, or is implicitly `Pure` (no side effects). The `Babalawo` static analyzer validates that:

1. Functions only perform operations consistent with their declared effects.
2. Functions that call effectful callees must propagate those effect declarations.
3. `Pure` functions cannot invoke any effectful operation.

---

## 2. Effect Enum

Defined at `ifa-types/src/ast.rs:10-23`:

```rust
pub enum Effect {
    Pure,      // No side effects (default when no annotation)
    Async,     // Yields/pauses (Osa / Concurrency domain)
    Network,   // Outbound network access (Otura domain)
    FileIO,    // File I/O access (Odi domain)
    State,     // Modifies global/closure state
    Impure,    // Opaque/Unsafe FFI or Bridge calls (superset — implies all)
}
```

`Impure` acts as a supertype: a function declared `effects(Impure)` satisfies any effect requirement.

---

## 3. Syntax

Functions declare effects after the return arrow:

```ifa
# Pure function (implicit — no annotation needed)
ese add(a, b) {
    da a + b;
}

# Function with a single effect
ese fetch_data(url) -> effects(Network) {
    da Otura.get(url);
}

# Function with multiple effects
ese sync_file(path, url) -> effects(Network, FileIO) {
    ayanmo data = Otura.get(url);
    Odi.si(path);
    Odi.ko(path, data);
    Odi.pa(path);
}
```

---

## 4. Domain-to-Effect Mapping

The `EffectChecker` maps Odù domains to their required effects at `ifa-babalawo/src/effects.rs:53-61`:

| Domain | Effect | Rationale |
|--------|--------|-----------|
| `Osa` | `Async` | Concurrency / actor spawning |
| `Otura` | `Network` | HTTP, sockets, outbound I/O |
| `Odi`, `Storage` | `FileIO` | Filesystem access |
| `Ofun`, `Sys`, `Coop` | `Impure` | Capabilities, system calls, FFI |
| All others | `Pure` if `!has_side_effects()`, else `Impure` | Conservative default |

---

## 5. EffectChecker State Machine

Defined at `ifa-babalawo/src/effects.rs:4-51`:

```
EffectChecker {
    current_effects: Vec<Effect>   — effects declared by the enclosing function
    errors: Vec<IfaError>          — accumulated violations
}
```

### 5.1 `enter_function(effects)`

Sets `current_effects` to the function's declared effect list. If the function has no `effects(...)` annotation, `current_effects` is empty (implicitly Pure).

### 5.2 `leave_function()`

Clears `current_effects`.

### 5.3 `check_call(callee_effects, file, line, column)`

```
for each effect in callee_effects:
    if effect == Pure:
        continue  — Pure calls are always allowed

    if current_effects contains Pure:
        error: "Pure function cannot call function with effect {effect}"

    if current_effects does NOT contain effect
       AND current_effects does NOT contain Impure:
        error: "Function is missing effect declaration {effect}"
```

### 5.4 Integration Point

The `EffectChecker` is a field on `LintContext` (`checks.rs:58`). It is invoked during AST walking when the checker encounters a domain method call (e.g., `Osa.ran`, `Otura.get`). The `domain_effects()` function translates the callee's domain into the required `Effect` variants.

---

## 6. Invariants

1. **Effect monotonicity**: A function's declared effects are the *minimum* set. A function declared `effects(Network)` may call `Pure` functions freely, but may not call `FileIO` functions without also declaring `FileIO`.

2. **Impure is the universal effect**: `effects(Impure)` satisfies any check. It is the escape hatch for FFI and unsafe operations.

3. **No inference**: Effects are not inferred from the function body. They must be explicitly declared. This is a deliberate design choice — effect signatures are part of the public API contract.

4. **Compositionality**: If function A calls function B, and B requires `effects(Network)`, then A must also declare `effects(Network)` (or `effects(Impure)`).
