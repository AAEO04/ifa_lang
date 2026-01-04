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

---

## Getting Started

### Installation

```bash
# Clone the repository
git clone https://github.com/AAEO04/ifa-lang.git
cd ifa-lang

# Install dependencies
pip install -r requirements.txt

# Verify installation
python src/cli.py --version
```

### Running Your First Program

Create a file called `hello.ifa`:

```ifa
ìbà Irosu;
Irosu.fo("Hello, World!");
àṣẹ;
```

Run it:
```bash
python src/cli.py run hello.ifa
```

---

## Hello World

Ifá-Lang supports **two syntaxes**: Yoruba and English.

### Yoruba Style
```ifa
ìbà Irosu;                          // Import the Irosu (Output) domain
Irosu.fo("Ẹ kú àbọ̀ sí Ifá-Lang!"); // Print greeting
àṣẹ;                                // End program (It is done!)
```

### English Style
```ifa
import Log;
Log.print("Welcome to Ifá-Lang!");
end;
```

Both versions produce the same output!

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
   - `ifa debug --gpc program.ifa` - GPC call stack tracing
   - `ifa check --ebo program.ifa` - Ẹbọ lifecycle validation

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

**Àṣẹ!** *(It is done!)*


