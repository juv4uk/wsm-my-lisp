use std::process::Command;

fn assert_fails_closed(mode: &str) {
    let output = Command::new(env!("CARGO_BIN_EXE_harness-sid8-trigger"))
        .arg(mode)
        .output()
        .unwrap_or_else(|error| panic!("run SID8 trigger {mode:?}: {error}"));

    assert_eq!(
        output.status.code(),
        Some(97),
        "{mode}: rejected SID8 transport must terminate via wsm_fail; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("unrecoverable condition"),
        "{mode}: wsm_fail evidence missing"
    );
}

#[test]
fn out_of_range_bits_are_not_truncated() {
    assert_fails_closed("out-of-range");
}

#[test]
fn ordinary_fixnum_is_not_coerced_to_sid8() {
    assert_fails_closed("fixnum");
}

#[test]
fn host_shaped_boxed_handle_without_runtime_entry_is_not_accepted() {
    assert_fails_closed("foreign-boxed");
}
