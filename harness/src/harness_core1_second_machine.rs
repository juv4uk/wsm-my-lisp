//! Native lower-bound witness for the ALREADY IMPLEMENTED SENS Core1 C1-SECOND.
//!
//! Source-side exact D2/D3 and physical T5 bytes are proven separately by
//! SENS PR #4472 (22-byte .sens, Rust SENS oracle). This executable proves
//! ONLY the target-mechanism projection of that known closed specialization:
//!     C1-SECOND X = CAR(CDR X)
//!     X = CONS(NIL, CONS(NIL, NIL))
//! Do not treat this as compiling or decoding a physical .sens source.
use wsm_os_target::{tag, Tag, NIL};

core::arch::global_asm!(include_str!("../../asm/nucleus.s"), options(att_syntax));

unsafe extern "C" {
    fn wsm_cons(context: *mut core::ffi::c_void, car: u64, cdr: u64) -> u64;
    fn wsm_car(context: *mut core::ffi::c_void, pair: u64) -> u64;
    fn wsm_cdr(context: *mut core::ffi::c_void, pair: u64) -> u64;
}

fn main() {
    let context = core::ptr::null_mut();

    // Source authority for EMPTY is SENS D3:000; its native storage
    // projection is imported from the pinned wsm-target-contract, not
    // hardcoded as an untyped integer or historical T/NIL SENS law.
    let tail = unsafe { wsm_cons(context, NIL, NIL) };
    assert_eq!(tag(tail), Tag::Cons as u64);
    let list = unsafe { wsm_cons(context, NIL, tail) };
    assert_eq!(tag(list), Tag::Cons as u64);

    // Both accesses are real calls into the hand-written x86-64 nucleus,
    // with the required context+word target ABI. No Rust implementation
    // of CONS/CAR/CDR and no duplicated SENS evaluator is involved.
    let rest = unsafe { wsm_cdr(context, list) };
    assert_eq!(rest, tail);
    assert_eq!(tag(rest), Tag::Cons as u64);
    let second = unsafe { wsm_car(context, rest) };
    assert_eq!(second, NIL, "Core1 C1-SECOND native effect must yield EMPTY");

    println!("native Core1 C1-SECOND mechanism: CAR(CDR(CONS(NIL,CONS(NIL,NIL)))) == NIL");
    println!("NOTE: WSM target mechanism only; physical T5→compiled-native handoff pending");
}
