//! Core1 S5 mechanism witness for target-contract v7 exact SID8 transport.
//!
//! The asm nucleus is semantics-blind: it only canonicalizes the already
//! ratified 8-bit payload into a BoxedKind::Sid8 runtime value.

use wsm_os_target::{CANONICAL_T, NIL, Tag, decode_boxed, encode_fixnum, tag};

core::arch::global_asm!(include_str!("../../asm/nucleus.s"), options(att_syntax));

unsafe extern "C" {
    fn wsm_sid8_new(context: *mut core::ffi::c_void, bits: u64) -> u64;
    fn wsm_sid8_bits(context: *mut core::ffi::c_void, value: u64) -> u64;
    fn wsm_eq(context: *mut core::ffi::c_void, left: u64, right: u64) -> u64;
}

fn main() {
    let context = core::ptr::null_mut();
    let mut five = None;

    for raw in 0_u16..=255 {
        let word = unsafe { wsm_sid8_new(context, u64::from(raw)) };
        assert_eq!(tag(word), Tag::Boxed as u64, "SID8 must use the ratified Boxed tag");
        assert!(decode_boxed(word).is_some(), "SID8 must carry a non-zero runtime handle");
        assert_eq!(
            unsafe { wsm_sid8_bits(context, word) },
            u64::from(raw),
            "boxed SID8 must preserve the exact original 8 bits"
        );

        if raw == 5 {
            five = Some(word);
            assert_ne!(
                word,
                encode_fixnum(5).expect("5 is a valid fixnum"),
                "SID8 00000101 must never alias numeric fixnum 5"
            );
        }
    }

    let five = five.expect("domain includes 5");
    let five_again = unsafe { wsm_sid8_new(context, 5) };
    let six = unsafe { wsm_sid8_new(context, 6) };
    assert_eq!(
        five_again, five,
        "same exact SID8 bits must canonicalize to one target word"
    );
    assert_eq!(unsafe { wsm_eq(context, five, five_again) }, CANONICAL_T);
    assert_eq!(unsafe { wsm_eq(context, five, six) }, NIL);

    println!("sid8 boxed transport: 256/256 exact");
}
