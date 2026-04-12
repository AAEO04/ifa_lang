use ifa_embedded::{EmbeddedValue, EmbeddedVm};

#[test]
fn test_embedded_ptr_ops() {
    let mut vm = EmbeddedVm::<256, 64>::default();

    // 1. Push Value (42) -> Stack: [42]
    // 2. Push Address (5) using Ref (A0) -> Stack: [42, Ptr(5)]
    // 3. StoreDeref (A2) -> Opon[5] = 42, Stack: []
    // 4. Push Address (5) using Ref (A0) -> Stack: [Ptr(5)]
    // 5. Deref (A1) -> Stack: [42]
    // 6. Halt

    let bytecode = [
        0x01, 42, 0, 0, 0, // PushInt(42)
        0xA0, 5, 0, 0, 0,    // Ref(5) -> Ptr(5)
        0xA2, // StoreDeref
        0xA0, 5, 0, 0, 0,    // Ref(5) -> Ptr(5)
        0xA1, // Deref
        0xFF, // Halt
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
        0x01, 1, 0, 0, 0, 0xA0, 44, 1, 0, 0, // 300 (0x12C = 44 01 00 00)
        0xA2, 0xFF,
    ];

    let result = vm.start(&bytecode);
    assert!(result.is_err());
}
