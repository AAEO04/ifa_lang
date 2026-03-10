//! Binary file format for compiled Ifa bytecode (.ifab files)



/// Magic bytes identifying an Ifa bytecode file: "IFA\0"
pub const MAGIC: [u8; 4] = [0x49, 0x46, 0x41, 0x00];

/// Current bytecode format version
pub const VERSION: u16 = 1;

/// Header for .ifab files (14 bytes)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BytecodeHeader {
    /// Magic bytes: "IFA\0"
    pub magic: [u8; 4],

    /// Format version (current: 1)
    pub version: u16,

    /// Instruction section size in bytes
    pub instruction_size: u32,

    /// Constant pool size in bytes
    pub constant_size: u32,
}

impl BytecodeHeader {
    /// Create a new header with the given section sizes.
    pub fn new(instruction_size: u32, constant_size: u32) -> Self {
        Self {
            magic: MAGIC,
            version: VERSION,
            instruction_size,
            constant_size,
        }
    }

    /// Validate the header magic and version.
    pub fn validate(&self) -> Result<(), FormatError> {
        if self.magic != MAGIC {
            return Err(FormatError::InvalidMagic);
        }
        if self.version != VERSION {
            return Err(FormatError::UnsupportedVersion(self.version));
        }
        Ok(())
    }

    /// Serialize header to 14 bytes (little-endian).
    pub fn to_bytes(&self) -> [u8; 14] {
        let mut bytes = [0u8; 14];
        bytes[0..4].copy_from_slice(&self.magic);
        bytes[4..6].copy_from_slice(&self.version.to_le_bytes());
        bytes[6..10].copy_from_slice(&self.instruction_size.to_le_bytes());
        bytes[10..14].copy_from_slice(&self.constant_size.to_le_bytes());
        bytes
    }

    /// Parse header from bytes (min 14 bytes).
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, FormatError> {
        if bytes.len() < 14 {
            return Err(FormatError::TooShort);
        }

        let header = Self {
            magic: [bytes[0], bytes[1], bytes[2], bytes[3]],
            version: u16::from_le_bytes([bytes[4], bytes[5]]),
            instruction_size: u32::from_le_bytes([bytes[6], bytes[7], bytes[8], bytes[9]]),
            constant_size: u32::from_le_bytes([bytes[10], bytes[11], bytes[12], bytes[13]]),
        };

        header.validate()?;
        Ok(header)
    }
}

/// Errors that can occur when parsing or validating bytecode format.
#[derive(Debug)]
pub enum FormatError {
    /// Magic bytes did not match "IFA\0"
    InvalidMagic,
    /// Version not supported by this runtime
    UnsupportedVersion(u16),
    /// Input data too short for header
    TooShort,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_roundtrip() {
        let header = BytecodeHeader::new(100, 200);
        let bytes = header.to_bytes();
        let decoded = BytecodeHeader::from_bytes(&bytes).unwrap();

        assert_eq!(decoded.magic, MAGIC);
        assert_eq!(decoded.version, VERSION);
        assert_eq!(decoded.instruction_size, 100);
        assert_eq!(decoded.constant_size, 200);
    }
}
