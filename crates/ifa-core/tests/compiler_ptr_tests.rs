use ifa_core::bytecode::OpCode;
use ifa_core::compiler::compile;

#[test]
fn test_compile_address_of() {
    // &0x1234
    // Must be a statement: ayanmo x = &4660;
    let bytecode = compile("ayanmo x = &4660;").unwrap(); // 4660 = 0x1234

    // Check for Ref(4660) opcode.
    // It will be wrapped in StoreLocal code.
    // Scan the code vector.

    // Pattern: [..., Ref(0xA0), 0x34, 0x12, 0, 0, ...]
    // 4660 = 0x1234 -> Little Endian: 34 12 00 00

    let mut found = false;
    for i in 0..bytecode.code.len() {
        if bytecode.code[i] == OpCode::Ref as u8 {
            // Check payload
            if i + 4 < bytecode.code.len() {
                let b1 = bytecode.code[i + 1];
                let b2 = bytecode.code[i + 2];
                // 4660 = 0x1234
                if b1 == 0x34 && b2 == 0x12 {
                    found = true;
                    break;
                }
            }
        }
    }
    assert!(found, "Did not find Ref(4660) instruction");
}

#[test]
fn test_compile_deref_read() {
    // *(&4660)
    let bytecode = compile("ayanmo y = *(&4660);").unwrap();
    // Scan for Deref (Load8)
    assert!(bytecode.code.contains(&(OpCode::Load8 as u8)));
}

#[test]
fn test_compile_deref_write() {
    // *(&4660) = 1;
    let bytecode = compile("*(&4660) = 1;").unwrap();
    // 1. Push 1 (Val)
    // 2. Ref 4660 (Addr)
    // 3. StoreDeref (Pop addr, Pop val)

    // Scan for StoreDeref (Store8)
    assert!(bytecode.code.contains(&(OpCode::Store8 as u8)));
}
