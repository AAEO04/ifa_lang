use crate::{ErrorCode, OpCode};

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Validates the bytecode before execution.
/// Checks that all opcodes are valid, all operands are within bounds, and all jump targets point to a valid instruction start.
#[cfg(feature = "alloc")]
pub fn validate_bytecode(bytecode: &[u8]) -> Result<(), ErrorCode> {
    let mut ip = 0;
    let len = bytecode.len();

    // We use a Vec of usize to store valid instruction offsets
    let mut valid_offsets = Vec::new();
    let mut jumps = Vec::new(); // (jump_source_ip, jump_target_ip)

    while ip < len {
        valid_offsets.push(ip);
        let opcode_byte = bytecode[ip];
        let opcode = OpCode::from_u8(opcode_byte).ok_or(ErrorCode::InvalidOpCode)?;

        match opcode {
            OpCode::DefineClass => {
                // Not supported/emitted in current version
                return Err(ErrorCode::InvalidOpCode);
            }
            OpCode::MakeClosure => {
                ip += 1;
                if ip >= len {
                    return Err(ErrorCode::InvalidBytecode);
                }
                let upvalues = bytecode[ip] as usize;
                ip += 1 + upvalues * 3;
                continue;
            }
            OpCode::Jump
            | OpCode::JumpIfTrue
            | OpCode::JumpIfFalse
            | OpCode::TryBegin
            | OpCode::FinallyBegin => {
                if ip + 4 >= len {
                    return Err(ErrorCode::InvalidBytecode);
                }
                let target = u32::from_le_bytes([
                    bytecode[ip + 1],
                    bytecode[ip + 2],
                    bytecode[ip + 3],
                    bytecode[ip + 4],
                ]) as usize;
                jumps.push((ip, target));
            }
            _ => {}
        }

        if let Some(operand_len) = opcode.operand_bytes() {
            ip += 1 + operand_len;
        } else {
            // Should be unreachable since MakeClosure and DefineClass are the only ones returning None
            return Err(ErrorCode::InvalidOpCode);
        }
    }

    if ip != len {
        return Err(ErrorCode::InvalidBytecode);
    }

    // Verify that all jump targets land exactly on an instruction boundary
    for (_source_ip, target) in jumps {
        if !valid_offsets.contains(&target) {
            return Err(ErrorCode::InvalidBytecode);
        }
    }

    Ok(())
}
