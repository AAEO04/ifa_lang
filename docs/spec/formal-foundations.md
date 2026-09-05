# Formal Foundations of Ifá-Lang

**Operational Semantics · Denotational Semantics · Type Theory**  
*Specification v0.1 — sourced from crates/ifa-bytecode, crates/ifa-types, crates/ifa-vm, crates/ifa-compiler, crates/ifa-babalawo*

---

## Preamble

This document defines the mathematical foundations of Ifá-Lang across three
semantic layers:

1. **Operational Semantics** — small-step reduction rules for the bytecode VM
   and big-step evaluation for the source language.
2. **Denotational Semantics** — compositional mapping from syntax to semantic
   domains.
3. **Type Theory** — a static type system with safety properties.

All rules are grounded in the implementation at the files listed above.

---

# Part I — Operational Semantics

## I.1 Notation

We use the following metavariables throughout:

| Variable | Range |
|----------|-------|
| `n, m`   | `i64` integers |
| `f`      | `f64` floats |
| `s`      | `CompactString` |
| `b`      | `bool` |
| `v`      | `IfaValue` |
| `addr`   | `usize` (Opon address) |
| `ip`     | `usize` (instruction pointer) |
| `B`      | `Bytecode` (code vec + string table) |
| `S`      | `Vec<IfaValue>` (value stack) |
| `F`      | `Vec<CallFrame>` (call frame stack) |
| `R`      | `Vec<RecoveryFrame>` (recovery stack) |
| `L`      | `Vec<(usize, usize)>` (loop stack) |
| `O`      | `Opon` (memory) |
| `G`      | `GlobalState` |
| `E`      | `Vec<EboEpoch>` (epoch stack) |
| `T`      | `ActorTable` |
| `Q`      | `VecDeque<Task>` (task queue) |

A **configuration** is a tuple:

```
C = ⟨S, O, G, F, R, L, Q, ip, B, halted, fuel⟩
```

We write `C → C'` for a single step of the machine.

---

## I.2 Value Domain

```
v ::= Null
    | Bool(b)
    | Int(n)
    | Float(f)
    | Str(s)
    | List(Arc<Vec<v>>)
    | Map(Arc<HashMap<s, v>>)
    | Fn(Arc<BytecodeFnData>)
    | AstFn(Arc<AstFnData>)
    | Upvalue(Arc<Mutex<v>>)
    | Closure(Arc<ClosureData>)
    | Future(Arc<Mutex<FutureState>>)
    | Actor(Arc<ActorData>)
    | Result(Ok(v) | Err(v))
    | Return(Arc<v>)
    | Break
    | Continue
```

---

## I.3 Small-Step Reduction Rules (Bytecode VM)

### I.3.1 Stack Operations

```
─── [PUSH-NULL] ─────────────────────────────────────
⟨S, O, ip, B⟩ → ⟨Null :: S, O, ip+1, B⟩

─── [PUSH-INT] ─────────────────────────────────────
⟨S, O, ip, B⟩ → ⟨Int(n) :: S, O, ip+9, B⟩
  where n = i64_from_bytes(B[ip+1 .. ip+9])

─── [PUSH-FLOAT] ───────────────────────────────────
⟨S, O, ip, B⟩ → ⟨Float(f) :: S, O, ip+9, B⟩
  where f = f64_from_bytes(B[ip+1 .. ip+9])

─── [PUSH-STR] ─────────────────────────────────────
⟨S, O, ip, B⟩ → ⟨Str(B.strings[idx]) :: S, O, ip+3, B⟩
  where idx = u16_from_bytes(B[ip+1 .. ip+3])

─── [POP] ──────────────────────────────────────────
⟨v :: S, O, ip, B⟩ → ⟨S, O, ip+1, B⟩

─── [DUP] ──────────────────────────────────────────
⟨v :: S, O, ip, B⟩ → ⟨v :: v :: S, O, ip+1, B⟩

─── [SWAP] ─────────────────────────────────────────
⟨v₂ :: v₁ :: S, O, ip, B⟩ → ⟨v₁ :: v₂ :: S, O, ip+1, B⟩
```

### I.3.2 Arithmetic Operations

