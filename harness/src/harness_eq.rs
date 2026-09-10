//! Direct primitive test for wsm_eq (no existing CML-generated fixture calls
//! it in isolation, so this calls it directly rather than through a
//! wsm_entry). Fixnums and truth values come from the ratified target
//! contract; the witness does not duplicate their bit encodings locally.

use wsm_os_target::{encode_fixnum, CANONICAL_T, NIL};

core::arch::global_asm!(include_str!("../../asm/nucleus.s"), options(att_syntax));

unsafe extern "C" {
    fn wsm_eq(context: *mut core::ffi::c_void, left: u64, right: u64) -> u64;
}

fn main() {
    let ctx = core::ptr::null_mut();
    let forty_one = encode_fixnum(41).expect("41 must fit target fixnum range");
    let forty_two = encode_fixnum(42).expect("42 must fit target fixnum range");
    let same = unsafe { wsm_eq(ctx, forty_one, forty_one) };
    let diff = unsafe { wsm_eq(ctx, forty_one, forty_two) };
    assert_eq!(same, CANONICAL_T, "equal fixnums must yield canonical Symbol(\"t\")");
    assert_eq!(diff, NIL, "unequal fixnums must yield canonical ()");
    println!("wsm_eq: (eq 41 41) -> t, (eq 41 42) -> ()  [both correct]");
}
