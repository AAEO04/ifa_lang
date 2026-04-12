use ifa_embedded::{EmbeddedConfig, EmbeddedValue, EmbeddedVm, VmExit};

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
        0x01, 10, 0, 0, 0, // PushInt(10)
        0xF0, 0xE8, 0x03, 0x00, 0x00, // Yield(1000) (0x3E8 = 1000)
        0x01, 20, 0, 0, 0,    // PushInt(20)
        0x20, // Add
        0xFF, // Halt
    ];

    // Start
    let result = vm.start(&bytecode).unwrap();

    // Expect Yield(1000)
    if let VmExit::Yield(duration) = result {
        assert_eq!(duration, 1000);
        // Verify stack preserved
        // We can't easily peek into private fields unless we use a run-loop or inspect via public API if available.
        // But we can just resume and check result.
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
        0x01, 0, 0, 0, 0, 0x51, 0, // StoreLocal(0)
        // Loop Start (Offset 7)
        // Load i
        0x50, 0, // Push 1
        0x01, 1, 0, 0, 0,    // Add
        0x20, // Store i
        0x51, 0, // Yield(0)
        0xF0, 0, 0, 0, 0, // Load i
        0x50, 0, // Push 3
        0x01, 3, 0, 0, 0, // Lt
        0x32,
        // JumpIfFalse (Exit) -> Offset ?
        // We want: if i < 3 (True), Jump Loop.
        // JumpIfFalse jumps if False. So if i >= 3, jump exit.
        // Wait, standard `while` logic: while condition { body }.
        // Loop: Body... Evaluate Condition... If True Jump Loop.
        // Using JumpIfFalse: If !True Jump Exit. Else Fallthrough (Start loop again? No, fallthrough goes down).

        // Let's use simple Jump for loop back, and JumpIfFalse to break?
        // Let's invert: if i < 3, Jump Loop.
        // 0x61 (JumpIfFalse) -> if False jump to "Skip Jump Loop".
        // But we want "If True Jump Loop".
        // We don't have JumpIfTrue opcode?
        // We have Jump (0x60) and JumpIfFalse (0x61).

        // Pattern:
        //   Condition
        //   JumpIfFalse Exit
        //   Body
        //   Jump Start
        // Exit: ...

        // No, I entered loop already.
        // Pattern for "do-while":
        //   Body
        //   Yield
        //   Condition
        //   Not (so False means True)
        //   JumpIfFalse Start
        //   Wait, JumpIfFalse jumps if top is False.
        //   If (i < 3) is True. Not -> False. JumpIfFalse -> Jump!
        //   Yes.

        // Load i
        0x50, 0, // Push 3
        0x01, 3, 0, 0, 0,    // Lt (i < 3) -> True
        0x32, // Not -> False
        0x42, // JumpIfFalse to Loop Start (7)
        0x61, 7, 0, // Halt (Offset 37)
        0xFF,
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
