# Ambitious Features — Cross-Cutting Design Work Needed

These features touch multiple subsystems (parser, type system, VM, Babalawo) and need careful design before implementation.

---

## 1. OponView Borrow Checker (Partially Implemented)

**Goal:** A static borrow checker for Ifá-Lang that runs as a Babalawo pass, ensuring memory safety for references and shared state without a tracing GC.

**Concept:** Named after *Opon* (the sacred divination board / memory container). The OponView system tracks:
- **Unique references** (`&mut T`): exclusive write access, no aliasing
- **Shared references** (`&T`): read-only, multiple allowed
- **Ownership transfer**: move semantics by default, copy for primitives

**Syntax:**
```
let x = 42;
let r = &x;              // shared borrow
print(*r);               // deref

let mut v = [1, 2, 3];
let m = &mut v;          // mutable borrow
m[0] = 99;

// Ownership
let s = Ogbe.str("hello");
let t = s;               // move: s is now invalid
print(t);                // OK
// print(s);             // Babalawo error: use of moved value
```

**Key Design Decisions:**

| Question | Proposed Answer |
|----------|----------------|
| Default semantics | Move (like Rust). Copy for Int/Float/Bool |
| Borrow check granularity | Per-variable, per-scope (lexical, not NLL) for v1 |
| Syntax for references | `&expr`, `&mut expr`, `*expr` |
| Interior mutability | Provided by Odù domains (e.g. `Okanran`) and `IfaShared` |
| Closure captures | Infer borrow kind from body; or explicit `move` keyword |
| `'static` lifetime | Domain-level: `Ogbe.str("literal")` is always `'static` |

**Implementation Status:**
1. ✅ **Done** — AST nodes exist: `Expression::Iso(Box<Expression>)` at ast.rs:528, `UnaryOperator::AddressOf` and `UnaryOperator::AddressOfMut` at ast.rs:588-589, `TypeHint::Ref(Box<TypeHint>)` and `TypeHint::RefMut(Box<TypeHint>)` at ast.rs:372-374.
2. ✅ **Done** — Parser: `&` / `&mut` prefix operators in grammar.pest:316, `*` dereference prefix. Parser maps `&mut` to `AddressOfMut` and `&` to `AddressOf`.
3. ✅ **Done** — Babalawo pass: `IwaEngine` in `checks.rs` — `borrow()` called for `AddressOf` (line 1774), `borrow_mut()` called for `AddressOfMut` (line 1795). `enter_scope()`/`exit_scope()` wired into all block types (if/while/for/match/ebo/defer/ailewu/try/catch/finally). State history buffer records lifecycle transitions.
4. ✅ **Done** — Compiler: emits `OpCode::Ref` for both `AddressOf` and `AddressOfMut` (compiler/src/lib.rs:1412-1427). Currently only literal integer addresses are supported.
5. ✅ **Done** — No runtime borrow-check overhead: all checks are static via Babalawo.

**Remaining Work:**
- `release_borrow()` not called from `checks.rs` — implicit via scope exit, but explicit release could be useful.
- Compiler only supports literal integer addresses for `&`/`&mut` — variable-address references not yet compiled.
- `*` dereference not yet emitted as a dedicated opcode (compiler compiles `Expression::Iso` as pass-through).
- Closure capture borrow inference not implemented.
- Non-lexical lifetimes (NLL) deferred to future work.

---

## 2. Compile-Time Oracle (Àwọn Àpéjọ)

**Goal:** A subset of Ifá-Lang that runs at compile time — function evaluation, constant folding, and domain-call evaluation during static analysis.

**Concept:** Named *Àwọn Àpéjọ* (The Assembly — a gathering of wise diviners). Think Zig's `comptime` or Rust's `const fn`.

**Syntax:**
```
oracle fibonacci(n: Int) -> Int {
    if n <= 1 { n }
    else { fibonacci(n - 1) + fibonacci(n - 2) }
}

const FIB_10 = fibonacci(10);  // evaluated at compile time
```

**Rules:**
- `oracle` functions are pure (no side effects, no I/O, no random)
- All parameters and return types must be explicit
- Only a subset of Odù domains are available at compile time: `Obara` (math), `Oyeku` (random? — no, banned), `Ika` (string manipulation)
- Oracle calls in non-oracle contexts inline the result as a constant

**Implementation Plan:**
1. Add `FunctionType::Oracle` to `Statement::EseDef`
2. Babalawo validates oracle body (no side-effect calls, no mutation of outer scope)
3. Compiler evaluates oracle calls during bytecode compilation using a lightweight interpreter (reuse existing `IfaVM` with compile-time restrictions)
4. Result is stored as a constant pool entry

**What's evaluable at compile time:**

| Domain | Methods Available | Notes |
|--------|------------------|-------|
| `Obara` | add, sub, mul, div, mod, pow, neg, abs, min, max | Pure math |
| `Ika` | len, concat, slice, contains, replace | Pure string ops |
| `Ogbe` | to_str, to_int, to_float | Type coercion only |
| `Ofun` | — | All banned (capabilities don't exist at compile time) |

---

## 3. `select` Over Channels

**Goal:** A `select` statement that waits on multiple channel operations simultaneously, inspired by Go's `select` and Rust's `tokio::select!`.

**Syntax:**
```
select {
    msg <- channel1 => {
        print("got from channel1: ", msg);
    }
    channel2 -> msg => {
        print("got from channel2: ", msg);
    }
    after 1s => {
        print("timeout");
    }
    default => {
        print("no channel ready");
    }
}
```

**Semantics:**
- `msg <- chan` — receive from channel, binding result to `msg`
- `chan -> val` — send `val` to channel
- `after duration` — timeout branch
- `default` — non-blocking: fires if no other branch is immediately ready
- If multiple branches are ready, one is chosen pseudo-randomly (via `Oyeku`)
- All receive expressions in non-selected branches are NOT evaluated (no side effects)

**Implementation Plan:**
1. Add `Statement::Select { arms: Vec<SelectArm>, span: Span }` to AST
2. `SelectArm` has variants: `Recv { var, channel, body }`, `Send { channel, value, body }`, `After { duration, body }`, `Default { body }`
3. Parser grammar:
   ```pest
   select_stmt = { select_kw ~ "{" ~ select_arm+ ~ "}" }
   select_arm = { recv_arm | send_arm | after_arm | default_arm }
   recv_arm = { ident ~ "<-" ~ expression ~ "=>" ~ block }
   send_arm = { expression ~ "->" ~ ident ~ "=>" ~ block }
   after_arm = { "after" ~ expression ~ "=>" ~ block }
   default_arm = { "default" ~ "=>" ~ block }
   ```
4. `Osa` domain already has channels (`Osa.ise`, `Osa.gba`). VM needs:
   - `OpCode::SelectBegin` — takes N branches + timeout
   - Runtime polls all channels, pseudorandomly picks ready one
   - Jumps to selected branch's bytecode
5. Babalawo: verify channel types match across branches

**V1 Limitation:** No `default` and `after` in the initial implementation. Only `recv` and `send` arms.
