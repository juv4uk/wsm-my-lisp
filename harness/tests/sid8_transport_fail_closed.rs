use std::process::Command;

fn assert_rejected(executable: &str) {
    let output = Command::new(executable)
        .output()
        .expect("run SID8 negative witness");
    assert_eq!(output.status.code(), Some(97));
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("wsm-my-lisp asm nucleus: unrecoverable condition"));
}

#[test]
fn non_boxed_sid8_input_fails_closed() {
    assert_rejected(env!("CARGO_BIN_EXE_harness-sid8-type-trigger"));
}

#[test]
fn forged_sid8_boxed_handle_fails_closed() {
    assert_rejected(env!("CARGO_BIN_EXE_harness-sid8-abi-trigger"));
}
