//! Ifa VM Bytecode Definitions
//!
//! This crate defines the binary interface between the Ifa compiler and all Ifa runtimes.
//! It is intentionally minimal and has zero dependencies to ensure:
//! - no_std compatibility for embedded targets
//! - Binary stability (opcode values never change)
//! - Fast compilation
//!
//! # Stability Guarantee
//!
//! Opcode discriminant values are part of the public API and will never change.
//! Adding new opcodes is permitted; changing existing values is a breaking change.

#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

/// Error types
pub mod error;
pub use error::{ErrorCode, InvalidOpCode};

/// Binary file format
#[cfg(feature = "alloc")]
pub mod format;
#[cfg(feature = "alloc")]
pub use format::{BytecodeHeader, FormatError, MAGIC, VERSION};

/// The canonical Ifa VM instruction set.
///
/// Each variant has a fixed byte value that defines the binary encoding.
/// These values are **stable** - changing them breaks compatibility with
/// existing `.ifab` bytecode files.
#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OpCode {
    // === Stack Operations (0x01-0x0F) ===
    /// Push a constant (generic 8-bit index)
    Push = 0x01,
    /// Pop the top value from the stack
    Pop = 0x02,
    /// Duplicate the top stack value
    Dup = 0x03,
    /// Swap the top two stack values
    Swap = 0x04,

    /// Push Null value
    PushNull = 0x05,
    /// Push True boolean
    PushTrue = 0x06,
    /// Push False boolean
    PushFalse = 0x07,

    /// Push Integer (followed by 8 bytes)
    PushInt = 0x08,
    /// Push Float (followed by 8 bytes)
    PushFloat = 0x09,
    /// Push String (followed by generic index)
    PushStr = 0x0A,

    // === Memory Operations (0x10-0x1F) ===
    // Standardized from ifa-embedded's Deref8/Store8 → Load8/Store8
    /// Load 8-bit value from memory address on stack
    Load8 = 0x10,
    /// Load 16-bit value from memory address on stack
    Load16 = 0x11,
    /// Load 32-bit value from memory address on stack
    Load32 = 0x12,
    /// Load 64-bit value from memory address on stack
    Load64 = 0x13,

    /// Store 8-bit value to memory address on stack
    Store8 = 0x14,
    /// Store 16-bit value to memory address on stack
    Store16 = 0x15,
    /// Store 32-bit value to memory address on stack
    Store32 = 0x16,
    /// Store 64-bit value to memory address on stack
    Store64 = 0x17,

    /// Load from Local Variable (followed by 2-byte index)
    LoadLocal = 0x18,
    /// Store to Local Variable (followed by 2-byte index)
    StoreLocal = 0x19,
    /// Load from Global Variable (followed by 2-byte name index)
    LoadGlobal = 0x1A,
    /// Store to Global Variable (followed by 2-byte name index)
    StoreGlobal = 0x1B,

    /// Get Reference/Address (generic)
    Ref = 0x1C,

    // === Arithmetic Operations (0x20-0x2F) ===
    /// Add top two stack values
    Add = 0x20,
    /// Subtract top two stack values
    Sub = 0x21,
    /// Multiply top two stack values
    Mul = 0x22,
    /// Divide top two stack values
    Div = 0x23,
    /// Modulo of top two stack values
    Mod = 0x24,
    /// Negate top stack value
    Neg = 0x25,
    /// Power (exponentiation)
    Pow = 0x26,

    // === Bitwise Operations (0x30-0x3F) ===
    /// Bitwise AND
    And = 0x30,
    /// Bitwise OR
    Or = 0x31,
    /// Bitwise XOR
    Xor = 0x32,
    /// Bitwise NOT
    Not = 0x33,
    /// Shift left
    Shl = 0x34,
    /// Shift right (logical)
    Shr = 0x35,
    /// Shift right (arithmetic)
    Sar = 0x36,

    // === Comparison Operations (0x40-0x4F) ===
    /// Equal
    Eq = 0x40,
    /// Not equal
    Ne = 0x41,
    /// Less than
    Lt = 0x42,
    /// Less than or equal
    Le = 0x43,
    /// Greater than
    Gt = 0x44,
    /// Greater than or equal
    Ge = 0x45,

    // === Control Flow (0x50-0x5F) ===
    /// Unconditional jump
    Jump = 0x50,
    /// Jump if true
    JumpIfTrue = 0x51,
    /// Jump if false
    JumpIfFalse = 0x52,
    /// Call function
    Call = 0x53,
    /// Return from function
    Return = 0x54,
    /// Halt execution
    Halt = 0x55,
    /// Yield execution
    Yield = 0x56,

    // === Type Operations (0x60-0x6F) ===
    /// Convert to Integer
    ToInt = 0x60,
    /// Convert to Float
    ToFloat = 0x61,
    /// Convert to String
    ToString = 0x62,
    /// Convert to Boolean
    ToBool = 0x63,

    // === Collections & High Level (0x70-0x7F) ===
    /// Build a list from stack items
    BuildList = 0x70,
    /// Build a map from stack items
    BuildMap = 0x71,
    /// Get index from collection
    GetIndex = 0x72,
    /// Set index in collection
    SetIndex = 0x73,
    /// Get length of collection
    Len = 0x74,
    /// Append to collection
    Append = 0x75,
    /// Push list literal (legacy)
    PushList = 0x76,
    /// Push map literal (legacy)
    PushMap = 0x77,

    // === IO & System (0x80-0x8F) ===
    /// Print to stdout
    Print = 0x80,
    /// Print raw (no newline)
    PrintRaw = 0x81,
    /// Read from stdin
    Input = 0x82,
    /// Import module
    Import = 0x83,
    /// Define a class
    DefineClass = 0x84,

    // === Advanced Calls (0x90-0x9F) ===
    /// Call object method
    CallMethod = 0x90,
    /// Call system Odu
    CallOdu = 0x91,
    /// Push function reference
    PushFn = 0x92,

    // === Exception Handling (0xA0-0xAF) ===
    /// Begin try block (followed by 4-byte jump offset to catch)
    TryBegin = 0xA0,
    /// End try block (successful exit)
    TryEnd = 0xA1,
    /// Throw exception (top of stack)
    Throw = 0xA2,
}

