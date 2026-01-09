//! # Bytecode Module
//!
//! OpCode definitions and .ifab bytecode format for Ifá-Lang VM.

use crate::error::{IfaError, IfaResult};
use serde::{Deserialize, Serialize};

/// Magic bytes for .ifab files: "IFAB" in ASCII
pub const BYTECODE_MAGIC: [u8; 4] = [0x49, 0x46, 0x41, 0x42];

/// Current bytecode version
pub const BYTECODE_VERSION: u8 = 1;

/// VM opcodes - the instruction set for Ifá-Lang
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum OpCode {
    // =========================================================================
    // STACK OPERATIONS
    // =========================================================================
    /// Push null onto stack
    PushNull = 0x00,
    /// Push integer constant (followed by 8 bytes)
    PushInt = 0x01,
    /// Push float constant (followed by 8 bytes)
    PushFloat = 0x02,
    /// Push string constant (followed by length + bytes)
    PushStr = 0x03,
    /// Push boolean true
    PushTrue = 0x04,
    /// Push boolean false
    PushFalse = 0x05,
    /// Push empty list
    PushList = 0x06,
    /// Push empty map
    PushMap = 0x07,

    /// Pop and discard top of stack
    Pop = 0x10,
    /// Duplicate top of stack
    Dup = 0x11,
    /// Swap top two stack elements
    Swap = 0x12,

    // =========================================================================
    // ARITHMETIC (Ọ̀bàrà & Òtúúrúpọ̀n)
    // =========================================================================
    /// Add top two values
    Add = 0x20,
    /// Subtract
    Sub = 0x21,
    /// Multiply
    Mul = 0x22,
    /// Divide
    Div = 0x23,
    /// Modulo
    Mod = 0x24,
    /// Negate (unary minus)
    Neg = 0x25,
    /// Power
    Pow = 0x26,

    // =========================================================================
    // COMPARISON
    // =========================================================================
    /// Equal
    Eq = 0x30,
    /// Not equal
    Ne = 0x31,
    /// Less than
    Lt = 0x32,
    /// Less than or equal
    Le = 0x33,
    /// Greater than
    Gt = 0x34,
    /// Greater than or equal
    Ge = 0x35,

    // =========================================================================
    // LOGIC
    // =========================================================================
    /// Logical AND
    And = 0x40,
    /// Logical OR
    Or = 0x41,
    /// Logical NOT
    Not = 0x42,

    // =========================================================================
    // VARIABLES
    // =========================================================================
    /// Load local variable (followed by index)
    LoadLocal = 0x50,
    /// Store to local variable
    StoreLocal = 0x51,
    /// Load global variable (followed by name)
    LoadGlobal = 0x52,
    /// Store to global variable
    StoreGlobal = 0x53,

    // =========================================================================
    // CONTROL FLOW
    // =========================================================================
    /// Unconditional jump (followed by offset)
    Jump = 0x60,
    /// Jump if top of stack is falsy
    JumpIfFalse = 0x61,
    /// Jump if top of stack is truthy
    JumpIfTrue = 0x62,

    // =========================================================================
    // FUNCTIONS
    // =========================================================================
    /// Call function (followed by arg count)
    Call = 0x70,
    /// Return from function
    Return = 0x71,
    /// Call Odù domain method (followed by domain + method IDs)
    CallOdu = 0x72,
    /// Call method on object
    CallMethod = 0x73,

    // =========================================================================
    // COLLECTIONS
    // =========================================================================
    /// Get index (list\[i\] or map\[k\])
    GetIndex = 0x80,
    /// Set index
    SetIndex = 0x81,
    /// Get length
    Len = 0x82,
    /// Append to list
    Append = 0x83,
    /// Build list from N stack items
    BuildList = 0x84,
    /// Build map from N key-value pairs on stack
    BuildMap = 0x85,

    // =========================================================================
    // I/O (Ìrosù)
    // =========================================================================
    /// Print (with newline)
    Print = 0x90,
    /// Print (no newline)
    PrintRaw = 0x91,
    /// Read input
    Input = 0x92,

    // =========================================================================
    // SYSTEM
    // =========================================================================
    /// Halt execution
    Halt = 0xFF,
}

