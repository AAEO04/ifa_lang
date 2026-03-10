# Ifá Bytecode (ifa-bytecode)

The foundational crate for the Ifá-Lang runtime ecosystem. It defines the instruction set architecture (ISA), binary file format (`.ifab`), and standard error codes.

## Features

- **Standard OpCodes**: `OpCode` enum defining the VM instruction set.
- **Binary Format**: `format` module defining the `.ifab` file header and structure.
- **Error Codes**: `ErrorCode` enum for consistent error reporting across FFI and runtimes.
- **No-std Compatible**: Designed for use in both embedded (`no_std`) and full desktop (`std`) environments.

## Stack Machine Semantics

Ifá uses a **pure stack machine** model. All operands come from the stack, and all results go to the stack.

### Stack Notation
`[before] -> [after]` (Rightmost is TOP of stack)

Examples:
- `Add`: `[a, b] -> [a+b]`
- `Dup`: `[a] -> [a, a]`

## Binary File Format (.ifab)

Compiled Ifá binaries use a simple, compact format:

| Offset | Size | Type | Description |
|--------|------|------|-------------|
| 0      | 4    | `[u8; 4]` | Magic: `IFA\0` (0x49 0x46 0x41 0x00) |
| 4      | 2    | `u16`     | Version: 1 |
| 6      | 4    | `u32`     | Instruction Section Size (bytes) |
| 10     | 4    | `u32`     | Constant Pool Size (bytes) |
| 14     | ...  | `[u8]`    | **Instructions** (Raw OpCodes) |
| ...    | ...  | `[u8]`    | **Constant Pool** (Tagged Values) |

### Constant Pool Encoding
Constants are serialized with a leading type tag:
- `0x00`: Nil
- `0x01`: Bool (1 byte)
- `0x02`: Integer (8 bytes, little-endian)
- `0x03`: Float (8 bytes, little-endian)
- `0x04`: String (4 bytes length + UTF-8 bytes)

## Error Codes

Standard `u16` error codes facilitate cross-runtime communication:
- `0x00xx`: VM Support (e.g., `StackOverflow`)
- `0x01xx`: Memory (e.g., `OutOfMemory`)
- `0x02xx`: Types (e.g., `TypeMismatch`)
- `0x03xx`: Math (e.g., `DivByZero`)
- `0x04xx`: System (e.g., `FileNotFound`)
