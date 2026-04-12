use ifa_core::bytecode::{Bytecode, OpCode};
use ifa_core::vm::IfaVM;
use ifa_types::IfaValue;
// use ifa_types::IfaError; // Not needed if we check values

#[test]
fn test_okanran_div_by_zero_recovery() {
    // 1. Setup VM
    let mut vm = IfaVM::new();
    let mut bc = Bytecode::new("test_recovery.ifa");

    // 2. Construct Bytecode manually (Simulator of `gbiyanju`)
    // Address 0: TryBegin(Jmp +8?) -> To Address 10
    // Address 5: Push 10
    // Address 14: Push 0
    // Address 23: Div -> Error!
    // Address 24: TryEnd
    // Address 25: Jump(+5) -> To End
    // Address 30: (Catch Handler) Pop (error), Push 999
    // Address XX: Halt

    // Note: Bytecode builder in tests usually easier if we had an assembler.
    // We'll write raw bytes or use OpCode instructions if helper exists.
    // Since `instruction_size` varies, constructing raw code vector is tedious.
    // Let's assume we can push bytes.

    // Helper to push instruction
    let mut code = Vec::new();

    // 0: TryBegin (offset=25 bytes to catch handler @ 30)
    // OpCode::TryBegin is 0xA0. Operand is u32 (4 bytes).
    code.push(OpCode::TryBegin as u8);
    code.extend_from_slice(&(25u32).to_le_bytes()); // 5 bytes total

    // 5: PushInt 10 (OpCode 0x08 + 8 bytes)
    code.push(OpCode::PushInt as u8);
    code.extend_from_slice(&(10i64).to_le_bytes()); // 9 bytes -> Total 14

    // 14: PushInt 0 (OpCode 0x08 + 8 bytes)
    code.push(OpCode::PushInt as u8);
    code.extend_from_slice(&(0i64).to_le_bytes()); // 9 bytes -> Total 23

    // 23: Div (OpCode 0x23) -> Total 24
    code.push(OpCode::Div as u8);

    // 24: TryEnd (OpCode 0xA1) -> Total 25
    // Should be skipped if error occurs at 23
    code.push(OpCode::TryEnd as u8);

    // 25: Jump (OpCode 0x50 + 4 bytes offset=5 to skip catch) -> Total 30
    code.push(OpCode::Jump as u8);
    code.extend_from_slice(&(10u32).to_le_bytes()); // Jump far ahead (placeholder)

    // --- CATCH HANDLER ---
    // Total so far: 0..23 (try block start), 23 is Div call.
    // If we crash at 23, we jump to catch_ip = ip + offset.
    // ip at TryBegin execution was 0 (opcode read) -> Wait.
    // VM increments ip *after* reading opcode but *before* matching.
    // For TryBegin(offset), ip points to *next* instruction?
    // VM: `let offset = self.read_u32(bytecode)?; let catch_ip = self.ip + offset;`
    // read_u32 advances ip by 4. So `self.ip` is now at instruction start + 5.
    // So catch_ip = (start+5) + offset.
    // We want catch handler at offset 30 (after the code above).
    // Current pos: 5. Need +25.
    // Let's adjust offset in TryBegin to 25.

    // 30: Catch Handler. Stack has [ErrorString].
    // Pop error
    code.push(OpCode::Pop as u8);
    // Push 999
    code.push(OpCode::PushInt as u8);
    code.extend_from_slice(&(999i64).to_le_bytes());

    // Exit
    code.push(OpCode::Halt as u8);

    bc.code = code;

    // 3. Execute
    let result = vm.execute(&bc);

    // 4. Verify
    assert!(result.is_ok(), "VM execution panic instead of recovered!");
    let val = result.unwrap();

    match val {
        IfaValue::Int(i) => assert_eq!(i, 999, "Failed to recover with correct value"),
        _ => panic!("Expected Int(999), got {:?}", val),
    }
}
