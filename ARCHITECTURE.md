# Ifá-Lang Complete Architecture

## Overview

Ifá-Lang is a modular, tiered programming language ecosystem based on Yoruba cosmology and philosophy. The architecture enables execution from constrained embedded devices (Microcontrollers) to full desktop/server environments, organized in distinct layers with clear separation of concerns.

## Architecture Layers

### Layer 1: Foundation Layer
- **ifa-bytecode**: Binary interface contract and instruction set
- **ifa-types**: Shared type system and data structures

### Layer 2: Core Runtime Layer  
- **ifa-core**: VM engine, compiler, interpreter, memory management

### Layer 3: Standard Library Layer
- **ifa-std**: 16 Odù domains + specialized stacks + infrastructure

### Layer 4: Platform Runtimes
- **ifa-cli**: Native command-line interface
- **ifa-wasm**: WebAssembly browser bindings
- **ifa-embedded**: no_std embedded runtime
- **ifa-sandbox**: WASM-based security sandbox

### Layer 5: Developer Tools
- **ifa-babalawo**: LSP server and compile-time checking
- **ifa-macros**: Procedural macros for language features
- **ifa-fmt**: Code formatter

### Layer 6: Distribution
- **ifa-installer-core**: Cross-platform installation logic
- **ifa-installer-gui**: Graphical installer interface

---

## Detailed Crate Architecture

### Foundation Layer

#### `ifa-bytecode` (Binary Interface Contract)
**Purpose**: Defines the universal bytecode format and instruction set

**Core Components**:
- `OpCode` enum - Complete instruction set for VM operations
- `.ifab` binary format - Serialized bytecode container
- `ErrorCode` system - Unified error codes across platforms
- `InvalidOpCode` handling - Validation and error reporting

**Constraints**: Pure `no_std`, zero dependencies (except optional serde)
**Features**: `std`, `alloc`, `serde` for serialization support

#### `ifa-types` (Shared Type System)
**Purpose**: Central type system used by ALL other crates

**Core Types**:
- `IfaValue` - Dynamic value enum (Int, Float, Str, Bool, List, Map, Object, Fn, Null, Return)
- `IfaError` - Rich error types with Yoruba proverbs
- `OduDomain` - 16 traditional Odù + infrastructure + stacks
- `IfaShared` - Thread-safe shared values (Arc/RwLock)
- `ResourceToken` - Capability-based resource management

**VM Extensions** (`vm` feature):
- `Statement` - AST node types
- `Bytecode` - High-level serialization logic

**Constraints**: `no_std` + `alloc` compatible
**Features**: `std`, `serde`, `dashmap`, `vm`, `alloc`

---

### Core Runtime Layer

#### `ifa-core` (Virtual Machine Engine)
**Purpose**: Main runtime, compiler, and execution engine

**Core Subsystems**:

**Compiler Pipeline**:
- `lexer.rs` - Tokenization with logos
- `parser.rs` - PEG parsing with pest
- `compiler.rs` - AST to bytecode compilation
- `transpiler/` - AST to Rust source for native builds

**Execution Engine**:
- `vm.rs` - Stack-based virtual machine (34KB)
- `interpreter/` - Tree-walking interpreter with 19 domain handlers
- `vm_ikin.rs` - Ikin optimization module
- `vm_iroke.rs` - Iroke optimization module

**Memory Management**:
- `opon.rs` - "Calabash" managed heap with Rc/RefCell patterns
- `ebo.rs` - Ẹbọ resource lifecycle (RAII)
- `ajose.rs` - Àjọṣe reactive relationships (19KB)

**Language Support**:
- `ast.rs` - Abstract syntax tree definitions
- `grammar.pest` - Complete grammar specification
- `native.rs` - Native function bindings

**Error Handling**:
- `error.rs` - Comprehensive error types
- `iwa_pele.rs` - Ìwà Pẹ̀lẹ́ graceful error handling

**Features**: `native`, `parallel`, `sysinfo`, `network`, `audio`, `gpu`, `persistence`

---

### Standard Library Layer

#### `ifa-std` (16 Odù Domains)
**Purpose**: User-facing standard library organized by Yoruba cosmology

**Core Domains** (Always Available):
- `ogbe.rs` (1111) - System, CLI args, lifecycle management
- `oyeku.rs` (0000) - Exit, sleep, process termination  
- `iwori.rs` (0110) - Time, iteration, chrono integration
- `irosu.rs` (1100) - Console I/O, logging, crossterm
- `owonrin.rs` (0011) - Random number generation
- `obara.rs` (1000) - Mathematics (addition/multiplication)
- `okanran.rs` (0001) - Error handling, assertions
- `ogunda.rs` (1110) - Arrays, processes, std::process
- `ika.rs` (0100) - Strings, regex, ropey
- `oturupon.rs` (0010) - Mathematics (subtraction/division)

