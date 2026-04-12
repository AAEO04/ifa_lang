# Ifá Babalawo (ifa-babalawo)

The **Babalawo** (Priest) is the static analysis and diagnostic engine for Ifá-Lang. It ensures that programs are not only syntactically correct but also architecturally sound and resource-balanced.

## Core Responsibilities

- **Static Analysis**: Identifies undefined variables, unused bindings, and type mismatches.
- **Ìwà Engine**: Validates resource lifecycle symmetry (ensuring every `si`/`open` has a corresponding `pa`/`close`).
- **Èèwọ̀ Enforcer**: Enforces architectural "taboos" (forbidden dependencies between modules).
- **Wisdom System**: Transforms dry compiler errors into helpful, proverb-based diagnostics mapped to the 16 Odù domains.

## Key Features

### 1. The Ìwà Engine (Lifecycle Validation)
The Ìwà engine tracks "Resource Debt". If a resource is acquired (e.g., opening a file in the `Òdí` domain), it must be released. The Babalawo raises an error if a program terminates with outstanding debt.

### 2. Èèwọ̀ (Architectural Taboos)
Define forbidden call paths to maintain clean architecture.
```ifa
# Directives can define taboos
èèwọ̀ "frontend" -> "database"; # Frontend cannot call database directly
```

### 3. Proverb-Based Diagnostics
Diagnostics are categorized by the 16 Odù. If a loop is malformed, the Babalawo might invoke **Ìwòrì** (The Mirror):
> *"The river does not flow backwards. Check your loop conditions."*

## Usage

### In Rust
```rust
use ifa_babalawo::{check_program, BabalawoConfig};

let config = BabalawoConfig { include_wisdom: true };
let results = check_program(&ast, "my_script.ifa");

if results.has_errors() {
    println!("{}", results.format());
}
```

### Via CLI
```bash
ifa babalawo path/to/script.ifa --strict
```

## Diagnostics Summary
- **Errors (Aṣiṣe)**: Hard failures that prevent execution.
- **Warnings (Ìkìlọ̀)**: Potential issues like unused variables or unclosed resources.
- **Wisdom (Ìmọ̀ràn)**: Contextual advice and proverbs to guide the developer.
