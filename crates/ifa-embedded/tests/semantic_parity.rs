use ifa_embedded::{EmbeddedConfig, EmbeddedOpCode, EmbeddedValue, EmbeddedVm, VmExit};
use ifa_vm::{Bytecode, IfaVM, IfaValue, OpCode};

fn transpile_for_embedded(bc: &Bytecode) -> Vec<u8> {
    let mut out = Vec::new();
    let mut ip = 0;

    while ip < bc.code.len() {
        let op_byte = bc.code[ip];
        ip += 1;

        let op = OpCode::from_u8(op_byte)
            .unwrap_or_else(|| panic!("Unknown std opcode: {op_byte:#04x}"));

        match op {
            OpCode::PushInt => {
                let val_bytes: [u8; 8] = bc.code[ip..ip + 8].try_into().unwrap();
                let val = i64::from_le_bytes(val_bytes);
                ip += 8;
                out.push(EmbeddedOpCode::PushInt as u8);
                out.extend_from_slice(&(val as i32).to_le_bytes());
            }
            OpCode::Add => out.push(EmbeddedOpCode::Add as u8),
            OpCode::Sub => out.push(EmbeddedOpCode::Sub as u8),
            OpCode::Mul => out.push(EmbeddedOpCode::Mul as u8),
            OpCode::Div => out.push(EmbeddedOpCode::Div as u8),
            OpCode::Halt => out.push(EmbeddedOpCode::Halt as u8),
            _ => {
                if let Some(size) = op.operand_bytes() {
                    ip += size;
                } else {
                    panic!("Cannot skip variable length opcode {:?}", op);
                }
            }
        }
    }

    out
}

fn run_std(code: &[u8]) -> IfaValue {
    let mut vm = IfaVM::new();
    let mut bc = Bytecode::new("semantic_parity");
    bc.code = code.to_vec();
    vm.execute(&bc).expect("standard VM should succeed")
}

fn run_embedded(code: &[u8]) -> EmbeddedValue {
    let mut vm = EmbeddedVm::<256, 64>::new(EmbeddedConfig::standard());
    match vm.start(code).expect("embedded VM should succeed") {
        VmExit::Halted(value) => value,
        other => panic!("unexpected embedded exit: {:?}", other),
    }
}

fn build_std_program(lhs: i64, rhs: i64, op: OpCode) -> Bytecode {
    let mut bc = Bytecode::new("semantic_parity");
    bc.code.push(OpCode::PushInt as u8);
    bc.code.extend_from_slice(&lhs.to_le_bytes());
    bc.code.push(OpCode::PushInt as u8);
    bc.code.extend_from_slice(&rhs.to_le_bytes());
    bc.code.push(op as u8);
    bc.code.push(OpCode::Halt as u8);
    bc
}

#[test]
fn standard_and_embedded_agree_on_basic_arithmetic() {
    let cases = [
        (10, 20, OpCode::Add, 30),
        (42, 17, OpCode::Sub, 25),
        (6, 7, OpCode::Mul, 42),
        (84, 7, OpCode::Div, 12),
    ];

    for (lhs, rhs, op, expected) in cases {
        let bc = build_std_program(lhs, rhs, op);
        let embedded = transpile_for_embedded(&bc);

        let std_result = run_std(&bc.code);
        let embedded_result = run_embedded(&embedded);

        assert_eq!(std_result, IfaValue::Int(expected));
        assert_eq!(embedded_result, EmbeddedValue::Int(expected as _));
    }
}