impl OpCode {
    /// Convert a byte to an OpCode.
    #[inline]
    pub const fn from_u8(byte: u8) -> Option<Self> {
        match byte {
            0x01 => Some(OpCode::Push),
            0x02 => Some(OpCode::Pop),
            0x03 => Some(OpCode::Dup),
            0x04 => Some(OpCode::Swap),
            0x05 => Some(OpCode::PushNull),
            0x06 => Some(OpCode::PushTrue),
            0x07 => Some(OpCode::PushFalse),
            0x08 => Some(OpCode::PushInt),
            0x09 => Some(OpCode::PushFloat),
            0x0A => Some(OpCode::PushStr),

            0x10 => Some(OpCode::Load8),
            0x11 => Some(OpCode::Load16),
            0x12 => Some(OpCode::Load32),
            0x13 => Some(OpCode::Load64),
            0x14 => Some(OpCode::Store8),
            0x15 => Some(OpCode::Store16),
            0x16 => Some(OpCode::Store32),
            0x17 => Some(OpCode::Store64),
            0x18 => Some(OpCode::LoadLocal),
            0x19 => Some(OpCode::StoreLocal),
            0x1A => Some(OpCode::LoadGlobal),
            0x1B => Some(OpCode::StoreGlobal),
            0x1C => Some(OpCode::Ref),

            0x20 => Some(OpCode::Add),
            0x21 => Some(OpCode::Sub),
            0x22 => Some(OpCode::Mul),
            0x23 => Some(OpCode::Div),
            0x24 => Some(OpCode::Mod),
            0x25 => Some(OpCode::Neg),
            0x26 => Some(OpCode::Pow),

            0x30 => Some(OpCode::And),
            0x31 => Some(OpCode::Or),
            0x32 => Some(OpCode::Xor),
            0x33 => Some(OpCode::Not),
            0x34 => Some(OpCode::Shl),
            0x35 => Some(OpCode::Shr),
            0x36 => Some(OpCode::Sar),

            0x40 => Some(OpCode::Eq),
            0x41 => Some(OpCode::Ne),
            0x42 => Some(OpCode::Lt),
            0x43 => Some(OpCode::Le),
            0x44 => Some(OpCode::Gt),
            0x45 => Some(OpCode::Ge),

            0x50 => Some(OpCode::Jump),
            0x51 => Some(OpCode::JumpIfTrue),
            0x52 => Some(OpCode::JumpIfFalse),
            0x53 => Some(OpCode::Call),
            0x54 => Some(OpCode::Return),
            0x55 => Some(OpCode::Halt),
            0x56 => Some(OpCode::Yield),

            0x60 => Some(OpCode::ToInt),
            0x61 => Some(OpCode::ToFloat),
            0x62 => Some(OpCode::ToString),
            0x63 => Some(OpCode::ToBool),

            0x70 => Some(OpCode::BuildList),
            0x71 => Some(OpCode::BuildMap),
            0x72 => Some(OpCode::GetIndex),
            0x73 => Some(OpCode::SetIndex),
            0x74 => Some(OpCode::Len),
            0x75 => Some(OpCode::Append),
            0x76 => Some(OpCode::PushList),
            0x77 => Some(OpCode::PushMap),

            0x80 => Some(OpCode::Print),
            0x81 => Some(OpCode::PrintRaw),
            0x82 => Some(OpCode::Input),
            0x83 => Some(OpCode::Import),
            0x84 => Some(OpCode::DefineClass),

            0x90 => Some(OpCode::CallMethod),
            0x91 => Some(OpCode::CallOdu),
            0x92 => Some(OpCode::PushFn),

            0xA0 => Some(OpCode::TryBegin),
            0xA1 => Some(OpCode::TryEnd),
            0xA2 => Some(OpCode::Throw),

            _ => None,
        }
    }