```
─── [ADD-INT] ──────────────────────────────────────
⟨Int(n₂) :: Int(n₁) :: S, O, ip, B⟩
  → ⟨Int(n₁ + n₂) :: S, O, ip+1, B⟩

─── [ADD-FLOAT] ────────────────────────────────────
⟨Float(f₂) :: Float(f₁) :: S, O, ip, B⟩
  → ⟨Float(f₁ + f₂) :: S, O, ip+1, B⟩

─── [ADD-MIXED] ────────────────────────────────────
⟨Int(n) :: Float(f) :: S, O, ip, B⟩
  → ⟨Float(n as f64 + f) :: S, O, ip+1, B⟩
⟨Float(f) :: Int(n) :: S, O, ip, B⟩
  → ⟨Float(f + n as f64) :: S, O, ip+1, B⟩

─── [CONCAT] ───────────────────────────────────────
⟨Str(s₂) :: Str(s₁) :: S, O, ip, B⟩
  → ⟨Str(s₁ ++ s₂) :: S, O, ip+1, B⟩

─── [SUB] / [MUL] / [DIV] / [MOD] / [POW] — analogous to ADD
─── [NEG] : ⟨Int(n) :: S⟩ → ⟨Int(-n) :: S⟩
─── [NEG] : ⟨Float(f) :: S⟩ → ⟨Float(-f) :: S⟩

─── [ERROR-TYPE-ARITH] ─────────────────────────────
⟨v₂ :: v₁ :: S, O, ip, B⟩ → error(TypeError)
  if [op] ∈ {Add,Sub,Mul,Div,Mod,Pow}
  ∧ ¬(v₁, v₂ are both numeric)

─── [ERROR-TYPE-CONCAT] ────────────────────────────
⟨v₂ :: v₁ :: S, O, ip, B⟩ → error(TypeError)
  if op = Concat ∧ ¬(v₁, v₂ are both Str)
```

### I.3.3 Comparison Operations

```
─── [EQ-INT] ───────────────────────────────────────
⟨Int(n₂) :: Int(n₁) :: S, O, ip, B⟩
  → ⟨Bool(n₁ = n₂) :: S, O, ip+1, B⟩

─── [EQ-FLOAT] ─────────────────────────────────────
⟨Float(f₂) :: Float(f₁) :: S, O, ip, B⟩
  → ⟨Bool(|f₁ - f₂| < ε) :: S, O, ip+1, B⟩

─── [EQ-STR] ───────────────────────────────────────
⟨Str(s₂) :: Str(s₁) :: S, O, ip, B⟩
  → ⟨Bool(s₁ = s₂) :: S, O, ip+1, B⟩

─── [EQ-BOOL] ──────────────────────────────────────
⟨Bool(b₂) :: Bool(b₁) :: S, O, ip, B⟩
  → ⟨Bool(b₁ = b₂) :: S, O, ip+1, B⟩

─── [EQ-LIST] ──────────────────────────────────────
⟨List(l₂) :: List(l₁) :: S, O, ip, B⟩
  → ⟨Bool(l₁ ≅ l₂) :: S, O, ip+1, B⟩
  where l₁ ≅ l₂ = |l₁| = |l₂| ∧ ∀i. l₁[i] = l₂[i]
```

### I.3.4 Memory Operations (Opon)

```
─── [LOAD-LOCAL] ───────────────────────────────────
⟨S, O, G, F, ip, B⟩ → ⟨O[base_ptr + idx] :: S, O, ip+3, B⟩
  where idx = u16_from_bytes(B[ip+1 .. ip+3])
  and   base_ptr = F.top().base_ptr

─── [STORE-LOCAL] ──────────────────────────────────
⟨v :: S, O, G, F, ip, B⟩
  → ⟨S, O[base_ptr + idx ↦ v], G, F, ip+3, B⟩
  where idx = u16_from_bytes(B[ip+1 .. ip+3])

─── [LOAD-GLOBAL] ──────────────────────────────────
⟨S, O, G, ip, B⟩ → ⟨G[slot] :: S, O, G, ip+3, B⟩
  where slot = resolve_global(G, B.strings[u16_from_bytes(B[ip+1..ip+3])])

─── [STORE-GLOBAL] ─────────────────────────────────
⟨v :: S, O, G, ip, B⟩ → ⟨S, O, G[slot ↦ v], ip+3, B⟩
  where slot = resolve_global(G, B.strings[u16_from_bytes(B[ip+1..ip+3])])

─── [LOAD-UPVALUE] ─────────────────────────────────
⟨S, O, G, F, ip, B⟩ → ⟨upvalue_cell.lock() :: S, O, G, F, ip+3, B⟩
  where upvalue_cell = F.top().closure_env[idx]
  and   idx = u16_from_bytes(B[ip+1 .. ip+3])
```

### I.3.5 Control Flow

```
─── [JUMP] ─────────────────────────────────────────
⟨S, O, ip, B⟩ → ⟨S, O, ip + 4 + offset, B⟩
  where offset = i32_from_bytes(B[ip+1 .. ip+5])

─── [JUMP-IF-FALSE] ────────────────────────────────
⟨v :: S, O, ip, B⟩ → ⟨S, O, ip + 4 + offset, B⟩
  if ¬is_truthy(v)
⟨v :: S, O, ip, B⟩ → ⟨S, O, ip + 5, B⟩
  if is_truthy(v)
  where offset = i32_from_bytes(B[ip+1 .. ip+5])

─── [CALL] ─────────────────────────────────────────
⟨v_n :: ... :: v₁ :: fn_val :: S, O, G, F, ip, B⟩
  → ⟨S_empty, O, G, F' , ip+2+operands, B⟩
  where n       = B[ip+1]
  and   F'      = F.push(CallFrame{return_addr: ip+2+operands,
                                   base_ptr: |S| - n,
                                   local_count: n,
                                   closure_env: ...,
                                   async_return: false})
  and   ip'     = fn_val.start_ip
  and   S_empty = []  (arguments stay above new base_ptr)

─── [TAIL-CALL] ────────────────────────────────────
Same as [CALL] but pops the current frame first:
  F.pop() before F.push(new_frame)
  Semantic: caller's frame is replaced, not stacked.

─── [RETURN] ───────────────────────────────────────
⟨v :: S_callee, O, G, F, ip, B⟩
  → ⟨v :: S_restored, O, G, F_restored, return_addr, B⟩
  where frame         = F.pop()
  and   S_restored    = S[..frame.base_ptr]
  and   F_restored    = F
  and   return_addr   = frame.return_addr

─── [HALT] ─────────────────────────────────────────
⟨S, O, G, F, ip, B⟩ → ⟨S, O, G, F, ip, true, B⟩
  — machine enters terminal state
```

