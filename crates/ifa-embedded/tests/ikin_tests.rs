use ifa_embedded::{
    EmbeddedConfig, EmbeddedValue, EmbeddedVm, VmExit, embedded_ikin::EmbeddedIkin,
};

#[test]
fn test_ikin_consult() {
    // Bytecode: PushStr 0x00 0x01 (index 1), Halt
    let bytecode: &[u8] = &[0x0A, 0x01, 0x00, 0x55];

    // Mock flash memory: [0x04 (len), 't', 'e', 's', 't', 0x02 (len), 'o', 'k']
    // Offsets: [0, 5]
    let flash_data: &[u8] = &[4, b't', b'e', b's', b't', 2, b'o', b'k'];
    let offsets: &[u16] = &[0, 5];

    let ikin = EmbeddedIkin::new(flash_data, Some(offsets));

    let mut vm = EmbeddedVm::<256, 64>::new(EmbeddedConfig::minimal());
    vm.attach_ikin(ikin);

    let exit = vm.start(bytecode).expect("Execution failed");

    assert_eq!(exit, VmExit::Halted(EmbeddedValue::FlashString(1)));

    // Verify string resolves correctly
    let resolved = vm
        .ikin
        .unwrap()
        .consult(1)
        .expect("Failed to consult string");
    assert_eq!(resolved, "ok");
}
