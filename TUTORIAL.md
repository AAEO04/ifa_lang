# 🎓 Ifá-Lang Tutorial

A step-by-step guide for beginners to learn Ifá-Lang, the Yoruba programming language.

---

## Table of Contents

1. [Getting Started](#getting-started)
2. [Hello World](#hello-world)
3. [Variables](#variables)
4. [Math Operations](#math-operations)
5. [Strings](#strings)
6. [Control Flow](#control-flow)
7. [Functions](#functions)
8. [Classes](#classes)
9. [File I/O](#file-io)
10. [Network](#network)
11. [Sandbox](#igbale-sandbox-secure-execution)
12. [FFI](#ffi-foreign-function-interface)
13. [Ose (Graphics)](#ose-graphics-domain)
14. [Ọpẹlẹ Divination](#ọpẹlẹ-divination)
15. [Ewo (Assertions)](#ewo-assertions)
16. [Visibility](#visibility-gbangba--ikoko)

---

## Getting Started

### Installation

**Option 1: Download Binary (Recommended)**
```bash
# Download from GitHub Releases
# Windows: ifa-lang-windows.zip
# Linux: ifa-lang-linux.tar.gz
# macOS: ifa-lang-macos.tar.gz

# Extract and add to PATH
# Verify installation
ifa --version
```

**Option 2: Build from Source**
```bash
# Clone the repository
git clone https://github.com/AAEO04/ifa-lang.git
cd ifa-lang

# Build with Cargo
cargo build --release

# The binary is at: target/release/ifa
```

### Running Your First Program

Create a file called `hello.ifa`:

```ifa
Irosu.fo("Hello, World!");
àṣẹ;
```

Run it:
```bash
ifa run hello.ifa
```

---

## Hello World

Ifá-Lang supports **two syntaxes**: Yoruba and English.

### Yoruba Style
```ifa
Irosu.fo("Ẹ kú àbọ̀ sí Ifá-Lang!"); // Print greeting
àṣẹ;                                // End program (It is done!)
```

### English Style
```ifa
Log.print("Welcome to Ifá-Lang!");
end;
```

Both versions produce the same output!

> **Note:** The `àṣẹ` / `end` statement is **optional**. Programs will execute correctly without it. It's a stylistic choice that emphasizes completion in the Yoruba tradition ("It is done!").

### About Imports (`ìbà` / `import`)

The **16 Odù domains** (Ogbe, Oyeku, Iwori, Odi, Irosu, Owonrin, Obara, Okanran, Ogunda, Osa, Ika, Oturupon, Otura, Irete, Ose, Ofun) are **built-in** and **always available** without imports:

```ifa
// ✅ No import needed - domains are built-in
Irosu.fo("Hello!");
Obara.fikun(5, 3);  // Math operations
Ose.ko(10, 5, "Canvas text");
```

**Imports are only needed for:**
- User-defined modules (your own `.ifa` files)
- External libraries
- Compound Odù (advanced combinations like `Òtúrá_Ìká`)

```ifa
// Import your own module
ìbà my_utils;

// Import compound Odù
ìbà Òtúrá_Ìká;
```

**Style Note:** Some examples include `ìbà` statements for built-in domains purely for **documentation/clarity** - they make the code self-documenting but aren't technically required.

---

## Variables

Declare variables with `ayanmọ` (Yoruba) or `let` (English):

```ifa
// Yoruba style
ayanmọ name = "Adé";
ayanmọ age = 25;
ayanmọ is_student = otito;  // true

// English style
let city = "Lagos";
let population = 21000000;
let growing = true;
```

### Data Types

| Type | Yoruba | Examples |
|------|--------|----------|
| Number | Nọ́mbà | `42`, `3.14`, `-17` |
| String | Ọ̀rọ̀ | `"Ifá"`, `'Lang'` |
| Boolean | Òtítọ́ | `otito`, `iro` (true/false) |
| Array | Àkójọ | `[1, 2, 3]` |
| Map | Àwòrán | `{"key": "value"}` |

---

## Math Operations

The **Ọ̀bàrà** and **Òtúúrúpọ̀n** domains handle math:

```ifa
ìbà Obara;     // Math+ (addition, multiplication)
ìbà Oturupon;  // Math- (subtraction, division)
ìbà Irosu;

// Addition
ayanmọ sum = Obara.fikun(10, 5);     // 15
Irosu.fo("Sum: " + sum);

// Subtraction
ayanmọ diff = Oturupon.din(10, 3);   // 7
Irosu.fo("Difference: " + diff);

// Multiplication
ayanmọ product = Obara.isodipupo(6, 7);  // 42
Irosu.fo("Product: " + product);

// Division
ayanmọ quotient = Oturupon.pin(20, 4);   // 5.0
Irosu.fo("Quotient: " + quotient);

// Power
ayanmọ power = Obara.agbara(2, 8);   // 256
Irosu.fo("2^8: " + power);

àṣẹ;
```

---

## Strings

The **Ìká** domain handles string operations:

```ifa
ìbà Ika;
ìbà Irosu;

ayanmọ greeting = "Hello";
ayanmọ name = "Ifá";

// Concatenate
ayanmọ message = Ika.so(greeting, " ", name, "!");
Irosu.fo(message);  // "Hello Ifá!"

// Length
ayanmọ len = Ika.gigun(message);
Irosu.fo("Length: " + len);  // 11

// Uppercase / Lowercase
Irosu.fo(Ika.nla("hello"));     // "HELLO"
Irosu.fo(Ika.kekere("WORLD"));  // "world"

// Find substring
ayanmọ pos = Ika.wa(message, "Ifá");
Irosu.fo("Found at: " + pos);  // 6

// Split
ayanmọ parts = Ika.pin("a,b,c", ",");
// parts = ["a", "b", "c"]

àṣẹ;
```

---

## Control Flow

### If/Else

```ifa
ayanmọ age = 18;

ti age >= 18 {
    Irosu.fo("You are an adult");
} bibẹkọ {
    Irosu.fo("You are a minor");
}
```

### While Loop

```ifa
ayanmọ count = 0;

nigba count < 5 {
    Irosu.fo("Count: " + count);
    count = count + 1;
}
```

### For Loop

```ifa
ayanmọ fruits = ["apple", "banana", "orange"];

fun fruit ninu fruits {
    Irosu.fo("I like " + fruit);
}
```

### Match Statement

```ifa
ayanmọ status = 200;

yàn (status) {
    200 => Irosu.fo("Success");
    404 => Irosu.fo("Not Found");
    500 => Irosu.fo("Server Error");
    _   => Irosu.fo("Unknown");
}
```

---

## Functions

Define functions with `ese` (Yoruba) or `fn` (English):

```ifa
ìbà Irosu;

// Define a function
ese greet(name) {
    Irosu.fo("Hello, " + name + "!");
}

// Call the function
greet("Adé");   // "Hello, Adé!"
greet("Tayo");  // "Hello, Tayo!"

// Function with return value
ese add(a, b) {
    padà a + b;   // return
}

ayanmọ result = add(5, 3);
Irosu.fo(result);  // 8

àṣẹ;
```

---

## Classes

Define classes with `odù` (Yoruba) or `class` (English):

```ifa
ìbà Irosu;

odù Calculator {
    ayanmọ value = 0;
    
    // Constructor
    ese dá(initial) {
        value = initial;
    }
    
    ese add(n) {
        value = value + n;
        padà value;
    }
    
    ese subtract(n) {
        value = value - n;
        padà value;
    }
    
    ese get_value() {
        padà value;
    }
}

// Create instance
ayanmọ calc = Calculator.dá(10);
Irosu.fo(calc.add(5));       // 15
Irosu.fo(calc.subtract(3));  // 12
Irosu.fo(calc.get_value());  // 12

àṣẹ;
```

---

## File I/O

The **Òdí** domain handles file operations:

```ifa
ìbà Odi;
ìbà Irosu;

ayanmọ filename = "data.txt";

// Write to file
Odi.ko(filename, "Hello from Ifá-Lang!\nLine 2\nLine 3");
Irosu.fo("File written");

// Check if file exists
ti Odi.wa(filename) {
    // Read file
    ayanmọ content = Odi.ka(filename);
    Irosu.fo("File contents:");
    Irosu.fo(content);
}

// Append to file
Odi.fi(filename, "\nAppended line");

// Delete file
// Odi.pa_faili(filename);

àṣẹ;
```

---

## Network

The **Òtúrá** domain handles networking:

### UDP Multicast (Ether)
```ifa
ìbà Otura;
ìbà Irosu;

// Join channel 1
Otura.ether_de(1);
Irosu.fo("Joined Ether channel 1");

// Broadcast a message
Otura.ether_ran("Hello from Ifá-Lang!");

// Receive messages (with timeout)
ayanmọ msg = Otura.ether_gba(10);
Irosu.fo("Received: " + msg);

// Leave channel
Otura.ether_pa();
àṣẹ;
```

---

## The 16 Odù Domains

| Binary | Odù | Purpose |
|--------|-----|---------|
| 1111 | Ogbe | System, CLI Args |
| 0000 | Oyeku | Exit, Sleep |
| 0110 | Iwori | Time, Loops |
| 1001 | Odi | Files, Storage |
| 1100 | Irosu | Console I/O |
| 0011 | Owonrin | Random |
| 1000 | Obara | Math (Add/Mul) |
| 0001 | Okanran | Error Handling |
| 1110 | Ogunda | Arrays |
| 0111 | Osa | Concurrency |
| 0100 | Ika | Strings |
| 0010 | Oturupon | Math (Sub/Div) |
| 1011 | Otura | Network |
| 1101 | Irete | Crypto |
| 1010 | Ose | Graphics |
| 0101 | Ofun | Permissions |

---

## IDE Support (LSP)

Ifá-Lang includes a full Language Server Protocol (LSP) implementation for IDE integration.

### VS Code Setup

1. Install the Ifá-Lang extension (if available) or configure manually:

```json
// .vscode/settings.json
{
    "ifa.lsp.path": "python",
    "ifa.lsp.args": ["-m", "src.lsp"]
}
```

2. Or run the LSP server manually:
```bash
python -m src.lsp
```

### Features

| Feature | Description |
|---------|-------------|
| **Autocompletion** | Context-aware suggestions for variables, functions, Odù domains |
| **Hover** | Documentation for keywords, Odù methods, and your own symbols |
| **Go to Definition** | Jump to variable/function definitions |
| **Document Symbols** | Outline view of all symbols in file |
| **Signature Help** | Parameter hints when calling functions |
| **Diagnostics** | Real-time error detection |

---

## Benchmarking

Compare Ifá-Lang performance across different execution modes.

### Running Benchmarks

```bash
# Run all benchmarks
python benchmarks/benchmark.py

# Run specific tests
python benchmarks/benchmark.py --fib      # Fibonacci
python benchmarks/benchmark.py --primes   # Prime sieve
python benchmarks/benchmark.py --strings  # String ops
python benchmarks/benchmark.py --arrays   # Array ops

# More iterations for accuracy
python benchmarks/benchmark.py -n 10
```

### Execution Modes

| Mode | Command | Speed |
|------|---------|-------|
| **Interpreter** | `ifa run file.ifa` | Baseline |
| **Bytecode** | `ifa runb file.ifab` | ~2-5x faster |
| **Native (Rust)** | `./compiled_binary` | ~10-50x faster |

### Compiling for Speed

```bash
# Compile to bytecode
ifa bytecode program.ifa -o program.ifab
ifa runb program.ifab

# Compile to native binary (requires Rust)
ifa build program.ifa -o program
./program
```

---

## Advanced Examples

Check out the `examples/05_advanced/` folder for complex, real-world examples:

### Blockchain (`blockchain.ifa`)
- Custom classes for Block and Blockchain
- SHA256 hashing with proof-of-work
- Chain validation

### API Client (`api_client.ifa`)
- HTTP GET/POST requests
- JSON parsing and serialization
- REST API integration

### Database (`database.ifa`)
- File-based JSON database
- CRUD operations (Create, Read, Update, Delete)
- Query by field

### Chat Server (`chat_server.ifa`)
- TCP socket server
- Multi-client support
- Room-based messaging

---

## Next Steps

1. **Explore Examples**: Check `examples/` folder
   - `01_basics/` - Hello world, variables, math
   - `02_features/` - OOP, lambdas, pattern matching
   - `03_compounds/` - Custom Odù definitions
   - `04_apps/` - Web server, file processor
   - `05_advanced/` - Blockchain, API client, database

2. **Read DOCS.md**: Full API reference for all 16 Odù domains

3. **Performance Optimization**:
   - `ifa bytecode program.ifa` - Compile to bytecode
   - `ifa build program.ifa` - Compile to native

4. **IDE Integration**: Set up LSP for autocompletion

5. **Package Management**: 
   - `ifa oja init` - Create new project
   - `ifa oja add <url>` - Add dependencies

6. **Debugging**:
   - `ifa check program.ifa` - Syntax and type checking
   - `ifa check --ebo program.ifa` - Ẹbọ lifecycle validation
   - Use `--verbose` flag for detailed error messages

7. **Sandbox Execution**:
   - `ifa sandbox run script.ifa` - Run in isolated container
   - `ifa sandbox demo` - Demo sandbox features
   - See [sandbox.html](docs/sandbox.html) for full documentation

---

## Ìgbálẹ̀ Sandbox (Secure Execution)

Run untrusted code safely using the Ìgbálẹ̀ sandbox:

```bash
# Run script in sandbox with 30s timeout
ifa sandbox run script.ifa --timeout 30

# Demo sandbox features
ifa sandbox demo

# List active containers
ifa sandbox list
```

### Security Features

| Feature | Description |
|---------|-------------|
| **Filesystem** | Isolated virtual filesystem |
| **Network** | Blocked by default |
| **CPU** | Time limits with watchdog |
| **Memory** | 100MB default limit |
| **Processes** | Max 10 child processes |

### When to Use

- Running untrusted user code
- Online playgrounds
- Testing potentially dangerous scripts
- Multi-tenant code execution

---

## Opon Memory Configuration

The **Opon** (Sacred Calabash) is the memory container for the Ifá-Lang VM. Size is configurable for different deployment scenarios.

### Size Presets

| Size | Yoruba | English Aliases | Slots | Memory |
|------|--------|-----------------|-------|--------|
| Kekere | kẹ́kẹ́rẹ́ | `small`, `tiny`, `embedded`, `micro` | 256 | ~4KB |
| Arinrin | àrínrin | `medium`, `standard`, `default`, `normal` | 4,096 | ~64KB |
| Nla | nlá | `large`, `big`, `mega`, `xl` | 65,536 | ~1MB |
| Ailopin | àìlópin | `unlimited`, `dynamic`, `infinite`, `max` | Dynamic | Grows |

### Setting Opon Size

```ifa
// Via directive (at start of file)
#opon large

// Or with English alias
#opon mega
```

### Rust API

```rust
use ifa_core::opon::{Opon, OponSize};

// Parse from string (supports both Yoruba and English)
let size = OponSize::from_str("large").unwrap();  // → Nla
let size = OponSize::from_str("kekere").unwrap(); // → Kekere

// Create Opon with specific size
let opon = Opon::new(OponSize::Nla);

// Embedded (4KB) preset
let embedded_opon = Opon::embedded();

// Default (64KB) preset  
let default_opon = Opon::create_default();
```

## FFI (Foreign Function Interface)

Ifá-Lang can call external C/Rust libraries and expose its functions to other languages.

### Calling External Libraries

```rust
use ifa_std::ffi::{IfaFfi, FfiValue};

// Load a shared library
let mut ffi = IfaFfi::new();
ffi.load("mymath", None)?;  // mymath.dll / libmymath.so

// Bind a function with types
ffi.bind("mymath", "add", &["i32", "i32"], "i32")?;

// Call it
let result = ffi.call("mymath", "add", &[
    FfiValue::I32(5),
    FfiValue::I32(3)
])?;
```

### Exposing Ifá Functions via API

```rust
use ifa_std::ffi::{IfaApi, IfaType, FfiValue, FfiResult};

let mut api = IfaApi::new();

// Expose a function
api.expose("math.add", &[IfaType::I32, IfaType::I32], IfaType::I32, |args| {
    let a = args[0].as_i32().unwrap_or(0);
    let b = args[1].as_i32().unwrap_or(0);
    Ok(FfiValue::I32(a + b))
});

// Call it
let result = api.call("math.add", &[FfiValue::I32(10), FfiValue::I32(20)])?;

// Generate C header for external use
let c_header = api.generate_c_header();
// Output: int32_t ifa_math_add(int32_t arg0, int32_t arg1);
```

### JSON-RPC Server

Expose Ifá functions over HTTP:

```rust
use ifa_std::ffi::{IfaRpcServer, create_stdlib_api};

// Create server with stdlib API
let api = create_stdlib_api();
let server = IfaRpcServer::new(api, 8080);

// Handle JSON-RPC request
let request = r#"{"jsonrpc":"2.0","method":"obara.fikun","params":[5,3],"id":1}"#;
let response = server.handle_request(request);
// Returns: {"jsonrpc":"2.0","result":8,"id":1}
```

### Stdlib API Endpoints

| Endpoint | Description |
|----------|-------------|
| `obara.fikun(a, b)` | Add two numbers |
| `obara.isodipupo(a, b)` | Multiply two numbers |
| `oturupon.din(a, b)` | Subtract |
| `oturupon.pin(a, b)` | Divide |
| `ika.gigun(s)` | String length |
| `iwori.epoch()` | Current Unix timestamp |
| `owonrin.afesona(min, max)` | Random number |
| `ogbe.version()` | Ifá version string |

### Secure FFI (Sandboxed)

For untrusted code, use `SecureFfi` with whitelisting:

```rust
use ifa_std::ffi::SecureFfi;

let mut ffi = SecureFfi::new();

// Must whitelist libraries first
ffi.allow("mylib");
ffi.load("mylib", None)?;

// Dangerous symbols are blocked automatically
ffi.bind("libc", "system", &["str"], "i32");  // ERROR: blocked!
```

### Supported Languages

| Language | Direction | Method |
|----------|-----------|--------|
| C | Bidirectional | Native FFI + header generation |
| C++ | Bidirectional | `extern "C"` |
| Rust | Bidirectional | Native crate |
| Python | Call from Ifá | Via ctypes |
| JavaScript | Call Ifá | Via JSON-RPC |
| Go/Zig | Call Ifá | Via C header |

### Coop Domain in Ifá-Lang

Use the `Coop` (Àjọṣe) domain directly in your Ifá-Lang code:

```ifa
// Call Python function
ayanmo result = Coop.py("math", "sqrt", 16);
Irosu.fo(result);  // 4.0

// Execute JavaScript via Node.js
Coop.js("console.log(JSON.stringify({sum: 1+2}))");

// Run shell command
ayanmo output = Coop.sh("echo Hello from shell");
Irosu.fo(output);

// Compile and run C code (requires gcc/clang)
Coop.c("#include <stdio.h>\nint main() { printf(\"Hi from C\"); return 0; }");

// Generic FFI call (load library, call function)
Coop.ffi("mylib", "add", 5, 3);

ase;
```

| Method | Language | Requirements |
|--------|----------|--------------|
| `Coop.py(module, func, args...)` | Python | Python 3 installed |
| `Coop.js(code)` | JavaScript | Node.js installed |
| `Coop.c(code)` | C | gcc or clang installed |
| `Coop.sh(cmd)` | Shell | Bash/cmd.exe |
| `Coop.ffi(lib, func, args...)` | C/Rust | Shared library (.dll/.so) |

---

## Ose (Graphics Domain)

Ose is the graphics domain for visual output. Currently it supports **ASCII canvas** rendering.

### ASCII Canvas Mode

```ifa
// Initialize canvas
Ose.nu();             // Yoruba: Clear canvas
Ose.iwon(40, 20);     // Yoruba: Set size (40x20)

// Or in English
Ose.clear();
Ose.size(40, 20);

// Drawing commands
Ose.ila(5, 5, 30, 5, "#");    // Line from (5,5) to (30,5)
Ose.iyika(20, 10, 8, "*");    // Circle at (20,10) radius 8
Ose.ko(10, 15, "Hello!");     // Text at (10,15)

// Render to output
Ose.han();  // Yoruba: Display
// Or: Ose.render();

ase;
```

### Available Methods

| Yoruba | English | Description |
|--------|---------|-------------|
| `Ose.nu()` | `Ose.clear()` | Clear canvas |
| `Ose.iwon(w, h)` | `Ose.size(w, h)` | Set canvas dimensions |
| `Ose.ya(x, y, c)` | `Ose.plot(x, y, c)` | Plot single character |
| `Ose.ila(x1,y1,x2,y2,c)` | `Ose.line(...)` | Draw line |
| `Ose.iyika(x,y,r,c)` | `Ose.circle(...)` | Draw circle |
| `Ose.ko(x, y, text)` | `Ose.text(...)` | Draw text |
| `Ose.han()` | `Ose.render()` | Output to console |

### CSS Support (Future Enhancement)

Currently Ose renders **ASCII art** only. CSS support would require modifications:

**Why CSS isn't currently supported:**
- Ose is designed for terminal/console output
- ASCII art works in CLI, SSH, and minimal environments
- No dependency on browsers or GUI frameworks

**How CSS support could be added:**

1. **HTML Mode** for WASM playground:
   ```ifa
   Ose.mode("html");  // Switch to HTML output
   Ose.style("background:#1a1a2e; padding:20px;");
   Ose.rect(0, 0, 200, 100, "border:2px solid gold;");
   Ose.text(50, 50, "Styled!", "color:#e94560; font-size:24px;");
   Ose.render();  // Outputs <div> elements with CSS
   ```

2. **SVG Mode** for vector graphics:
   ```ifa
   Ose.mode("svg");
   Ose.circle(100, 100, 50, "fill:purple;");
   Ose.save("output.svg");
   ```

3. **Canvas API** for browser (WASM only):
   ```ifa
   Ose.mode("canvas");  // Uses HTML5 Canvas API
   Ose.fillStyle("#e94560");
   Ose.fillRect(10, 10, 100, 50);
   ```

**Current workaround:** For styled output in the playground, combine Ose ASCII with custom CSS in your HTML wrapper.

---

## Ọpẹlẹ Divination

The Ọpẹlẹ (divination chain) module provides two features: a verifiable chain structure and traditional Odù divination.

### What is Ọpẹlẹ?

In Ifá tradition, the Ọpẹlẹ is a chain of 8 cowries or seeds. When cast, each half-chain (4 elements) falls open or closed, creating one of 16 base patterns. Two casts combine into one of 256 possible Odù, each with associated verses and guidance.

### OpeleChain - Verifiable Log

Create tamper-evident, append-only logs where each entry is cryptographically linked to the previous:

```rust
use ifa_std::opele::{OpeleChain};

// Create a new chain
let mut chain = OpeleChain::new();

// Cast (append) entries - returns &mut Self for chaining
chain.cast("User logged in")
     .cast("Permission granted")
     .cast("Action completed");

// Verify integrity
if chain.verify() {
    println!("Chain is intact - {} entries", chain.len());
} else {
    println!("Chain has been tampered with!");
}

// Access entries
for entry in chain.entries() {
    println!("{}: {}", entry.index, entry.data);
}
```

### Odù Divination

Cast the Ọpẹlẹ to receive one of 256 Odù patterns:

```rust
use ifa_std::opele::{cast, divine, Odu, PrincipalOdu};

// Cast a random Odù
let odu = cast();
println!("Odù: {}", odu.name());  // e.g., "Ogbe Meji" or "Iwori Oyeku"

// Check if it's a principal (double) Odù
if odu.is_principal() {
    println!("This is a Meji - double power!");
}

// Full divination with interpretation
let result = divine("Should I proceed with this project?");
println!("{}", result);
// Output:
// Question: Should I proceed with this project?
// Odù: Obara Osa
// Proverb: Obara says: What you give, returns to you multiplied.
// Guidance: The combination of Obara and Osa suggests balance...
```

### The 16 Principal Odù

| Binary | Name | Meaning |
|--------|------|---------|
| 1111 | Ogbe | Clear path, move forward |
| 0000 | Oyeku | Wisdom in darkness |
| 0110 | Iwori | Look within first |
| 1001 | Odi | Doors opening/closing |
| 1100 | Irosu | Speak truth |
| 0011 | Owonrin | Embrace change |
| 1000 | Obara | Generosity returns |
| 0001 | Okanran | Words have power |
| 1110 | Ogunda | Clear path with patience |
| 0111 | Osa | Release what doesn't serve |
| 0100 | Ika | Choose words carefully |
| 0010 | Oturupon | Seek balance |
| 1011 | Otura | Journey teaches more |
| 1101 | Irete | Secrets bring freedom |
| 1010 | Ose | Beauty from peace |
| 0101 | Ofun | Honor ancestors |

### Use Cases

- **Audit Logs**: Track system events with verification
- **Blockchain-lite**: Simple tamper-evident records
- **Random Selection**: Fair, culturally-meaningful randomization
- **Educational**: Learn traditional Ifá concepts

### Opele in Ifá-Lang

Use the `Opele` or `Oracle` domain directly in your Ifá-Lang code:

```ifa
// Cast a simple 2-level compound Odù (256 patterns)
ayanmo odu = Opele.cast();
Irosu.fo(odu);  // e.g., "Ogbe_Iwori"

// Cast compound Odù with custom depth
ayanmo gpc = Opele.cast_compound(3);  // 3-level GPC (4,096 patterns)
Irosu.fo(gpc);  // e.g., "Obara_Osa_Iwori"

// Get lineage description
Irosu.fo(Opele.lineage(gpc));
// Output: "Obara → Osa → Iwori (depth 3)"

// Get depth
Irosu.fo(Opele.depth(gpc));  // 3

// Divine with guidance
Irosu.fo(Opele.divine("Should I pursue this path?"));
// Output:
// Question: Should I pursue this path?
// Guidance: What you give, returns to you multiplied.

ase;
```

**English alias:**
```ifa
ayanmo odu = Oracle.cast_compound(4);
Irosu.fo(Oracle.divine("What wisdom awaits?"));
ase;
```

---

## Ewo (Assertions)

Ewo (ẹ̀wọ̀) means "taboo" in Yoruba. In Ifá-Lang, the `ewo` statement enforces runtime constraints - values that must remain true.

### Syntax

```ifa
// Yoruba
ewo condition;
ewo condition, "optional message";

// English
assert condition;
```

### Examples

```ifa
// Basic assertion
ayanmo balance = 100;
ewo balance > 0;  // Passes

// With message
ayanmo amount = -5;
ewo amount >= 0, "Amount must be non-negative";
// Runtime error: [ẹ̀wọ̀] Taboo violated: Amount must be non-negative

// In validation
ese validate_age(age) {
    ewo age >= 0;
    ewo age <= 150;
    Irosu.fo("Age is valid: " + age);
}
```

### Use Cases

- **Input validation**: Ensure values meet requirements
- **Invariants**: Assert conditions that must always hold
- **Debugging**: Catch unexpected states early
- **Documentation**: Make assumptions explicit

---

## Visibility (gbangba / ikoko)

Ifá-Lang follows Rust's visibility model: **private by default**.

### Keywords

| Yoruba | English | Meaning |
|--------|---------|---------|
| *(default)* | *(default)* | Private - accessible within same module/class |
| `gbangba` | `pub` | Public - accessible from anywhere |
| `ikoko` / `àdáni` | `private` | Explicit private (optional) |
| `gbangba(ile)` | `pub(crate)` | Package-internal (planned) |

### Example

```ifa
odù User {
    // Private field (default)
    ayanmọ _password = "";
    
    // Public method
    gbangba ẹsẹ create(name) {
        Irosu.fo("Creating user: " + name);
    }
    
    // Private method
    ẹsẹ hash_password(pw) {
        padà Irete.sha256(pw);
    }
    
    // Public getter for controlled access
    gbangba ẹsẹ get_name() {
        padà _name;
    }
}

// Public class
gbangba odù PublicAPI { }
```

### Language Comparison

| Aspect | Ifá-Lang | Rust | Zig | C | Go |
|--------|----------|------|-----|---|-----|
| **Default** | Private | Private | Private (file) | Public | Private (lowercase) |
| **Public keyword** | `gbangba` / `pub` | `pub` | `pub` | - | Uppercase name |
| **Private keyword** | `ikoko` (optional) | - | - | - | lowercase name |
| **Field privacy** | ✓ Yes | ✓ Yes | ✗ No | Convention | ✓ Yes |
| **Module privacy** | ✓ Yes | ✓ Yes | ✓ File-based | Header files | ✓ Package |
| **Crate internal** | `gbangba(ile)` | `pub(crate)` | - | - | `internal` package |
| **Getters pattern** | Methods | Methods | Methods | Functions | Methods |
| **Escape hatch** | `àìléwu` block | `unsafe` + raw ptr | `@ptrCast` | Direct access | - |

### Accessing Private Fields

Use **getter methods** for controlled access:

```ifa
odù Config {
    ayanmọ _secret_key = "abc123";
    
    // Public read-only access
    gbangba ẹsẹ get_secret() {
        padà _secret_key;
    }
    
    // No setter = truly private
}

ayanmọ c = Config.new();
Irosu.fo(c.get_secret());  // ✓ Works via getter
Irosu.fo(c._secret_key);   // ✗ ERROR: Private field
```

---

## Ẹbọ - Resource Lifecycle (RAII)

**Ẹbọ** (Sacrifice) is Ifá-Lang's RAII system for automatic resource cleanup. Like Go's `defer` or Rust's `Drop`.

### Basic Usage

```ifa
// Yoruba syntax
ẹbọ {
    ayanmọ file = Odi.si("data.txt");
    // File automatically closed when block ends
}

// English alias
sacrifice {
    let conn = Database.connect();
    // Connection cleaned up on exit
}
```

### Rust API

```rust
use ifa_core::ebo::{Ebo, EboScope};

// Simple cleanup guard
let _guard = Ebo::new("tempfile", || {
    std::fs::remove_file("temp.txt").ok();
});
// Cleanup runs when _guard drops

// Scoped resource with value access
let scoped = EboScope::new(
    File::create("data.txt")?,
    |f| { f.sync_all().ok(); }  // Cleanup function
);
scoped.write_all(b"data")?;
// sync_all() called automatically on drop
```

### Methods

| Method | Description |
|--------|-------------|
| `Ebo::new(name, fn)` | Create guard with named cleanup |
| `.dismiss()` | Cancel cleanup (won't run) |
| `.sacrifice()` | Run cleanup early |

---

## Àjọṣe - Reactive Bindings

**Àjọṣe** (Relationship) provides signal-based reactivity for automatic UI/data synchronization.

### Signals

```rust
use ifa_core::ajose::{Signal, effect};

// Create reactive signal
let count = Signal::new(0);
let label = Signal::new(String::new());

// Subscribe to changes
count.subscribe(|val| {
    println!("Count changed to: {}", val);
});

// Create effect (derived computation)
effect(move || {
    label.set(format!("Count: {}", count.get()));
});

count.set(5);  // label automatically updates to "Count: 5"
```

### Computed Values

```rust
use ifa_core::ajose::Computed;

let first = Signal::new("John");
let last = Signal::new("Doe");

// Computed value that auto-updates
let full_name = Computed::new(move || {
    format!("{} {}", first.get(), last.get())
});

println!("{}", full_name.get());  // "John Doe"
first.set("Jane");
println!("{}", full_name.get());  // "Jane Doe" - automatic!
```

### UI Bindings (ajose! macro)

```rust
use ifa_macros::ajose;

// Bind counter value to label text
ajose!(counter.value => label.text);

// When counter.value changes, label.text updates automatically
```

---

## Èèwọ̀ - Taboo Declarations

**Èèwọ̀** (Taboo) declares forbidden interactions between Odù domains, enforcing architectural boundaries.

### Directive Syntax

```ifa
// Forbid UI (Ose) from directly accessing Files (Odi)
#ewọ Ose → Odi

// Forbid all network access
#ewọ Otura.*

// English alias
#taboo UI → Database
```

### Use Cases

| Pattern | Directive | Reason |
|---------|-----------|--------|
| MVC separation | `#ewọ Ose → Odi` | UI shouldn't touch files directly |
| Security | `#ewọ Otura.*` | Block all network in sandbox |
| Dependency direction | `#ewọ Ogbe → Irosu` | Core shouldn't know about logging |

### Enforcement

When a taboo is violated:
```
ERROR [Èèwọ̀]: Forbidden call from Ose to Odi
  Declared at line 2: #ewọ Ose → Odi
  Violation at line 45: Odi.ka("file.txt")
```

### Legacy Python API

```python
from src.directives import DirectiveParser

parser = DirectiveParser()
directives = parser.parse(source_code)

for ewo in directives.ewos:
    print(f"Taboo: {ewo.source_domain} → {ewo.target_domain}")
```

---

## Ìwà Pẹ̀lẹ́ - Graceful Error Handling

**Ìwà Pẹ̀lẹ́** (Gentle Character) is Ifá-Lang's philosophy for graceful error handling with Yoruba proverbs.

### Error Messages with Proverbs

When an error occurs, Iwa Pele provides culturally-meaningful guidance:

```
[Iwa Pele] Missing: Configuration file not found
  Proverb: Ohun tí a wá kò sí, ṣùgbọ́n ohun mìíràn wà.
           (What we seek is not here, but something else is.)
  Hint: Try using default configuration.
```

### Error Kinds

| Kind | English | Proverb |
|------|---------|---------|
| `Missing` | Not found | "What we seek is not here, but something else is." |
| `Timeout` | Timed out | "Patience is the father of character." |
| `PermissionDenied` | No access | "The commander's authority doesn't exceed God's." |
| `OperationFailed` | Failed | "If we fall, we rise again." |
| `Imbalanced` | Unbalanced | "What is opened must be closed." |

### Rust API

```rust
use ifa_core::iwa_pele::{IwaPele, IwaPeleResult, with_patience};

// Graceful fallback
let config = load_config().or_gentle(default_config);

// Recovery function
let db = connect_db().or_recover(|| {
    println!("Using in-memory database");
    Database::memory()
});

// Retry with patience (3 attempts)
let result: IwaPeleResult<Data> = with_patience(3, || {
    fetch_from_api()
});
```

### Compile-Time Balance Checking

The `#[iwa_pele]` macro checks that resources are properly balanced:

```rust
use ifa_macros::iwa_pele;

#[iwa_pele]
fn process_file() -> Result<()> {
    let file = open("data.txt")?;  // open
    process(&file);
    close(file)?;                   // close - balanced!
    Ok(())
}
// Compile-time: "[Iwa Pele] Balanced function: process_file"
```

---

**Àṣẹ!** *(It is done!)*