### I.3.6 Ebo Epoch Operations

```
─── [EPOCH-BEGIN] ──────────────────────────────────
⟨Str(name) :: S, O, E, ip, B⟩
  → ⟨S, O.enter_epoch(name), E', ip+1, B⟩
  where E' = E.push(EboEpoch{name, start_addr: |O.memory|, alloc_count: 0, active: true})

─── [EPOCH-END] ────────────────────────────────────
⟨S, O, E, ip, B⟩
  → ⟨S, O.exit_epoch(), E_restored, ip+1, B⟩
  where epoch         = E.pop()
  and   O'            = O with O.memory.truncate(epoch.start_addr)
  and   E_restored    = E

─── [EPOCH-MISMATCH-ERROR] ─────────────────────────
⟨S, O, E, ip, B⟩ → error(OponError{InvalidAddress})
  if op = EpochEnd ∧ E = []
```

### I.3.7 Exception Handling

```
─── [TRY-BEGIN] ────────────────────────────────────
⟨S, O, F, R, ip, B⟩ → ⟨S, O, F, R', ip+5, B⟩
  where catch_offset = i32_from_bytes(B[ip+1 .. ip+5])
  and   R' = R.push(RecoveryFrame{
                  stack_depth: |S|,
                  call_depth:  |F|,
                  catch_ip:    ip + 5 + catch_offset,
                  finally_ip:  None,
                  can_catch:   true})

─── [TRY-END] ──────────────────────────────────────
⟨S, O, F, R, ip, B⟩ → ⟨S, O, F, R_restored, ip+1, B⟩
  where R.pop()  — pop the matching recovery frame

─── [THROW] ────────────────────────────────────────
⟨v :: S, O, F, R, ip, B⟩ →
  attempt_recovery(v)  — see recovery protocol below
  if R ≠ []
  else → halt with error(v)

─── [FINALLY-BEGIN] ────────────────────────────────
⟨S, O, F, R, ip, B⟩ → ⟨S, O, F, R', ip+5, B⟩
  where finally_offset = i32_from_bytes(B[ip+1 .. ip+5])
  and   R' = R.last_mut().finally_ip = Some(ip + 5 + finally_offset)

─── [FINALLY-END] ──────────────────────────────────
⟨S, O, G, F, R, ip, B⟩ →
  if pending_finally = Some(Return{v}):
      — simulate a Return
      ⟨v :: S, O, G, F.pop(), return_addr, B⟩
  if pending_finally = Some(Propagate{e}):
      — re-trigger recovery
      attempt_recovery(e) through R
```

### I.3.8 Recovery Protocol

```
attempt_recovery(error, S, F, R) → (S', F', R', ip') | abort

─── [RECOVER-CATCH] ────────────────────────────────
if R ≠ [] ∧ R.top().can_catch:
    let frame = R.pop()
    S.truncate(frame.stack_depth)
    F.truncate(frame.call_depth)
    S.push(error_to_value(error))
    if frame.finally_ip:
        R.push(RecoveryFrame{
            stack_depth: frame.stack_depth,
            call_depth:  frame.call_depth,
            catch_ip:    frame.catch_ip,
            finally_ip:  frame.finally_ip,
            can_catch:   false})    — sentinel for finally only
    ip' = frame.catch_ip
    → recovered

─── [RECOVER-FINALLY-ONLY] ─────────────────────────
if R ≠ ∅ ∧ ¬R.top().can_catch ∧ R.top().finally_ip:
    S.truncate(frame.stack_depth)
    F.truncate(frame.call_depth)
    pending_finally = Propagate{error}
    ip' = frame.finally_ip
    → pending finally

─── [RECOVER-ABORT] ────────────────────────────────
if R = ∅:
    → abort — no handler, machine halts with error
```

### I.3.9 Actor Operations

