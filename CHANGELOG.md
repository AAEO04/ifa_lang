# Changelog

All notable changes to Ifá-Lang will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

#### Opele/Oracle Domain Integration
- **Opele interpreter handler**: Full integration with ifa-core interpreter
  - `Opele.cast()` / `Oracle.cast()` - Cast simple 2-level Odù
  - `Opele.cast_compound(n)` - Cast n-level compound (2-5 levels)
  - `Opele.lineage(compound)` - Get genealogical description
  - `Opele.depth(compound)` - Get compound depth
  - `Opele.divine(question)` - Divination with proverb guidance
- **Grammar updates**: Added Opele, Oracle, Coop to `odu_name` rule
- **Parser support**: Added pseudo-domain parsing for Opele/Oracle

#### Ewo Assertion System
- **New `ewo` statement**: Runtime assertions for value constraints
  - Yoruba: `ewo balance > 0;`
  - English: `assert amount >= 0;`
- **Lexer tokens**: `ewo`, `ẹ̀wọ̀`, `assert`
- **Grammar rule**: `ewo_stmt` for assertion parsing
- **Interpreter handler**: Validates boolean expressions, throws on failure

#### CLI Improvements
- Removed emojis and decorative box art for cleaner output
- Updated version display to v1.2.0
- Fixed unused import warnings

#### Playground Updates
- Added 6 comprehensive examples:
  - Hello World, Math Demo, Opele/Oracle, Charonboat (Ose canvas), Ewo/Assert, All Domains
- All examples available in both Yoruba and English syntax
- New example buttons in UI

#### Security & Sandbox Enhancements
- **Wasmtime v40 Upgrade**: Major security update fixing all known vulnerabilities
  - RUSTSEC-2025-0118: Shared linear memory issue (Low)
  - RUSTSEC-2025-0046: fd_renumber panic (Low)
  - RUSTSEC-2024-0438: Windows device filename sandboxing
  - RUSTSEC-2024-0445: cap-primitives sandboxing (Low)
- **WasiView Implementation**: Proper WASI Preview 1 integration with `wasmtime_wasi::p1`
- **Store Limits**: Added memory, table, and instance limits to sandbox
- **Epoch Interruption**: Added timeout enforcement via epoch deadlines
- **Migration Guide**: Added `MIGRATION.md` documenting wasmtime v17→v40 changes
- **CapabilitySet (Ọ̀fún)**: Unified permission system for all sandbox backends
  - `Ofun` enum: ReadFiles, WriteFiles, Network, Environment, Time, Random, Stdio, Execute
  - `CapabilitySet.grant()` / `.check()` methods for permission management
  - Interpreter now validates capabilities before privileged operations
- **CLI Permission Flags**:
  - `--allow-read=PATH` - Grant file read access
  - `--allow-write=PATH` - Grant file write access
  - `--allow-net=DOMAIN` - Grant network access
  - `--allow-env=KEY` - Grant environment variable access
  - `--allow-time` - Grant time/date access
  - `--allow-random` - Grant RNG access (default: on)
  - `--allow-all` - Grant all permissions (use with caution)
- **OmniBox WASM Sandbox**: Fully refactored for wasmtime v40 API
- **Igbale Integration**: Native sandbox now uses `CapabilitySet` for consistent permissions

### Fixed
- **Removed non-existent `Ose.iwon()` method** from playground and documentation
- **Fixed rustdoc link errors** in bytecode.rs and ast.rs (escaped square brackets)
- **Fixed Clippy warnings**: Default impl for Opon, mem::take in interpreter, thread_local const
- **Fixed sandbox test**: `allow_network` → `allowed_network_domains.is_empty()`
- **Removed broken domain tests**: Tests were outdated after CapabilitySet refactoring

#### Opon Memory Configuration
- **English aliases for OponSize**: small/tiny → Kekere, medium/standard → Arinrin, large/mega → Nla, unlimited/dynamic → Ailopin
- **New methods**: `OponSize::from_str()`, `display_name()`, `approx_memory()`

