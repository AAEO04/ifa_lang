#  Ifá-Lang

**The Yoruba Programming Language** - A modern language rooted in the ancient wisdom of the Ifá divination system.

```
╔══════════════════════════════════════════════════════════════════════════════╗
║                              IFÁ-LANG v1.0                                   ║
║                       "Code with good character"                             ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

##  Features

- **256 Amúlù Instructions** - 8-bit ISA with 16 Verbs × 16 Nouns
- **16 Odù Domains** - Each principal Odù represents a computing domain
- **Balance Checker (Ìwà-Pẹ̀lẹ́)** - Semantically meaningful lifecycle management
- **Dual Runtime** - Interpreted (Python) or transpiled to native (Rust)
- **User Libraries (Ọjà)** - Create and publish libraries using Git
- **VS Code Extension** - Full Intellisense and Debugging support

## 📚 Creating Libraries (Ọjà)
You can share your Ifá code with the world!

### 1. Structure
Create a library file (e.g., `lib.ifa`) and use the `gbangba` keyword to invoke public export.

```javascript
// lib.ifa
gbangba odù Calculator {
    gbangba ese add(a, b) {
        padà a + b;
    }
}
```

### 2. Publishing
Push your code to a Git repository (e.g., GitHub).

### 3. Installing
Other users can install your library using:
```bash
ifa oja add https://github.com/username/my-lib.git
```

This will automatically link your library for use in their projects!
```javascript
ìbà my_lib; // Use the repo name
```

##  Project Structure

```
ifa-lang/
├── bin/
│   └── ifa                 # CLI entry point
├── src/                    # Compiler Core (10 modules)
│   ├── __init__.py         # Package with lazy imports
│   ├── cli.py              # CLI (run, build, debug, check, matrix, library, repl)
│   ├── lexer.py            # Tokenizer with Yoruba Unicode
│   ├── parser.py           # Ese Parser (high-level syntax)
│   ├── validator.py        # Ìwà Engine (balance checker)
│   ├── transpiler.py       # Rust code generator
│   ├── vm.py               # OponVM + Babalawo debugger
│   ├── ffi.py              # Foreign Function Interface
│   ├── isa.py              # Amúlù 8-bit ISA (256 instructions)
│   └── memory.py           # 12-bit Calabash (4KB memory)
├── lib/
│   ├── std/                # Standard Library (18 modules)
│   │   ├── __init__.py     # StandardLibrary registry
│   │   ├── base.py         # OduModule base class
│   │   ├── ogbe.py         # System (1111)
│   │   ├── oyeku.py        # Process (0000)
│   │   ├── iwori.py        # Time (0110)
│   │   ├── odi.py          # File I/O (1001)
│   │   ├── irosu.py        # Console (1100)
│   │   ├── owonrin.py      # Random (0011)
│   │   ├── obara.py        # Math+ (1000)
│   │   ├── okanran.py      # Errors (0001)
│   │   ├── ogunda.py       # Arrays (1110)
│   │   ├── osa.py          # Concurrency (0111) - Async & JSON/CSV
│   │   ├── ika.py          # Strings (0100)
│   │   ├── oturupon.py     # Math- (0010)
│   │   ├── otura.py        # Network (1011) - Real UDP Ether
│   │   ├── irete.py        # Crypto (1101) - Hash & Compression
│   │   ├── ose.py          # Graphics (1010)
│   │   └── ofun.py         # Permissions (0101) - Meta & Config
│   └── core.rs             # Rust runtime
├── examples/
│   ├── hello.ifa           # Hello World
│   ├── demo.ifa            # Math & Network demo
│   └── math.ifa            # Arithmetic operations
├── tests/
│   └── test_balance.py     # Ìwà Engine tests
├── ifa.toml                # Project configuration
└── README.md
```

## 📥 Download & Install

### Windows (Recommended)
Download the installer - **no Python required!**

[![Download for Windows](https://img.shields.io/badge/Download-Windows%20Installer-blue?style=for-the-badge&logo=windows)](https://github.com/AAEO04/ifa-lang/releases/latest)

1. Download `ifa-lang-*-windows-setup.exe`
2. Run the installer
3. Restart your terminal
4. Ready! Run `ifa --help`

### macOS (Homebrew)
```bash
brew tap AAEO04/ifa-lang
brew install ifa-lang
```

### Linux
```bash
# Download and extract
wget https://github.com/AAEO04/ifa-lang/releases/latest/download/ifa-lang-1.0.0-linux.tar.gz
tar -xzf ifa-lang-*.tar.gz
cd ifa-lang-*