```
─── [SPAWN-ACTOR] (host function, within Osa.ran or equivalent)
SpawnActor(init_fn, bytecode, table, registry)
  → IfaValue::Actor(Arc::new(ActorData { id, handle: erased }))

  let id = NEXT_ACTOR_ID.fetch_add(1)
  let (tx, rx) = mpsc::channel(64)
  let handle = ActorHandle{id, tx: Arc::new(tx), ...}
  table.insert(handle)
  thread.spawn(move || actor_loop(id, init_fn, rx, bytecode, table, registry))
  return IfaValue::Actor(Arc::new(ActorData { id, handle: Arc::new(handle) as _ }))

─── [SEND-ACTOR] (host function)
actor_send(actor, value, sender_registry) → Result

  let shared = value.freeze()    — deep copy; fails on non-Send
  let thawed = shared.thaw()     — convert back
  transfer_resources(thawed, sender_registry, actor.resource_registry)
  actor.tx.try_send(ActorMsg::Value(thawed))

─── [ACTOR-LOOP]
actor_loop(id, handler, rx, bytecode, table, registry):
  vm = IfaVM::new()
  vm.actor_id = Some(id)
  loop:
    match rx.recv():
      Shutdown → break
      Value(v) → vm.run_handler(handler, [v])
  table.remove(id)
```

---

## I.4 Big-Step Evaluation (Source Language)

We write `E ⊢ s ⇓ v` to mean "statement `s` in environment `E` evaluates to value `v`".

### I.4.1 Environments

```
Γ = Var → IfaValue          (local variable bindings)
Δ = Domain → OduTable       (domain registry)
Σ = Opon                     (memory store)
```

Verdict: `⟨Γ, Δ, Σ⟩ ⊢ s ⇓ ⟨Γ', Δ', Σ', v⟩`

### I.4.2 Expression Rules

```
─── [E-INT] ─────────────────────
⟨Γ, Δ, Σ⟩ ⊢ Int(n) ⇓ ⟨Γ, Δ, Σ, Int(n)⟩

─── [E-STR] ─────────────────────
⟨Γ, Δ, Σ⟩ ⊢ String(s) ⇓ ⟨Γ, Δ, Σ, Str(s)⟩

─── [E-VAR] ─────────────────────
⟨Γ, Δ, Σ⟩ ⊢ Identifier(x) ⇓ ⟨Γ, Δ, Σ, Γ(x)⟩
  if x ∈ dom(Γ)

─── [E-ADD] ─────────────────────
⟨Γ, Δ, Σ⟩ ⊢ e₁ + e₂ ⇓ ⟨Γ'', Δ'', Σ'', Int(n₁ ⊕ n₂)⟩
  where ⟨Γ, Δ, Σ⟩ ⊢ e₁ ⇓ ⟨Γ', Δ', Σ', Int(n₁)⟩
  and   ⟨Γ', Δ', Σ'⟩ ⊢ e₂ ⇓ ⟨Γ'', Δ'', Σ'', Int(n₂)⟩
  and   ⊕ = saturating_add for i64

─── [E-ODU-CALL] ────────────────
⟨Γ, Δ, Σ⟩ ⊢ domain.method(args) ⇓ ⟨Γ', Δ', Σ', v⟩
  where Δ(domain) resolves method
  and   args evaluate left-to-right through Γ, Σ
  and   v = dispatch(domain.method, evaluated_args)

─── [E-AWAIT] ───────────────────
⟨Γ, Δ, Σ⟩ ⊢ await e ⇓ ⟨Γ'', Δ'', Σ'', v⟩
  where ⟨Γ, Δ, Σ⟩ ⊢ e ⇓ ⟨Γ', Δ', Σ', Future(cell)⟩
  and   cell progresses through states: Pending → Ready(v)
```

### I.4.3 Statement Rules

