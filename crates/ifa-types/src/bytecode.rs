//! # Bytecode Module
//!
//! OpCode definitions and .ifab bytecode format for Ifá-Lang VM.

use crate::IfaValue;
use crate::error::{IfaError, IfaResult};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "alloc")]
use alloc::{format, string::String, string::ToString, vec::Vec};
#[cfg(feature = "std")]
use std::{string::String, string::ToString, vec::Vec};
#[cfg(feature = "std")]
use std::io::{Cursor, Read, Write};

/// Bytecode chunk - a compiled unit of code
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Bytecode {
    /// Raw bytecode bytes
    pub code: Vec<u8>,
    /// Constant pool (unified)
    pub constants: Vec<IfaValue>,
    /// Legacy string pool (kept for backward compat, populated from constants)
    pub strings: Vec<String>,
    /// Source file name (for debugging)
    pub source_name: String,
    /// Line number mapping (offset -> line)
    pub lines: Vec<(usize, u32)>,
    /// Entry point offset
    pub entry_point: usize,
}

impl Bytecode {
    /// Create new empty bytecode
    pub fn new(source_name: &str) -> Self {
        Bytecode {
            code: Vec::new(),
            constants: Vec::new(),
            strings: Vec::new(),
            source_name: source_name.to_string(),
            lines: Vec::new(),
            entry_point: 0,
        }
    }

    /// Add a constant to the pool and return its index
    pub fn add_constant(&mut self, val: IfaValue) -> u32 {
        let idx = self.constants.len() as u32;
        self.constants.push(val);
        idx
    }

    /// Serialize to .ifab binary format
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut output = Vec::new();
        let mut constant_bytes = Vec::new();
        
        // Encode constants using new writer
        let _ = self.write_constants(&mut constant_bytes); // Should handle error? using unwrap/defaults for simple logic