impl OpCode {
    /// Get opcode from byte value
    pub fn from_byte(byte: u8) -> IfaResult<OpCode> {
        match byte {
            0x00 => Ok(OpCode::PushNull),
            0x01 => Ok(OpCode::PushInt),
            0x02 => Ok(OpCode::PushFloat),
            0x03 => Ok(OpCode::PushStr),
            0x04 => Ok(OpCode::PushTrue),
            0x05 => Ok(OpCode::PushFalse),
            0x06 => Ok(OpCode::PushList),
            0x07 => Ok(OpCode::PushMap),
            0x10 => Ok(OpCode::Pop),
            0x11 => Ok(OpCode::Dup),
            0x12 => Ok(OpCode::Swap),
            0x20 => Ok(OpCode::Add),
            0x21 => Ok(OpCode::Sub),
            0x22 => Ok(OpCode::Mul),
            0x23 => Ok(OpCode::Div),
            0x24 => Ok(OpCode::Mod),
            0x25 => Ok(OpCode::Neg),
            0x26 => Ok(OpCode::Pow),
            0x30 => Ok(OpCode::Eq),
            0x31 => Ok(OpCode::Ne),
            0x32 => Ok(OpCode::Lt),
            0x33 => Ok(OpCode::Le),
            0x34 => Ok(OpCode::Gt),
            0x35 => Ok(OpCode::Ge),
            0x40 => Ok(OpCode::And),
            0x41 => Ok(OpCode::Or),
            0x42 => Ok(OpCode::Not),
            0x50 => Ok(OpCode::LoadLocal),
            0x51 => Ok(OpCode::StoreLocal),
            0x52 => Ok(OpCode::LoadGlobal),
            0x53 => Ok(OpCode::StoreGlobal),
            0x60 => Ok(OpCode::Jump),
            0x61 => Ok(OpCode::JumpIfFalse),
            0x62 => Ok(OpCode::JumpIfTrue),
            0x70 => Ok(OpCode::Call),
            0x71 => Ok(OpCode::Return),
            0x72 => Ok(OpCode::CallOdu),
            0x73 => Ok(OpCode::CallMethod),
            0x80 => Ok(OpCode::GetIndex),
            0x81 => Ok(OpCode::SetIndex),
            0x82 => Ok(OpCode::Len),
            0x83 => Ok(OpCode::Append),
            0x84 => Ok(OpCode::BuildList),
            0x85 => Ok(OpCode::BuildMap),
            0x90 => Ok(OpCode::Print),
            0x91 => Ok(OpCode::PrintRaw),
            0x92 => Ok(OpCode::Input),
            0xFF => Ok(OpCode::Halt),
            _ => Err(IfaError::UnknownOpcode(byte)),
        }
    }
}

/// Bytecode chunk - a compiled unit of code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bytecode {
    /// Raw bytecode bytes
    pub code: Vec<u8>,
    /// String constant pool
    pub strings: Vec<String>,
    /// Source file name (for debugging)
    pub source_name: String,
    /// Line number mapping (offset -> line)
    pub lines: Vec<(usize, u32)>,
}

impl Bytecode {
    /// Create new empty bytecode
    pub fn new(source_name: &str) -> Self {
        Bytecode {
            code: Vec::new(),
            strings: Vec::new(),
            source_name: source_name.to_string(),
            lines: Vec::new(),
        }
    }

    /// Serialize to .ifab format
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();

        // Magic header
        result.extend_from_slice(&BYTECODE_MAGIC);
        result.push(BYTECODE_VERSION);

        // Use bincode for the rest
        if let Ok(encoded) = bincode::serialize(self) {
            result.extend(encoded);
        }

        result
    }

    /// Deserialize from .ifab format
    pub fn from_bytes(bytes: &[u8]) -> IfaResult<Self> {
        if bytes.len() < 5 {
            return Err(IfaError::Custom("Invalid bytecode: too short".to_string()));
        }

        // Check magic
        if &bytes[0..4] != &BYTECODE_MAGIC {
            return Err(IfaError::Custom("Invalid bytecode: bad magic".to_string()));
        }

        // Check version
        if bytes[4] != BYTECODE_VERSION {
            return Err(IfaError::Custom(format!(
                "Bytecode version mismatch: expected {}, got {}",
                BYTECODE_VERSION, bytes[4]
            )));
        }

        // Deserialize rest
        bincode::deserialize(&bytes[5..])
            .map_err(|e| IfaError::Custom(format!("Bytecode decode error: {}", e)))
    }

    /// Get line number for instruction at offset
    pub fn get_line(&self, offset: usize) -> Option<u32> {
        for (off, line) in self.lines.iter().rev() {
            if *off <= offset {
                return Some(*line);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_roundtrip() {
        let opcodes = [OpCode::Add, OpCode::Sub, OpCode::Mul, OpCode::Halt];
        for op in opcodes {
            let byte = op as u8;
            assert_eq!(OpCode::from_byte(byte).unwrap(), op);
        }
    }

    #[test]
    fn test_bytecode_serialize() {
        let mut bc = Bytecode::new("test.ifa");
        bc.code = vec![0x01, 0x20, 0xFF]; // PushInt, Add, Halt
        bc.strings = vec!["hello".to_string()];

        let bytes = bc.to_bytes();
        let decoded = Bytecode::from_bytes(&bytes).unwrap();

        assert_eq!(decoded.code, bc.code);
        assert_eq!(decoded.strings, bc.strings);
    }
}