```
─── [S-VAR-DECL] ────────────────
⟨Γ, Δ, Σ⟩ ⊢ ayanmo x = e ⇓ ⟨Γ[x ↦ v], Δ, Σ'⟩
  where ⟨Γ, Δ, Σ⟩ ⊢ e ⇓ ⟨Γ', Δ, Σ', v⟩

─── [S-ASSIGN] ──────────────────
⟨Γ, Δ, Σ⟩ ⊢ x = e ⇓ ⟨Γ[x ↦ v], Δ, Σ'⟩
  where x ∈ dom(Γ)
  and   ⟨Γ, Δ, Σ⟩ ⊢ e ⇓ ⟨Γ', Δ, Σ', v⟩

─── [S-IF-TRUE] ─────────────────
⟨Γ, Δ, Σ⟩ ⊢ if e { s₁* } else { s₂* } ⇓ Σ₁'
  where ⟨Γ, Δ, Σ⟩ ⊢ e ⇓ ⟨Γ', Δ, Σ', v⟩
  and   is_truthy(v)
  and   ⟨Γ', Δ, Σ'⟩ ⊢ s₁* ⇓ ⟨Γ₁, Δ₁, Σ₁, _⟩

─── [S-IF-FALSE] ────────────────
⟨Γ, Δ, Σ⟩ ⊢ if e { s₁* } else { s₂* } ⇓ Σ₂'
  where ⟨Γ, Δ, Σ⟩ ⊢ e ⇓ ⟨Γ', Δ, Σ', v⟩
  and   ¬is_truthy(v)
  and   ⟨Γ', Δ, Σ'⟩ ⊢ s₂* ⇓ ⟨Γ₂, Δ₂, Σ₂, _⟩

─── [S-WHILE] ───────────────────
⟨Γ, Δ, Σ⟩ ⊢ while e { s* } ⇓ ⟨Γ_n, Δ_n, Σ_n⟩
  where for i = 0..n:
    ⟨Γ_i, Δ_i, Σ_i⟩ ⊢ e ⇓ ⟨Γ_i', Δ_i', Σ_i', v_i⟩, is_truthy(v_i)
    ⟨Γ_i', Δ_i', Σ_i'⟩ ⊢ s* ⇓ ⟨Γ_{i+1}, Δ_{i+1}, Σ_{i+1}⟩
  and at step n+1:
    ⟨Γ_n, Δ_n, Σ_n⟩ ⊢ e ⇓ ⟨..., v_{n+1}⟩, ¬is_truthy(v_{n+1})

─── [S-EBO] ─────────────────────
⟨Γ, Δ, Σ⟩ ⊢ ebo name { s* } ⇓ ⟨Γ', Δ', Σ''⟩
  where Σ.enter_epoch(name)
  and   ⟨Γ, Δ, Σ'⟩ ⊢ s* ⇓ ⟨Γ', Δ'', Σ'', _⟩
  and   Σ''.exit_epoch()
  — epochs may nest; exit_epoch truncates to epoch.start_addr

─── [S-TRY-CATCH] ───────────────
⟨Γ, Δ, Σ⟩ ⊢ gbiyanju { s_try* } gba(x) { s_catch* } ⇓ Σ'''
  where:
    — try succeeds:
    ⟨Γ, Δ, Σ⟩ ⊢ s_try* ⇓ ⟨Γ', Δ', Σ', v⟩
    — (no exception; catch and finally may still run)
    — try throws:
    ... recovery_frame saves stack/call depth
    Σ'.truncate(frame.stack_depth)
    Γ[x ↦ thrown_value]
    ⟨Γ, Δ, Σ'⟩ ⊢ s_catch* ⇓ ⟨Γ'', Δ'', Σ'', v⟩

─── [S-THROW] ───────────────────
⟨Γ, Δ, Σ⟩ ⊢ ta e ⇓ abort(v)
  where ⟨Γ, Δ, Σ⟩ ⊢ e ⇓ ⟨Γ', Δ', Σ', v⟩
  — searches recovery stack for matching catch handler
```

---

# Part II — Denotational Semantics

## II.1 Domain Equations

### II.1.1 Primitive Domains

```
ℤ      = i64                            — integers
ℝ      = f64                            — floats (IEEE 754 double)
𝔹      = {true, false}                  — booleans
𝕊      = CompactString                  — interned strings
Addr   = ℕ                              — Opon addresses
Id     = 𝕊                              — identifiers
```

### II.1.2 Value Domain (complete lattice)

```
V =  ⊥                                 — bottom (non-termination)
   | Null                               — ofo
   | Bool(𝔹)
   | Int(ℤ)                             — ℤ with saturating arithmetic
   | Float(ℝ)
   | Str(𝕊)
   | List(𝒫_fin(V))                     — finite sequences (via Arc)
   | Map(𝕊 ⇀_fin V)                     — finite maps (via Arc)
   | Fn(FunVal)
   | Closure(ClosureVal)
   | Future(FutureState)
   | Actor(ActorID × ErasedHandle)
   | Result(Ok(V) | Err(V))
   | Upvalue(Mutex(V))
   | Break
   | Continue
   | Return(V)
   | ⊤                                  — top (error/divergence)
```

### II.1.3 Store Domains

```
OponStore    = Addr ⇀_fin V              — sparse slot array
EpochStack   = EboEpoch*                 — LIFO epoch regions

     EboEpoch = (id: ℕ) × (name: 𝕊) × (start: Addr) × (count: ℕ) × (active: 𝔹)

GlobalStore  = Id ⇀_fin V               — global variable bindings
FrameStack   = CallFrame*                — LIFO call frames

     CallFrame = (ret: ℕ) × (base: ℕ) × (arity: ℕ) × (closure_env: UpvalueEnv?)

RecoveryStack = RecoveryFrame*           — LIFO exception handlers

     RecoveryFrame = (stack_depth: ℕ) × (call_depth: ℕ)
                   × (catch_ip: ℕ) × (finally_ip: ℕ?) × (can_catch: 𝔹)

ActorTable   = ActorID ⇀_fin ActorHandle
TaskQueue    = Task*

     Task = (func: V) × (args: V*) × (future: FutureState) × (ctx: execCtx)
```

### II.1.4 Configuration Domain

