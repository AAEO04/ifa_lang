# Ifá VM (ifa-vm)

The **Ifá VM** is the high-performance execution engine for Ifá-Lang. It handles the **Iroke** (Action) role—taking the analyzed bytecode from the compiler and "striking" the hardware via the **ifa-infra** layer.

As of the Phase 2 Unification, this VM is the **singular execution path** for the entire language, powering everything from the interactive REPL to production-grade distributed clusters.

## Core Responsibilities

- **Bytecode Dispatch**: Rapidly fetches and executes opcodes using a optimized stack-based architecture.
- **ResourceToken Enforcement**: Performs real-time security checks against the **#opon** (Calabash) to ensure the sandbox is never breached.
- **Hardware Orchestration**: Manages the **ifa-infra** task graphs for CPU, GPU, and Storage domains.
- **REPL Hot-Patching**: Supports the dynamic injection of new bytecode into a running session without state loss.

## Performance Architecture

### 1. Iroke Dispatcher (vm_iroke.rs)
The core loop uses a "Tapper" pattern (Instruction Fetch) that is optimized for branch prediction and zero-overhead dispatch. 

### 2. The Ikin Constant Pool (vm_ikin.rs)
All strings and complex constants are interned into a shared pool. This reduces memory pressure and allows for O(1) comparison speeds.

### 3. Indirection Tables
To support the dynamic nature of a REPL without sacrificing production speed, the VM uses indirection slots for user-redefinable functions. 
*   **REPL Mode**: Calls go through the table, allowing for on-the-fly redefinitions.
*   **Production Mode**: Calls are direct to memory, maximizing hardware throughput.

## Security & Sandboxing

The VM is inherently "Capability-Aware." Before executing any "Effectful" instruction (like a file write or a GPU draw), it verifies that the current context possesses the necessary **ResourceToken**. 

If a program tries to "strike" a domain it hasn't been granted access to, the VM halts execution with a detailed **Ìwòrì** (Mirror) diagnostic, explaining the violation.

## Hardware Integration

The VM communicates directly with the **ifa-infra** crate:
- **CPU**: Parallel task execution via Rayon.
- **GPU**: Compute and Graphics shaders via WGPU (Metal/Vulkan/DX12).
- **Storage**: High-speed persistence via OduStore (LSM-Tree).

## Usage

### Running a Bytecode File
```bash
ifa run path/to/program.ifab
```

### Embedding in Rust
```rust
use ifa_vm::IfaVM;
use ifa_types::Bytecode;

let mut vm = IfaVM::new();
vm.load_bytecode(my_bytecode)?;
vm.run()?;
```
