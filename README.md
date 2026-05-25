# Ifá-Lang: The Wisdom of the Machine

**Ifá-Lang** is a high-performance, high-assurance systems programming language that bridges ancient philosophical principles with modern heterogeneous computing. It is designed for developers who demand both the speed of raw hardware and the safety of capability-based sandboxing.

## 🏛️ The Philosophy: Wisdom & Action

Ifá-Lang architecture is divided into two inseparable halves:
- **Ikin (The Wisdom)**: The **ifa-babalawo** analysis engine. It performs static analysis, scope validation, type inference, and Ìwà (balance) verification *before* execution.
- **Iroke (The Action)**: The **ifa-vm** execution engine. A high-speed, stack-based VM that dispatches to the standard library's hardware domains (CPU/GPU/Storage/Sys).

## 🚀 Key Features

- **Unified VM Architecture**: A single, high-performance execution path for both REPL sessions and production binaries.
- **Capability-Based Security (`#opon`)**: Fine-grained hardware access control via the Òfún domain and `CapabilitySet`. The `Opon` managed heap provides RAII-based memory safety.
- **16 Odù Domains**: Standard library organized into 16 functional domains from **Ògbe** (system lifecycle) to **Òfún** (permissions), plus hardware domains (CPU/GPU/Storage/Sys).
- **Ìwà Pẹ̀lẹ́ Diagnostics**: Error messages enriched with Yoruba proverbs via the `iwa_pele` system, guiding developers toward better architectural "character".

## 📦 Workspace Structure

The project is organized into a modular Rust workspace:

- **`crates/ifa-vm`**: The high-performance "Iroke" execution engine.
- **`crates/ifa-compiler`**: The AST-to-Bytecode "Forge".
- **`crates/ifa-babalawo`**: The "Priest" - LSP, static analysis, and partial evaluator.
- **`crates/ifa-std`**: The Standard Library domains (16 Odù).
- **`crates/ifa-bytecode`**: The stable binary interface definition.
- **`crates/ifa-std` hardware modules**: CPU/GPU/Storage/Sys dispatch domains (in `crates/ifa-std/src/hardware/`).

## 📥 Quick Start

### Installation
```bash
# Install the CLI via Cargo
cargo install ifa-cli --git https://github.com/AAEO04/ifa_lang
```

### Running Your First Program
```ifa
ìbà Irosu; # Import the Console domain

Irosu.fo("Ẹ kú àbọ̀ sí Ifá-Lang!"); # Print a greeting
ase; # End of ritual
```

```bash
ifa run hello.ifa
```

## 🛠️ Modern Architecture Highlights

### The Opon (Calabash) Managed Heap
The `Opon` is the VM's memory manager with variable sizing (kekere/arinrin/nla/ailopin), supporting both embedded and desktop targets. Combined with `Ebo` epochs for scoped allocation and `defer` for resource cleanup.

### Storage via Òdí Domain
The **Òdí** domain (feature `backend`) provides file read/write operations and integrates with the Storage hardware domain for key-value persistence.

### Compute via GPU Domain
Ifá-Lang exposes GPU compute through the dedicated hardware domain (feature `gpu`), supporting buffer allocation, compute pipeline dispatch, and synchronization.

## 🛠️ Development

1. Open the repository in VS Code.
2. Run `npm install` in `vscode_extension`.
3. Press `F5` to launch a Debug Extension Host.

```bash
# Run tests
cargo test --workspace
```

## 📜 License

MIT License - Created by Charon

---

**Àṣẹ!** *(It is done!)*
