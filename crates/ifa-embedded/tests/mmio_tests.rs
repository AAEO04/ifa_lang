use ifa_embedded::{EmbeddedOpCode, EmbeddedValue, EmbeddedVm, MmioBus};

/// Mock MMIO implementation
struct MockMmio {
    // Map address -> value
    // For simplicity, we just store writes to a specific address
    last_write_addr: u32,
    last_write_val: u32,
    last_write_width: u8, // 8, 16, 32
    read_val: u32,
}

impl MockMmio {
    fn new() -> Self {
        MockMmio {
            last_write_addr: 0,
            last_write_val: 0,
            last_write_width: 0,
            read_val: 0,
        }
    }
}

impl MmioBus for MockMmio {
    fn read(&mut self, addr: u32) -> u32 {
        // Return a magic value for testing
        if addr == 0x4000_0000 {
            return 0xDEAD_BEEF;
        }
        self.read_val
    }

    fn write(&mut self, addr: u32, val: u32) {
        self.last_write_addr = addr;
        self.last_write_val = val;
        self.last_write_width = 32;
    }

    fn write_u16(&mut self, addr: u32, val: u16) {
        self.last_write_addr = addr;
        self.last_write_val = val as u32;
        self.last_write_width = 16;
    }

    fn write_u8(&mut self, addr: u32, val: u8) {
        self.last_write_addr = addr;
        self.last_write_val = val as u32;
        self.last_write_width = 8;
    }
}

#[test]
fn test_mmio_write() {
    let mut vm = EmbeddedVm::<256, 64>::default();
    let mut mock = MockMmio::new();

    vm.attach_mmio(&mut mock);

    // Ref(0x4000_1000)
    // PushInt(1)
    // StoreDeref
    // Halt

    let bytecode = [
        EmbeddedOpCode::PushInt as u8,
        1,
        0,
        0,
        0, // Push 1
        EmbeddedOpCode::Ref as u8,
        0x00,
        0x10,
        0x00,
        0x40,                             // Ref 0x4000_1000 (LE)
        EmbeddedOpCode::StoreDeref as u8, // StoreDeref
        EmbeddedOpCode::Halt as u8,
    ];

    vm.start(&bytecode).unwrap();

    // Since scope issues prevent checking mock after run if VM holds borrow,
    // we use a scoped test in test_mmio_write_scoped.
}

#[test]
fn test_mmio_write_scoped() {
    let mut mock = MockMmio::new();

    {
        let mut vm = EmbeddedVm::<256, 64>::default();
        vm.attach_mmio(&mut mock);

        // Push 42, Ref 0x4000_1000, StoreDeref
        let bytecode = [
            EmbeddedOpCode::PushInt as u8,
            42,
            0,
            0,
            0, // Push 42
            EmbeddedOpCode::Ref as u8,
            0x00,
            0x10,
            0x00,
            0x40,                             // Ref 0x4000_1000
            EmbeddedOpCode::StoreDeref as u8, // StoreDeref
            EmbeddedOpCode::Halt as u8,
        ];

        vm.start(&bytecode).unwrap();
    } // vm drops here, releasing mock borrow

    assert_eq!(mock.last_write_addr, 0x4000_1000);
    assert_eq!(mock.last_write_val, 42);
    assert_eq!(mock.last_write_width, 32);
}

#[test]
fn test_mmio_read() {
    let mut mock = MockMmio::new();

    {
        let mut vm = EmbeddedVm::<256, 64>::default();
        vm.attach_mmio(&mut mock);

        // Ref(0x4000_0000) (Mock returns 0xDEAD_BEEF / 3735928559)
        // Deref
        // Halt

        let bytecode = [
            EmbeddedOpCode::Ref as u8,
            0x00,
            0x00,
            0x00,
            0x40,                        // Ref 0x4000_0000
            EmbeddedOpCode::Deref as u8, // Deref
            EmbeddedOpCode::Halt as u8,
        ];

        let res = vm.start(&bytecode).unwrap();

        if let ifa_embedded::VmExit::Halted(EmbeddedValue::Int(val)) = res {
            // 0xDEAD_BEEF as i32 is -559038737
            assert_eq!(val as u32, 0xDEAD_BEEF);
        } else {
            panic!("Expected Int result");
        }
    }
}

#[test]
fn test_mmio_sized_write() {
    let mut mock = MockMmio::new();

    // Test Store8
    {
        let mut vm = EmbeddedVm::<256, 64>::default();
        vm.attach_mmio(&mut mock);
        // Push 0x12345678, Ref 0x4000_0000, Store8
        let code = [
            EmbeddedOpCode::PushInt as u8,
            0x78,
            0x56,
            0x34,
            0x12,
            EmbeddedOpCode::Ref as u8,
            0x00,
            0x00,
            0x00,
            0x40,
            EmbeddedOpCode::Store8 as u8, // Store8
            EmbeddedOpCode::Halt as u8,
        ];
        vm.start(&code).unwrap();
    }
    assert_eq!(mock.last_write_width, 8);
    assert_eq!(mock.last_write_val, 0x78);

    // Test Store16
    {
        let mut vm = EmbeddedVm::<256, 64>::default();
        vm.attach_mmio(&mut mock);
        // Push 0x12345678, Ref 0x4000_0000, Store16
        let code = [
            EmbeddedOpCode::PushInt as u8,
            0x78,
            0x56,
            0x34,
            0x12,
            EmbeddedOpCode::Ref as u8,
            0x00,
            0x00,
            0x00,
            0x40,
            EmbeddedOpCode::Store16 as u8, // Store16
            EmbeddedOpCode::Halt as u8,
        ];
        vm.start(&code).unwrap();
    }
    assert_eq!(mock.last_write_width, 16);
    assert_eq!(mock.last_write_val, 0x5678);
}
