//! Proves each primitive's Type-error abort path (asm/nucleus.s)
//! actually diverges via `wsm_fail` instead of silently returning
//! garbage or segfaulting on a wrong-typed word -- same
//! subprocess-boundary pattern `dll/tests/oom_path.rs` used for the
//! OOM path before `dll/` was deleted at Phase D (a real abort's exit
//! code can't be observed by catching a signal/panic in the same
//! process, since wsm_fail's own `syscall`-based exit(97) bypasses
//! Rust's runtime entirely). Covers `wsm_car` (a non-Cons word) and
//! `wsm_closure_definition` (a non-Closure word) -- `wsm_cdr` shares
//! `wsm_car`'s exact check and `wsm_fail` jump so is not separately
//! triggered, and `wsm_closure_environment` mirrors
//! `wsm_closure_definition`'s check for the same reason.

use std::process::Command;

#[test]
fn wsm_car_on_a_fixnum_aborts_with_the_documented_exit_code() {
    let output = Command::new(env!("CARGO_BIN_EXE_harness-car-type-trigger"))
        .output()
        .expect("failed to spawn harness-car-type-trigger");

    let code = output.status.code();
    assert_eq!(
        code,
        Some(97),
        "wsm_car(Fixnum) must abort via wsm_fail's exit(97), got {code:?}; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unrecoverable condition"),
        "wsm_fail's own message must reach stderr, got: {stderr}"
    );
    assert!(
        !stderr.contains("BUG:"),
        "the trigger's own failure path fired, meaning wsm_car returned instead of aborting: {stderr}"
    );
}

#[test]
fn wsm_closure_definition_on_a_fixnum_aborts_with_the_documented_exit_code() {
    let output = Command::new(env!("CARGO_BIN_EXE_harness-closure-type-trigger"))
        .output()
        .expect("failed to spawn harness-closure-type-trigger");

    let code = output.status.code();
    assert_eq!(
        code,
        Some(97),
        "wsm_closure_definition(Fixnum) must abort via wsm_fail's exit(97), got {code:?}; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unrecoverable condition"),
        "wsm_fail's own message must reach stderr, got: {stderr}"
    );
    assert!(
        !stderr.contains("BUG:"),
        "the trigger's own failure path fired, meaning wsm_closure_definition returned instead of aborting: {stderr}"
    );
}
