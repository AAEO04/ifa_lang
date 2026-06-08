//! # Bytecode Module
//!
//! OpCode definitions and .ifab bytecode format for Ifá-Lang VM.

use crate::IfaValue;
use crate::error::{IfaError, IfaResult};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::{format, string::String, string::ToString, vec::Vec};
#[cfg(feature = "std")]
use std::io::{Cursor, Read, Write};
#[cfg(feature = "std")]
use std::{string::String, string::ToString, vec::Vec};

/// Memory pool sizing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum OponSize {
    Kekere,
    #[default]
    Arinrin,
    Nla,
    Ailopin,
}

impl OponSize {
    pub fn limits(&self) -> (Option<usize>, Option<usize>) {
        match self {
            OponSize::Kekere => (Some(256), Some(64)),
            OponSize::Arinrin => (Some(4096), Some(512)),
            OponSize::Nla => (Some(65536), Some(4096)),
            OponSize::Ailopin => (None, None),
        }
    }

    pub fn from_u8(val: u8) -> Self {
        match val {
            0 => OponSize::Kekere,
            1 => OponSize::Arinrin,
            2 => OponSize::Nla,
            _ => OponSize::Ailopin,
        }
    }

    pub fn to_u8(self) -> u8 {
        match self {
            OponSize::Kekere => 0,
            OponSize::Arinrin => 1,
            OponSize::Nla => 2,
            OponSize::Ailopin => 3,
        }
    }

    /// GC collection policy for this memory tier.
    /// Returns (check_interval_ticks, suspect_threshold).
    pub fn gc_policy(&self) -> (u64, usize) {
        match self {
            OponSize::Kekere => (128, 8),
            OponSize::Arinrin => (512, 32),
            OponSize::Nla => (2048, 128),
            OponSize::Ailopin => (4096, 256),
        }
    }
}

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
    /// Exported symbol names (module exports)
    pub exports: Vec<String>,
    /// Bytecode format version
    pub version: u16,
    /// Source file name (for debugging)
    pub source_name: String,
    /// Line number mapping (offset -> line)
    pub lines: Vec<(usize, u32)>,
    /// Entry point offset
    pub entry_point: usize,
    /// Opon (Memory) configuration directive
    pub opon_size: OponSize,
}

// SAFETY: Bytecode constants only contain primitives and strings (no IfaGc cycles).
// It is safely shared across actor threads.
unsafe impl Send for Bytecode {}
unsafe impl Sync for Bytecode {}

impl Bytecode {
    /// Create new empty bytecode
    pub fn new(source_name: &str) -> Self {
        Bytecode {
            code: Vec::new(),
            constants: Vec::new(),
            strings: Vec::new(),
            exports: Vec::new(),
            version: ifa_bytecode::format::VERSION,
            source_name: source_name.to_string(),
            lines: Vec::new(),
            entry_point: 0,
            opon_size: OponSize::default(),
        }
    }

    /// Add a constant to the pool and return its index
    pub fn add_constant(&mut self, val: IfaValue) -> u32 {
        let idx = self.constants.len() as u32;
        self.constants.push(val);
        idx
    }

