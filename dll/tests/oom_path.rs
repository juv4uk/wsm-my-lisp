//! Exercises the real OOM path (asm/nucleus-win64.s's wsm_cons_oom ->
//! dll/src/lib.rs's wsm_fail_win64) end to end, via a subprocess -- this
//! can't be a plain #[test] in lib.rs because wsm_fail_win64 calls
//! std::process::exit(97), which would kill the whole in-process test
//! runner rather than just the one test.

use std::process::Command;

#[test]
fn arena_exhaustion_exits_97_with_diagnostic_on_stderr() {
    let output = Command::new(env!("CARGO_BIN_EXE_oom-trigger"))
        .output()
        .expect("failed to run oom-trigger subprocess");

    assert_eq!(
        output.status.code(),
        Some(97),
        "expected wsm_fail_win64's exit(97); got {:?}, stdout={:?}, stderr={:?}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unrecoverable condition"),
        "expected wsm_fail_win64's own diagnostic message on stderr, got: {stderr}"
    );
    // ErrorCode::OutOfMemory = 1, per asm/nucleus-win64.s's wsm_cons_oom
    // (movl $1, %ecx before calling wsm_fail_win64).
    assert!(
        stderr.contains("code=1"),
        "expected OOM's ErrorCode=1 in the diagnostic, got: {stderr}"
    );

    // No stdout output expected: the "did not exhaust" println in
    // oom_trigger.rs must never run if the arena is really 4096 bytes /
    // 256 cells as documented.
    assert!(
        output.stdout.is_empty(),
        "oom-trigger printed to stdout, meaning it did NOT hit OOM as expected: {:?}",
        String::from_utf8_lossy(&output.stdout)
    );
}