**Feature-Gated Domains**:
- `odi.rs` (1001) - Files, database (`backend` feature)
- `osa.rs` (0111) - Concurrency, async, tokio (`backend` feature)
- `otura.rs` (1011) - Networking, reqwest, TLS (`backend` feature)
- `irete.rs` (1101) - Crypto, compression, ring (`crypto` feature)
- `ose.rs` (1010) - Graphics, UI, ratatui (`game` feature)
- `ofun.rs` (0101) - Permissions, capabilities, reflection

**Specialized Stacks**:
- `stacks/crypto.rs` - SHA, HMAC, Argon2, Base64, SecureRng (15KB)
- `stacks/backend.rs` - HTTP, Request/Response, ORM (6KB)
- `stacks/frontend.rs` - XSS-safe HTML, Element builder (12KB)
- `stacks/gamedev.rs` - Vec2, AABB, ECS, Animation (22KB)
- `stacks/ml.rs` - Tensor operations, activations, matmul (24KB)
- `stacks/iot.rs` - GPIO, Serial, I2C/SPI protocols (16KB)
- `stacks/fusion.rs` - Fullstack IPC runtime (3KB)

**Infrastructure Layer**:
- `infra/cpu.rs` - Parallel iterators, task graphs, rayon (15KB)
- `infra/gpu.rs` - WGPU compute, memory pools (28KB)
- `infra/storage.rs` - OduStore key-value database (15KB)
- `infra/kernel.rs` - System info, memory statistics
- `infra/shaders.rs` - Pre-built WGSL compute shaders (9KB)
- `infra/runtime.rs` - Runtime management and coordination

**Foreign Function Interface**:
- `ffi.rs` - Polyglot support: JavaScript (boa_engine), Python (pyo3), native C (libffi) (44KB)

**Features**: `backend`, `frontend`, `game`, `iot`, `crypto`, `ml`, `fusion`, `parallel`, `gpu`, `persistence`, `kernel`, `audio`, `js`, `python`, `native_ffi`, `wasm`

---

### Platform Runtimes

#### `ifa-cli` (Native Command-Line Interface)
**Purpose**: Full-featured CLI with REPL, LSP integration, and self-updating

**Core Components**:
- Clap-based command-line argument parsing
- Integrated REPL with syntax highlighting
- LSP server integration via ifa-babalawo
- Self-update mechanism with signature verification
- Project generation and management tools
- Memory tracking and system monitoring

**Features**: Full std, sysinfo, parallel, gpu, persistence

#### `ifa-wasm` (WebAssembly Bindings)
**Purpose**: Browser playground and web deployment

**Core Components**:
- wasm-bindgen JavaScript interop
- Web-sys console integration
- Optimized WASM build configuration
- Limited feature set (core logic + GPU)
- No native IO capabilities

**Build Configuration**:
- `cdylib` and `rlib` crate types
- WASM optimization flags
- Small binary size focus

#### `ifa-embedded` (Bare-Metal Runtime)
**Purpose**: no_std runtime for embedded/IoT devices

**Core Components**:
- `EmbeddedVm` - Integer-heavy, non-blocking execution
- Bounded collections using heapless
- Memory-mapped I/O bindings (`mmio`)
- Interrupt-safe critical sections
- Target-specific HAL integration points

**Target Support**:
- `esp32` - ESP32 (Xtensa) with esp-hal
- `stm32` - STM32 (Cortex-M) with embassy-stm32  
- `rp2040` - RP2040 (Cortex-M0+) with embassy-rp

**Constraints**: Strict `no_std`, `no_alloc` (optional)

#### `ifa-sandbox` (Security Sandbox)
**Purpose**: WASM-based capability sandbox for safe code execution

**Core Components**:
- Wasmtime WASM runtime integration
- WASI (WebAssembly System Interface) support
- Capability-based security model
- Network capability controls
- Resource limiting and monitoring

**Features**: `network` for optional network capabilities

---

### Developer Tools

#### `ifa-babalawo` (LSP Server & Language Tools)
**Purpose**: Language server with proverb-based error messages and compile-time checking

**Core Components**:
- LSP protocol implementation (lsp-server, lsp-types)
- Compile-time error checking and analysis
- Proverb-based error messages for cultural context
- Syntax highlighting and code completion
- Refactoring tools and code navigation
- Integration with editors and IDEs

#### `ifa-macros` (Procedural Macros)
**Purpose**: Code generation for cultural safety features and language constructs

**Core Components**:
- Procedural macro implementations
- Ẹbọ (sacrifice/offering) code generation
- Àjọṣe (relationship) macro expansions
- Cultural safety compile-time checks
- Custom derive implementations

#### `ifa-fmt` (Code Formatter)
**Purpose**: Opinionated code formatter for Ifá-Lang

**Core Components**:
- AST-based code formatting
- Configurable style rules
- Integration with ifa-core parser
- IDE integration support

---

### Distribution Layer

#### `ifa-installer-core` (Cross-Platform Installation)
**Purpose**: Core installation logic and system integration

**Core Components**:
- Cross-platform system detection
- Download and verification with SHA256
- Archive extraction (zip, tar, gzip)
- Registry integration (Windows)
- PATH and environment setup
- Dependency checking and resolution

#### `ifa-installer-gui` (Graphical Installer)
**Purpose**: User-friendly graphical installation interface

