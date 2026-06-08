use std::process::Command;

#[test]
fn test_cli_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_ifa"))
        .arg("--help")
        .output()
        .expect("Failed to execute ifa binary");

    assert!(output.status.success(), "CLI help should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Ifá-Lang"),
        "Help should contain language name"
    );
    assert!(stdout.contains("run"), "Help should list run command");
    assert!(stdout.contains("check"), "Help should list check command");
}

#[test]
fn test_cli_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_ifa"))
        .arg("--version")
        .output()
        .expect("Failed to execute ifa binary");

    assert!(output.status.success(), "CLI version should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ifa"), "Version should contain binary name");
}
