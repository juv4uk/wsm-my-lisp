//! `wsm_atom`'s negative branch (a `Cons` word is NOT an atom) had zero
//! witness coverage -- `harness-atom` only ever proved the positive
//! case (`()` IS an atom). Found while auditing this repo's own
//! primitive coverage for wsm-my-lisp#16. Mirrors
//! `external/my-lisp/tests/fixtures/conformance.my`'s own
//! `(atom (quote (radio antenna)))` -> `()` oracle fixture (line 15,
//! not itself `compiler-corpus`-tagged, but the same shape).
//!
//! Builds a real cons cell via `wsm_cons` (exercising the bump
//! allocator, same as `harness-cons`) rather than fabricating a
//! Cons-tagged word by hand, so this proves `wsm_atom` against an
//! actually-allocated pair, not just a bit pattern that happens to
//! decode as Cons.

use wsm_os_target::{encode_symbol, tag, CANONICAL_T, NIL, Tag};

core::arch::global_asm!(include_str!("../../asm/nucleus.s"), options(att_syntax));

unsafe extern "C" {
    fn wsm_cons(context: *mut core::ffi::c_void, car: u64, cdr: u64) -> u64;
    fn wsm_atom(context: *mut core::ffi::c_void, value: u64) -> u64;
}

fn main() {
    let ctx = core::ptr::null_mut();
    let radio = encode_symbol(1).expect("symbol id 1 must encode");
    let antenna = encode_symbol(2).expect("symbol id 2 must encode");

    let pair = unsafe { wsm_cons(ctx, radio, antenna) };
    assert_eq!(tag(pair), Tag::Cons as u64, "wsm_cons must yield a Cons-tagged word");

    let atom_of_nil = unsafe { wsm_atom(ctx, NIL) };
    let atom_of_pair = unsafe { wsm_atom(ctx, pair) };

    assert_eq!(atom_of_nil, CANONICAL_T, "() must be an atom");
    assert_eq!(atom_of_pair, NIL, "a real allocated Cons must NOT be an atom");
    println!("wsm_atom: (atom ()) -> t, (atom (cons radio antenna)) -> ()  [both correct]");
}
