# Ifá-Lang Rust Workspace

This directory contains the Rust implementation of Ifá-Lang.

## Crates

| Crate | Description |
|-------|-------------|
| `ifa-core` | Core VM, bytecode, parser, interpreter, and IfaValue type system |
| `ifa-std` | Standard library - 16 Odù domains + stacks + infra |
| `ifa-cli` | Command-line interface and REPL |
| `ifa-embedded` | no_std runtime for embedded/IoT devices |
| `ifa-sandbox` | WASM sandbox with capability-based security |
| `ifa-macros` | Procedural macros (ẹbọ, àjọṣe, etc.) |
| `ifa-babalawo` | LSP server & developer tools |
| `ifa-wasm` | WASM bindings for browser playground |
| `ifa-installer-core` | Cross-platform installer logic |
| `ifa-installer-gui` | Graphical installer UI |

## Building

```bash
# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Build release
cargo build --workspace --release

# Run CLI
cargo run -p ifa-cli -- --help
```

## Standard Library (ifa-std)

### 16 Odù Domains

| Binary | Name | Purpose | Key Features |
|--------|------|---------|--------------|
| 1111 | Ọ̀gbè | System | CLI args, env vars |
| 0000 | Ọ̀yẹ̀kú | Exit | Process termination |
| 0110 | Ìwòrì | Time | chrono, timers |
| 1001 | Òdí | Files | rusqlite, I/O |
| 1100 | Ìrosù | Console | I/O, logging |
| 0011 | Ọ̀wọ́nrín | Random | rand_chacha |
| 1000 | Ọ̀bàrà | Math | arithmetic |
| 0001 | Ọ̀kànràn | Errors | Result types |
| 1110 | Ògúndá | Arrays | std::process |
| 0111 | Ọ̀sá | Flow | async, tokio |
| 0100 | Ìká | Strings | regex, ropey |
| 0010 | Òtúúrúpọ̀n | Reduce | sub/div/mod |
| 1011 | Òtúrá | Net | reqwest, TLS |
| 1101 | Ìrẹtẹ̀ | Crypto | ring, zstd |
| 1010 | Ọ̀ṣẹ́ | UI | ratatui |
| 0101 | Òfún | Perms | capabilities |

### Domain Stacks

| Stack | Description |
|-------|-------------|
| `stacks/crypto` | SHA, HMAC, Argon2, Base64, SecureRng |
| `stacks/backend` | HTTP, Request/Response, ORM |
| `stacks/frontend` | XSS-safe HTML, Element builder |
| `stacks/gamedev` | Vec2, AABB, ECS, Animation |
| `stacks/ml` | Tensor, activations, matmul |
| `stacks/iot` | GPIO, Serial, I2C/SPI |
| `stacks/fusion` | Fullstack IPC runtime |

### Infrastructure

| Module | Description |
|--------|-------------|
| `infra/cpu` | Parallel iterators (rayon), task graphs |
| `infra/gpu` | WGPU compute, memory pools |
| `infra/storage` | OduStore key-value database |
| `infra/kernel` | System info, memory stats |
| `infra/shaders` | Pre-built WGSL compute shaders |

## Architecture

```
crates/
├── ifa-core/          # Core runtime
│   └── src/
│       ├── lib.rs     # Module exports
│       ├── value.rs   # IfaValue enum
│       ├── bytecode.rs # OpCode definitions
│       ├── vm.rs      # Stack-based VM
│       ├── opon.rs    # Memory (calabash)
│       └── error.rs   # Error types
├── ifa-std/           # Standard library
│   └── src/
│       ├── lib.rs     # Domain exports
│       ├── stacks/    # Crypto, ML, IoT, etc.
│       └── infra/     # CPU, GPU, Storage
├── ifa-cli/           # CLI
│   └── src/main.rs    # Clap-based CLI
├── ifa-embedded/      # Embedded runtime
│   └── src/lib.rs     # no_std VM
├── ifa-sandbox/       # WASM sandbox
├── ifa-macros/        # Proc macros
├── ifa-babalawo/      # LSP server
├── ifa-wasm/          # Browser bindings
├── ifa-installer-core/# Installer logic
└── ifa-installer-gui/ # Installer UI
```
