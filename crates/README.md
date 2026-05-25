# Ifá-Lang Workspace (crates/)

This workspace contains the Rust implementation of the Ifá-Lang ecosystem. It is designed around a **Decoupled Architecture** to ensure high-performance, modularity, and cross-platform safety.

## 🏛️ Core Architecture

The system is divided into three distinct execution layers:

| Layer | Concept | Crates | Role |
| :--- | :--- | :--- | :--- |
| **Analysis** | **Ikin** (Wisdom) | `ifa-babalawo`, `ifa-parser` | Static analysis, constant evaluation, and safety validation. |
| **Translation** | **Forge** | `ifa-compiler`, `ifa-bytecode`, `ifa-transpiler` | Transforming AST into optimized execution formats. |
| **Execution** | **Iroke** (Action) | `ifa-vm`, `ifa-std`, `ifa-embedded` | Striking the hardware and enforcing sandboxed capabilities. |

## 📦 Crate Registry

### Core Execution
- **`ifa-vm`**: High-performance stack-based bytecode engine. The "Iroke" wand.
- **`ifa-compiler`**: AST-to-Bytecode compiler with support for incremental REPL injection.
- **`ifa-bytecode`**: The binary interface definition. 100% stable, `no_std` compatible.
- **`ifa-types`**: Shared type system (IfaValue) and Odù domain definitions.

### Frontend & Analysis
- **`ifa-parser`**: High-speed parser generating the canonical Abstract Syntax Tree.
- **`ifa-babalawo`**: LSP engine and Partial Evaluator. Handles "Constant Divination."
- **`ifa-fmt`**: Source code formatter and stylistic enforcer.

### Platforms & Tools
- **`ifa-std`**: The standard library (16 Odù domains). Transitioning low-level logic to `ifa-infra`.
- **`ifa-cli`**: The unified command-line tool (`ifa run`, `ifa check`, `ifa repl`).
- **`ifa-sandbox`**: WASM-based execution environment for browser/edge deployment.
- **`ifa-embedded`**: `no_std` runtime for IoT and microcontrollers.

## 🛠️ Building & Testing

```bash
# Build the entire workspace
cargo build --workspace

# Run the full test suite (including drift tests)
cargo test --workspace

# Run the CLI locally
cargo run -p ifa-cli -- --help
```

## 📜 Standard Library (16 Odù)

The `ifa-std` crate implements the high-level API for the hardware domains:

| Domain | Name | Purpose | Hardware Bridge |
| :--- | :--- | :--- | :--- |
| **0111** | Ọ̀sá | Flow | Async/Parallelism |
| **1001** | Òdí | Seal | Storage/Persistence |
| **1010** | Ọ̀ṣẹ́ | Painter | Graphics/UI |
| **1011** | Òtúrá | Messenger | Networking |
| **1111** | Ọ̀gbè | Source | System/Kernel |

## 🛡️ Security Model (#opon)

Ifá-Lang utilizes **Capability-Based Security**. Access to hardware (via `ifa-std` or `ifa-infra`) is gated by the **#opon** (Calabash) configuration.
1.  **Validation**: `ifa-babalawo` checks resource requirements at build-time.
2.  **Enforcement**: `ifa-vm` verifies `ResourceTokens` in the hot-path before every hardware strike.

## 🚀 Performance Strategy

1.  **Zero-Copy UMA**: Optimized for Unified Memory architectures (Apple Silicon/APUs).
2.  **Constant Folding**: Babalawo evaluates constant expressions during the "Wisdom" phase.
3.  **Slab Allocation**: Using `ifa-infra` slab pools to avoid the system `malloc` in the VM.
```