    /// Convert OpCode to byte.
    #[inline]
    pub const fn to_u8(self) -> u8 {
        self as u8
    }

    /// Get the mnemonic for the OpCode.
    pub const fn mnemonic(self) -> &'static str {
        match self {
            OpCode::Push => "push",
            OpCode::Pop => "pop",
            OpCode::Dup => "dup",
            OpCode::Swap => "swap",
            OpCode::PushNull => "push_null",
            OpCode::PushTrue => "push_true",
            OpCode::PushFalse => "push_false",
            OpCode::PushInt => "push_int",
            OpCode::PushFloat => "push_float",
            OpCode::PushStr => "push_str",

            OpCode::Load8 => "load8",
            OpCode::Load16 => "load16",
            OpCode::Load32 => "load32",
            OpCode::Load64 => "load64",
            OpCode::Store8 => "store8",
            OpCode::Store16 => "store16",
            OpCode::Store32 => "store32",
            OpCode::Store64 => "store64",
            OpCode::LoadLocal => "load_local",
            OpCode::StoreLocal => "store_local",
            OpCode::LoadGlobal => "load_global",
            OpCode::StoreGlobal => "store_global",
            OpCode::Ref => "ref",

            OpCode::Add => "add",
            OpCode::Sub => "sub",
            OpCode::Mul => "mul",
            OpCode::Div => "div",
            OpCode::Mod => "mod",
            OpCode::Neg => "neg",
            OpCode::Pow => "pow",

            OpCode::And => "and",
            OpCode::Or => "or",
            OpCode::Xor => "xor",
            OpCode::Not => "not",
            OpCode::Shl => "shl",
            OpCode::Shr => "shr",
            OpCode::Sar => "sar",

            OpCode::Eq => "eq",
            OpCode::Ne => "ne",
            OpCode::Lt => "lt",
            OpCode::Le => "le",
            OpCode::Gt => "gt",
            OpCode::Ge => "ge",

            OpCode::Jump => "jump",
            OpCode::JumpIfTrue => "jump_if_true",
            OpCode::JumpIfFalse => "jump_if_false",
            OpCode::Call => "call",
            OpCode::Return => "return",
            OpCode::Halt => "halt",
            OpCode::Yield => "yield",

            OpCode::ToInt => "to_int",
            OpCode::ToFloat => "to_float",
            OpCode::ToString => "to_string",
            OpCode::ToBool => "to_bool",

            OpCode::BuildList => "build_list",
            OpCode::BuildMap => "build_map",
            OpCode::GetIndex => "get_index",
            OpCode::SetIndex => "set_index",
            OpCode::Len => "len",
            OpCode::Append => "append",
            OpCode::PushList => "push_list",
            OpCode::PushMap => "push_map",

            OpCode::Print => "print",
            OpCode::PrintRaw => "print_raw",
            OpCode::Input => "input",
            OpCode::Import => "import",
            OpCode::DefineClass => "define_class",

            OpCode::CallMethod => "call_method",
            OpCode::CallOdu => "call_odu",
            OpCode::PushFn => "push_fn",

            OpCode::TryBegin => "try_begin",
            OpCode::TryEnd => "try_end",
            OpCode::Throw => "throw",
        }
    }

    /// Get the number of operand bytes.
    pub const fn operand_bytes(self) -> Option<usize> {
        match self {
            OpCode::Pop
            | OpCode::Dup
            | OpCode::Swap
            | OpCode::PushNull
            | OpCode::PushTrue
            | OpCode::PushFalse
            | OpCode::PushList
            | OpCode::PushMap
            | OpCode::Load8
            | OpCode::Load16
            | OpCode::Load32
            | OpCode::Load64
            | OpCode::Store8
            | OpCode::Store16
            | OpCode::Store32
            | OpCode::Store64
            | OpCode::Add
            | OpCode::Sub
            | OpCode::Mul
            | OpCode::Div
            | OpCode::Mod
            | OpCode::Neg
            | OpCode::Pow
            | OpCode::And
            | OpCode::Or
            | OpCode::Xor
            | OpCode::Not
            | OpCode::Shl
            | OpCode::Shr
            | OpCode::Sar
            | OpCode::Eq
            | OpCode::Ne
            | OpCode::Lt
            | OpCode::Le
            | OpCode::Gt
            | OpCode::Ge
            | OpCode::Return
            | OpCode::Halt
            | OpCode::ToInt
            | OpCode::ToFloat
            | OpCode::ToString
            | OpCode::ToBool
            | OpCode::GetIndex
            | OpCode::SetIndex
            | OpCode::Len
            | OpCode::Append
            | OpCode::Print
            | OpCode::PrintRaw
            | OpCode::TryEnd
            | OpCode::Throw
            | OpCode::Input => Some(0),

            // Fixed length operands
            OpCode::Push
            | OpCode::Jump
            | OpCode::JumpIfTrue
            | OpCode::JumpIfFalse
            | OpCode::Call
            | OpCode::Yield
            | OpCode::TryBegin
            | OpCode::Ref => Some(4),
            OpCode::PushInt | OpCode::PushFloat => Some(8),
            OpCode::LoadLocal
            | OpCode::StoreLocal
            | OpCode::LoadGlobal
            | OpCode::StoreGlobal
            | OpCode::PushStr
            | OpCode::CallMethod => Some(2),

            // Variable length operands
            OpCode::PushFn
            | OpCode::Import
            | OpCode::DefineClass
            | OpCode::CallOdu
            | OpCode::BuildList
            | OpCode::BuildMap => None,
        }
    }

    /// Get the stack effect.
    pub const fn stack_effect(self) -> Option<(usize, usize)> {
        match self {
            // Stack operations
            OpCode::Push => Some((0, 1)),
            OpCode::Pop => Some((1, 0)),
            OpCode::Dup => Some((1, 2)),
            OpCode::Swap => Some((2, 2)),
            OpCode::PushNull
            | OpCode::PushTrue
            | OpCode::PushFalse
            | OpCode::PushInt
            | OpCode::PushFloat
            | OpCode::PushStr
            | OpCode::PushList
            | OpCode::PushMap
            | OpCode::PushFn => Some((0, 1)),

            // Memory operations
            OpCode::Load8
            | OpCode::Load16
            | OpCode::Load32
            | OpCode::Load64
            | OpCode::LoadLocal
            | OpCode::LoadGlobal => Some((0, 1)), // Note: Locals/Globals load from env, not stack addr

            OpCode::Ref => Some((0, 1)), // Pushes address

            OpCode::Store8 | OpCode::Store16 | OpCode::Store32 | OpCode::Store64 => Some((2, 0)), // [addr, val] -> []
            OpCode::StoreLocal | OpCode::StoreGlobal => Some((1, 0)), // [val] -> [] (index in operand)

            // Arithmetic
            OpCode::Add
            | OpCode::Sub
            | OpCode::Mul
            | OpCode::Div
            | OpCode::Mod
            | OpCode::And
            | OpCode::Or
            | OpCode::Xor
            | OpCode::Shl
            | OpCode::Shr
            | OpCode::Sar
            | OpCode::Eq
            | OpCode::Ne
            | OpCode::Lt
            | OpCode::Le
            | OpCode::Gt
            | OpCode::Ge => Some((2, 1)),

            OpCode::Neg | OpCode::Not => Some((1, 1)),
            OpCode::Pow => Some((2, 1)),

            // Control flow
            OpCode::Jump => Some((0, 0)),
            OpCode::JumpIfTrue | OpCode::JumpIfFalse => Some((1, 0)),
            OpCode::Call | OpCode::CallMethod | OpCode::CallOdu => Some((0, 0)), // Dynamic return
            OpCode::Return | OpCode::Halt | OpCode::Yield => Some((0, 0)),

            // Type conversions
            OpCode::ToInt | OpCode::ToFloat | OpCode::ToString | OpCode::ToBool => Some((1, 1)),

            // Collections
            OpCode::GetIndex => Some((2, 1)), // [col, idx] -> [val]
            OpCode::SetIndex => Some((3, 1)), // [col, idx, val] -> [col] (or void?) - Ifa returns collection usually
            OpCode::Len => Some((1, 1)),      // [val] -> [int]
            OpCode::Append => Some((2, 1)),   // [list, val] -> [list]

            OpCode::BuildList | OpCode::BuildMap => None, // Variable input

            // IO
            OpCode::Print | OpCode::PrintRaw => Some((1, 0)),
            OpCode::Input => Some((0, 1)),
            OpCode::Import | OpCode::DefineClass => Some((0, 0)), // Side effects or pushes? Import pushes module.

            // Exception Handling
            OpCode::TryBegin => Some((0, 0)), // Pushes internal frame, no value stack effect
            OpCode::TryEnd => Some((0, 0)),   // Pops internal frame
            OpCode::Throw => Some((1, 0)),    // [err] -> [] (unwinds)
        }
    }

    /// Get the stack effect description.
    pub const fn stack_effect_description(self) -> &'static str {
        "Refer to stack_effect() values"
    }
}