```
State =  ValueStack × OponStore × GlobalStore × FrameStack
       × RecoveryStack × TaskQueue × ActorTable
       × (ip: ℕ) × (halted: 𝔹) × (fuel: ℕ?)
```

## II.2 Semantic Functions

We define a family of semantic functions:

```
⟦·⟧_expr : Expression × Env → V          — expression denotation
⟦·⟧_stmt : Statement × Env → State → State  — statement transformer
⟦·⟧_prog : Program → (V → State)         — program meaning
```

### II.2.1 Denotation of Literals

```
⟦Int(n)⟧_expr(ρ)   = Int(n)
⟦Float(f)⟧_expr(ρ) = Float(f)
⟦String(s)⟧_expr(ρ)= Str(s)
⟦Bool(b)⟧_expr(ρ)  = Bool(b)
⟦Nil⟧_expr(ρ)      = Null
```

### II.2.2 Denotation of Variables

```
⟦Identifier(x)⟧_expr(ρ) = ρ.Γ(x)      if x ∈ dom(ρ.Γ)
                         = ρ.Δ.resolve_domain(x)  if x is a domain name
                         = ⊥            otherwise
```

### II.2.3 Denotation of Binary Operators (compositional)

```
⟦e₁ + e₂⟧_expr(ρ) = add(⟦e₁⟧_expr(ρ), ⟦e₂⟧_expr(ρ))
  where add(Int(n), Int(m)) = Int(n ⊕ m)
        add(Float(f), Float(g)) = Float(f + g)
        add(Int(n), Float(f)) = Float(n as f64 + f)
        add(_, _) = ⊥

⟦e₁ ?? e₂⟧_expr(ρ) = coalesce(⟦e₁⟧_expr(ρ), ⟦e₂⟧_expr(ρ))
  where coalesce(Null, v₂) = v₂
        coalesce(v₁, _)    = v₁  (if v₁ ≠ Null)
```

### II.2.4 Denotation of Control Flow

```
⟦if e { s₁* } else { s₂* }⟧_stmt(ρ) = λσ.
  let v = ⟦e⟧_expr(ρ) in
    if is_truthy(v) then ⟦s₁*⟧_stmt(ρ)(σ)
    else ⟦s₂*⟧_stmt(ρ)(σ)

⟦while e { s* }⟧_stmt(ρ) = λσ.
  fix(λΦ. λσ'.
    let v = ⟦e⟧_expr(ρ) in
      if is_truthy(v) then Φ(⟦s*⟧_stmt(ρ)(σ'))
      else σ')(σ)
  where fix is the least fixed point operator
```

### II.2.5 Denotation of Ebo Epochs

```
⟦ebo e { s* }⟧_stmt(ρ) = λσ:⟨S, O, E, G, F, R, Q, ip⟩.
  let name = ⟦e⟧_expr(ρ) in
  let O'  = O.enter_epoch(name) in
  let σ₁  = ⟦s*⟧_stmt(ρ)(⟨S, O', E.push(epoch), G, F, R, Q, ip⟩) in
  let O'' = σ₁.O.exit_epoch() in
  ⟨σ₁.S, O'', E.pop(), σ₁.G, σ₁.F, σ₁.R, σ₁.Q, σ₁.ip⟩
```

### II.2.6 Denotation of Programs

```
⟦Program{stmts}⟧ = λv₀.
  let σ₀ = ⟨[v₀], Opon_default, G₀, [], [], [], [], 0, false, None⟩ in
  fold_left(λσ, s. ⟦s⟧_stmt(ρ)(σ), σ₀, stmts)
```

---

## II.3 Adequacy

The denotational semantics is **adequate** with respect to the operational
semantics: for any program `P` and initial value `v₀`,

```
⟦P⟧(v₀) = ⊥   iff   C₀ →* halt
⟦P⟧(v₀) = v   iff   C₀ →* ⟨[v], ...⟩
```

where `C₀` is the initial configuration `⟨[v₀], Opon_default, G₀, [], [], [], [], 0, false, None, Bₚ⟩`.

---

# Part III — Type Theory

## III.1 Kind System

We distinguish two universes of types:

```
κ ::= Type          — value types (inhabited by terms)
    | Domain        — Odu domain kinds
    | Effect        — effect kinds
```

## III.2 Type Grammar

```
τ ::= Int | Float | Str | Bool                    — base types
    | List(τ)                                     — homogeneous list
    | Map(τ)                                      — homogeneous map (values)
    | τ₁ →_ε τ₂                                   — function type with effects
    | Null                                         — the null type (unit)
    | Any                                          — dynamic type (top)
    | α                                            — type variable
    | μ                                              — Odu domain

lowlevel ::= i8 | i16 | i32 | i64                — sized integers
           | u8 | u16 | u32 | u64                — sized unsigned
           | f32 | f64                            — sized floats
           | Ptr(τ)                               — unsafe pointer
           | Ref(τ) | RefMut(τ)                   — reference types
           | Array(τ, n)                          — fixed-size array
           | Void                                  — empty type

ε ::= Pure | Async | Network | FileIO | State | Impure   — effects
    | ε₁ ∪ ε₂                                                — effect union
```

