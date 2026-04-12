use ifa_core::vm::IfaVM;
use ifa_types::IfaValue;
use ifa_types::OpCode;
use ifa_types::bytecode::Bytecode;

fn u16le(n: u16) -> [u8; 2] {
    n.to_le_bytes()
}

fn u32le(n: u32) -> [u8; 4] {
    n.to_le_bytes()
}

#[test]
fn makeclosure_loadupvalue_storeupvalue_roundtrip() {
    // This test constructs a minimal bytecode chunk directly:
    //
    // main:
    //   local0 = 0
    //   fn_template = PushFn(start_ip = func_ip, arity=0)
    //   closure = MakeClosure(capture local0)
    //   closure(); closure();
    //   return local0   (proves: captured local was boxed into an upvalue cell)
    //   Halt
    //
    // func:
    //   v = LoadUpvalue(0) + 1
    //   StoreUpvalue(0, v)
    //   Return(v)
    //
    // Expected return value: 2

    let mut bytecode = Bytecode::new("vm_closure_opcode_tests");
    bytecode.strings.push("inc".to_string()); // name_idx = 0

    let mut code: Vec<u8> = Vec::new();

    // main: PushInt 0
    code.push(OpCode::PushInt as u8);
    code.extend_from_slice(&0i64.to_le_bytes());

    // PushFn(name_idx=0, start_ip=?, arity=0)
    let pushfn_ip = code.len();
    code.push(OpCode::PushFn as u8);
    code.extend_from_slice(&u16le(0)); // name_idx
    code.extend_from_slice(&u32le(0)); // start_ip placeholder
    code.push(0); // arity
    code.push(0); // is_async

    // MakeClosure(capture_count=1, (kind=0 local, idx=0))
    code.push(OpCode::MakeClosure as u8);
    code.push(1); // capture_count
    code.push(0); // kind=0 (local)
    code.extend_from_slice(&u16le(0)); // idx=0

    // Call argc=0 (call #1, keep closure for later)
    code.push(OpCode::Dup as u8);
    code.push(OpCode::Call as u8);
    code.push(0);

    // Call argc=0 (call #2)
    code.push(OpCode::Swap as u8);
    code.push(OpCode::Call as u8);
    code.push(0);

    // Discard call results
    code.push(OpCode::Pop as u8);
    code.push(OpCode::Pop as u8);

    // Return local0 (must be 2 if upvalue boxing + store worked)
    code.push(OpCode::LoadLocal as u8);
    code.extend_from_slice(&u16le(0));

    // Halt
    code.push(OpCode::Halt as u8);

    // Function starts here.
    let func_ip = code.len() as u32;

    // Patch PushFn start_ip
    let start_ip_offset = pushfn_ip + 1 + 2; // opcode + name_idx
    code[start_ip_offset..start_ip_offset + 4].copy_from_slice(&u32le(func_ip));

    // func: LoadUpvalue 0
    code.push(OpCode::LoadUpvalue as u8);
    code.extend_from_slice(&u16le(0));
    // PushInt 1
    code.push(OpCode::PushInt as u8);
    code.extend_from_slice(&1i64.to_le_bytes());
    // Add
    code.push(OpCode::Add as u8);
    // Dup (keep one copy for return)
    code.push(OpCode::Dup as u8);
    // StoreUpvalue 0
    code.push(OpCode::StoreUpvalue as u8);
    code.extend_from_slice(&u16le(0));
    // Return
    code.push(OpCode::Return as u8);

    bytecode.code = code;

    let mut vm = IfaVM::new();
    let result = vm.execute(&bytecode).expect("vm execute failed");
    assert_eq!(result, IfaValue::Int(2));
}