# Install
./install.sh
```

### VS Code Extension
Search **"Ifá-Lang"** in VS Code Extensions marketplace.

---

## 🚀 Quick Start

```bash
# Run a program
ifa run examples/hello.ifa

# Interactive REPL
ifa repl

# Compile to bytecode
ifa bytecode examples/hello.ifa

# Compile to native binary (requires Rust)
ifa build examples/hello.ifa -o myapp

# Browse standard library
ifa library

# Check code balance (Ìwà)
ifa check examples/demo.ifa

# Manage Packages (Ọjà)
ifa oja add https://github.com/user/repo.git
```

> **Full Documentation**: See [`DOCS.md`](DOCS.md) or visit [aaeo04.github.io/ifa-lang](https://aaeo04.github.io/ifa-lang/)

## Language Syntax

### Hello World
```ifa
ìbà Irosu;
Irosu.fo("Ẹ kú àbọ̀ sí Ifá-Lang!");
ase;
```

### Math Operations
```ifa
ayanmo x = 50;
Obara.fikun(10);      # Add 10 → 60
Oturupon.din(5);      # Subtract 5 → 55
Oturupon.ku(3);       # Modulo 3 → 1
Irosu.fo(x);          # Print result
ase;
```

### Network (Ether)
```ifa
# Real UDP multicast between Opon instances
Otura.ether_de(1);    # Join channel 1
Otura.ether_ran(42);  # Broadcast value
Otura.ether_gba();    # Receive from network
```

##  The 16 Odù Domains

| Binary | Odù | Verb | Noun | Domain |
|--------|-----|------|------|--------|
| 1111 | Ogbè | INIT | GLOBAL | System Init |
| 0000 | Ọ̀yẹ̀kú | HALT | VOID | Termination |
| 0110 | Ìwòrì | LOOP | STACK | Iteration |
| 1001 | Òdí | SAVE | DISK | File I/O |
| 1100 | Ìrosù | EMIT | CONSOLE | Output |
| 0011 | Ọ̀wọ́nrín | SWAP | POINTER | Random |
| 1000 | Ọ̀bàrà | ADD | ACCUM | Math+ |
| 0001 | Ọ̀kànràn | THROW | ERRLOG | Errors |
| 1110 | Ògúndá | ALLOC | HEAP | Arrays |
| 0111 | Ọ̀sá | JUMP | FLAG | Concurrency |
| 0100 | Ìká | PACK | ARRAY | Strings |
| 0010 | Òtúúrúpọ̀n | SUB | CONST | Math- |
| 1011 | Òtúrá | SEND | SOCKET | Network |
| 1101 | Ìrẹtẹ̀ | FREE | GARBAGE | Crypto |
| 1010 | Ọ̀ṣẹ́ | DRAW | SCREEN | Graphics |
| 0101 | Òfún | NEW | OBJECT | Permissions |

##  Ìwà-Pẹ̀lẹ́ (Good Character)

The Ìwà Engine enforces balance - every action must have a reaction:

| Open Action | Required Close |
|-------------|----------------|
| `Odi.si()` | `Odi.pa()` |
| `Otura.ether_de()` | `Otura.ether_pa()` |
| `Ogunda.ge()` | `Irete.tu()` |

Unbalanced code will not compile.

##  Architecture

- **8-bit Amúlù ISA**: 256 instructions (16 verbs × 16 nouns)
- **12-bit Memory**: 4,096 addressable locations (4KB Calabash)
- **Memory Regions**: BOOT, STACK, HEAP, STATIC, IO, NETWORK, GRAPHICS, etc.
- **Registers**: OKE (IP), ISALE (Accumulator), OTUN (X), OSI (Y)

##  Testing (Ìdánwò)
Run unit tests with the `test` command. It discovers all `*_test.ifa` files.

```bash
ifa test
# or specific file
ifa test examples/math_test.ifa
```

Assertions use the **Ọ̀kànràn** (Error) domain:
```ifa
Òkànràn.jé(x == 10, "Value mismatch error");
```

##  VS Code Extension (Ilé Ìwé)
This repository includes a standalone VS Code extension in `vscode_extension/`.

### Features
- **Syntax Highlighting**: Colors for Odù, Keywords, Strings.
- **Intellisense**: Autocomplete for standard library functions.
- **Diagnostics**: Real-time error checking (linting).
- **Snippets**: Quick expansions for common patterns.

### Development
1. Open the repository in VS Code.
2. Run `npm install` in `vscode_extension`.
3. Press `F5` to launch a Debug Extension Host.

```bash
python -m pytest tests/ -v
```

##  License

MIT License - Created by Charon

---

**Àṣẹ!** *(It is done!)*
