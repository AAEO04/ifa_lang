# Mathematical Foundations of Ifá-Lang

**Audit date:** 2026-05-29  
**Codebase:** `crates/` tree

---

## 1. Lambda Calculus

### 1.1 Functions as First-Class Values

Ifá-Lang supports first-class functions through two mechanisms:

1. **Named functions** (`EseDef` at `ast.rs:104`): `fn foo(x) { ... }` — compiled to `BytecodeFnData` with a `start_ip` address.
2. **Anonymous lambdas** (grammar `pest:230`): `ese(params) { body }` / `fn(params) { body }` — compiled to `<lambda@offset>` with implicit closure capture.

Lambda representation at `ast.rs:476-479`:
```rust
Lambda {
    params: Vec<String>,
    body: Vec<Statement>,
}
```

### 1.2 Closure Implementation

**Compiler** (`compiler/src/lib.rs:55`): `FunctionContext` has `upvalues: Vec<Upvalue>`, tracked per-function.

**Upvalue resolution** (`compiler/src/lib.rs:258-276`): Standard lexical (static) scoping with a three-level lookup chain:
1. `resolve_local` — check current and ancestor scopes (up to enclosing function boundary).
2. `resolve_upvalue` — recurse into parent `FunctionContext`s, adding an `Upvalue` entry at each level.
3. `LoadGlobal` — fallback for module-level names.

**Upvalue kinds** (`Upvalue` struct at `compiler/src/lib.rs:75-78`):
```rust
struct Upvalue {
    name: String,
    index: usize,    // local slot or parent upvalue index
    is_local: bool,  // true = captured from current function, false = from parent closure
}
```

**Codegen pattern** (`compiler/src/lib.rs:1142-1164`): After compiling a function body that captures variables:
```
[upvalue values on stack]
PushFn <name>, <start_ip>, <arity>, <is_async>
MakeClosure <count>
  → consumes `count` stack values + `PushFn` result → produces `Closure(Arc<ClosureData>)`
```

**OpCodes** (`bytecode/src/lib.rs:100-102, 232-233`):
- `LoadUpvalue (0x1D)`: Push captured upvalue onto stack.
- `StoreUpvalue (0x1E)`: Pop stack into captured upvalue (mutable closure state).
- `MakeClosure (0x93)`: Wrap bytecode function pointer with captured environment.

**VM dispatch** (`vm.rs:1462-1528`): `MakeClosure` reads `capture_count` bytes from the instruction stream — each byte is a `(kind << 7) | index` pair where `kind=0` means a local (immediate) capture and `kind=1` means an upvalue from the parent's closure env.

### 1.3 Beta-Reduction (Function Application)

