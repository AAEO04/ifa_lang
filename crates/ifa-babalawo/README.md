# Ifá Babalawo (ifa-babalawo)

The **Babalawo** (Priest) is the static analysis, partial evaluation, and diagnostic engine for Ifá-Lang. It ensures that programs are not only syntactically correct but also architecturally sound, resource-balanced, and hardware-safe.

In the Ifá-Lang architecture, Babalawo fulfills the role of **Ikin** (The Palm Nuts)—providing the wisdom and analysis required before the **Iroke** (The VM) takes action.

## Core Responsibilities

- **Constant Divination (Partial Evaluation)**: Interprets and "folds" constant expressions at compile-time to minimize VM runtime overhead.
- **Ìwà Engine (Lifecycle Validation)**: Validates resource lifecycle symmetry (ensuring every `si`/`open` has a corresponding `pa`/`close`).
- **Hardware & Sandbox Guard**: Validates that hardware requests (GPU/Storage) fit within the allocated `#opon` (Calabash) boundaries.
- **Èèwọ̀ Enforcer**: Enforces architectural "taboos" (forbidden dependencies between modules).
- **Wisdom System**: Transforms dry technical errors into helpful, proverb-based diagnostics mapped to the 16 Odù domains.

## Key Features

### 1. Constant Divination (The Ikin Role)
Babalawo contains a gas-limited symbolic interpreter. It can "pre-run" pure functions and mathematical expressions during compilation.
*   **Optimization**: `const x = 10 * 5` is evaluated by Babalawo so the VM only sees the result `50`.
*   **Safety**: Identifies potential division-by-zero or out-of-bounds errors before the code ever runs.

### 2. Symbolic Infrastructure Simulation
Before the VM touches the actual hardware via `ifa-infra`, Babalawo simulates the request.
*   It checks if your GPU task graph is valid.
*   It verifies that your `#opon` size is sufficient for the requested buffers.
*   **Result**: No more "Out of Memory" crashes at runtime; Babalawo catches them at build-time.

### 3. The Ìwà Engine (Lifecycle Validation)
The Ìwà engine tracks "Resource Debt". If a resource is acquired (e.g., opening a file in the `Òdí` domain), it must be released. The Babalawo raises an error if a program terminates with outstanding debt.

### 4. Èèwọ̀ (Architectural Taboos)
Define forbidden call paths to maintain clean architecture.
```ifa
èèwọ̀ "frontend" -> "database"; # Frontend cannot call database directly
```

### 5. Proverb-Based Diagnostics
Diagnostics are categorized by the 16 Odù. If a loop is malformed, the Babalawo might invoke **Ìwòrì** (The Mirror):
> *"The river does not flow backwards. Check your loop conditions."*

## Performance Architecture
To ensure that "Deep Analysis" does not slow down development, Babalawo utilizes:
*   **Demand-Driven Analysis**: Powered by a query-based system (Salsa-style), analyzing only what is currently being edited.
*   **Gas Limits**: Constant evaluation is strictly capped to prevent the compiler from hanging on infinite loops.
*   **Incremental Caching**: Results of partial evaluations are cached to ensure sub-millisecond response times in the LSP.

## Usage

### In Rust
```rust
use ifa_babalawo::{check_program, BabalawoConfig};

let config = BabalawoConfig { 
    include_wisdom: true,
    evaluation_gas: 1000 
};
let results = check_program(&ast, "my_script.ifa");

if results.has_errors() {
    println!("{}", results.format());
}
```

## Diagnostics Summary
- **Errors (Aṣiṣe)**: Hard failures that prevent execution.
- **Warnings (Ìkìlọ̀)**: Potential issues like unused variables or unclosed resources.
- **Wisdom (Ìmọ̀ràn)**: Contextual advice and proverbs to guide the developer.