#### Advanced Features Documentation
- **Ẹbọ (RAII)**: Documented sacrifice-based resource cleanup with `dismiss()` and `sacrifice()` methods
- **Àjọṣe (Reactive)**: Documented Signal-based reactivity and `ajose!` macro
- **Èèwọ̀ (Taboo)**: Documented `#ewọ` directive for forbidden domain interactions
- **Ìwà Pẹ̀lẹ́ (Graceful Errors)**: Documented proverb-based error handling

### Documentation
- New `docs/advanced.html` - Complete advanced features reference
- Updated `docs/sandbox.html` - Full sandbox architecture with security analysis
- Updated `docs/ofun.html` - Capability types and CLI usage
- Added Security & Advanced Features section to `docs/index.html`
- TUTORIAL.md: Added Opon sizing, Ẹbọ, Àjọṣe, Èèwọ̀, Ìwà Pẹ̀lẹ́ sections
- Note that `àṣẹ`/`end` is optional

## [1.2.0] - 2026-01-07

### Added

#### Ọpẹlẹ Module - Divination Chain
- **OpeleChain**: Tamper-evident, append-only log structure
  - `.cast(data)` - Add entries to the chain
  - `.verify()` - Verify chain integrity
  - Each entry hashed with previous, creating unbroken sequence
- **256 Odù Patterns**: Complete traditional divination system
  - 16 Principal Odù (Meji) + 240 compound combinations
  - `cast()` - Random Odù generation with proper 2-leg structure
  - `divine(question)` - Full divination with proverbs and guidance
- **PrincipalOdu enum**: All 16 base patterns with names and binary codes

#### FFI (Foreign Function Interface)
- **IfaFfi**: Load and call C/Rust shared libraries
- **SecureFfi**: Sandboxed FFI with library whitelisting
- **IfaApi**: Expose Ifá functions to external code
- **IfaRpcServer**: JSON-RPC over HTTP
- **create_stdlib_api()**: Pre-built stdlib endpoints

#### VS Code Extension (v1.1.0)
- **30+ code snippets** for common Odù patterns
- **FFI syntax highlighting** for new types and functions
- **Stack highlighting** for IoT, ML, Crypto, etc.
- **Commands**: Run file (Ctrl+Shift+R), REPL, Format
- **LSP client** with restart command

#### Distribution
- **Static linking** for zero-dependency binaries
  - Windows: Static CRT (no Visual C++ Runtime required)
  - Linux: musl target (works on any distro)
- Updated GitHub Actions CI for musl builds
- Windows installer updated for Rust binary

#### Visibility System
- **Private by default** - Rust-style visibility model
- **`gbangba` / `pub`** - Mark items as public
- **`ikoko` / `private`** - Explicit private (optional)
- **`Visibility` enum** - Private, Public, Crate variants
- Visibility applies to: classes, functions, and fields

#### Web Playground
- **Interactive browser playground** at docs/playground.html
- **Dual language toggle**: Yoruba (àṣẹ) / English (end)
- **5 example programs**: Hello World, Math, Fibonacci, Opele, Random
- **Share via URL**: Base64-encoded code sharing
- **ifa-wasm crate**: WebAssembly bindings for browser

#### Babalawo Type Checker (ifa-babalawo crate)
- **Proverb-based error messages** ported from Python legacy
- **16 Odù wisdom entries** with advice and proverbs
- **40+ error-to-Odù mappings** (division by zero → Oturupon, etc.)
- **Compile-time checks**: undefined vars, unused vars, missing returns
- **CLI command**: `ifa babalawo script.ifa`
- **Output formats**: minimal (default), compact, json, verbose

#### Test Runner (Idanwo)
- **`ifa test`** CLI command for running tests
- **Auto-discovery**: finds `*_test.ifa` and `test_*.ifa` files
- **Pass/fail reporting** with timing

### Changed
- Removed emojis and box-drawing characters from error messages
- Simplified error format: `ERROR at file:line:col: message`
- Reduced Odù domain aliases to max 2 per domain

### Documentation
- Added Opele divination section to TUTORIAL.md
- Added FFI section to TUTORIAL.md and docs/index.html
- Added Visibility section with language comparison table
- Updated docs with FFI card grid section
- Added "Try Playground" button to docs/index.html

