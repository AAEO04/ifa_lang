# Phase 5: Ẹgbẹ́ (Actor) Concurrency Implementation Plan
## Erlang-Style VM Isolation Model

> **Architectural Decision:** `IfaValue` will NOT be changed to `Send + Sync`. Upvalues will retain `Rc<RefCell>` for maximum single-threaded performance. Actors will run in completely isolated `IfaVM` instances, communicating purely through `bincode`-serialized message passing via the existing `IfaValueSurrogate`.

---

## Part 1: Syntax and AST

Actors in Ifá-Lang are declared using the `egbe` (society/group) keyword. They define internal state and message handlers.

### 1.1 Grammar (`grammar.pest`)
```pest
// Add alongside odu_def and ese_def
egbe_def = { public_mod? ~ egbe_kw ~ ident ~ "{" ~ egbe_body ~ "}" }
egbe_body = { (var_decl | handler_def)* }
handler_def = { "gba" ~ "(" ~ ident ~ ")" ~ "{" ~ statement* ~ "}" }
egbe_kw = _{ "egbe" | "ẹgbẹ́" | "actor" }
```

### 1.2 AST (`ifa-types/src/ast.rs`)
```rust
Statement::EgbeDef {
    name: String,
    visibility: Visibility,
    state: Vec<Statement>, // var_decls
    handlers: Vec<EgbeHandler>,
    span: Span,
}

pub struct EgbeHandler {
    pub msg_param: String,
    pub body: Vec<Statement>,
}
```

---

## Part 2: The Communication Boundary (Osa Domain)

Actors communicate via the `Osa` (async/task) domain. 

### 2.1 Spawning (`Osa.da`)
Spawning an actor creates a new thread, a new `IfaVM` instance, and returns an `ActorRef` (an integer ID).

```ifa
ayanmo counter = Osa.da(CounterEgbe);
```

### 2.2 Sending (`Osa.ebo`)
`ebo` (sacrifice/offering) is the primitive for sending a message.
```ifa
Osa.ebo(counter, "increment");
```

### 2.3 The Surrogate Channel (Rust Implementation)
In `ifa-std/src/osa.rs`, the actor registry manages communication:

```rust
use std::sync::mpsc;

// The message payload is serialized bytes, guaranteeing absolute isolation.
pub struct ActorMsg(Vec<u8>);

pub struct ActorRef {
    id: u64,
    tx: mpsc::Sender<ActorMsg>,
}

// Inside Osa.ebo(id, val) native handler:
let surrogate_bytes = bincode::serialize(&val).map_err(|_| 
    IfaError::Runtime("Cannot send closures or unserializable data to actor".into())
)?;
tx.send(ActorMsg(surrogate_bytes));
```

---

## Part 3: Runtime Isolation (The VM Engine)

An actor is literally a `std::thread` running a dedicated `IfaVM`. There is no global interpreter lock (GIL).

### 3.1 Actor Execution Loop
When `Osa.da` is called, the host runtime spawns the actor:

```rust
fn spawn_actor(egbe_bytecode: Bytecode) -> ActorRef {
    let (tx, rx) = mpsc::channel::<ActorMsg>();
    
    std::thread::spawn(move || {
        let mut vm = IfaVM::new();
        // 1. Execute the Egbe initialization (var_decls)
        vm.execute(&egbe_bytecode.init_code).unwrap();
        
        // 2. Event loop
        while let Ok(msg) = rx.recv() {
            // Deserialize back into an IfaValue in the NEW heap
            let val: IfaValue = bincode::deserialize(&msg.0).unwrap();
            
            // Push handler arguments and invoke the `gba` handler
            vm.push(val);
            vm.execute_handler();
        }
    });
    
    ActorRef { tx }
}
```

---

## Part 4: Babalawo Static Guarantees

While `bincode` serialization provides a runtime safety net (it will return a runtime error if you try to serialize a closure), we want compile-time guarantees.

### 4.1 Enforce `Èèwọ̀` (Taboo) on `Osa.ebo`
Add a check in `ifa-babalawo/src/checks.rs`:

```rust
fn check_actor_message(expr: &Expression, baba: &mut Babalawo) {
    if let Expression::MethodCall { object, method, args, .. } = expr {
        if is_osa_domain(object) && method == "ebo" {
            let payload = &args[1];
            // If the payload is statically known to be an `ese` (closure)
            // or an `odu` instance with methods, emit error:
            if contains_closure(payload) {
                baba.error(BabalawoError::TabooViolation(
                    "Cannot send closures across egbe boundaries".into()
                ));
            }
        }
    }
}
```

---

## Summary of the Flow

1. **User writes:** `Osa.ebo(actor, [1, 2, 3])`
2. **Babalawo checks:** "Is `[1, 2, 3]` a closure?" No. Passes.
3. **Sender VM executes:** `Osa.ebo` native handler intercepts `IfaValue::List`.
4. **Serialization:** `bincode` converts the list into a `Vec<u8>`. All `Arc` pointers are dereferenced. The data is deep-copied into a flat byte array.
5. **Channel:** The `Vec<u8>` is sent over a standard `mpsc::channel`.
6. **Receiver VM executes:** The actor thread wake up, `bincode::deserialize` reconstructs a *brand new* `IfaValue::List` with a fresh `Arc` in the receiver's memory space.
7. **Execution:** The receiver VM processes the message with zero lock contention against the sender.

This plan gives you 100% of Erlang's memory safety and concurrency semantics while keeping your single-threaded interpreter completely free of atomic locking overhead.
