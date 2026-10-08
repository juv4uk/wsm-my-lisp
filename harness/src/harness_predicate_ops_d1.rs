//! Current exact-D1 predicate mechanism witness for wsm-my-lisp#78.
//!
//! SENS owns the law. This harness only proves that the native nucleus projects
//! admitted current ATOM/EQ results into the ratified target v8 carriers while
//! historical wsm_atom/wsm_eq remain separate compatibility entrypoints.

use wsm_os_target::{encode_fixnum, NIL};

core::arch::global_asm!(include_str!("../../asm/nucleus.s"), options(att_syntax));

unsafe extern "C" {
    fn wsm_cons(context: *mut core::ffi::c_void, car: u64, cdr: u64) -> u64;
    fn wsm_atom_predicate_bit(context: *mut core::ffi::c_void, value: u64) -> u64;
    fn wsm_eq_predicate_bit(context: *mut core::ffi::c_void, left: u64, right: u64) -> u64;
    fn wsm_predicate_bit_bits(context: *mut core::ffi::c_void, value: u64) -> u64;
}

fn bit(ctx: *mut core::ffi::c_void, value: u64) -> u64 {
    unsafe { wsm_predicate_bit_bits(ctx, value) }
}

fn main() {
    let ctx = core::ptr::null_mut();
    let one = encode_fixnum(1).expect("1 fits target fixnum");
    let two = encode_fixnum(2).expect("2 fits target fixnum");
    let pair = unsafe { wsm_cons(ctx, one, two) };

    // ATOM is total in current SENS.
    let atom_nil = unsafe { wsm_atom_predicate_bit(ctx, NIL) };
    let atom_fixnum = unsafe { wsm_atom_predicate_bit(ctx, one) };
    let atom_pair = unsafe { wsm_atom_predicate_bit(ctx, pair) };
    assert_eq!(bit(ctx, atom_nil), 1, "EMPTY is still an atom for ATOM");
    assert_eq!(bit(ctx, atom_fixnum), 1, "fixnum is an atom");
    assert_eq!(bit(ctx, atom_pair), 0, "CONS is not an atom");

    // EQ is atom-domain only: admitted atom/atom calls return D1.
    let eq_same = unsafe { wsm_eq_predicate_bit(ctx, one, one) };
    let eq_diff = unsafe { wsm_eq_predicate_bit(ctx, one, two) };
    assert_eq!(bit(ctx, eq_same), 1, "same admitted atoms -> D1:1");
    assert_eq!(bit(ctx, eq_diff), 0, "distinct admitted atoms -> D1:0");

    if std::env::args().any(|arg| arg == "--pair-eq-type-error") {
        // Contract 11.8 requires the nucleus to terminate through its named
        // Type failure path. Returning from this call is itself a regression.
        let _ = unsafe { wsm_eq_predicate_bit(ctx, pair, pair) };
        panic!("pair EQ returned instead of failing Type");
    }

    println!("current-d1 predicate mechanisms: ATOM total, EQ atom-domain exact D1");
}