---

## [1.0.1] - 2026-01-06

### Added

#### Priority Stacks (Major Improvements)
- **Crypto Stack**: Constant-time comparison, HMAC-SHA256, SecureRng with SplitMix64, Secret zeroization
- **Frontend Stack**: XSS protection with `escape_html()`, `SafeHtml` wrapper, auto-escaping text
- **ML Stack**: In-place ops (`add_mut`, `scale_mut`), `log_softmax`, `SGD` optimizer, `TensorError`
- **IoT Stack**: `EmbeddedError` enum, `GpioPin` HAL traits, proper initialization checks
- **Gamedev Stack**: ECS `World`, `SpatialGrid` for collision, `Entity`/`Transform`/`Velocity` components
- **Backend Stack**: Existing HTTP server and ORM (no changes this version)

### Security
- Crypto: All comparison operations now use constant-time algorithms
- Crypto: Secrets are zeroed in memory on drop
- Frontend: All user text is HTML-escaped by default

### Documentation
- Updated `docs/index.html` with Priority Stacks section
- Added security feature descriptions for each stack

---

## [1.0.0] - 2026-01-05

### Added

#### Core Runtime (ifa-core)
- **IfaValue**: Dynamic type system with Int, Float, Str, Bool, List, Map, Object, Fn, Null
- **Checked Arithmetic**: Overflow promotes to Float instead of panic
- **Bytecode Engine**: OpCode definitions with `.ifab` binary format
- **VM**: Stack-based bytecode interpreter
- **Opon**: Memory management with configurable sizes and flight recorder
- **Error System**: Yoruba proverbs for educational error messages

#### Standard Library (ifa-std)
- All 16 Odù domains implemented:
  - Ọ̀gbè (1111): System/Lifecycle
  - Ọ̀yẹ̀kú (0000): Exit/Sleep with RAII Ẹbọ guards
  - Ìwòrì (0110): Time/Iteration with chrono
  - Òdí (1001): Files/Database with sandboxing and rusqlite
  - Ìrosù (1100): Console I/O with crossterm
  - Ọ̀wọ́nrín (0011): Crypto-seeded random with ChaCha20
  - Ọ̀bàrà (1000): Math add/mul with statistics
  - Ọ̀kànràn (0001): Errors/Assertions
  - Ògúndá (1110): Arrays/Processes
  - Ọ̀sá (0111): Concurrency with tokio
  - Ìká (0100): Strings with ropey and regex
  - Òtúúrúpọ̀n (0010): Checked division with rounding modes
  - Òtúrá (1011): Networking with SSRF protection
  - Ìrẹtẹ̀ (1101): Crypto (ring) and compression (zstd)
  - Ọ̀ṣẹ́ (1010): Terminal UI with ratatui
  - Òfún (0101): Capability-based permissions

#### Priority Stacks
- **IoT/Embedded**: GPIO, Timer, Serial, I2C, SPI abstractions
- **Backend**: HTTP server, Request/Response, ORM, Middleware
- **ML**: Tensor operations, Linear layers, activations, loss functions

#### Tooling
- **Ọjà Package Manager**: Cargo wrapper with `init`, `add`, `build`, `run`, `test`
- **Ìgbálẹ̀ Sandbox**: OS-level isolation (Linux/macOS/Windows)
- **CLI**: Full command set with `ifa run`, `ifa build`, `ifa flash`, etc.

#### CI/CD
- Cross-platform builds (Windows, Linux, macOS)
- Code coverage with cargo-llvm-cov
- Security audit with cargo-audit
- MSRV check (Rust 1.75.0)
- Automatic releases with installers

### Changed
- Migrated from Python interpreter to Rust VM
- Reorganized as Cargo workspace

### Deprecated
- Python-based interpreter (to be removed in 2.0)

### Security
- SSRF protection in Òtúrá networking domain
- Sandboxed file access in Òdí domain
- Cryptographic random numbers via ring

---

## Links
- [GitHub Repository](https://github.com/AAEO04/ifa-lang)
- [Documentation](https://aaeo04.github.io/ifa_lang)
- [Tutorial](TUTORIAL.md)