        // Write header
        let header = ifa_bytecode::format::BytecodeHeader::new(
            self.code.len() as u32,
            constant_bytes.len() as u32,
        );
        output.extend_from_slice(&header.to_bytes());
        output.extend_from_slice(&self.code);
        output.extend_from_slice(&constant_bytes);
        output
    }

    /// Deserialize from .ifab binary format
    pub fn from_bytes(bytes: &[u8]) -> IfaResult<Self> {
        use ifa_bytecode::format::BytecodeHeader;

        let header = BytecodeHeader::from_bytes(bytes)
            .map_err(|_| IfaError::Custom("Invalid bytecode header".to_string()))?;

        let instructions_start = 14; 
        let instructions_end = instructions_start + header.instruction_size as usize;
        let constants_start = instructions_end;
        let constants_end = constants_start + header.constant_size as usize;

        if bytes.len() < constants_end {
            return Err(IfaError::Custom("Bytecode file too short".to_string()));
        }

        let code = bytes[instructions_start..instructions_end].to_vec();
        let mut constants_reader = Cursor::new(&bytes[constants_start..constants_end]);
        let constants = Self::read_constants(&mut constants_reader)?;

        // Backward compatibility: Populate strings from constants
        let strings: Vec<String> = constants
            .iter()
            .filter_map(|c| match c {
                IfaValue::Str(s) => Some(s.to_string()),
                _ => None,
            })
            .collect();

        Ok(Self {
            code,
            constants,
            strings,
            source_name: "unknown.ifab".to_string(),
            lines: Vec::new(),
            entry_point: 0,
        })
    }

    fn write_constants<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        for c in &self.constants {
            match c {
                IfaValue::Null => writer.write_all(&[0x00])?,
                IfaValue::Bool(b) => {
                    writer.write_all(&[0x01])?;
                    writer.write_all(&[if *b { 1 } else { 0 }])?;
                }
                IfaValue::Int(i) => {
                    writer.write_all(&[0x02])?;
                    writer.write_all(&i.to_le_bytes())?;
                }
                IfaValue::Float(f) => {
                    writer.write_all(&[0x03])?;
                    writer.write_all(&f.to_le_bytes())?;
                }
                IfaValue::Str(s) => {
                    writer.write_all(&[0x04])?;
                    let bytes = s.as_bytes();
                    writer.write_all(&(bytes.len() as u32).to_le_bytes())?;
                    writer.write_all(bytes)?;
                }
                _ => {
                    // Skip unsupported types for bytecode serialization for now
                }
            }
        }
        Ok(())
    }

    fn read_constants<R: Read>(reader: &mut R) -> Result<Vec<IfaValue>, IfaError> {
        let mut constants = Vec::new();
        // Read until EOF? No, header says specific size byte-wise, but we don't know count.
        // Wait, the format stores LENGTH in header (bytes), not count.
        // So we read until EOF of the slice provided to Cursor.
        
        // We need to robustly read tag then data until reader is empty.
        // Or checking stream position?
        
        loop {
            let mut tag = [0u8; 1];
            match reader.read_exact(&mut tag) {
                Ok(_) => {},
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break, // Done
                Err(e) => return Err(IfaError::Custom(e.to_string())),
            }

            match tag[0] {
                0x00 => constants.push(IfaValue::null()),
                0x01 => {
                    let mut b = [0u8; 1];
                    reader.read_exact(&mut b).map_err(|e| IfaError::Custom(e.to_string()))?;
                    constants.push(IfaValue::bool(b[0] != 0));
                }
                0x02 => {
                    let mut b = [0u8; 8];
                    reader.read_exact(&mut b).map_err(|e| IfaError::Custom(e.to_string()))?;
                    constants.push(IfaValue::int(i64::from_le_bytes(b)));
                }
                0x03 => {
                    let mut b = [0u8; 8];
                    reader.read_exact(&mut b).map_err(|e| IfaError::Custom(e.to_string()))?;
                    constants.push(IfaValue::float(f64::from_le_bytes(b)));
                }
                0x04 => {
                    let mut len_b = [0u8; 4];
                    reader.read_exact(&mut len_b).map_err(|e| IfaError::Custom(e.to_string()))?;
                    let str_len = u32::from_le_bytes(len_b) as usize;
                    let mut str_bytes = vec![0u8; str_len];
                    reader.read_exact(&mut str_bytes).map_err(|e| IfaError::Custom(e.to_string()))?;
                    let s = String::from_utf8(str_bytes).map_err(|e| IfaError::Custom(e.to_string()))?;
                    constants.push(IfaValue::str(s));
                }
                _ => return Err(IfaError::Custom("Invalid constant tag".to_string())),
            }
        }
        Ok(constants)
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
    use ifa_bytecode::OpCode;

    #[test]
    fn test_opcode_roundtrip() {
        let opcodes = [OpCode::Add, OpCode::Sub, OpCode::Mul, OpCode::Halt];
        for op in opcodes {
            let byte = op as u8;
            assert_eq!(OpCode::from_u8(byte).unwrap(), op);
        }
    }

    #[test]
    fn test_bytecode_serialize_v2() {
        let mut bc = Bytecode::new("test.ifa");
        bc.code = vec![0x01, 0x20, 0xFF]; // PushInt, Add, Halt
        bc.add_constant(IfaValue::int(42));
        bc.add_constant(IfaValue::str("hello"));

        let bytes = bc.to_bytes();
        let decoded = Bytecode::from_bytes(&bytes).unwrap();

        assert_eq!(decoded.code, bc.code);
        assert_eq!(decoded.constants.len(), 2);
        
        match &decoded.constants[0] {
            IfaValue::Int(i) => assert_eq!(*i, 42),
            _ => panic!("Expected Int"),
        }
        match &decoded.constants[1] {
            IfaValue::Str(s) => assert_eq!(s.as_ref(), "hello"),
            _ => panic!("Expected Str"),
        }

        // Check legacy strings population
        assert_eq!(decoded.strings.len(), 1);
        assert_eq!(decoded.strings[0], "hello");
    }
}
