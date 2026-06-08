//! Embedded OpCodes
//!
//! Minimal instruction set for the embedded target.

/// Embedded opcodes - minimal subset for constrained devices
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EmbeddedOpCode {
    /// Push null
    PushNull = 0x05,
    /// Push integer (followed by 4 bytes, little-endian)
    PushInt = 0x08,
    /// Push float (followed by 4 bytes, little-endian)
    PushFloat = 0x09,
    /// Push true
    PushTrue = 0x06,
    /// Push false
    PushFalse = 0x07,

    /// Pop and discard
    Pop = 0x02,
    /// Duplicate top
    Dup = 0x03,

    /// Add
    Add = 0x20,
    /// Subtract
    Sub = 0x21,
    /// Multiply
    Mul = 0x22,
    /// Divide
    Div = 0x23,

    /// Equal
    Eq = 0x40,
    /// Less than
    Lt = 0x42,
    /// Greater than
    Gt = 0x44,

    /// Logical NOT
    Not = 0x33,

    /// Load local variable (followed by 1-byte index)
    LoadLocal = 0x18,
    /// Store local variable (followed by 1-byte index)
    StoreLocal = 0x19,

    /// Jump (followed by 2-byte offset, little-endian)
    Jump = 0x50,
    /// Jump if false
    JumpIfFalse = 0x52,

    /// Halt execution
    Halt = 0x55,

    // ===================================
    // POINTER OPS (Address 0xA0 base)
    // ===================================
    /// Push Ref/Address (followed by 4 byte address/index)
    Ref = 0x1C,
    /// Dereference (Read): Pop addr -> Push value (32-bit load)
    Deref = 0x12,
    /// Store Dereference (Write): Pop addr, Pop val -> Write val to address (32-bit store)
    StoreDeref = 0x16,

    // Sized pointer ops
    /// Store 8-bit value to address
    Store8 = 0x14,
    /// Store 16-bit value to address
    Store16 = 0x15,
    /// Read 8-bit value from address
    Load8 = 0x10,
    /// Read 16-bit value from address
    Load16 = 0x11,

    /// Yield execution (pause without resetting)
    /// Followed by 4 byte duration hint (u32 microseconds), or 0 for indefinite
    Yield = 0x56,
}

impl EmbeddedOpCode {
    /// Parse opcode from byte
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x05 => Some(EmbeddedOpCode::PushNull),
            0x08 => Some(EmbeddedOpCode::PushInt),
            0x09 => Some(EmbeddedOpCode::PushFloat),
            0x06 => Some(EmbeddedOpCode::PushTrue),
            0x07 => Some(EmbeddedOpCode::PushFalse),
            0x02 => Some(EmbeddedOpCode::Pop),
            0x03 => Some(EmbeddedOpCode::Dup),
            0x20 => Some(EmbeddedOpCode::Add),
            0x21 => Some(EmbeddedOpCode::Sub),
            0x22 => Some(EmbeddedOpCode::Mul),
            0x23 => Some(EmbeddedOpCode::Div),
            0x40 => Some(EmbeddedOpCode::Eq),
            0x42 => Some(EmbeddedOpCode::Lt),
            0x44 => Some(EmbeddedOpCode::Gt),
            0x33 => Some(EmbeddedOpCode::Not),
            0x18 => Some(EmbeddedOpCode::LoadLocal),
            0x19 => Some(EmbeddedOpCode::StoreLocal),
            0x50 => Some(EmbeddedOpCode::Jump),
            0x52 => Some(EmbeddedOpCode::JumpIfFalse),
            0x55 => Some(EmbeddedOpCode::Halt),
            0x1C => Some(EmbeddedOpCode::Ref),
            0x12 => Some(EmbeddedOpCode::Deref),
            0x16 => Some(EmbeddedOpCode::StoreDeref),
            0x14 => Some(EmbeddedOpCode::Store8),
            0x15 => Some(EmbeddedOpCode::Store16),
            0x10 => Some(EmbeddedOpCode::Load8),
            0x11 => Some(EmbeddedOpCode::Load16),
            0x56 => Some(EmbeddedOpCode::Yield),
            _ => None,
        }
    }
}