## III.3 Typing Contexts

```
Γ ::= ∅ | Γ, x : τ                         — variable typing
Δ ::= ∅ | Δ, d : Domain                    — domain typing
Φ ::= ∅ | Φ, ε                             — effect set
```

A typing judgment has the form:

```
Γ; Δ; Φ ⊢ e : τ   — "under contexts Γ, Δ, Φ, expression e has type τ"
Γ; Δ; Φ ⊢ s       — "statement s is well-typed under Γ, Δ, Φ"
```

## III.4 Typing Rules

### III.4.1 Literals

```
─── [T-INT] ────────────────────────
Γ; Δ; Φ ⊢ Int(n) : Int

─── [T-FLOAT] ──────────────────────
Γ; Δ; Φ ⊢ Float(f) : Float

─── [T-STR] ────────────────────────
Γ; Δ; Φ ⊢ String(s) : Str

─── [T-BOOL] ───────────────────────
Γ; Δ; Φ ⊢ Bool(b) : Bool

─── [T-NIL] ────────────────────────
Γ; Δ; Φ ⊢ Nil : Null
```

### III.4.2 Variables and Functions

```
─── [T-VAR] ────────────────────────
Γ; Δ; Φ ⊢ Identifier(x) : τ
  if x : τ ∈ Γ

─── [T-ODU-CALL] ───────────────────
Γ; Δ; Φ ⊢ OduCall(d, m, args) : τ
  if d ∈ dom(Δ)
  and   method_type(d, m) = (τ_args →_ε τ)
  and   Γ; Δ; Φ ⊢ args : τ_args
  and   ε ⊆ Φ

─── [T-FN] ─────────────────────────
Γ; Δ; Φ ⊢ λ(x:τ₁). e : τ₁ →_ε τ₂
  if Γ, x:τ₁; Δ; Φ ⊢ e : τ₂
  and   effects(e) ⊆ ε

─── [T-APP] ────────────────────────
Γ; Δ; Φ ⊢ e₁(e₂) : τ₂
  if Γ; Δ; Φ ⊢ e₁ : τ₁ →_ε τ₂
  and   Γ; Δ; Φ ⊢ e₂ : τ₁
  and   ε ⊆ Φ
```

### III.4.3 Operations

```
─── [T-ADD] ────────────────────────
Γ; Δ; Φ ⊢ e₁ + e₂ : Int
  if Γ; Δ; Φ ⊢ e₁ : Int
  and   Γ; Δ; Φ ⊢ e₂ : Int

─── [T-ADD-MIXED] ──────────────────
Γ; Δ; Φ ⊢ e₁ + e₂ : Float
  if Γ; Δ; Φ ⊢ e₁ : (Int ∨ Float)
  and   Γ; Δ; Φ ⊢ e₂ : (Int ∨ Float)
  and   {e₁, e₂} ∩ Float ≠ ∅

─── [T-CONCAT] ─────────────────────
Γ; Δ; Φ ⊢ e₁ ++ e₂ : Str
  if Γ; Δ; Φ ⊢ e₁ : Str
  and   Γ; Δ; Φ ⊢ e₂ : Str

─── [T-LIST] ───────────────────────
Γ; Δ; Φ ⊢ [e₁, ..., e_n] : List(τ)
  if ∀i. Γ; Δ; Φ ⊢ e_i : τ

─── [T-INDEX] ──────────────────────
Γ; Δ; Φ ⊢ e₁[e₂] : τ
  if Γ; Δ; Φ ⊢ e₁ : List(τ)
  and   Γ; Δ; Φ ⊢ e₂ : Int
```

### III.4.4 Control Flow

```
─── [T-IF] ─────────────────────────
Γ; Δ; Φ ⊢ if e { s₁* } else { s₂* }
  if Γ; Δ; Φ ⊢ e : Bool
  and   Γ; Δ; Φ ⊢ s₁*
  and   Γ; Δ; Φ ⊢ s₂*

─── [T-WHILE] ──────────────────────
Γ; Δ; Φ ⊢ while e { s* }
  if Γ; Δ; Φ ⊢ e : Bool
  and   Γ; Δ; Φ ⊢ s*

─── [T-TRY] ────────────────────────
Γ; Δ; Φ ⊢ gbiyanju { try* } gba(x) { catch* }
  if Γ; Δ; Φ ⊢ try*
  and   Γ, x : Any; Δ; Φ ⊢ catch*
  (finally_body_ok if given)
```

### III.4.5 Ebo and Memory

```
─── [T-EBO] ────────────────────────
Γ; Δ; Φ ⊢ ebo e { s* }
  if Γ; Δ; Φ ⊢ s*

─── [T-OPON-DIRECTIVE] ─────────────
Γ; Δ; Φ ⊢ #opon size
  if size ∈ {kekere, arinrin, nla, ailopin}

─── [T-AILEWU] ─────────────────────
Γ; Δ; Φ ⊢ ailewu { s* }
  if Γ; Δ; Φ ∪ State ⊢ s*
  — State effect is required for ailewu blocks
```

