# Ifá-Lang Rust Workspace

This directory contains the Rust implementation of Ifá-Lang.

## Crates

| Crate | Description |
|-------|-------------|
| `ifa-core` | Core VM, bytecode, and IfaValue type system |
| `ifa-std` | Standard library - 16 Odù domains |
| `ifa-cli` | Command-line interface |
| `ifa-embedded` | no_std runtime for embedded devices |

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

## 16 Odù Domains

| Binary | Name | Purpose | Key Crates |
|--------|------|---------|------------|
| 1111 | Ọ̀gbè | System/Lifecycle | std::env |
| 0000 | Ọ̀yẹ̀kú | Exit/Sleep | RAII guards |
| 0110 | Ìwòrì | Time/Iteration | chrono |
| 1001 | Òdí | Files/Database | rusqlite |
| 1100 | Ìrosù | Console I/O | crossterm |
| 0011 | Ọ̀wọ́nrín | Random | rand_chacha |
| 1000 | Ọ̀bàrà | Math (Add/Mul) | std::ops |
| 0001 | Ọ̀kànràn | Errors | thiserror |
| 1110 | Ògúndá | Arrays/Processes | std::process |
| 0111 | Ọ̀sá | Concurrency | tokio |
| 0100 | Ìká | Strings | ropey, regex |
| 0010 | Òtúúrúpọ̀n | Math (Sub/Div) | checked ops |
| 1011 | Òtúrá | Networking | reqwest, rustls |
| 1101 | Ìrẹtẹ̀ | Crypto/Compress | ring, zstd |
| 1010 | Ọ̀ṣẹ́ | Graphics/UI | ratatui |
| 0101 | Òfún | Permissions | capability-based |

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
│       ├── traits.rs  # OduDomain trait
│       └── *.rs       # 16 domain modules
├── ifa-cli/           # CLI
│   └── src/
│       └── main.rs    # Clap-based CLI
└── ifa-embedded/      # Embedded runtime
    └── src/
        └── lib.rs     # no_std scaffold
```
