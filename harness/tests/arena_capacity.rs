use std::process::Command;

#[test]
fn core1_full_source_crosses_old_256_cell_limit() {
    let output = Command::new(env!("CARGO_BIN_EXE_harness-arena-capacity"))
        .arg("success-503")
        .output()
        .expect("run 503-cell arena witness");
    assert!(
        output.status.success(),
        "503-cell Core1 source transport must fit bounded S5 arena; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn bounded_arena_still_fails_closed_beyond_2048_cells() {
    let output = Command::new(env!("CARGO_BIN_EXE_harness-arena-capacity"))
        .arg("overflow")
        .output()
        .expect("run bounded arena overflow witness");
    assert_eq!(output.status.code(), Some(97));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("unrecoverable condition"),
        "overflow must terminate through wsm_fail"
    );
}