    /// Compute a stable hash of the compiled bytecode
    pub fn hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        self.to_bytes().hash(&mut hasher);
        hasher.finish()
    }

    /// Serialize to .ifab binary format
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut output = Vec::new();
        let mut constant_bytes = Vec::new();
        let mut strings = self.strings.clone();
        let mut index_map: std::collections::HashMap<String, u16> =
            std::collections::HashMap::new();
        for (i, s) in strings.iter().enumerate() {
            index_map.insert(s.clone(), i as u16);
        }
        for constant in &self.constants {
            if let IfaValue::Str(s) = constant {
                let string_value = s.to_string();
                index_map.entry(string_value.clone()).or_insert_with(|| {
                    strings.push(string_value);
                    (strings.len() - 1) as u16
                });
            }
        }
        for name in &self.exports {
            if !index_map.contains_key(name) {
                strings.push(name.clone());
                index_map.insert(name.clone(), (strings.len() - 1) as u16);
            }
        }
        let export_indices: Vec<u16> = self
            .exports
            .iter()
            .filter_map(|n| index_map.get(n).copied())
            .collect();

        // First, write the string pool count so the deserializer knows how many
        // strings belong to the pool vs. general constants.
        let string_count = strings.len() as u32;
        constant_bytes.extend_from_slice(&string_count.to_le_bytes());
        let export_count = export_indices.len() as u32;
        constant_bytes.extend_from_slice(&export_count.to_le_bytes());
        for idx in &export_indices {
            constant_bytes.extend_from_slice(&idx.to_le_bytes());
        }

        // Write the string pool entries first (these are indexed by u16 in bytecode)
        for s in &strings {
            constant_bytes.push(0x04); // Str tag
            let bytes = s.as_bytes();
            constant_bytes.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
            constant_bytes.extend_from_slice(bytes);
        }

        // Then write the general constants
        let _ = self.write_constants(&mut constant_bytes);

        // Write header
        let header = ifa_bytecode::format::BytecodeHeader {
            magic: ifa_bytecode::format::MAGIC,
            version: self.version,
            instruction_size: self.code.len() as u32,
            constant_size: constant_bytes.len() as u32,
            opon_size: self.opon_size.to_u8(),
        };
        output.extend_from_slice(&header.to_bytes());
        output.extend_from_slice(&self.code);
        output.extend_from_slice(&constant_bytes);

        // Append source name & lines metadata
        let source_bytes = self.source_name.as_bytes();
        output.extend_from_slice(&(source_bytes.len() as u32).to_le_bytes());
        output.extend_from_slice(source_bytes);

        let line_count = self.lines.len() as u32;
        output.extend_from_slice(&line_count.to_le_bytes());
        for (offset, line) in &self.lines {
            output.extend_from_slice(&(*offset as u32).to_le_bytes());
            output.extend_from_slice(&line.to_le_bytes());
        }

        output
    }

    /// Deserialize from .ifab binary format
    pub fn from_bytes(bytes: &[u8]) -> IfaResult<Self> {
        use ifa_bytecode::format::BytecodeHeader;

        const MAX_BYTECODE_SIZE: usize = 64 * 1024 * 1024; // 64 MB limit
        if bytes.len() > MAX_BYTECODE_SIZE {
            return Err(IfaError::Custom(format!(
                "Bytecode size exceeds limit of {} bytes",
                MAX_BYTECODE_SIZE
            )));
        }

        let header = BytecodeHeader::from_bytes(bytes)
            .map_err(|_| IfaError::Custom("Invalid bytecode header".to_string()))?;

        let instructions_start: usize = 15;
        let instruction_size = header.instruction_size as usize;
        let constant_size = header.constant_size as usize;

        let instructions_end = instructions_start
            .checked_add(instruction_size)
            .ok_or_else(|| IfaError::Custom("Instruction size overflow".to_string()))?;

        let constants_start = instructions_end;
        let constants_end = constants_start
            .checked_add(constant_size)
            .ok_or_else(|| IfaError::Custom("Constant size overflow".to_string()))?;

        if bytes.len() < constants_end {
            return Err(IfaError::Custom("Bytecode file too short".to_string()));
        }

        let code = bytes[instructions_start..instructions_end].to_vec();
        let mut constants_reader = Cursor::new(&bytes[constants_start..constants_end]);

        // Read the string pool count
        let mut count_bytes = [0u8; 4];
        constants_reader
            .read_exact(&mut count_bytes)
            .map_err(|e| IfaError::Custom(format!("Failed to read string pool count: {}", e)))?;
        let string_count = u32::from_le_bytes(count_bytes) as usize;

        if string_count > 65536 {
            return Err(IfaError::Custom(
                "String pool count exceeds limit".to_string(),
            ));
        }

        let mut export_indices: Vec<u16> = Vec::new();
        if header.version >= 2 {
            let mut export_bytes = [0u8; 4];
            constants_reader
                .read_exact(&mut export_bytes)
                .map_err(|e| IfaError::Custom(format!("Failed to read export count: {}", e)))?;
            let export_count = u32::from_le_bytes(export_bytes) as usize;
            if export_count > 65536 {
                return Err(IfaError::Custom("Export count exceeds limit".to_string()));
            }
            for _ in 0..export_count {
                let mut idx_b = [0u8; 2];
                constants_reader
                    .read_exact(&mut idx_b)
                    .map_err(|e| IfaError::Custom(format!("{}", e)))?;
                export_indices.push(u16::from_le_bytes(idx_b));
            }
        }

        // Read string pool entries first (preserving their indices)
        let mut strings = Vec::with_capacity(string_count);
        for _ in 0..string_count {
            let mut tag = [0u8; 1];
            constants_reader
                .read_exact(&mut tag)
                .map_err(|e| IfaError::Custom(format!("Failed to read string tag: {}", e)))?;
            if tag[0] != 0x04 {
                return Err(IfaError::Custom(format!(
                    "Expected string tag 0x04, got 0x{:02x}",
                    tag[0]
                )));
            }
            let mut len_b = [0u8; 4];
            constants_reader
                .read_exact(&mut len_b)
                .map_err(|e| IfaError::Custom(format!("{}", e)))?;
            let str_len = u32::from_le_bytes(len_b) as usize;
            if str_len > 1_048_576 {
                return Err(IfaError::Custom("String in pool too large".to_string()));
            }
            let mut str_bytes = vec![0u8; str_len];
            constants_reader
                .read_exact(&mut str_bytes)
                .map_err(|e| IfaError::Custom(format!("{}", e)))?;
            let s = String::from_utf8(str_bytes).map_err(|e| IfaError::Custom(format!("{}", e)))?;
            strings.push(s);
        }

        // Read remaining general constants
        let constants = Self::read_constants(&mut constants_reader)?;

        let exports: Vec<String> = export_indices
            .iter()
            .filter_map(|i| strings.get(*i as usize).cloned())
            .collect();

        let mut source_name = "unknown.ifab".to_string();
        let mut lines = Vec::new();

        if bytes.len() >= constants_end + 4 {
            let mut offset = constants_end;
            let source_len = u32::from_le_bytes([
                bytes[offset],
                bytes[offset + 1],
                bytes[offset + 2],
                bytes[offset + 3],
            ]) as usize;
            offset += 4;
            if bytes.len() >= offset + source_len + 4 {
                if let Ok(s) = String::from_utf8(bytes[offset..offset + source_len].to_vec()) {
                    source_name = s;
                }
                offset += source_len;

                let line_count = u32::from_le_bytes([
                    bytes[offset],
                    bytes[offset + 1],
                    bytes[offset + 2],
                    bytes[offset + 3],
                ]) as usize;
                offset += 4;

                if bytes.len() >= offset + line_count * 8 {
                    for _ in 0..line_count {
                        let instr_off = u32::from_le_bytes([
                            bytes[offset],
                            bytes[offset + 1],
                            bytes[offset + 2],
                            bytes[offset + 3],
                        ]) as usize;
                        let line_num = u32::from_le_bytes([
                            bytes[offset + 4],
                            bytes[offset + 5],
                            bytes[offset + 6],
                            bytes[offset + 7],
                        ]);
                        lines.push((instr_off, line_num));
                        offset += 8;
                    }
                }
            }
        }

        Ok(Self {
            code,
            constants,
            strings,
            exports,
            version: header.version,
            source_name,
            lines,
            entry_point: 0,
            opon_size: OponSize::from_u8(header.opon_size),
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
            if constants.len() > 65536 {
                return Err(IfaError::Custom(
                    "Constant pool size exceeds limit".to_string(),
                ));
            }

            let mut tag = [0u8; 1];
            match reader.read_exact(&mut tag) {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break, // Done
                Err(e) => return Err(IfaError::Custom(e.to_string())),
            }

            match tag[0] {
                0x00 => constants.push(IfaValue::null()),
                0x01 => {
                    let mut b = [0u8; 1];
                    reader
                        .read_exact(&mut b)
                        .map_err(|e| IfaError::Custom(e.to_string()))?;
                    constants.push(IfaValue::bool(b[0] != 0));
                }
                0x02 => {
                    let mut b = [0u8; 8];
                    reader
                        .read_exact(&mut b)
                        .map_err(|e| IfaError::Custom(e.to_string()))?;
                    constants.push(IfaValue::int(i64::from_le_bytes(b)));
                }
                0x03 => {
                    let mut b = [0u8; 8];
                    reader
                        .read_exact(&mut b)
                        .map_err(|e| IfaError::Custom(e.to_string()))?;
                    constants.push(IfaValue::float(f64::from_le_bytes(b)));
                }
                0x04 => {
                    let mut len_b = [0u8; 4];
                    reader
                        .read_exact(&mut len_b)
                        .map_err(|e| IfaError::Custom(e.to_string()))?;
                    let str_len = u32::from_le_bytes(len_b) as usize;
                    if str_len > 1_048_576 {
                        return Err(IfaError::Custom("String constant too large".to_string()));
                    }
                    let mut str_bytes = vec![0u8; str_len];
                    reader
                        .read_exact(&mut str_bytes)
                        .map_err(|e| IfaError::Custom(e.to_string()))?;
                    let s = String::from_utf8(str_bytes)
                        .map_err(|e| IfaError::Custom(e.to_string()))?;
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
