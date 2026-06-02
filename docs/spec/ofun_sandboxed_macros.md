# Òfún: Sandboxed Macro System — Capability-Gated Metaprogramming

**Status:** `DRAFT`  
**Supersedes:** Nothing (new capability)  
**Crate:** `ifa-babalawo` (macro expansion), `ifa-types` (`src/ast.rs`), `ifa-sandbox` (`src/lib.rs`)

---

## 1. Purpose

Ifá-Lang's macro system allows compile-time code generation via AST manipulation. Unlike Rust's proc-macros or C's preprocessor, macro execution is sandboxed by the `Ofun` capability system. A macro cannot read the filesystem, access the network, or perform any side effect unless the build manifest explicitly grants the capability to that specific dependency.

---

## 2. AST Node Types

Macros operate on typed AST nodes from `ifa-types/src/ast.rs`. The macro system introduces three primitives:

### 2.1 `ofun.da_ise(kind, fields) → Statement`

Constructs a new AST `Statement` node. The `kind` argument is a string literal matching a `Statement` variant name. The `fields` argument is a Map of field names to values.

```ifa
# Create a variable declaration AST node
ayanmo node = Ofun.da_ise("VarDecl", {
    "name": "count",
    "value": 42,
    "type_hint": "Int",
    "mutable": ooto,
});
```

**Return type:** `IfaValue::Map` containing a serialized `Statement` (JSON-compatible structure that the compiler can deserialize back into a typed AST node).

### 2.2 `ofun.yipada(ast, transform_fn) → Statement`

Walks an AST node and applies `transform_fn` to each sub-node. Returns the transformed tree.

```ifa
# Add logging to every function body
ese @add_logging(ast) -> effects(Pure) {
    da Ofun.yipada(ast, ese(node) {
        ti node.kind == "EseDef" {
            ayanmo log_stmt = Ofun.da_ise("ExprStmt", {
                "expr": { "kind": "MethodCall", "domain": "Obara", "method": "so", 
                          "args": ["Entering: " + node.name] }
            });
            node.body = [log_stmt] + node.body;
        }
        da node;
    });
}
```

**Traversal order:** Pre-order depth-first. The transform function receives each node, may modify it, and returns the replacement.

### 2.3 `ofun.iru(value) → String`

Returns the runtime type name of a value. Already implemented in the current codebase (`Ofun.iru` maps to `type_of`).

---

## 3. Macro Invocation

Macros are regular `ese` (function) definitions prefixed with `@`. The compiler recognizes `@`-prefixed function calls as macro invocations and executes them during compilation.

```ifa
# Definition
ese @derive_json(ast) -> effects(Pure) {
    # ... generate serialization methods ...
    da ast;
}

# Invocation — applied to the next declaration
@derive_json
iru Person {
    oruko: Str,
    ojo_ibi: Int,
}
```

### 3.1 Expansion Protocol

```
1. Parser encounters @macro_name before a declaration.
2. Parser parses the target declaration into an AST node.
3. Compiler serializes the AST node to an IfaValue::Map.
4. Compiler spawns a SANDBOXED compile-time VM (see §4).
5. The macro function executes with the serialized AST as its argument.
6. The return value (an IfaValue::Map) is deserialized back into an AST node.
7. The original declaration is replaced with the macro output.
8. Compilation continues with the expanded AST.
```

---

## 4. The Capability Sandbox

### 4.1 Compile-Time VM

When expanding a macro, the compiler spawns a fresh `IfaVM` instance with a restricted `CapabilitySet`:

```rust
let mut cap = CapabilitySet::new();
// Start with ZERO capabilities — no filesystem, no network, no execute.
// The macro can only perform pure computation on the AST.

// If the build manifest grants specific capabilities to this dependency:
if manifest.macro_permissions.contains(&dep, Ofun::Network) {
    cap.grant(Ofun::Network);
}

let mut vm = IfaVM::new();
vm.capabilities = cap;
```

### 4.2 Capability Violations

If a macro body calls a domain method that requires an ungrated capability, the `EffectChecker` + `CapabilitySet` will produce a **compile-time error** (not a runtime error):

```
error[OFUN_DENIED]: Macro '@fetch_schema' attempted Network access
  --> lib/schema_gen.ifa:5:12
  |
5 |     Otura.get("http://api.example.com/schema");
  |     ^^^^^^^^^^ Network capability not granted to macro
  |
  = help: Grant in oja.toml: [macro-permissions]
                              schema_gen = ["Network"]
```

### 4.3 Capability Grant Syntax (in `oja.toml`)

```toml
[dependencies]
schema_gen = "1.2.0"

[macro-permissions]
# Explicitly grant Network to schema_gen's macros
schema_gen = ["Network"]
# By default, macros have ZERO capabilities
```

---

## 5. Invariants

1. **Default deny**: A macro VM starts with an empty `CapabilitySet`. No side effects are possible unless explicitly granted in the build manifest.

2. **Macro purity by default**: Macros with `effects(Pure)` are guaranteed to be deterministic — same input AST produces same output AST. No I/O, no randomness, no time access.

3. **AST-in, AST-out**: Macros receive a serialized AST node and must return a serialized AST node. They cannot return arbitrary values, modify global state, or emit bytecode directly.

4. **No recursive macro expansion**: A macro's output is not re-scanned for further `@macro` invocations. This prevents infinite expansion loops and makes compilation time linear in the number of macro invocations.

5. **Isolation from host compiler**: The compile-time VM is a standard `IfaVM` instance. It does not have access to the compiler's internal state, symbol tables, or type information beyond what is serialized into the AST node.

---

## 6. Relationship to Other Specs

- **Depends on [effect_system.md](effect_system.md)**: Macro effect declarations use the same `Effect` enum. `effects(Pure)` on a macro means the `EffectChecker` validates that no effectful calls exist in the macro body.
- **Depends on [opon-ebo-actor-taboo-spec.md](opon-ebo-actor-taboo-spec.md)**: The compile-time VM uses the same `CapabilitySet` / `Ofun` enum defined in Section 4 of that spec.
- **Independent of memory model**: Macros execute during compilation and produce AST nodes. They are unaffected by runtime memory model choices (Arc/Rc/Slab).
