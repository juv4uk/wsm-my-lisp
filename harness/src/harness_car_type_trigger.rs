//! Deliberately triggers `wsm_car`'s new Type-error abort path (a
//! subprocess target for `harness/tests/type_error_path.rs`, same
//! pattern the old `dll/src/bin/oom_trigger.rs` used for the OOM path
//! before `dll/` was deleted at Phase D). Not meant to be run directly
//! as a "witness" -- it deliberately calls `wsm_car` on a `Fixnum`
//! word, which must abort the process via `wsm_fail`, not return.

use wsm_os_target::encode_fixnum;

core::arch::global_asm!(include_str!("../../asm/nucleus.s"), options(att_syntax));

unsafe extern "C" {
    fn wsm_car(context: *mut core::ffi::c_void, pair: u64) -> u64;
}

fn main() {
    let ctx = core::ptr::null_mut();
    let five = encode_fixnum(5).expect("5 must fit target fixnum range");
    let _ = unsafe { wsm_car(ctx, five) };
    // Unreachable if wsm_car's Type-error path correctly diverges via
    // wsm_fail's exit(97) -- reaching here at all is itself the bug.
    eprintln!("BUG: wsm_car(5) returned instead of aborting");
    std::process::exit(1);
}
