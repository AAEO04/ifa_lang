use ifa_bytecode::embedded::EmbeddedOpCode;
use ifa_embedded::{EmbeddedValue, EmbeddedVm, VmExit};

#[test]
fn test_yield_resume() {
    let mut vm = EmbeddedVm::<256, 64>::default();

    // Bytecode:
    // 1. Push 10
    // 2. Yield (1000us)
    // 3. Push 20
    // 4. Add
    // 5. Halt

    let bytecode = [
        EmbeddedOpCode::PushInt as u8,
        10,
        0,
        0,
        0, // PushInt(10)
        EmbeddedOpCode::Yield as u8,
        0xE8,
        0x03,
        0x00,
        0x00, // Yield(1000) (0x3E8 = 1000)
        EmbeddedOpCode::PushInt as u8,
        20,
        0,
        0,
        0,                          // PushInt(20)
        EmbeddedOpCode::Add as u8,  // Add
        EmbeddedOpCode::Halt as u8, // Halt
    ];

    // Start
    let result = vm.start(&bytecode).unwrap();

    // Expect Yield(1000)
    if let VmExit::Yield(duration) = result {
        assert_eq!(duration, 1000);
    } else {
        panic!("Expected Yield, got {:?}", result);
    }

    // Resume
    let result = vm.resume(&bytecode).unwrap();

    // Expect Halted(30)
    if let VmExit::Halted(val) = result {
        assert_eq!(val, EmbeddedValue::Int(30));
    } else {
        panic!("Expected Halt, got {:?}", result);
    }
}

#[test]
fn test_yield_loop() {
    let mut vm = EmbeddedVm::<256, 64>::default();

    // Simple counter loop with yield
    // i = 0
    // loop:
    //   i = i + 1
    //   yield(0)
    //   if i < 3 jump loop
    // halt

    // Locals: [0] = i

    let bytecode = [
        // Init i = 0
        EmbeddedOpCode::PushInt as u8,
        0,
        0,
        0,
        0,
        EmbeddedOpCode::StoreLocal as u8,
        0, // StoreLocal(0)
        // Loop Start (Offset 7)
        // Load i
        EmbeddedOpCode::LoadLocal as u8,
        0,
        // Push 1
        EmbeddedOpCode::PushInt as u8,
        1,
        0,
        0,
        0,
        // Add
        EmbeddedOpCode::Add as u8,
        // Store i
        EmbeddedOpCode::StoreLocal as u8,
        0,
        // Yield(0)
        EmbeddedOpCode::Yield as u8,
        0,
        0,
        0,
        0,
        // Load i
        EmbeddedOpCode::LoadLocal as u8,
        0,
        // Push 3
        EmbeddedOpCode::PushInt as u8,
        3,
        0,
        0,
        0,
        // Lt (i < 3) -> True
        EmbeddedOpCode::Lt as u8,
        // Not -> False
        EmbeddedOpCode::Not as u8,
        // JumpIfFalse to Loop Start (7)
        EmbeddedOpCode::JumpIfFalse as u8,
        7,
        0,
        // Halt (Offset 37)
        EmbeddedOpCode::Halt as u8,
    ];

    let mut steps = 0;
    let mut res = vm.start(&bytecode).unwrap();

    while let VmExit::Yield(_) = res {
        steps += 1;
        if steps > 10 {
            panic!("Infinite loop");
        }
        res = vm.resume(&bytecode).unwrap();
    }

    // Should yield for i=1, i=2, i=3.
    // i=0 -> i=1 -> Yield -> i<3 (1<3 T) -> Jump
    // i=1 -> i=2 -> Yield -> i<3 (2<3 T) -> Jump
    // i=2 -> i=3 -> Yield -> i<3 (3<3 F) -> Halt

    assert_eq!(steps, 3);
    if let VmExit::Halted(_) = res {
        // success
    } else {
        panic!("Expected Halt");
    }
}