**Call path** (`vm.rs:1533-1560`):
- `Call (0x53)`: Pop function + args, push frame, set `ip` to function start.
- `TailCall (0x57)**: Reuse current frame — no new `CallFrame` pushed, no return address saved.

**Arity checking** (`vm.rs:1017-1022`): Enforced at call site — `args.len() != data.arity` → `ArityMismatch` error.

### 1.4 What's Present vs Standard Lambda Calculus

| Concept | Status | Location |
|---------|--------|----------|
| Variable binding (λx.e) | `Lambda { params, body }` | `ast.rs:476` |
| Function application (e₁ e₂) | `Call { name, args }` + `Expr::Call` | `ast.rs:448` |
| Beta-reduction (runtime) | `call_value_task` | `vm.rs:1014` |
| Lexical (static) scoping | `resolve_upvalue` chain | `compiler/src/lib.rs:258` |
| Closure (environment capture) | `UpvalueCell = Arc<Mutex<IfaValue>>` | `value_union.rs:102` |
| Tail-call elimination | `TailCall` reuses frame | `compiler/src/lib.rs:733, vm.rs:1537` |
| Eta-conversion | Not implemented | — |
| Alpha-conversion | None (no hygienic naming) | — |
| Church numerals | Not built-in | — |
| Fixed-point combinator | Not built-in | — |

### 1.5 Key Lambda Calculus Gap

**No beta-normalization at compile time.** The constant folder (`fold_expression` at `compiler/src/lib.rs:1840`) only handles literal arithmetic — it cannot apply functions at compile time. There is no `comptime` evaluation, no partial evaluator, no specialization pass.

**No eta-expansion or eta-reduction.** The compiler emits calls exactly as written; there is no normalization strategy.

**No capture-avoiding substitution.** Closure capture uses `Arc<Mutex>` runtime indirection — there is no compile-time substitution of captured variables into the function body.

---

## 2. Automata Theory

### 2.1 The VM as a Pushdown Automaton

The Ifá bytecode VM (`vm.rs`) is a classic **stack-based pushdown automaton**:

- **Finite control**: The `execute` loop at `vm.rs:933-1002` — `while !halted && ip < code.len() { step(bytecode) }`.
- **Stack**: `ctx.stack: Vec<IfaValue>` — unbounded operand stack.
- **Call-stack**: `ctx.frames: Vec<CallFrame>` — grows with nested function calls (pushdown of activations).
- **State**: `ExecutionContext` at `vm.rs:201-212`:
  - `ip: usize` — instruction pointer (current state).
  - `stack: Vec<IfaValue>` — operand stack.
  - `frames: Vec<CallFrame>` — call stack.
  - `halted: bool` — accept/reject flag.
  - `recovery_stack: Vec<RecoveryFrame>` — exception handler stack.
  - `loop_stack: Vec<(usize, usize)>` — loop continue/break targets.

**Transition function**: `step(bytecode) -> Result<()>` at `vm.rs` — a 100+ arm match on `OpCode` that reads opcode bytes, pops/pushes stack values, and advances `ip`.

### 2.2 Formal Automaton Classes

| Class | Does Ifá implement it? | Evidence |
|-------|----------------------|----------|
| **Finite Automaton (DFA)** | The lexical tokenizer | `pest` PEG generates a DFA for token recognition at `parser.rs:12-14` |
| **Pushdown Automaton (PDA)** | The bytecode VM | Call stack + operand stack = 2-stack machine (technically a 2-PDA, equivalent to Turing machine) |
| **Turing Machine** | Effectively yes | Recursive functions + heap-allocated structures + I/O = Turing-complete |
| **Linear Bounded Automaton** | Opon restricts memory | `OponSize::Kekere/Small` bounds allocations, but not computationally |

### 2.3 Loop Semantics as Automaton Control

**Loop compilation** (`compiler/src/lib.rs:560-609`):
```
  [condition code]
  JumpIfFalse <exit>    ; if condition false → break
  [body code]
  Jump <loop-header>    ; back to condition
