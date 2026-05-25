# Advanced topics

## Execution model

Ifá-Lang has two parallel execution paths:

```text
Source (.ifa)
    │
    ▼
Parser (logos + pest) ──────────────────► Tree-walking Interpreter
    │                                         (ifa-interpreter)
    ▼
Compiler (AST → bytecode) ──► .ifab ──► Stack-based VM
                                             (ifa-vm)
```

- **Tree-walking interpreter**: Used by `ifa run`, `ifa repl`, `ifa test`. Walks the AST directly. Simpler debugging and faster startup.
- **Bytecode VM**: Used by `ifa runb`, `ifa build`. A stack-based virtual machine with opcode dispatch, optimized for execution speed.

Both paths share the same parser, AST types, standard library, and capability model.

### Bytecode format

Compiled `.ifab` files contain:

- **Header**: Magic bytes, version, sizes
- **Instruction stream**: Opcode bytes
- **Constant pool**: Interned strings and literal values
- **Export table**: Public symbol names
- **Line mappings**: Source line → bytecode offset

### Native transpilation

`ifa build` converts `.ifa` → Rust source → native binary:

```text
AST → Rust transpiler → Cargo project → native executable
```

The transpiler maps Ifá-Lang constructs to Rust equivalents, tracking which Rust dependencies are needed (tokio, reqwest, rand). Use `--project` to generate a reusable Rust project instead of immediately building.

## Capability-based security

All I/O operations are gated by capability tokens (`Ofun`). No domain can perform privileged operations without an explicit grant.

### Capability types

| Capability | Description |
|------------|-------------|
| `ReadFiles { root }` | Read files under a path |
| `WriteFiles { root }` | Write files under a path |
| `Network { domains }` | Network access to specific domains |
| `Execute { programs }` | Execute subprocesses |
| `Environment { keys }` | Read environment variables |
| `Time` | Access high-resolution time |
| `Random` | Generate random numbers |
| `Stdio` | Read/write stdin, stdout, stderr |
| `Bridge { language }` | FFI bridge (js, python) |

### Running with capabilities

```bash
# Allow reading /tmp and writing /tmp/output
ifa run script.ifa --allow-read /tmp --allow-write /tmp/output

# Allow network access to example.com
ifa run script.ifa --allow-net example.com

# Allow everything (insecure, development only)
ifa run script.ifa --allow-all
```

By default, `Stdio` and `Random` are granted. The script's own directory is granted read access for imports.

### Capability enforcement

Each domain handler checks the `CapabilitySet` before performing operations:

- **Irosu** (console): checks `Stdio` before print/input
- **Ogbe** (system): checks `Environment` before env var access
- **Odi** (files): checks `ReadFiles`/`WriteFiles` before file operations
- **Otura** (network): checks `Network` with SSRF protection (blocks localhost, private IPs, metadata endpoints)
- **Owonrin** (random): checks `Random`

## Memory management

### The Opon (Calabash)

The VM manages memory through the Opon system with configurable sizes:

| Size | Stack slots | Call frames | Use case |
|------|-------------|-------------|----------|
| `kekere` / `small` | 256 | 64 | Embedded |
| `arinrin` / `default` | 4,096 | 512 | General |
| `nla` / `large` | 65,536 | 4,096 | Computation-heavy |
| `ailopin` / `unlimited` | Dynamic | Dynamic | Interactive/REPL |

Set in source: `opon: nla;`

### IfaValue type system

The runtime uses two value types:

- **IfaValue** (`Arc<Mutex>`, `Arc<str>`): Thread-local, used for VM inner loop and interpreter operations. Uses `Arc` for heap allocation (strings, objects, closures).
- **IfaShared** (`Arc<RwLock>` / `DashMap`): Thread-safe, Send+Sync, for inter-thread communication and the global registry.

| Variant | Rust type | Description |
|---------|-----------|-------------|
| `Int` | `i64` | Signed integer |
| `Float` | `f64` | Double-precision float |
| `Str` | `Arc<str>` | Interned string |
| `Bool` | `bool` | Boolean |
| `List` | `Vec<IfaValue>` | Dynamic array |
| `Map` | `HashMap<Arc<str>, IfaValue>` | String-keyed map |
| `Object` | `Arc<Mutex<HashMap<...>>>` | Heap object with mutable state |
| `Fn` | `Arc<dyn Fn(...)>` | Function value |
| `Null` | unit | Null/nil |

## Async model

The language supports async/await via the Osa domain (requires `backend` feature):

```ifa
daro ese fetch_data(url) {
    reti Osa.ise("task");
    pada result;
}
```

- `daro` / `async` marks a function as asynchronous
- `reti` / `await` suspends execution until a future completes
- The VM maintains a `task_queue` of pending async tasks
- The interpreter polls async tasks between statements

## Error handling

### Try / Catch / Finally

```ifa
gbiyanju {
    ayanmo result = risky_operation();
    Irosu.fo("Success: " + result);
} gba (e) {
    Irosu.fo("Caught: " + e);
} nipari {
    Irosu.fo("Cleanup always runs");
}
```

### Throw

```ifa
Okanran.ta("Something went wrong");  // Throw recoverable error
```

### Assert

```ifa
ewo count > 0, "Count must be positive";
Okanran.beeni(count > 0, "Count must be positive");
```

### The Shield of Okanran

At runtime, errors trigger the recovery system:

1. If inside a `try` block with a `catch`, execution jumps to the catch handler
2. If inside a `try` with no catch but a `finally`, the finally block runs before the error propagates
3. If no handler exists, the program crashes with an error message

## Ikin (Interned constant pool)

The VM interns all string constants from the bytecode into an `Ikin` pool for O(1) lookup and deduplication. Maximum 65,536 interned strings.

## Static analysis (Babalawo)

The Babalawo engine (run by `ifa check`, `ifa babalawo`, or automatically before `ifa run`) performs:

- **Type checking**: Variable type hint validation
- **Iwa engine**: Resource lifecycle tracking (every open must have a close)
- **Taboo enforcer**: Architectural constraint validation
- **Move tracker**: Linear type discipline for actor-safe types
- **Capability inference**: Derives required capabilities from code
- **Wisdom generation**: Maps errors to cultural proverbs

```bash
ifa babalawo script.ifa --strict --format verbose
```

## FFI (Polyglot bridge)

The FFI system (gated by `native_ffi`) provides:

- **JavaScript**: Execute JS via Boa engine (`--allow-js`)
- **Python**: Execute Python via PyO3 (`--allow-python`)
- **Native C**: Load `.so`/`.dll`/`.dylib` via libloading with verification
- **RPC**: Export Ifá-Lang functions as JSON-RPC 2.0 HTTP endpoints

All FFI operations require explicit capability grants and are subject to `SecureFfi` symbol allowlisting.
