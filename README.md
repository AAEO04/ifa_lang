#  Ifá-Lang

**The Yoruba Programming Language** - A modern language rooted in the ancient wisdom of the Ifá divination system.

```
╔══════════════════════════════════════════════════════════════════════════════╗
║                              IFÁ-LANG v1.2.2                                 ║
║                       "Code with good character"                             ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

##  Features

- **256 Amúlù Instructions** - 8-bit ISA with 16 Verbs × 16 Nouns
- **16 Odù Domains** - Each principal Odù represents a computing domain
- **Balance Checker (Ìwà-Pẹ̀lẹ́)** - Semantically meaningful lifecycle management
- **Native Compilation** - Build standalone executables with `ifa build`
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
├── crates/
│   ├── ifa-core/           # VM & Runtime logic
│   ├── ifa-std/            # Standard Library (Odu domains)
│   ├── ifa-cli/            # Command Line Interface (ifa)
│   ├── ifa-embedded/       # no_std runtime for microcontrollers
│   ├── ifa-wasm/           # WebAssembly bindings
│   ├── ifa-babalawo/       # Debugger
│   ├── ifa-sandbox/        # Security & Isolation
│   ├── ifa-macros/         # Procedural macros
│   └── ifa-installer-gui/  # Graphical Installer
├── docs/                   # HTML Documentation
├── examples/               # extensive example code
└── vscode_extension/       # Editor support
```

## 📥 Download & Install

### Quick Install (Linux/macOS)
```bash
curl -sSL https://aaeo04.github.io/ifa_lang/install.sh | sh
```

### Download from Releases

[**→ GitHub Releases**](https://github.com/AAEO04/ifa_lang/releases)

| Platform | Download |
|----------|----------|
| **Windows** | `ifa-installer-v*-windows-x86_64.exe` (GUI installer) |
| Windows (manual) | `ifa-v*-windows-x86_64.exe` |
| macOS ARM | `ifa-v*-macos-arm64` |
| macOS Intel | `ifa-v*-macos-x86_64` |
| Linux x64 | `ifa-v*-linux-x86_64` |
| Linux ARM | `ifa-v*-linux-arm64` |

### 🔒 Verification

Download `checksums-v*.sha256` and verify:
```bash
sha256sum -c checksums-v*.sha256 --ignore-missing
```

### 🦀 From Source
```bash
cargo install ifa-cli --git https://github.com/AAEO04/ifa_lang
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

# Compile to native binary (Coming Soon)
# ifa build examples/hello.ifa -o myapp

# Browse standard library
ifa library

# Check code balance (Ìwà)
ifa check examples/demo.ifa

# Update Ifá-Lang
ifa oja upgrade

# Manage Packages (Ọjà)
ifa oja add https://github.com/user/repo.git
```

> **Full Documentation**: [aaeo04.github.io/ifa_lang](https://aaeo04.github.io/ifa_lang/)

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
Òkànràn.jé(x == 10, "Value mismatch"); // Assert true
Òkànràn.dogba(x, 10);                  // Assert equal
Òkànràn.yato(x, 5);                    // Assert not equal
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
