# Ifá Bytecode (ifa-bytecode)

The foundational crate for the Ifá-Lang runtime ecosystem. It defines the instruction set architecture (ISA), binary file format (`.ifab`), and standard error codes.

## Features

- **Standard OpCodes**: `OpCode` enum defining the VM instruction set with stable discriminants.
- **Binary Format**: `format` module defining the `.ifab` header, instruction section, and constant pool.
- **Error Codes**: `ErrorCode` enum for consistent error reporting across FFI and runtimes.
- **Stack Machine Semantics**: Documented stack effects for every instruction.
- **No-std Compatible**: Zero-dependency crate, perfect for `no_std` embedded targets.

## Instruction Set Architecture (ISA)

The ISA is divided into functional ranges:

| Range | Category | Examples |
| :--- | :--- | :--- |
| `0x01-0x0F` | Stack Ops | `Push`, `Pop`, `Dup`, `Swap` |
| `0x10-0x1F` | Memory Ops | `Load8`, `StoreLocal`, `Ref` |
| `0x20-0x2F` | Arithmetic | `Add`, `Sub`, `Mul`, `Div`, `Pow` |
| `0x30-0x3F` | Bitwise | `And`, `Or`, `Xor`, `Shl` |
| `0x40-0x4F` | Comparison | `Eq`, `Ne`, `Lt`, `Gt` |
| `0x50-0x5F` | Control Flow | `Jump`, `Call`, `Return`, `Await` |
| `0x60-0x6F` | Type Conversions| `ToInt`, `ToString` |
| `0x70-0x7F` | Collections | `BuildList`, `GetIndex`, `Len` |
| `0xA0-0xAF` | Exceptions | `TryBegin`, `TryEnd`, `Throw` |

## Stack Machine Semantics

Ifá uses a **pure stack machine** model. Every instruction defines its "Stack Effect" — what it consumes and what it produces.

### Stack Notation
`[bottom, ..., top-1, top] -> [bottom, ..., result]`

### Examples:
- **`Add` (0x20)**: `[a, b] -> [a + b]`
- **`Load8` (0x10)**: `[addr] -> [u8_value]`
- **`StoreLocal` (0x19)**: `[value] -> []` (Index is an instruction operand)
- **`Call` (0x53)**: `[arg1, ..., argN, fn_ptr] -> [result]`

## Binary File Format (.ifab)

Compiled Ifá binaries (`.ifab`) are composed of a header followed by three major sections:

1. **Header**: Magic bytes `IFA\0`, version, and section sizes.
2. **Instruction Section**: Raw byte stream of OpCodes and their immediate operands.
3. **Constant Pool**: Tagged values (Strings, Large Integers, Floats) referenced by index.

### Constant Pool Encoding
Constants are prefixed with a type tag:
- `0x00`: Nil
- `0x01`: Bool
- `0x02`: Integer (i64, Little-Endian)
- `0x03`: Float (f64, Little-Endian)
- `0x04`: String (u32 length + UTF-8 payload)

## Error Codes

Standard 16-bit error codes ensure that a `StackOverflow` or `DivByZero` is interpreted the same way whether it's running in a desktop VM, a WASM sandbox, or on an ESP32.
