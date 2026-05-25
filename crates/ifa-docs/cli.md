# CLI reference

The `ifa` binary is defined in `crates/ifa-cli/src/main.rs`.

## Usage

```bash
ifa [command] [options] [file]
```

If the first argument does not start with `-` and is not a known subcommand, `run` is automatically inserted:

```bash
ifa hello.ifa           # Equivalent to: ifa run hello.ifa
```

## Commands

### `ifa run <file>`

Run an Ifá-Lang source file using the tree-walking interpreter.

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--allow-all` | flag | false | Grant all permissions (insecure) |
| `--allow-read` | path[] | — | Allow reading from paths |
| `--allow-write` | path[] | — | Allow writing to paths |
| `--allow-net` | string[] | — | Allow network domains |
| `--allow-env` | string[] | — | Allow environment variables |
| `--allow-time` | flag | false | Allow time functions |
| `--allow-random` | flag | true | Allow random number generation |
| `--allow-js` | flag | false | Allow JavaScript FFI |
| `--allow-python` | flag | false | Allow Python FFI |
| `--sandbox` | string | none | Sandbox mode: `wasm`, `native`, `none` |
| `args...` | trailing | — | Arguments passed to the program |

The `run` command parses the source, runs Babalawo static analysis, then executes with the interpreter.

### `ifa bytecode <file>`

Compile an `.ifa` source file to `.ifab` bytecode.

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--output`, `-o` | path | `<file>.ifab` | Output path |

Run `cargo build -p ifa-cli --release` for release builds.

### `ifa runb <file.ifab>`

Run compiled `.ifab` bytecode.

The `.ifab` file **cannot run standalone**. The command requires a matching `.ifa` file in the same directory for Babalawo verification:

```bash
ifa runb program.ifab     # Requires program.ifa in same directory
```

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--allow-all` | flag | false | Grant all permissions |
| `--allow-read` | path[] | — | Allow reading from paths |
| `--allow-write` | path[] | — | Allow writing to paths |
| `--allow-net` | string[] | — | Allow network domains |
| `--allow-env` | string[] | — | Allow environment variables |
| `--allow-time` | flag | false | Allow time functions |
| `--allow-random` | flag | true | Allow random number generation |
| `--allow-js` | flag | false | Allow JavaScript FFI |
| `--allow-python` | flag | false | Allow Python FFI |
| `args...` | trailing | — | Arguments passed to the program |

### `ifa build <file>`

Transpile an `.ifa` file to Rust, then compile to a native binary.

Requires the Rust toolchain (`rustc` and `cargo`).

| Flag | Type | Default | Description |
|------|------|---------|-------------|
| `--output`, `-o` | path | `<file_stem>` | Output path |
| `--target` | string | — | Target triple |
| `--project` | flag | false | Generate reusable Cargo project instead of building |
| `--backend` | flag | false | Build for Backend domain |
| `--frontend` | flag | false | Build for Frontend (WASM) |
| `--game` | flag | false | Build for Game domain |
| `--iot` | flag | false | Build for IoT (no_std) |
| `--crypto` | flag | false | Build for Crypto |
| `--ml` | flag | false | Build for ML/AI |
| `--fullstack` | flag | false | Build as Fullstack (backend + frontend) |

### `ifa check <file>`

Parse and run Babalawo static analysis without executing.

### `ifa fmt <file> --unstable`

Format Ifá-Lang source code.

**Requires the `--unstable` flag.** Without it, the command hard-errors:

```
`ifa fmt` is still unstable. Re-run with `--unstable`.
```

| Flag | Default | Description |
|------|---------|-------------|
| `--unstable` | false | Required acknowledgement |
| `--check` | false | Check only, do not modify |

### `ifa test [path]`

Run Ifá-Lang test files. Matches files named `*_test.ifa` or `test_*.ifa`.

| Flag | Default | Description |
|------|---------|-------------|
| `--verbose`, `-v` | false | Show detailed output |

The test runner parses and executes each matching `.ifa` file, reporting pass/fail per file.

### `ifa babalawo <path>`

Run the Babalawo static analysis linter and type checker on a file or directory.

| Flag | Default | Description |
|------|---------|-------------|
| `--strict` | false | Warnings become errors |
| `--format` | minimal | Output format: `minimal`, `compact`, `json`, `verbose` |
| `--fast` | false | Skip wisdom/proverb generation |

If `path` is a directory, all `.ifa` files in it are checked.

### `ifa repl`

Start the interactive REPL.

```
ifá> Irosu.fo("Hello");
=> Hello
```

REPL commands:

| Command | Short | Description |
|---------|-------|-------------|
| `.help` | `.h` | Show help |
| `.clear` | `.c` | Clear interpreter state |
| `.vars` | `.v` | Show variables |
| `.odu` | | List Odù domains |
| `.quit` | `.q` | Exit |

### `ifa lsp`

Start the Language Server Protocol server on stdin/stdout.

### `ifa doc <input> <output>`

Generate HTML documentation from `.ifa` source files.

| Argument | Description |
|----------|-------------|
| `input` | Input directory containing `.ifa` files |
| `--output`, `-o` | Output directory (default: `docs`) |

### `ifa oja <command>`

Package manager commands:

| Subcommand | Description |
|------------|-------------|
| `init <name>` | Initialize new project |
| `add <url>` | Add dependency |
| `remove <name>` | Remove dependency |
| `build` | Build project |
| `run [args]` | Run project |
| `test` | Run project tests |
| `install` | Install dependencies |
| `list` | List dependencies |
| `update` | Update dependencies |
| `tree` | Show dependency tree |
| `search <query>` | Search registry |
| `audit` | Audit vulnerabilities |
| `publish` | Publish to registry |
| `upgrade` | Upgrade ifa CLI |

`oja init` accepts `--domain` (basic, fullstack, game, ml, iot).

### `ifa deploy`

Zero-config deployment scanner for the current directory.

### `ifa debug --file <path>`

Start the Debug Adapter Protocol (DAP) server.

**Requires `--file`** to specify the program to debug.

### `ifa flash <file> --target <target>`

Flash to an embedded device.

| Flag | Description |
|------|-------------|
| `--target` | Target device (e.g., `esp32`, `stm32f4`) |
| `--port` | Serial port |

### `ifa sandbox <command>`

Sandbox management:

| Subcommand | Description |
|------------|-------------|
| `run <file>` | Run script in sandbox |
| `demo` | Demo sandbox features |
| `list` | List active containers |

### `ifa version`

Show version and platform information.

```bash
# Output:
╔═══════════════════════════════════════════╗
║  Ifá-Lang v1.2.0                          ║
║  The Yoruba Programming Language          ║
╠═══════════════════════════════════════════╣
║  Platform: linux / x86_64                 ║
║  16 Odù Domains Active                    ║
╚═══════════════════════════════════════════╝
```
