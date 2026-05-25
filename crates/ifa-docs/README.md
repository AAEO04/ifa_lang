# Ifá-Lang

**Ifá-Lang** is a programming language with a dual Yoruba/English lexicon, 16 domain-standard-library, and two execution paths: a tree-walking interpreter and a stack-based bytecode VM. Built in Rust.

## Philosophy

The architecture divides into two inseparable halves:

- **Ikin (The Wisdom)** — the `ifa-babalawo` analysis engine: compile-time checking, constant evaluation, and safety verification *before* execution.
- **Iroke (The Action)** — the execution engine that runs your code via either interpreted tree-walking or compiled bytecode.

## Quick start

```bash
# Install via Cargo
cargo install ifa-cli

# Run a program
ifa run hello.ifa

# Bare filename is implicitly 'run'
ifa hello.ifa
```

A minimal program:

```ifa
Irosu.fo("Hello, Ifá-Lang!");
ase;
```

## Documentation

| Document | Covers |
|----------|--------|
| [Language guide](guide.md) | Syntax, types, variables, functions, control flow, operators, imports, classes |
| [Standard library](std-library.md) | Complete API reference for all 16 Odù domains |
| [CLI reference](cli.md) | All `ifa` commands, flags, and usage |
| [Advanced topics](advanced.md) | Execution model, bytecode, capabilities, async, FFI, memory |

## Key features

- **Bilingual syntax** — All keywords exist in Yoruba and English; mix freely in the same file.
- **16 Odù domains** — Standard library organized by the 16 principal Odù of Ifá divination.
- **Two execution paths** — Tree-walking interpreter for fast iteration, bytecode VM for performance.
- **Capability-based security** — Fine-grained permission tokens gate all I/O, network, and system access.
- **Native compilation** — Transpile `.ifa` to Rust, then compile to a standalone binary.
- **Dual-lexicon APIs** — Each domain method has both a Yoruba name and an English alias.