impl core::fmt::Display for OpCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.mnemonic())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opcode_size_is_one_byte() {
        assert_eq!(core::mem::size_of::<OpCode>(), 1);
    }

    #[test]
    fn opcode_roundtrip_all_valid() {
        for byte in 0u8..=255 {
            if let Some(op) = OpCode::from_u8(byte) {
                assert_eq!(
                    op.to_u8(),
                    byte,
                    "Roundtrip failed for opcode {}",
                    op.mnemonic()
                );
            }
        }
    }

    #[test]
    fn critical_opcodes_have_stable_values() {
        // These values are part of the binary format and must never change
        assert_eq!(OpCode::Push as u8, 0x01);
        assert_eq!(OpCode::Pop as u8, 0x02);
        assert_eq!(OpCode::Load8 as u8, 0x10);
        assert_eq!(OpCode::Store8 as u8, 0x14);
        assert_eq!(OpCode::Add as u8, 0x20);
        assert_eq!(OpCode::Jump as u8, 0x50);
        assert_eq!(OpCode::Call as u8, 0x53);
        assert_eq!(OpCode::Return as u8, 0x54);
        assert_eq!(OpCode::Halt as u8, 0x55);
    }

    #[test]
    fn invalid_bytes_return_none() {
        assert_eq!(OpCode::from_u8(0x00), None);
        assert_eq!(OpCode::from_u8(0xFF), None);
        assert_eq!(OpCode::from_u8(0x70), None);
    }

    #[test]
    fn mnemonic_is_lowercase() {
        assert_eq!(OpCode::Push.mnemonic(), "push");
        assert_eq!(OpCode::JumpIf.mnemonic(), "jumpif");
    }
}