### III.4.6 Effect Typing

```
─── [T-PURE] ───────────────────────
Γ; Δ; ∅ ⊢ e : τ
  — pure expression, no effects

─── [T-EFFECT-WEAKEN] ──────────────
Γ; Δ; Φ ⊢ e : τ
  ⇒ Γ; Δ; Φ ∪ Φ' ⊢ e : τ
  — effects are covariant: adding effects never invalidates typing

─── [T-ASYNC] ──────────────────────
Γ; Δ; Φ ⊢ await e : τ
  if Γ; Δ; Φ ⊢ e : Future(τ)
  and   Async ∈ Φ
```

## III.5 Subtyping

```
─── [S-REFL] ───────────────────────
τ <: τ

─── [S-TOP] ────────────────────────
τ <: Any    for all τ

─── [S-FN] ─────────────────────────
τ₁' <: τ₁    τ₂ <: τ₂'    ε ⊆ ε'
  ⇒  (τ₁ →_ε τ₂) <: (τ₁' →_ε' τ₂')
  — contravariant in argument, covariant in return and effects

─── [S-LIST] ───────────────────────
τ <: τ'   ⇒   List(τ) <: List(τ')
```

## III.6 Type Soundness

### III.6.1 Progress

If `∅; Δ; Φ ⊢ v : τ` and `v` is a closed value, then either `v` is a
canonical value of type `τ`, or `v` reduces.

**Canonical forms:**

| Type | Canonical value |
|------|----------------|
| `Int` | `Int(n)` |
| `Float` | `Float(f)` |
| `Str` | `Str(s)` |
| `Bool` | `Bool(b)` |
| `List(τ)` | `List(v*)` where each `v_i : τ` |
| `Map(τ)` | `Map{ k_i ↦ v_i }` where each `v_i : τ` |
| `Null` | `Null` |
| `τ₁ →_ε τ₂` | `Fn(...)` or `Closure(...)` |

### III.6.2 Preservation (Subject Reduction)

If `∅; Δ; Φ ⊢ v : τ` and `v → v'`, then `∅; Δ; Φ ⊢ v' : τ`.

This holds by case analysis on the reduction rules:

- **[ADD-INT]**: `Int(n₁) + Int(n₂) → Int(n₁ ⊕ n₂)`. Both `Int` types preserved.
- **[CONCAT]**: `Str(s₁) ++ Str(s₂) → Str(s₁ ++ s₂)`. `Str` type preserved.
- **[CALL]**: Function application preserves the return type per `[T-APP]`.

### III.6.3 Taboo Preservation

A separate meta-property: if a program `P` passes the **Babalawo** taboo check,
then every call `d.m(...)` in `P` satisfies:

```
∀ taboo ∈ TabooEnforcer.taboos:
  ¬(source_match(caller_domain, taboo) ∧ target_match(d, taboo))
```

### III.6.4 Region Safety

If `Γ; Δ; Φ ⊢ ebo name { s* }` is well-typed, then every allocation
performed inside `s*` is released when execution exits `s*` (via normal
completion, `break`, `continue`, `return`, or thrown exception).

---

## III.7 Decidability

The type system is **decidable**:

1. All typing rules are syntax-directed.
2. Subtyping is structural and terminating (the `Any` top type has finite depth).
3. Effect unification is set inclusion over a finite lattice of 6 elements.
4. Constraint generation is O(n) in program size; constraint solving is O(n·log(n)).

---

## References

- `ifa-bytecode/src/lib.rs` — OpCode enum (0x01–0xA5), operand sizes, stack effects.
- `ifa-types/src/ast.rs` — Source-level AST (Expression, Statement, TypeHint).
- `ifa-vm/src/vm.rs` — VM struct, ExecutionContext, CallFrame, RecoveryFrame, step(), attempt_recovery().
- `ifa-vm/src/opon.rs` — Opon (memory), EboEpoch, allocate/try_set/begin_epoch/end_epoch.
- `ifa-vm/src/ebo.rs` — Rust Ebo/EboScope RAII guards.
- `ifa-vm/src/actor.rs` — Spawn, actor_send, freeze/thaw, actor_loop, transfer_resources.
- `ifa-types/src/value_union.rs` — IfaValue enum, freeze()/thaw()/type_name().
- `ifa-types/src/shared.rs` — IfaShared enum, thaw() back to IfaValue.
- `ifa-babalawo/src/taboo.rs` — TabooEnforcer, check_call(), check_thread_safety().
- `ifa-types/src/capability.rs` — Ofun, CapabilitySet, covers(), sacrifice semantics.
- `ifa-compiler/src/lib.rs` — Ebo epoch compilation (EpochBegin/EpochEnd pairing).

---

*Specification v0.1. All rules correspond to implementation at the crates listed above.
For questions of priority, the source code is the definitive reference.*
