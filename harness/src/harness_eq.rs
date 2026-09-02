//! Direct primitive test for wsm_eq (no existing CML-generated fixture calls
//! it in isolation, so this calls it directly rather than through a
//! wsm_entry). Encodes two fixnums the same way wsm-os-target::encode_fixnum
//! does ((value << 3) | Tag::Fixnum(3)) and checks eq/not-eq both ways.

core::arch::global_asm!(include_str!("../../asm/nucleus.s"), options(att_syntax));

unsafe extern "C" {
    fn wsm_eq(context: *mut core::ffi::c_void, left: u64, right: u64) -> u64;
}

fn encode_fixnum(value: i64) -> u64 {
    ((value as u64) << 3) | 3
}

fn main() {
    let ctx = core::ptr::null_mut();
    let same = unsafe { wsm_eq(ctx, encode_fixnum(41), encode_fixnum(41)) };
    let diff = unsafe { wsm_eq(ctx, encode_fixnum(41), encode_fixnum(42)) };
    assert_eq!(same, 2, "equal fixnums must yield Tag::True (2)");
    assert_eq!(diff, 1, "unequal fixnums must yield Tag::Nil (1)");
    println!("wsm_eq: (eq 41 41) -> t, (eq 41 42) -> ()  [both correct]");
}
