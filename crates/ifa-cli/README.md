# Ifá-Lang CLI (ifa-cli)

The `ifa` command-line interface is the primary toolchain for developing, building, and deploying Ifá-Lang programs. It provides a unified interface for the interpreter, compiler, package manager, and developer tools.

## Installation

```bash
cargo install --path crates/ifa-cli
```

## Core Commands

### 1. Running Programs

- **`ifa run <file>`**: Runs a `.ifa` source file using the tree-walking interpreter.
  - `--allow-read <path>`: Grant read access to specific directories.
  - `--allow-net <domain>`: Grant network access to specific domains.
  - `--sandbox wasm`: Run in the OmniBox WASM sandbox for maximum isolation.
- **`ifa runb <file.ifab>`**: Runs pre-compiled bytecode in the high-performance VM.
- **`ifa repl`**: Starts an interactive session for rapid prototyping.

### 2. Building & Deployment

- **`ifa build <file>`**: Compiles Ifá source to a native Rust binary.
  - `--project`: Instead of a binary, generates a complete, reusable Cargo project.
  - `--target <triple>`: Cross-compile for different architectures.
- **`ifa flash <file> --target <mcu>`**: Compiles and flashes a program directly to an embedded device (e.g., ESP32).
- **`ifa bytecode <file>`**: Compiles source to a standalone `.ifab` bytecode file.

### 3. Package Management (Ọjà)

- **`ifa oja init`**: Initializes a new Ifá project with a domain template.
- **`ifa oja add <url>`**: Adds a dependency from a Git repository or local path.
- **`ifa oja install`**: Resolves and installs all project dependencies.

### 4. Developer Tools

- **`ifa babalawo <path>`**: Runs the static analysis engine to check for errors and architectural taboos.
- **`ifa lsp`**: Starts the Language Server for IDE integration (VS Code, Vim, etc.).
- **`ifa fmt <file>`**: Formats source code according to the canonical Ifá style.
- **`ifa test`**: Runs all tests (`*_test.ifa`) in the current project.
- **`ifa doc`**: Generates HTML documentation from doc-comments.

## Security Model
The CLI enforces **Capability-Based Security**. By default, programs have NO access to the file system, network, or environment variables. You must explicitly grant these permissions using `--allow-*` flags during `ifa run`.
