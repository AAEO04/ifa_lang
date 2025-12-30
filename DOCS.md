# 📖 Ifá-Lang Documentation

**The Yoruba Programming Language** - A modern programming language rooted in the ancient wisdom of the Ifá divination system.

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Language Features](#language-features)
3. [Language Syntax](#language-syntax)
4. [The 16 Odù Domains](#the-16-odù-domains)
5. [Data Types](#data-types)
6. [Control Flow](#control-flow)
7. [Functions & Classes](#functions--classes)
8. [Standard Library](#standard-library)
9. [CLI Commands](#cli-commands)
10. [Project Architecture](#project-architecture)
11. [Building & Deployment](#building--deployment)

---

## Language Features

Ifá-Lang is not just another programming language — it's a **culturally-rooted, production-ready** tool with unique capabilities.

### 🌍 Dual Lexicon (Yoruba + English)

Write in the language you prefer — or mix both!

| Feature | Yoruba | English |
|---------|--------|---------|
| Variable | `ayanmọ x = 5;` | `let x = 5;` |
| Class | `odù Player {}` | `class Player {}` |
| Function | `ese init() {}` | `func init() {}` |
| Import | `ìbà Ìrosù;` | `import Log;` |
| If | `ti condition {}` | `if condition {}` |
| Return | `padà value;` | `return value;` |

```ifa
// Both are 100% equivalent!
ìbà Ìrosù;
ayanmọ x = 10;
Ìrosù.fọ̀(x);

// OR

import Log;
let x = 10;
Log.print(x);
```

### 🔮 Babalawo Debugger ( Errors)

Errors aren't just stack traces — they're **proverbs with wisdom**.

```
╔══════════════════════════════════════════════════════════════╗
║ 🔮 BABALAWO DIAGNOSTICS                                      ║
╠══════════════════════════════════════════════════════════════╣
║  OTURUPON-OYEKU (Division by Zero)                         ║
║                                                              ║
║  "One cannot carry a load that does not exist."            ║
║                                                              ║
║  You attempted to divide by zero. Check your denominator.  ║
╚══════════════════════════════════════════════════════════════╝
```

###  Ìwà-Pẹ̀lẹ́ Balance Checker

Every resource must be properly closed — **file handles, connections, locks**.

```bash
$ ifa check myprogram.ifa

 Balance Check Passed!
   Files: 2 opened, 2 closed
   Connections: 1 opened, 1 closed
   
🧘 Your code has good character (Ìwà-Pẹ̀lẹ́).
```

### 🏗️ Object-Oriented Programming (dá Constructor)

Create classes and instances with the `dá` (create) constructor.

```javascript
odù Player {
    ayanmọ name = "";
    ayanmọ health = 100;
    
    ese dá(n) {        // Constructor
        name = n;
    }
    
    ese attack() {
        Ìrosù.fọ̀(name + " attacks!");
    }
}

ayanmọ hero = Player.dá("Sango");
hero.attack();  // "Sango attacks!"
```

### 🎯 Match Statements (yàn)

Pattern matching inspired by Rust's `match`.

```javascript
ayanmọ status = 200;

yàn (status) {
    200 => Ìrosù.fọ̀("Success");
    404 => Ìrosù.fọ̀("Not Found");
    500 => Ìrosù.fọ̀("Server Error");
    _   => Ìrosù.fọ̀("Unknown");
}
```

### ⚡ Lambdas (Arrow Functions)

First-class functions for functional programming.

```javascript
ayanmọ double = (x) -> { padà x * 2; };

ayanmọ result = double(5);  // 10

// Pass to higher-order functions
ayanmọ mapped = Ògúndá.map([1, 2, 3], double);  // [2, 4, 6]
```

### 📦 Package Manager (Ọjà)

Install libraries directly from Git repositories.

```bash
$ ifa oja add https://github.com/user/my-library.git

📦 Adding package from https://github.com/user/my-library.git...
✅ Added dependency 'my-library'!
🔗 Linked to: https://github.com/user/my-library.git
```

### 💻 VS Code Extension (Ilé Ìwé)

Full IDE support with:
- **Syntax Highlighting** — Distinct colors for Odù, keywords, strings
- **Intellisense (LSP)** — Autocomplete for all 16 Odù domains
- **Debugging (DAP)** — Breakpoints, stepping, variable inspection
- **Error Squiggles** — Real-time linting

### 🦀 Dual Runtime (Python + Rust)

| Mode | Command | Speed | Use Case |
|------|---------|-------|----------|
| **Interpreted** | `ifa run file.ifa` | Instant | Development, scripting |
| **Compiled** | `ifa build file.ifa` | Native | Production, performance |

```bash
# Instant execution (Python)
$ ifa run myapp.ifa

# Compiled to native binary (Rust)
$ ifa build myapp.ifa -o myapp
$ ./myapp
```

---

## Quick Start

### 1. Installation

Ifá-Lang is built on Python (for the interpreter) and Rust (for the compiler).

**Prerequisites:**
- Python 3.8+
- Git
- (Optional) Rust/Cargo (for native compilation)

**Steps:**
```bash
# 1. Clone the repository
git clone https://github.com/AAEO04/ifa-lang.git
cd ifa-lang

# 2. Install Python dependencies
pip install -r requirements.txt

# 3. Add to PATH (Optional)
# Windows: set PATH=%PATH%;C:\path\to\ifa-lang\bin
# Linux/Mac: export PATH=$PATH:/path/to/ifa-lang/bin
```

### 2. How to Use

Ifá-Lang supports three modes of execution:

#### A. Interpreted Mode (Default)
Fastest for development. Runs directly on the Python VM.
```bash
# Run a file
python src/cli.py run examples/hello.ifa
```

#### B. Computed Mode (Bytecode)
Compiles an efficient `.ifab` binary format, optimized for IoT/Embedded.
```bash
# Compile to bytecode
python src/cli.py bytecode examples/hello.ifa

# Run bytecode (Fast startup)
python src/cli.py runb examples/hello.ifab
```

#### C. Native Mode (Rust)
Transpiles to Rust for maximum performance. Requires `rustc`.
```bash
# Transpile to hello.rs and compile to binary
python src/cli.py build examples/hello.ifa -o hello

# Run the native binary
./hello
```

---

## Language Syntax

### Dual-Lexicon System

Ifá-Lang supports **both Yoruba and English keywords**. Both compile to the exact same AST:

| Concept | Yoruba | English | Purpose |
|---------|--------|---------|---------|
| Import | `ìbà` | `import` | Import module |
| Variable | `ayanmo` | `let`, `var` | Declare variable |
| Class | `odù` | `domain`, `class` | Define class/module |
| Function | `ese` | `verse`, `fn`, `def` | Define function |
| If | `ti`, `bí` | `if`, `divine` | Conditional |
| Else | `bibẹkọ` | `else`, `otherwise` | Else branch |
| While | `nigba` | `while`, `cycle` | While loop |
| For | `fun...ninu` | `for...in`, `each` | For loop |
| Try | `dida_ewu` | `try`, `attempt` | Try block |
| Catch | `kaka_ewu` | `catch`, `recover` | Catch block |
| Return | `pada` | `return` | Return statement |
| End | `àṣẹ` | `end` | End program |
| Taboo | `èèwọ̀` | `taboo`, `forbid` | Architectural constraint |
| Critical | `àṣẹ_pàtàkì` | `critical`, `atomic` | Transaction block |

### The 16 Odù Standard Library (Yoruba + English)

#### Lifecycle

| # | Yoruba | English | Domain | Functions |
|---|--------|---------|--------|-----------|
| 1 | **Ogbè** | Init, Start, System | Input/Init | `bi`, `gba`, `env` |
| 2 | **Ọ̀yẹ̀kú** | Exit, End, Halt | Termination | `ku`, `sun`, `nu` |

#### I/O & Communication

| # | Yoruba | English | Domain | Functions |
|---|--------|---------|--------|-----------|
| 3 | **Òtúrá** | Net, Network, Http | Networking | `ran`, `de`, `gba` |
| 4 | **Òdí** | File, Memory, Store | Filesystem | `fi`, `gba`, `pa` |
| 5 | **Ìrosù** | Log, Print, Out | Output | `fo`, `so`, `pe` |

#### Mathematics

| # | Yoruba | English | Domain | Functions |
|---|--------|---------|--------|-----------|
| 6 | **Ọ̀bàrà** | Add, Math, Plus | Addition | `ro`, `fikun` |
| 7 | **Òtúúrúpọ̀n** | Sub, Subtract, Minus | Subtraction | `din`, `ge`, `ku` |

#### Data Structures

| # | Yoruba | English | Domain | Functions |
|---|--------|---------|--------|-----------|
| 8 | **Ògúndá** | Array, List, Vec | Arrays | `ge`, `ya`, `to`, `fi`, `mu` |
| 9 | **Ìká** | Text, String, Str | Strings | `ka`, `sopo`, `wa` |
| 10 | **Ìrẹtẹ̀** | Crypto, Hash, Zip | Compression | `fun`, `di`, `tu` |

#### Logic & Time

| # | Yoruba | English | Domain | Functions |
|---|--------|---------|--------|-----------|
| 11 | **Ìwòrì** | Time, Clock, ML | Logic/Debug | `ago`, `duro`, `royin`, `nu` |
| 12 | **Ọ̀wọ́nrín** | Rand, Random, Chaos | Randomness | `bo`, `paaro`, `da` |
| 13 | **Ọ̀sá** | Async, Proc, Thread | Concurrency | `sa`, `duro`, `ago` |

#### Safety & Authority

| # | Yoruba | English | Domain | Functions |
|---|--------|---------|--------|-----------|
| 14 | **Ọ̀kànràn** | Error, Except, Test | Errors | `binu`, `je` |
| 15 | **Ọ̀ṣẹ́** | Draw, Graphics, UI | Display | `ya`, `han`, `nu` |
| 16 | **Òfún** | Meta, Reflect, Root | Permissions | `ase`, `fun`, `ka` |

### Example: Same Logic, Two Styles

**Yoruba Style:**
```ifa
ìbà std.irosu;
ayanmo x = 10;
ti x > 5 {
    Ìrosù.fọ̀("Large");
}
àṣẹ;
```

**English Style:**
```ifa
import std.log;
let x = 10;
if x > 5 {
    Log.fo("Large");
}
end;
```

Both compile to the **exact same AST** and bytecode!

### Comments

```ifa
// Single line comment
# Also valid comment
```

### Imports

```ifa
// Yoruba style
iba std.otura;

// English style  
import std.net;
```

### Variable Declaration

```ifa
// Yoruba style
ayanmo x = 50;

// English style
let x = 50;
var name = "Adé";
```

---

## The 16 Odù Domains

Each Odù represents a domain of functionality:

| # | Odù | Binary | Domain | Key Functions |
|---|-----|--------|--------|---------------|
| 1 | **Ogbè** | `1111` | Initialization/Input | `bi`, `gba`, `oruko` |
| 2 | **Ọ̀yẹ̀kú** | `0000` | Termination/Exit | `ku`, `duro`, `gbale`, `pana` |
| 3 | **Ìwòrì** | `0110` | Time/Loops | `ago`, `duro`, `royin`, `mo` |
| 4 | **Òdí** | `1001` | Files/Memory | `fi`, `gba`, `pamo`, `ti` |
| 5 | **Ìrosù** | `1100` | Output/Print | `fo`, `so`, `pe`, `san` |
| 6 | **Ọ̀wọ́nrín** | `0011` | Random/Chance | `bo`, `paaro`, `da` |
| 7 | **Ọ̀bàrà** | `1000` | Addition/Math | `fikun`, `ro`, `so`, `kun` |
| 8 | **Ọ̀kànràn** | `0001` | Error Handling | `binu`, `je`, `gbe` |
| 9 | **Ògúndá** | `1110` | Division/Arrays | `ge`, `ya`, `to`, `mu` |
| 10 | **Ọ̀sá** | `0111` | Concurrency | `sa`, `duro`, `fo` |
| 11 | **Ìká** | `0100` | Strings | `ka`, `fun`, `tu`, `wa` |
| 12 | **Òtúúrúpọ̀n** | `0010` | Subtraction/Math | `din`, `pin`, `ge`, `kekere` |
| 13 | **Òtúrá** | `1011` | Network/Comms | `ran`, `de`, `gbo`, `so_po` |
| 14 | **Ìrẹtẹ̀** | `1101` | Crypto/Compression | `dajo`, `dan`, `fun`, `di` |
| 15 | **Ọ̀ṣẹ́** | `1010` | Graphics/Display | `ya`, `han`, `kunle`, `botini` |
| 16 | **Òfún** | `0101` | Permissions/Meta | `ase`, `fun`, `ka_iwe` |

### Usage Example

```ifa
// Initialize with Ogbè
Ogbe.bi(100);

// Math with Ọ̀bàrà and Òtúúrúpọ̀n
Obara.fikun(50);      // Add 50
Oturupon.din(25);     // Subtract 25

// Output with Ìrosù
Irosu.fo("Result calculated!");

// Network with Òtúrá
Otura.ran("Hello from Ifá!");

// End with Ọ̀yẹ̀kú
ase;
```

---

## Data Types

Ifá-Lang supports **hybrid typing** - dynamic by default, with optional static types for performance.

| Type | Yoruba | Examples |
|------|--------|----------|
| Integer | Nọ́mbà | `42`, `-17`, `0` |
| Float | Ìdá | `3.14`, `-0.5` |
| String | Ọ̀rọ̀ | `"Hello"`, `'World'` |
| Boolean | Òtítọ́/Èké | `true`, `false`, `otito`, `eke` |
| Array | Àkójọ | `[1, 2, 3]` |
| HashMap | Àwòrán | `{"key": "value"}` |
| Null | Àìsí | `null` |

### Dynamic Typing (Default)

```ifa
ayanmo x = 10;          // Int (wrapped in IfaValue)
x = "hello";            // Now String (valid!)
x = x + " world";       // String concatenation
```

### Orí System - Optional Static Types

For **native performance**, add type hints:

```ifa
ayanmo x: Int = 50;           // Native i64 - FAST!
ayanmo name: Str = "Adé";     // Native String
ayanmo pi: Float = 3.14159;   // Native f64
ayanmo active: Bool = true;   // Native bool
ayanmo items: List = [1,2,3]; // Vec<IfaValue>
ayanmo data: Map = {};        // HashMap
```

**Type Names:**

| Type | Aliases (Yoruba) |
|------|------------------|
| `Int` | `Nọmbà`, `Number` |
| `Float` | `Ìdá`, `Ida` |
| `Str` | `Ọ̀rọ̀`, `Oro`, `String` |
| `Bool` | `Òtítọ́`, `Otito` |
| `List` | `Àkójọ`, `Akojo`, `Array` |
| `Map` | `Àwòrán`, `Aworan`, `Dict` |
| `Any` | `Àìyẹ`, `Dynamic` |

**Why Type Hints?**
- Dynamic: Flexible but ~10-20x slower (runtime type checks)
- Typed: Native speed (single CPU instruction for math)

---

## Control Flow

### If/Else

```ifa
ti x > 10 {
    Irosu.fo("Large");
} bibẹkọ {
    Irosu.fo("Small");
}
```

### While Loop

```ifa
ayanmo count = 0;
nigba count < 5 {
    Irosu.fo(count);
    count = count + 1;
}
```

### For Loop

```ifa
ayanmo items = [1, 2, 3, 4, 5];
fun item ninu items {
    Irosu.fo(item);
}
```

### Try/Catch

```ifa
dida_ewu {
    Otura.ran("Risky network call");
} kaka_ewu (err) {
    Irosu.fo("Error:", err);
}
```

### Èèwọ̀ (Taboo) - Architectural Constraints

Declare forbidden patterns that the compiler will enforce:

```ifa
// The UI module cannot call Database directly
eewo: Ose(UI) -> Odi(DB);

// No network calls allowed in this file
eewo: Otura.*;

// This will cause a compile error:
// Odi.ka("users.db");  // Error! Taboo violated
```

**Use Cases:**
- Enforce layered architecture (UI → Service → DB)
- Block network calls in pure computation modules
- Prevent direct hardware access in untrusted code

### Àṣẹ (Authority) - Critical/Atomic Blocks

Code inside an `àṣẹ` block is treated as **high criticality**:

```ifa
ase_pataki {
    // Critical transaction logic
    Bank.transfer(500);
    Account.debit(500);
}
```

**Features:**
- **Atomic Execution**: Creates a transaction checkpoint
- **Auto-Rollback**: If anything fails, state is restored
- **No Interrupts**: On embedded systems, disables interrupts

---

## Functions & Classes

### Function Definition (Ese)

```ifa
ese greet(name) {
    Irosu.fo("Hello, " + name + "!");
}

greet("Adé");
```

### Class Definition (Odù)

```ifa
odu Calculator {
    ayanmo value = 0;
    
    ese add(n) {
        value = value + n;
        pada value;
    }
    
    ese subtract(n) {
        value = value - n;
        pada value;
    }
}
```

---

## Standard Library: Dual Lexicon Reference

The Standard Library is organized into 16 Domains (Odù). Each function has a **Yoruba Name** (primary) and an **English Alias/Meaning**.

### 1. Lifecycle & System

#### **Ogbè** (The Opener - System)
| Yoruba | English | Description |
|--------|---------|-------------|
| `bi(x)` | `init` | Initialize system/environment |
| `gba()` | `input` | Get input from user/env |
| `oruko()` | `user` | Get current user/identity |
| `env(k)` | `env` | Get environment variable |

#### **Ọ̀yẹ̀kú** (The Closer - Termination)
| Yoruba | English | Description |
|--------|---------|-------------|
| `ku(code)` | `exit` | Exit program with code |
| `duro()` | `halt` | Stop execution gracefully |
| `gbale()` | `gc` | Garbage collect / Clean up |
| `pana()` | `shutdown` | Shutdown system |

### 2. I/O & Communication

#### **Ìrosù** (The Voice - Output)
| Yoruba | English | Description |
|--------|---------|-------------|
| `fo(msg)` | `print` | Print with newline |
| `so(l, v)` | `log` | Log with label |
| `pe()` | `alert` | Beep/Alert sound |
| `san()` | `flush` | Flush output stream |
| `kigbe(e)` | `error` | Log error to stderr |

#### **Òdí** (The Womb - Files)
| Yoruba | English | Description |
|--------|---------|-------------|
| `ko(f, d)` | `write` | Write data to file (overwrite) |
| `fi(f, d)` | `append` | Append data to file |
| `gba(f)` | `read` | Read data from file |
| `si(path)` | `open` | Open file handle |
| `pa()` | `close` | Close active file |
| `pamo()` | `save` | Save/Commit changes |
| `ti()` | `lock` | Lock/Close access |

#### **Òtúrá** (The Messenger - Network)
| Yoruba | English | Description |
|--------|---------|-------------|
| `ran(d)` | `send` | Send data packet |
| `gba()` | `recv` | Receive data packet |
| `de(p)` | `bind` | Bind to port |
| `so_po(h,p)`| `connect`| Connect to host |
| `gbo()` | `listen` | Listen for connections |

### 3. Mathematics

#### **Ọ̀bàrà** (The Expander - Addition)
| Yoruba | English | Description |
|--------|---------|-------------|
| `ro(a, b)` | `add` | Add two numbers |
| `fikun(n)` | `incr` | Increment value |
| `so(a, b)` | `mul` | Multiply two numbers |
| `kun(lst)` | `sum` | Sum a list of numbers |

#### **Òtúúrúpọ̀n** (The Bearer - Subtraction)
| Yoruba | English | Description |
|--------|---------|-------------|
| `din(a, b)` | `sub` | Subtract two numbers |
| `pin(a, b)` | `div` | Divide two numbers |
| `ku(a, b)` | `mod` | Modulo (Remainder) |
| `ge(a, b)` | `cut/div` | Divide (Alias) |
| `kekere()` | `min` | Get minimum value |

### 4. Data Structures

#### **Ògúndá** (The Cutter - Arrays)
| Yoruba | English | Description |
|--------|---------|-------------|
| `ge(n)` | `create` | Create new array |
| `fi(l, v)` | `push` | Push to array |
| `mu(l)` | `pop` | Pop from array |
| `ya(l, i)` | `split` | Split array at index |
| `to(l)` | `sort` | Sort array |

#### **Ìká** (The Constrictor - Strings)
| Yoruba | English | Description |
|--------|---------|-------------|
| `so(str)` | `concat` | Concatenate strings |
| `ge(s, i)` | `slice` | Slice string |
| `ka(s)` | `len` | Get string length |
| `fun(t)` | `format` | Format string |
| `tu(s)` | `split` | Split/Parse string |

#### **Ìrẹtẹ̀** (The Crusher - Crypto/Compression)
| Yoruba | English | Description |
|--------|---------|-------------|
| `di()` | `hash` | Hash data (SHA256/MD5) |
| `fun()` | `compress` | Compress data (zlib) |
| `tu()` | `decompress` | Decompress data |
| `si_base64()` | `encode64` | Encode to base64 |
| `lati_base64()` | `decode64` | Decode from base64 |

### 5. Time & Randomness

#### **Ìwòrì** (The Reflector - Time)
| Yoruba | English | Description |
|--------|---------|-------------|
| `ago()` | `time` | Get current time |
| `duro(ms)` | `sleep` | Sleep (milliseconds) |
| `royin(o)` | `report` | Debug/Report object |
| `mo(d)` | `know` | Predict/Analyze (ML) |
| `wo(o)` | `look` | Introspect object |

#### **Ọ̀wọ́nrín** (The Reverser - Random)
| Yoruba | English | Description |
|--------|---------|-------------|
| `bo(m)` | `rand` | Random integer (0-m) |
| `paaro()` | `shuffle`| Shuffle list |
| `da(v)` | `flip` | Bit flip / Fuzz |

#### **Ọ̀sá** (The Runner - Concurrency)
| Yoruba | English | Description |
|--------|---------|-------------|
| `sa(fn)` | `spawn` | Spawn thread/task |
| `duro()` | `wait` | Wait for task |
| `fo(lbl)` | `jump` | Jump/Goto Label |

### 6. Safety & Meta

#### **Ọ̀kànràn** (The Stuck One - Errors)
| Yoruba | English | Description |
|--------|---------|-------------|
| `binu(e)` | `raise` | Raise error |
| `je(e)` | `catch` | Handle error |
| `gbe(fn)` | `rescue` | Wrap function safe |

#### **Ọ̀ṣẹ́** (The Beautifier - Graphics)
| Yoruba | English | Description |
|--------|---------|-------------|
| `ya(x, y)` | `draw` | Draw pixel/shape |
| `han()` | `show` | Render frame |
| `kunle()` | `render` | Render (Alias) |
| `botini()` | `button` | Draw UI button |
| `fihan()` | `display`| Show (Alias) |

#### **Òfún** (The Elder - Permissions)
| Yoruba | English | Description |
|--------|---------|-------------|
| `ase()` | `sudo` | Request root logic |
| `fun(p)` | `grant` | Grant permission |
| `ka_iwe()` | `docs` | Read manifest/docs |







---

## CLI Commands

```bash
# Run a program (interpreted)
python src/cli.py run program.ifa

# Build to Rust binary
python src/cli.py build program.ifa -o output

# Compile to bytecode (.ifab)
python src/cli.py bytecode program.ifa

# Run bytecode
python src/cli.py runb program.ifab

# Disassemble bytecode
python src/cli.py disasm program.ifab

# Start REPL
python src/cli.py repl

# Static analysis (linting)
python src/cli.py lint program.ifa

# Generate documentation
python src/cli.py doc ./src -o docs

# Show instruction matrix
python src/cli.py matrix

# Show version
python src/cli.py --version
```

## Ọjà: The Market (Package Manager)

**Ọjà** (The Market) is the decentralized package manager for Ifá-Lang. It treats code exchange as commerce between spirits.

### 1. Philosophy
- **Decentralized**: No central registry (runs on Git).
- **Verifiable**: Cryptographic `ifa.lock` ensures code integrity.
- **Manifest**: `ifa.toml` tracks what you "buy" (install).

### 2. Commands

| Command | Yoruba | Description |
|---------|--------|-------------|
| `init` | - | Initialize a new project (`ifa.toml`, `src/`) |
| `add <url>` | `ra` (Buy) | Download & add a dependency from Git |
| `install` | `ra` (Buy) | Install/Sync all dependencies from `ifa.toml` |
| `remove <name>`| `ta` (Sell) | Remove/Uninstall a dependency |
| `update` | - | Pull latest changes for packages |
| `lock` | - | detailed checksums to `ifa.lock` |
| `verify` | - | Verify package integrity (CRC/SHA256) |
| `list` | - | List installed inventory |

### 3. Usage Guide

**Initialize a Project:**
```bash
ifa oja init my-app
cd my-app
```

**Add a Library:**
You "buy" libraries directly from their Git source.
```bash
# Add from GitHub
ifa oja add https://github.com/myuser/mylib.git
```
This adds it to `ifa.toml` and downloads to `libs/mylib`.

**Install Dependencies (e.g., after cloning):**
```bash
ifa oja install
```

**Security & Locking:**
Generate a lock file to ensure everyone uses the exact same version.
```bash
# Generate/Update lock file
ifa oja lock

# Verify packages haven't been tampered with
ifa oja verify
```

### 4. Manifest (`ifa.toml`)

The `ifa.toml` file tracks your project metadata and inventory.

```toml
[package]
name = "my-app"
version = "0.1.0"
description = "My Ifá Project"

[dependencies]
# name = "git_url"
standard-lib = "https://github.com/ifa-lang/std.git"
crypto-utils = "https://github.com/ade/crypto.git"
```

### Linter (Babalawo)

```bash
# Lint a file
ifa lint program.ifa

# Lint directory
ifa lint ./src

# Error codes:
# E100 - Undefined variable
# E200 - Unknown type hint
# W100 - Unused import
# W101 - Unused variable
# S001 - Trailing whitespace
```

---

## Project Architecture

```
ifa_lang/
├── src/                    # Core Python implementation
│   ├── cli.py             # Command-line interface
│   ├── grammar.lark       # Formal EBNF grammar (dual-lexicon)
│   ├── lark_parser.py     # AST parser (Lark-based)
│   ├── interpreter.py     # Python interpreter
│   ├── transpiler.py      # Rust code generator
│   ├── bytecode.py        # .ifab bytecode compiler
│   ├── vm.py              # Virtual machine + debugger
│   ├── errors.py          # Babalawo error system
│   ├── linter.py          # Static analysis (ifa lint)
│   ├── oja.py             # Package manager + verification
│   ├── docgen.py          # Documentation generator
│   ├── isa.py             # 8-bit ISA definitions
│   ├── memory.py          # 12-bit Odù encoding, 4KB memory
│   ├── ffi.py             # Foreign function interface
│   └── __init__.py        # Package exports
│
├── lib/                    # Runtime libraries
│   ├── core.rs            # Rust runtime (IfaValue, panic handler)
│   └── std/               # Standard library (16 Odù modules)
│       ├── ogbe.py        # Initialization
│       ├── oyeku.py       # Termination
│       ├── iwori.py       # Time
│       ├── odi.py         # Files
│       ├── irosu.py       # Output
│       ├── owonrin.py     # Random
│       ├── obara.py       # Addition
│       ├── okanran.py     # Errors
│       ├── ogunda.py      # Arrays
│       ├── osa.py         # System
│       ├── ika.py         # Strings
│       ├── oturupon.py    # Subtraction
│       ├── otura.py       # Network
│       ├── irete.py       # Logic
│       ├── ose.py         # Graphics
│       └── ofun.py        # Reflection
│
├── examples/              # Example programs
│   ├── hello.ifa
│   ├── demo.ifa
│   └── math.ifa
│
├── tests/                 # Unit tests
│   └── test_balance.py
│
├── bin/                   # Executable scripts
│   └── ifa               # Main entry point
│
├── requirements.txt       # Python dependencies
├── ifa.toml              # Project configuration
└── README.md             # Project overview
```

---

## Building & Deployment

### Interpreted Mode (Python)

```bash
python src/cli.py run hello.ifa
```

### Compiled Mode (Rust)

```bash
# Generate Rust code
python src/cli.py build hello.ifa -o hello

# The generated code uses lib/core.rs runtime
# Compile with rustc:
rustc hello.rs -o hello
./hello
```

### Bytecode Mode

```bash
# Compile to bytecode
python src/cli.py bytecode hello.ifa

# Run bytecode (fast startup)
python src/cli.py runb hello.ifab
```

### File Formats

| Extension | Format | Purpose |
|-----------|--------|---------|
| `.ifa` | Source code | Human-readable Ifá source |
| `.ifab` | Bytecode | Compact binary for IoT |
| `.rs` | Rust source | Generated Rust code |

---

## Error Messages (Babalawo System)

Errors are displayed with Yoruba proverbs for wisdom:

```
══════════════════════════════════════════════════════════════
🔮 BABALAWO DIAGNOSTICS
══════════════════════════════════════════════════════════════

   ⛔ ERROR at line 5:
   
      Undefined variable 'x'
   
   💡 WISDOM: "Ẹni tó bá fẹ́ mọ ọ̀nà, kò ní sọnù"
      (One who seeks to know the path will not be lost)
   
   📖 SUGGESTION: Declare the variable with 'ayanmo x = value;'

══════════════════════════════════════════════════════════════
```

---

## The 256 Odù: Complete Instruction Matrix

The Ifá divination system recognizes **256 Odù** (combinations), derived from the 16 Principal Odù. In Ifá-Lang, this maps to our **8-bit Amúlù ISA**: `16 Nouns × 16 Verbs = 256 Instructions`.

### The 16 Principal Odù (Nouns/Domains)

| # | Yoruba | ASCII | Domain | English |
|---|--------|-------|--------|---------|
| 0 | Ogbè | ogbe | System/Init | Start |
| 1 | Ọ̀yẹ̀kú | oyeku | Exit/End | Exit |
| 2 | Ìwòrì | iwori | Time/Clock | Time |
| 3 | Òdí | odi | Storage/File | File |
| 4 | Ìrosù | irosu | I/O/Console | Log |
| 5 | Ọ̀wọ́nrín | owonrin | Random | Rand |
| 6 | Ọ̀bàrà | obara | Math/Add | Math |
| 7 | Ọ̀kànràn | okanran | Error | Error |
| 8 | Ògúndá | ogunda | Arrays | Array |
| 9 | Ọ̀sá | osa | Process | Proc |
| 10 | Ìká | ika | Text/Regex | String |
| 11 | Òtúúrúpọ̀n | oturupon | Subtract | Sub |
| 12 | Òtúrá | otura | Network | Net |
| 13 | Ìrẹtẹ̀ | irete | Logic | Bool |
| 14 | Ọ̀ṣẹ́ | ose | Graphics | Draw |
| 15 | Òfún | ofun | Meta/Reflect | Meta |

### The 16 Ẹsẹ (Verbs/Actions)

| # | Yoruba | English | Description |
|---|--------|---------|-------------|
| 0 | bí | birth | Initialize/Create |
| 1 | fí | store | Save to memory |
| 2 | wá | seek | Get/Retrieve |
| 3 | fọ̀ | speak | Print/Output |
| 4 | gbà | receive | Input/Read |
| 5 | ṣe | do | Execute/Run |
| 6 | yí | turn | Transform |
| 7 | pa | end | Terminate/Kill |
| 8 | fikun | add | Increment |
| 9 | din | subtract | Decrement |
| 10 | pọ̀ | multiply | Multiply |
| 11 | pin | divide | Divide |
| 12 | de | arrive | Connect/Bind |
| 13 | lọ | go | Jump/Branch |
| 14 | duro | wait | Sleep/Pause |
| 15 | padà | return | Return value |

### Complete 256 Instruction Matrix

Each cell shows the **opcode (hex)** for the combination `Noun.Verb`:

|  | bí (0) | fí (1) | wá (2) | fọ̀ (3) | gbà (4) | ṣe (5) | yí (6) | pa (7) | fikun (8) | din (9) | pọ̀ (A) | pin (B) | de (C) | lọ (D) | duro (E) | padà (F) |
|---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **Ogbè (0)** | 00 | 01 | 02 | 03 | 04 | 05 | 06 | 07 | 08 | 09 | 0A | 0B | 0C | 0D | 0E | 0F |
| **Ọ̀yẹ̀kú (1)** | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 | 18 | 19 | 1A | 1B | 1C | 1D | 1E | 1F |
| **Ìwòrì (2)** | 20 | 21 | 22 | 23 | 24 | 25 | 26 | 27 | 28 | 29 | 2A | 2B | 2C | 2D | 2E | 2F |
| **Òdí (3)** | 30 | 31 | 32 | 33 | 34 | 35 | 36 | 37 | 38 | 39 | 3A | 3B | 3C | 3D | 3E | 3F |
| **Ìrosù (4)** | 40 | 41 | 42 | 43 | 44 | 45 | 46 | 47 | 48 | 49 | 4A | 4B | 4C | 4D | 4E | 4F |
| **Ọ̀wọ́nrín (5)** | 50 | 51 | 52 | 53 | 54 | 55 | 56 | 57 | 58 | 59 | 5A | 5B | 5C | 5D | 5E | 5F |
| **Ọ̀bàrà (6)** | 60 | 61 | 62 | 63 | 64 | 65 | 66 | 67 | 68 | 69 | 6A | 6B | 6C | 6D | 6E | 6F |
| **Ọ̀kànràn (7)** | 70 | 71 | 72 | 73 | 74 | 75 | 76 | 77 | 78 | 79 | 7A | 7B | 7C | 7D | 7E | 7F |
| **Ògúndá (8)** | 80 | 81 | 82 | 83 | 84 | 85 | 86 | 87 | 88 | 89 | 8A | 8B | 8C | 8D | 8E | 8F |
| **Ọ̀sá (9)** | 90 | 91 | 92 | 93 | 94 | 95 | 96 | 97 | 98 | 99 | 9A | 9B | 9C | 9D | 9E | 9F |
| **Ìká (A)** | A0 | A1 | A2 | A3 | A4 | A5 | A6 | A7 | A8 | A9 | AA | AB | AC | AD | AE | AF |
| **Òtúúrúpọ̀n (B)** | B0 | B1 | B2 | B3 | B4 | B5 | B6 | B7 | B8 | B9 | BA | BB | BC | BD | BE | BF |
| **Òtúrá (C)** | C0 | C1 | C2 | C3 | C4 | C5 | C6 | C7 | C8 | C9 | CA | CB | CC | CD | CE | CF |
| **Ìrẹtẹ̀ (D)** | D0 | D1 | D2 | D3 | D4 | D5 | D6 | D7 | D8 | D9 | DA | DB | DC | DD | DE | DF |
| **Ọ̀ṣẹ́ (E)** | E0 | E1 | E2 | E3 | E4 | E5 | E6 | E7 | E8 | E9 | EA | EB | EC | ED | EE | EF |
| **Òfún (F)** | F0 | F1 | F2 | F3 | F4 | F5 | F6 | F7 | F8 | F9 | FA | FB | FC | FD | FE | FF |

### Opcode Encoding

```
Opcode = (Noun × 16) + Verb
       = (Noun << 4) | Verb

Example: Ìrosù.fọ̀ (Log.print)
  Noun = Ìrosù = 4
  Verb = fọ̀ = 3
  Opcode = (4 × 16) + 3 = 0x43
```

### The 256 Odù Pairs (Traditional Names)

| Pair | Right Odù | Left Odù | Combined Name |
|------|-----------|----------|---------------|
| 1 | Ogbè | Ogbè | **Ẹjì Ogbè** (Ogbè Méjì) |
| 2 | Ogbè | Ọ̀yẹ̀kú | **Ogbè Ọ̀yẹ̀kú** |
| 3 | Ogbè | Ìwòrì | **Ogbè Ìwòrì** |
| 4 | Ogbè | Òdí | **Ogbè Òdí** |
| ... | ... | ... | ... |
| 17 | Ọ̀yẹ̀kú | Ogbè | **Ọ̀yẹ̀kú Ogbè** |
| 18 | Ọ̀yẹ̀kú | Ọ̀yẹ̀kú | **Ẹjì Ọ̀yẹ̀kú** (Ọ̀yẹ̀kú Méjì) |
| ... | ... | ... | ... |
| 256 | Òfún | Òfún | **Ẹjì Òfún** (Òfún Méjì) |

> **Note**: When an Odù is paired with itself, it forms a "Méjì" (double), also called "Ẹjì". These are considered the most powerful configurations.

---

## The 4,096 ISA: Compound Odù Libraries

The full **12-bit instruction space** expands the 256 Compound Odù into specialized enterprise libraries.

### How It Works

```
256 Compound Odù × 16 Verbs = 4,096 Instructions
     (Parent_Child)   (Actions)
```

### 12-Bit Layout

```
┌────────────┬────────────┬────────────┐
│   Parent   │   Child    │   Verb     │
│  (4-bit)   │  (4-bit)   │  (4-bit)   │
└────────────┴────────────┴────────────┘

Example: Òtúrá_Ìká.dé_ọ̀nà (Block IP)
  Parent = Òtúrá = 0xC
  Child  = Ìká   = 0xA
  Verb   = dé    = 0xC
  Opcode = 0xCAC
```

### Implemented Compound Modules

| Compound | Opcode | Parent | Child | Function |
|----------|--------|--------|-------|----------|
| **Òtúrá_Ogbè** | 0xC0 | Network | Source | DNS / Network Init |
| **Òtúrá_Ìká** | 0xCA | Network | Control | Firewall / SSL |
| **Òdí_Ìwòrì** | 0x32 | Storage | Analysis | SQL / Queries |

### Usage

```ifa
// Import compound module
ìbà Òtúrá_Ìká;

// Block an IP (Firewall)
Òtúrá_Ìká.dé_ọ̀nà("192.168.1.50");

// Encrypt data
ayanmọ hash = Òtúrá_Ìká.pamọ("secret", "key123");
```

### Naming Convention

Every new module MUST follow the `Parent_Child` pattern:

| Pattern | Meaning | Examples |
|---------|---------|----------|
| `Òtúrá_X` | Network + X | WebSockets, VPN, TCP |
| `Òdí_X` | Storage + X | Archive, Backup, Cache |
| `Ìrosù_X` | Output + X | Logger, Formatter |
| `Ọ̀ṣẹ́_X` | Graphics + X | 3D, Animation, UI |

---

**Àṣẹ!** *(So it is!)*
