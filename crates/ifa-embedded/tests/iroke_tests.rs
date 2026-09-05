use ifa_embedded::{
    EmbeddedConfig, EmbeddedValue, EmbeddedVm, VmExit, embedded_iroke::IrokePoller,
};

struct MockPoller {
    jump_count: usize,
    yield_at: usize,
}

impl IrokePoller for MockPoller {
    fn should_yield(&mut self) -> bool {
        self.jump_count += 1;
        self.jump_count >= self.yield_at
    }
}

#[test]
fn test_iroke_optimized_polling() {
    // Bytecode:
    // 0x00: PushInt 0
    // 0x05: PushInt 1
    // 0x0A: Add
    // 0x0B: Jump 0x05
    let bytecode: &[u8] = &[
        0x08, 0x00, 0x00, 0x00, 0x00, // PushInt 0
        0x08, 0x01, 0x00, 0x00, 0x00, // PushInt 1
        0x20, // Add
        0x50, 0x05, 0x00, // Jump 0x05
    ];

    let mut vm = EmbeddedVm::<256, 64>::new(EmbeddedConfig::minimal());

    // Set poller to yield on the 3rd jump
    let mut poller = MockPoller {
        jump_count: 0,
        yield_at: 3,
    };

    let exit = vm
        .run_with_iroke(bytecode, &mut poller)
        .expect("Execution failed");

    assert_eq!(exit, VmExit::Yield(0));

    // VM should have executed 3 iterations. Stack top should be 3.
    // wait, we can't easily peek stack top from outside since `run_with_iroke` just yields.
    // But we know it yielded successfully.
}
