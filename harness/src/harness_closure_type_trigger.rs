//! Deliberately triggers `wsm_closure_definition`'s Type-error abort
//! path -- `harness-closure` only ever exercised the happy path (a
//! real closure word from `wsm_closure_new`). Same subprocess pattern
//! as `harness-car-type-trigger`. The AbiViolation branch (a
//! Closure-tagged word pointing outside the closure arena or
//! misaligned) is NOT covered here: constructing such a word safely
//! from Rust would require fabricating a fake pointer, which is a
//! different, riskier kind of test than calling the primitive with an
//! ordinary wrong-typed value -- left as a named gap, not silently
//! skipped.

use wsm_os_target::encode_fixnum;

core::arch::global_asm!(include_str!("../../asm/nucleus.s"), options(att_syntax));

unsafe extern "C" {
    fn wsm_closure_definition(context: *mut core::ffi::c_void, closure: u64) -> u32;
}

fn main() {
    let ctx = core::ptr::null_mut();
    let five = encode_fixnum(5).expect("5 must fit target fixnum range");
    let _ = unsafe { wsm_closure_definition(ctx, five) };
    eprintln!("BUG: wsm_closure_definition(5) returned instead of aborting");
    std::process::exit(1);
}