exit:
```

**Loop stack** (`vm.rs:209-211`): `loop_stack: Vec<(continue_ip, break_ip)>` — each entry stores:
- `continue_ip`: Instruction pointer of the loop condition (target for `continue`).
- `break_ip`: Instruction pointer after the loop body (target for `break`).

`Continue` pops nothing, jumps to `continue_ip`.  
`Break` pops the loop stack, jumps to `break_ip`.

### 2.4 Grammar as Context-Free Language

The PEG grammar (`grammar.pest`) defines a context-free language recognized by a **packrat parser** (PEG parsers are a form of recursive descent with unlimited backtracking and memoization — technically O(n) for unambiguous PEGs).

- `program → statement*` (Kleene star — regular at top level).
- `statement → import_stmt | var_decl | if_stmt | ...` (union of 25+ production rules).
- `if_stmt → if_kw expr block (else_kw block)?` — context-free nesting.

The grammar is unambiguous by PEG construction (prioritized choice `|` with ordered alternatives).

### 2.5 What's Missing

- No formal automaton for **type checking** — the Babalawo uses ad-hoc recursive traversal, not a type automaton (no attribute grammar, no constraint-generating DFA).
- No **state machine DSL** for user programs — there is no `state`, `transition`, or `event` keyword.
- No **regular expression** support at the language level.
- No **control-flow graph** (CFG) — the compiler emits bytecode linearly without building a graph representation.

---

## 3. Graph Theory

### 3.1 Dependency Graph (Oja Package Manager)

`oja.rs:710-823` implements a **directed dependency graph**:

- **Nodes**: Packages (name + version).
- **Edges**: `dependency` entries in `Iwe.toml` — directed from dependent → dependency.
- **Traversal**: BFS (`VecDeque` queue at line 723) starting from direct dependencies.
- **Cycle handling**: `seen: HashSet<String>` at line 721 — nodes are visited once; cycles are implicitly broken by not re-queuing seen nodes.

The traversal produces a **topologically-ordered** list (line 714: "Returns a topologically-ordered list of `LockedPackage` entries"), but the topological order is implicit from insertion order (BFS), not from a proper topological sort. The graph is a DAG by construction (package managers forbid cyclic dependencies).

### 3.2 TaskGraph (ifa-infra)

`cpu.rs:301-481` implements a **DAG task scheduler**:

- **Nodes**: `TaskId` → `TaskFn` (sync or async closure).
- **Edges**: `dependencies: HashMap<TaskId, Vec<TaskId>>` — directed `task → depends_on`.
- **Execution**: Kahn's algorithm (in-degree counting at `cpu.rs:375-377`):
  ```
  loop {
      ready = nodes where in_degree == 0 ∧ not completed ∧ not failed
      if ready empty → break (all done or cycle)
      parallel-for each ready task: execute
      for each completed task: decrement in-degree of dependents
  }
  ```
- **Cycle detection** (`cpu.rs:397-398`): If `ready` is empty but some tasks remain, returns `TaskError::CycleDetected`.

**Kahn's algorithm** is a textbook algorithm for topological ordering of DAGs — this is the only explicit graph algorithm in the codebase.

### 3.3 Scope Tree

`scope.rs:13-17`: The scope system is a **tree** (root module scope with child block scopes):

- **Root**: File-level scope.
- **Child scopes**: Created by blocks (`{ }`), function bodies, if/else branches, loops.
- **Lookup**: `resolve(name)` walks parent pointers — **tree traversal** from leaf to root, O(depth).
- **No graph cycle**: Parent pointers form a DAG (strict tree — each node has exactly one parent).

### 3.4 What's Missing (Graph Theory)

- **No control-flow graph (CFG)**: The compiler emits bytecode linearly; there is no CFG with basic blocks, no dominator tree, no loop forest.
- **No use-def chain**: Variable definitions and uses are not linked.
- **No call graph**: Inter-function call relationships are not tracked at compile time.
- **No interference graph**: Register allocation would need an interference graph; none exists.
- **No dataflow graph**: Values are not tracked as nodes in a computation graph.
- **No abstract syntax graph**: The AST is a tree, not a graph — no sharing between equivalent subtrees.

---

## 4. Compiler Optimization Theory

### 4.1 Current Optimization Passes

The compiler has exactly **two** optimization passes:

#### 4.1.1 Constant Folding

`compiler/src/lib.rs:1840-2122` — `fold_expression(expr: &Expression) -> Expression`:

**Intra-expression only** (no cross-statement analysis). Reduces literal binary/unary operations:

- `Int + Int → Int` (including `Power` with exponent ≤ 30).
- `Float + Float → Float`.
- `Int + Float → Float` (type coercion at compile time).
- `String + String → String` (concatenation).
- All comparison operators on literal pairs: `Eq`, `Neq`, `Lt`, `LtEq`, `Gt`, `GtEq`.
- Boolean operators: `And`, `Or`.
- Unary: `Neg(Int)`, `Neg(Float)`, `Not(Bool)`.

**Safety guard at `compiler/src/lib.rs:1856`**: `Div` and `Mod` folding is guarded by `r != 0` — if the divisor is zero, folding is skipped and the expression is emitted as-is (so the VM can produce `DivisionByZero` at runtime). This is correct but means `0 / 0` produces a runtime error rather than a compile-time error, which is a missed Babalawo integration.

#### 4.1.2 Tail-Call Optimization

`compiler/src/lib.rs:709-738`: When a `return` statement's value is a direct function call (`Expression::Call { name, args }`):

1. Push function reference via `LoadLocal`/`LoadUpvalue`/`LoadGlobal`.
2. Push each argument.
3. Emit `OpCode::TailCall (0x57)` + arity byte.

**VM dispatch** (`vm.rs:1537`): `TailCall` pops the current frame, pushes no return address, and jumps to the callee. This is a **frame reuse** strategy — it prevents stack growth for tail-position calls.

**What TCO handles**:
- `fn f(x) { return g(x); }` → `TailCall g(x)`.
- Direct calls only (`Call { name, args }`).

**What TCO does NOT handle**:
- Indirect calls via function pointers or closures.
- Method calls (`CallMethod`).
- Conditional tail calls (`return if c then f(x) else g(x)` — the `if` is an `If` statement, not a ternary expression).
- Mutual recursion across more than 2 functions.

#### 4.1.3 String Deduplication

`compiler/src/lib.rs` (no explicit pass, but string indices are interned): String literals with identical content share a single constant pool entry. This is standard in any bytecode compiler and is not an optimization pass per se.

### 4.2 Optimization Theory Framework

| Optimization | Status | Implementation |
|-------------|--------|---------------|
| Constant folding | ✅ Single-expression | `fold_expression` at `compiler:1840` |
| Constant propagation | ❌ | No use-def chains, no SSA |
| Dead code elimination | ❌ | Unused assignments remain |
| Dead store elimination | ❌ | Redundant stores remain |
| Copy propagation | ❌ | `let x = y; use(x)` stays |
| Common subexpression elimination (CSE) | ❌ | Same expression computed twice |
| Loop invariant code motion | ❌ | No loop analysis pass |
| Loop unrolling | ❌ | Not attempted |
| Strength reduction | ❌ | No pattern-based rewriting |
| Inlining | ❌ | No inline heuristic or pass |
| Peephole optimization | ❌ | Single-pass, no window-based optimization |
| Register allocation | ❌ | Stack-based VM (no registers to allocate) |
| Instruction scheduling | ❌ | Instructions in parse order |
| Escape analysis | ❌ | All heap allocations are permanent |
| Devirtualization | ✅ (partial) | `CallOduFast` for known domains at `compiler:1582-1598` |
| Speculative optimization | ❌ | None |
| Profile-guided optimization | ❌ | No profiling infrastructure |

### 4.3 Why So Few Optimizations

The compiler is optimized for **compile-time simplicity over runtime performance**:

- Single-pass compilation from AST → bytecode (no intermediate IR).
- Stack-based bytecode (no register pressure, no allocation problem).
- No basic blocks, no CFG — control flow is implicit in Jump opcodes.
- No SSA form — every variable is a mutable slot, not a definition.

This is a deliberate trade-off: Ifá's performance model depends on **transpilation to Rust** for release builds, not on optimizing the bytecode. The bytecode VM is the development/debugging path.

### 4.4 What Would Be Needed for Serious Optimization

1. **Intermediate representation**: A `Vec<IfaInstr>` CFG with basic blocks (linear IR, not SSA initially).
2. **Dominator tree**: For loop identification and code motion.
3. **Use-def chains**: Variable liveness analysis.
4. **Dataflow analysis framework**: A lattice-based `gen()`/`kill()` engine for reaching definitions, available expressions, etc.
5. **SSA construction**: For maximal optimization power (CSE, constant propagation, dead code elimination).

---

## 5. Information Theory

### 5.1 Compression

**Irete (Ìrẹtẹ̀) domain** — Cryptography, hashing, compression:

- **`irete.funpo(data, level)`** / `Irete.compress()` — zstd compression with configurable level (1-22) at `odu/irete.rs:236-253`.
- **`irete.tu(data)`** / `Irete.decompress()` — zstd decompression at `odu/irete.rs:241-248`.
- **`irete.iwon_funpo(original, compressed)`** — compression ratio calculation: `1.0 - (compressed / original)` at `odu/irete.rs:259-265`.

### 5.2 Entropy

- **`owonrin.aya()`** / `Owonrin.create()` — ChaCha20 CSPRNG seeded from OS entropy at `odu/owonrin.rs:28-31`.
- No Shannon entropy measurement, no information-theoretic analysis of data.
- No Kolmogorov complexity estimation.

### 5.3 Encoding

- **Bytecode format** (`bytecode/src/format.rs:1-109`): Binary format with magic bytes (`IFA\0`), version, section sizes. No compression, no entropy coding.
- **NaN boxing** (deleted): Previously encoded primitive values in `f64` bit patterns. The `nan_box.rs` file has been removed — `IfaValue` is now a plain Rust enum with direct variant dispatch.

### 5.4 What's Missing (Information Theory)

- No **entropy coding** (Huffman, arithmetic coding) in the standard library.
- No **information-theoretic metrics** (entropy, mutual information, KL divergence).
- No **channel coding** (error-correcting codes, Reed-Solomon, LDPC).
- No **rate-distortion** theory applications.
- Bytecode has no **instruction-level entropy encoding** — opcodes are fixed 1-byte values regardless of frequency.

### 5.5 Information-Theoretic Observations

**NaN boxing efficiency (historical)**: The NaN-boxing encoding (now deleted) previously used 64 bits per value for scalar types. With 5 scalar types (Null, Bool, Int, Float, pointer), the theoretical information content was log₂(5) ≈ 2.32 bits per value, plus the value payload. `IfaValue` now uses a plain Rust enum (16 bytes) with direct variant dispatch — trading density for simplicity and wider numeric range (full i64, not 47-bit).

---

## 6. Composite Analysis: Foundations for Advanced Compiler Work

### 6.1 The Missing Mathematical Layers

```
Current:                            Needed for advanced optimization:
─────────────────────────────────────────────────────────────────
Lexical scope tree                 →  CFG + dominator tree
Single-pass constant folding       →  SSA-based optimization framework
Stack-based bytecode               →  Register-allocated IR
Ad-hoc type checking               →  Constraint-based type inference (HM(X))
BFS dependency traversal           →  Full topological sort + SAT solving
No lambda normalization            →  Normalization-by-evaluation (NBE)
No proof system                    →  Dependent types / refinement types
```

### 6.2 Stack vs Register Machine Tradeoff

The stack-based VM is mathematically simpler (PDA with 2 stacks) but:

- **+** Simple compilation (no register allocation).
- **+** Compact bytecode (implicit operands).
- **−** Stack shuffling overhead (`Swap`, `Dup`, `Pop`).
- **−** No instruction-level parallelism.
- **−** Harder to reason about data flow.

Adding a **register-based IR** would require graph coloring or linear scan register allocation — both well-understood but adding significant compiler complexity.

### 6.3 Recursion Theory

The closure system implements **general recursion** (not just primitive recursion):

- Recursive functions call themselves via `LoadGlobal <name>` (the function is a global).
- Mutual recursion works via forward references (globals are resolved at runtime, not compile time).
- **No recursion depth limit** beyond stack memory — `frame_limit: Option<usize>` at `vm.rs:146` is configurable but default-unbounded.
- **No tail-recursion modulo cons** (TRMC) optimization.

### 6.4 Type-Theoretic Observations

From a type theory perspective:

- **Simply typed lambda calculus** (STLC) with base types: `Int`, `Float`, `Bool`, `Str`.
- **No polymorphism** (no System F, no Hindley-Milner).
- **No dependent types** (no Π types, no Σ types).
- **No subtyping** beyond `Null ⊑ T` (implicit via `Null ≤ all reference types`).
- **No recursive types** (though List/Map are de facto recursive).
- **No existential types** (no `exists T. ...`).
- **No linear types** at the type level (the `MoveTracker` is a syntactic check, not a type system feature).

### 6.5 Proof-Theoretic Potential

Ifá-Lang could benefit from:

1. **Curry-Howard correspondence**: Not exploited. Types do not encode propositions; functions do not encode proofs.
2. **Normalization-by-evaluation (NBE)**: Could enable compile-time evaluation of pure functions in a principled way.
3. **Logical frameworks**: The Odu domain system (16 named worlds with binary identifiers) could serve as a framework for modal logic or epistemic logic — each domain is a "possible world" with specific operations.
4. **Abstract interpretation**: The Babalawo is a primitive abstract interpreter; a proper abstract interpretation framework (lattice-based) would enable sound static analysis with measurable precision guarantees.

---

## 7. Summary: What Exists vs What's Needed

| Foundation | What exists | What's needed for advanced work |
|-----------|------------|----------------------------------|
| **Lambda calculus** | First-class functions, closures, lexical scoping, TCO | Comptime beta-reduction, hygienic macros, normalization |
| **Automata theory** | Stack VM (PDA), PEG grammar (CFL) | CFG construction, dataflow analysis framework, type automaton |
| **Graph theory** | Dependency graph (BFS), TaskGraph (Kahn's DAG), scope tree | Dominator tree, use-def chains, call graph, interference graph |
| **Compiler optimization theory** | Constant folding (expression-level), TCO | SSA, DCE, CSE, loop optimization, inlining, register allocation |
| **Information theory** | zstd compression, ChaCha20 CSPRNG | Entropy coding, bytecode compression, statistical profiling |
| **Type theory** | STLC + base types + implicit null subtypes | Hindley-Milner, linear types, effect rows, dependent types |
| **Proof theory** | None | Curry-Howard, NBE, abstract interpretation, formal verification |
