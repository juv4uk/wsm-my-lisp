//! Physical bounded-cons-arena capacity witness for Core1 S5.
//! This exercises only the target/runtime allocation mechanism; it carries no
//! Lisp semantic rule. success-503 crosses the former 256-cell ceiling.
//! overflow proves the new 2048-cell bound still fails closed via wsm_fail.

core::arch::global_asm!(include_str!("../../asm/nucleus.s"), options(att_syntax));

unsafe extern "C" {
    fn wsm_cons(context: *mut core::ffi::c_void, car: u64, cdr: u64) -> u64;
}

fn allocate(count: usize) {
    let ctx = core::ptr::null_mut();
    let mut tail = wsm_os_target::NIL;
    for _ in 0..count {
        tail = unsafe { wsm_cons(ctx, wsm_os_target::NIL, tail) };
    }
    assert_ne!(tail, wsm_os_target::NIL);
}

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some("success-503") => allocate(503),
        Some("overflow") => allocate(2049),
        other => panic!("expected success-503|overflow, got {other:?}"),
    }
}
