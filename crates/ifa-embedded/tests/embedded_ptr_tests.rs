use ifa_embedded::{EmbeddedOpCode, EmbeddedValue, EmbeddedVm};

#[test]
fn test_embedded_ptr_ops() {
    let mut vm = EmbeddedVm::<256, 64>::default();

    // 1. Push Value (42) -> Stack: [42]
    // 2. Push Address (5) using Ref -> Stack: [42, Ptr(5)]
    // 3. StoreDeref -> Opon[5] = 42, Stack: []
    // 4. Push Address (5) using Ref -> Stack: [Ptr(5)]
    // 5. Deref -> Stack: [42]
    // 6. Halt

    let bytecode = [
        EmbeddedOpCode::PushInt as u8,
        42,
        0,
        0,
        0, // PushInt(42)
        EmbeddedOpCode::Ref as u8,
        5,
        0,
        0,
        0,                                // Ref(5) -> Ptr(5)
        EmbeddedOpCode::StoreDeref as u8, // StoreDeref
        EmbeddedOpCode::Ref as u8,
        5,
        0,
        0,
        0,                           // Ref(5) -> Ptr(5)
        EmbeddedOpCode::Deref as u8, // Deref
        EmbeddedOpCode::Halt as u8,  // Halt
    ];

    let result = vm.start(&bytecode).unwrap();
    assert_eq!(result, ifa_embedded::VmExit::Halted(EmbeddedValue::Int(42)));
}

#[test]
fn test_ptr_memory_bounds() {
    let mut vm = EmbeddedVm::<256, 64>::default();
    // Default opon size is 256. Try to write to 300.

    // 1. Push Value (1)
    // 2. Push Addr (300)
    // 3. StoreDeref

    let bytecode = [
        EmbeddedOpCode::PushInt as u8,
        1,
        0,
        0,
        0,
        EmbeddedOpCode::Ref as u8,
        44,
        1,
        0,
        0, // 300 (0x12C = 44 01 00 00)
        EmbeddedOpCode::StoreDeref as u8,
        EmbeddedOpCode::Halt as u8,
    ];

    let result = vm.start(&bytecode);
    assert!(result.is_err());
}
