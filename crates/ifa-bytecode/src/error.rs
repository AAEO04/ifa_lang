/// Error type for invalid opcode values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidOpCode(pub u8);

impl core::fmt::Display for InvalidOpCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Invalid opcode: {:#x}", self.0)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for InvalidOpCode {}

/// Standard numeric error codes for Ifá-Lang runtimes.
///
/// These codes allow for consistent error reporting across:
/// - ifa-core (full runtime)
/// - ifa-embedded (no_std)
/// - FFI boundaries
///
/// Codes are grouped by category (u16):
/// - 0x0000..0x00FF: Virtual Machine Support
/// - 0x0100..0x01FF: Memory & Resource Management
/// - 0x0200..0x02FF: Type System
/// - 0x0300..0x03FF: Math & Logic
/// - 0x0400..0x04FF: IO & System
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ErrorCode {
    /// Success (no error)
    Ok = 0x0000,

    // === VM Support (0x00XX) ===
    /// Generic VM Error
    VmError = 0x0001,
    /// Stack Overflow
    StackOverflow = 0x0002,
    /// Stack Underflow
    StackUnderflow = 0x0003,
    /// Invalid Instruction/OpCode
    InvalidOpCode = 0x0004,
    /// Invalid Bytecode Format
    InvalidBytecode = 0x0005,
    /// Program Halted explicitly
    Halt = 0x0006,
    /// Yielded execution
    Yield = 0x0007,
    /// Async not available on this target
    AsyncNotAvailable = 0x0008,

    // === Memory (0x01XX) ===
    /// Out of Memory
    OutOfMemory = 0x0100,
    /// Invalid Memory Access (segfault equivalent)
    AccessViolation = 0x0101,
    /// Reference out of bounds
    OutOfBounds = 0x0102,

    // === Types (0x02XX) ===
    /// Type Mismatch
    TypeMismatch = 0x0200,
    /// Invalid Cast
    InvalidCast = 0x0201,
    /// Undefined Variable
    UndefinedVar = 0x0202,

    // === Math (0x03XX) ===
    /// Division by Zero
    DivByZero = 0x0300,
    /// Numeric Overflow
    Overflow = 0x0301,

    // === System (0x04XX) ===
    /// IO Error
    IoError = 0x0400,
    /// File Not Found
    FileNotFound = 0x0401,
    /// Permission Denied
    PermissionDenied = 0x0402,
    /// Timeout
    Timeout = 0x0403,

    /// Unknown Error
    Unknown = 0xFFFF,
}

impl ErrorCode {
    /// Convert u16 to ErrorCode
    pub const fn from_u16(code: u16) -> Self {
        match code {
            0x0000 => Self::Ok,
            0x0001 => Self::VmError,
            0x0002 => Self::StackOverflow,
            0x0003 => Self::StackUnderflow,
            0x0004 => Self::InvalidOpCode,
            0x0005 => Self::InvalidBytecode,
            0x0006 => Self::Halt,
            0x0007 => Self::Yield,
            0x0008 => Self::AsyncNotAvailable,

            0x0100 => Self::OutOfMemory,
            0x0101 => Self::AccessViolation,
            0x0102 => Self::OutOfBounds,

            0x0200 => Self::TypeMismatch,
            0x0201 => Self::InvalidCast,
            0x0202 => Self::UndefinedVar,

            0x0300 => Self::DivByZero,
            0x0301 => Self::Overflow,

            0x0400 => Self::IoError,
            0x0401 => Self::FileNotFound,
            0x0402 => Self::PermissionDenied,
            0x0403 => Self::Timeout,

            _ => Self::Unknown,
        }
    }

    /// Get the numeric value
    pub const fn code(self) -> u16 {
        self as u16
    }

    /// Get a human-readable message
    pub const fn message(self) -> &'static str {
        match self {
            Self::Ok => "Success",
            Self::VmError => "VM Error",
            Self::StackOverflow => "Stack Overflow",
            Self::StackUnderflow => "Stack Underflow",
            Self::InvalidOpCode => "Invalid OpCode",
            Self::InvalidBytecode => "Invalid Bytecode",
            Self::Halt => "Program Halted",
            Self::Yield => "Program Yielded",
            Self::AsyncNotAvailable => "Async Not Available",

            Self::OutOfMemory => "Out of Memory",
            Self::AccessViolation => "Access Violation",
            Self::OutOfBounds => "Index Out of Bounds",

            Self::TypeMismatch => "Type Mismatch",
            Self::InvalidCast => "Invalid Type Cast",
            Self::UndefinedVar => "Undefined Variable",

            Self::DivByZero => "Division by Zero",
            Self::Overflow => "Numeric Overflow",

            Self::IoError => "IO Error",
            Self::FileNotFound => "File Not Found",
            Self::PermissionDenied => "Permission Denied",
            Self::Timeout => "Operation Timed Out",

            Self::Unknown => "Unknown Error",
        }
    }
}

impl core::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Error 0x{:04X}: {}", self.code(), self.message())
    }
}
