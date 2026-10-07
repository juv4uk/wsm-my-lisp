//! Executable mechanism witness for draft target-contract v8 PredicateBit carrier.
//!
//! This witness checks representation only. It does not assign predicate meaning
//! to bit 0 or bit 1; SENS owns that law.

use wsm_os_target::{
    decode_boxed, encode_fixnum, tag, BoxedKind, BoxedPredicateBit, Tag, CANONICAL_T, NIL, TRUE,
};

core::arch::global_asm!(include_str!("../../asm/nucleus.s"), options(att_syntax));

unsafe extern "C" {
    fn wsm_predicate_bit_0(context: *mut core::ffi::c_void) -> u64;
    fn wsm_predicate_bit_1(context: *mut core::ffi::c_void) -> u64;
    fn wsm_predicate_bit_bits(context: *mut core::ffi::c_void, value: u64) -> u64;
}

fn main() {
    let context = core::ptr::null_mut();

    let bit0 = unsafe { wsm_predicate_bit_0(context) };
    let bit1 = unsafe { wsm_predicate_bit_1(context) };
    let bit0_again = unsafe { wsm_predicate_bit_0(context) };
    let bit1_again = unsafe { wsm_predicate_bit_1(context) };

    assert_eq!(tag(bit0), Tag::Boxed as u64);
    assert_eq!(tag(bit1), Tag::Boxed as u64);
    assert!(decode_boxed(bit0).is_some());
    assert!(decode_boxed(bit1).is_some());
    assert_ne!(bit0, bit1, "exact bit identities must be distinct");
    assert_eq!(bit0, bit0_again, "bit 0 must be a canonical singleton");
    assert_eq!(bit1, bit1_again, "bit 1 must be a canonical singleton");

    assert_eq!(unsafe { wsm_predicate_bit_bits(context, bit0) }, 0);
    assert_eq!(unsafe { wsm_predicate_bit_bits(context, bit1) }, 1);

    for forbidden in [
        NIL,
        TRUE,
        CANONICAL_T,
        encode_fixnum(0).expect("0 fits fixnum"),
        encode_fixnum(1).expect("1 fits fixnum"),
    ] {
        assert_ne!(bit0, forbidden, "PredicateBit(0) must not alias legacy/numeric truth carriers");
        assert_ne!(bit1, forbidden, "PredicateBit(1) must not alias legacy/numeric truth carriers");
    }

    let contract0 = BoxedPredicateBit::new(0).expect("target contract admits bit 0");
    let contract1 = BoxedPredicateBit::new(1).expect("target contract admits bit 1");
    assert_eq!(contract0.kind, BoxedKind::PredicateBit);
    assert_eq!(contract1.kind, BoxedKind::PredicateBit);
    assert_eq!(contract0.exact_bit(), 0);
    assert_eq!(contract1.exact_bit(), 1);
    assert_eq!(BoxedPredicateBit::new(2), None);

    println!("predicate-bit carrier: boxed singletons 0/1 exact");
}
