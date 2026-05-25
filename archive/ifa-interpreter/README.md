# Ifá Interpreter (ifa-interpreter)

> [!WARNING]
> **STATUS: DEPRECATED**
> This crate is the legacy "Tree-Walking" interpreter for Ifá-Lang. It is being phased out in favor of the **Unified VM Architecture** (`ifa-compiler` + `ifa-vm`).

## Overview

The Ifá Interpreter was the original bootstrap engine for the language. It executes Ifá-Lang code by "walking" the Abstract Syntax Tree (AST) directly. While this was useful during initial development, it is significantly slower and less secure than the new bytecode-based VM.

## Why this crate still exists (for now)

- **REPL Transition**: Currently provides the backend for the legacy REPL session logic while the new `ifa-vm` hot-patching system is finalized.
- **Reference Implementation**: Serves as a baseline for correctness. If the VM and the Interpreter produce different results, it helps identify bugs in the new compiler.
- **Bootstrapping**: Used for running the very first versions of the standard library before the compiler was fully self-hosted.

## Migration Path

If you are writing new code or embedding Ifá-Lang in a Rust project, **do not use this crate.**

- Use **`ifa-compiler`** to generate `.ifab` bytecode files.
- Use **`ifa-vm`** to execute that bytecode with native performance and hardened sandboxing.

## Future Plans

Once the **Phase 2 Unification** is complete:
1.  The execution logic will be removed from this crate.
2.  Any useful AST-walking code will be absorbed into **`ifa-babalawo`** to power its "Wisdom" (Static Analysis) and constant folding features.
3.  This crate will be archived.
