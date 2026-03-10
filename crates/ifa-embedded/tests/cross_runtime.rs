use ifa_core::{Bytecode, OpCode};
use ifa_embedded::{EmbeddedConfig, EmbeddedOpCode, EmbeddedValue, EmbeddedVm, VmExit};

/// Transpile standard 64-bit bytecode to 32-bit embedded bytecode
/// This is a simplified transpiler for testing compatibility.
/// Real transpiler would handle label resolution and complex control flow.
fn transpile_for_embedded(bc: &Bytecode) -> Vec<u8> {
    let mut out = Vec::new();
    let mut ip = 0;

    // In a real transpiler, we would do two passes to fix jump offsets.
    // For this test, we assume linear code or manual offset fixups if needed.

    while ip < bc.code.len() {
        let op_byte = bc.code[ip];
        ip += 1;

        let op = OpCode::from_u8(op_byte)
            .unwrap_or_else(|| panic!("Unknown std opcode: 0x{:02X}", op_byte));

        match op {
            OpCode::PushInt => {
                // Std: PushInt (0x08) + 8 bytes (i64)
                let val_bytes: [u8; 8] = bc.code[ip..ip + 8].try_into().unwrap();
                let val = i64::from_le_bytes(val_bytes);
                ip += 8;

                // Embedded: PushInt (0x01) + 4 bytes (i32)
                out.push(EmbeddedOpCode::PushInt as u8);
                out.extend_from_slice(&(val as i32).to_le_bytes());
            }
            OpCode::Add => {
                // Std: Add (0x20)
                // Embedded: Add (0x20)
                out.push(EmbeddedOpCode::Add as u8);
            }
            OpCode::Sub => {
                // Std: Sub (0x21)
                // Embedded: Sub (0x21)
                out.push(EmbeddedOpCode::Sub as u8);
            }
            OpCode::Mul => {
                out.push(EmbeddedOpCode::Mul as u8);
            }
            OpCode::Div => {
                out.push(EmbeddedOpCode::Div as u8);
            }
            OpCode::Halt => {
                // Std: Halt (0x55)
                // Embedded: Halt (0xFF)
                out.push(EmbeddedOpCode::Halt as u8);
            }
            _ => {
                // Determine implicit operand size to skip
                if let Some(size) = op.operand_bytes() {
                    println!("Skipping unsupported opcode {:?} with {} bytes", op, size);
                    ip += size;
                } else {
                    panic!("Cannot skip variable length opcode {:?}", op);
                }
            }
        }
    }

    out
}

#[test]
fn test_cross_runtime_arithmetic() {
    // 1. Create Standard Bytecode (High Level)
    // Program: 10 + 20 * 2 = 50
    // Stack: Push 10, Push 20, Push 2, Mul (40), Add (50), Halt
    let mut bc = Bytecode::new("test.ifa");

    // Push 10
    bc.code.push(OpCode::PushInt as u8);
    bc.code.extend_from_slice(&10i64.to_le_bytes());

    // Push 20
    bc.code.push(OpCode::PushInt as u8);
    bc.code.extend_from_slice(&20i64.to_le_bytes());

    // Push 2
    bc.code.push(OpCode::PushInt as u8);
    bc.code.extend_from_slice(&2i64.to_le_bytes());

    // Mul
    bc.code.push(OpCode::Mul as u8);

    // Add
    bc.code.push(OpCode::Add as u8);

    // Halt
    bc.code.push(OpCode::Halt as u8);

    // 2. Transpile to Embedded Bytecode (Low Level)
    let embedded_code = transpile_for_embedded(&bc);

    // 3. Execute on Embedded VM
    let mut vm = EmbeddedVm::<1024, 256>::new(EmbeddedConfig::standard());
    match vm.start(&embedded_code) {
        Ok(VmExit::Halted(val)) => {
            assert_eq!(val, EmbeddedValue::Int(50));
        }
        Ok(other) => panic!("VM did not halt correctly: {:?}", other),
        Err(e) => panic!("VM Error: {:?}", e),
    }
}