**Core Components**:
- eframe-based GUI framework
- Interactive installation wizard
- Progress tracking and feedback
- File dialogs for configuration
- System requirement validation

---

## Type System Architecture

### "The Village and The Hut" Philosophy

Ifá-Lang uses a split-type architecture balancing performance with scalability.

#### `IfaValue` (The Hut 🛖)
- **Use Case**: VM inner loop and interpreter operations
- **Semantics**: Thread-local, non-Send
- **Reference Counting**: `Rc<RefCell<T>>`
- **Performance**: Extremely fast, no atomic overhead
- **Limitation**: Cannot be passed between threads

#### `IfaShared` (The Village 🏘️)
- **Use Case**: Global registry, worker threads, messaging
- **Semantics**: Thread-safe, Send + Sync
- **Reference Counting**: `Arc<RwLock<T>>` or `DashMap`
- **Performance**: Slower due to atomics/locks
- **Advantage**: Serializable and movable between threads

### State Conversion Bridge

```rust
// Converting Local -> Shared (Deep Copy / Arc Wrap)
let shared: IfaShared = local_value.freeze()?;

// Converting Shared -> Local (Cow / Deep Copy)  
let local: IfaValue = shared.thaw();
```

---

## Execution Flow

### 1. Compilation Pipeline
```
Source (.ifa) 
    ↓ Lexer (logos)
Tokens 
    ↓ Parser (pest)
AST 
    ↓ Compiler
Bytecode (.ifab)
```

### 2. Execution Paths
**Bytecode** can be executed by:
- `ifa-core` VM (CLI/Server) - Full feature set
- `ifa-embedded` VM (Embedded) - Minimal subset
- `ifa-wasm` VM (Browser) - WASM-compatible subset

### 3. Native Compilation
```
AST 
    ↓ Transpiler
Rust Source 
    ↓ Cargo
Native Binary
```

---

## Cross-Runtime Compatibility

### Bytecode Portability
- **`.ifab` Format**: Write-once, run-anywhere (within resource limits)
- **Constant Pool**: Abstracts strings and large numbers for streaming
- **Error Codes**: Unified `ErrorCode` ensures consistent error meanings

### Feature Flag Matrix
- **Native**: Full tokio async, networking, GPU, audio, system info
- **WASM**: Core logic + GPU (wgpu), no native IO
- **Embedded**: no_std + heapless bounded collections

---

## Dependency Architecture

### Critical Shared Components
1. **ifa-types**: Single source of truth for all types
2. **Error Handling**: thiserror + eyre across all crates
3. **Serialization**: serde + serde_json for bytecode/config
4. **Feature System**: Granular feature flags control capabilities

### Platform-Specific Services
- **Native**: Full tokio async, networking, GPU, audio, system info
- **WASM**: Limited to core logic + GPU (wgpu compatible), no native IO
- **Embedded**: no_std + heapless bounded collections

---

## Security Architecture

### Capability-Based Security
- **ResourceToken**: Capability-based resource access control
- **Sandbox**: WASM-based isolation for untrusted code
- **Permissions**: Òfún domain for permission management
- **Safe Defaults**: All operations require explicit capabilities

### Memory Safety
- **Opon**: Managed heap with lifetime tracking
- **Ẹbọ**: RAII-based resource cleanup
- **No Unsafe**: Safe Rust with minimal unsafe blocks
- **Bounds Checking**: Comprehensive array access validation

---

## Performance Architecture

### Optimization Strategies
- **Ikin**: Specialized VM optimizations
- **Iroke**: Advanced execution optimizations
- **Parallel Processing**: Rayon-based parallel operations
- **GPU Acceleration**: WGPU compute shader integration
- **Memory Efficiency**: Rc/RefCell patterns for fast local operations

### Benchmarking
- **Criterion**: Performance benchmarking suite
- **Opcode Dispatch**: Optimized instruction execution
- **Memory Profiling**: Allocation and usage tracking

---

## Testing Architecture

### Test Organization
- **Unit Tests**: Per-module comprehensive testing
- **Integration Tests**: Cross-crate compatibility testing
- **Property-Based Tests**: Proptest for core algorithms
- **Benchmark Tests**: Performance regression testing

### Quality Assurance
- **Clippy**: Lint enforcement for code quality
- **Documentation**: Comprehensive API documentation
- **Examples**: Working code examples for all features

---

## Cultural Architecture

### Yoruba Cosmology Integration
- **16 Odù**: Domain organization based on Ifá divination
- **Proverb-Based Errors**: Culturally relevant error messages
- **Philosophical Naming**: Components named after Yoruba concepts
- **Cultural Safety**: Compile-time checks for cultural appropriateness

### Language Philosophy
- **Àjọṣe**: Reactive relationships between components
- **Ẹbọ**: Resource lifecycle and sacrifice (cleanup)
- **Ìwà Pẹ̀lẹ️**: Graceful error handling and character
- **Opon**: Calabash metaphor for memory container

This architecture represents a complete, production-ready programming language ecosystem that bridges traditional Yoruba wisdom with modern software engineering principles.
