//! Direct primitive test for `wsm_eq` on `Symbol` words -- the gap named
//! in `docs/canon-function-table.md` (wsm-my-lisp#16): `harness-eq`
//! only ever exercised `Fixnum` equality, so nothing in this repo
//! proved `wsm_eq` behaves correctly on the other tagged word shape it
//! must also compare. Mirrors the two `compiler-corpus` fixtures in
//! `external/my-lisp/tests/fixtures/conformance.my` for symbol `eq`:
//! `(eq (quote radio) (quote radio))` -> `t` and
//! `(eq (quote radio) (quote antenna))` -> `()`. Image-local symbol
//! ids are this harness's own choice (1 = "radio", 2 = "antenna"),
//! same convention `harness-cons`'s `render_symbol` already uses for
//! its A=1/B=2 pinned fixture -- these ids carry no meaning beyond
//! "two distinct interned symbols" for this equality check.

use wsm_os_target::{encode_symbol, CANONICAL_T, NIL};

core::arch::global_asm!(include_str!("../../asm/nucleus.s"), options(att_syntax));

unsafe extern "C" {
    fn wsm_eq(context: *mut core::ffi::c_void, left: u64, right: u64) -> u64;
}

fn main() {
    let ctx = core::ptr::null_mut();
    let radio = encode_symbol(1).expect("symbol id 1 must encode");
    let radio_again = encode_symbol(1).expect("symbol id 1 must encode");
    let antenna = encode_symbol(2).expect("symbol id 2 must encode");

    let same = unsafe { wsm_eq(ctx, radio, radio_again) };
    let diff = unsafe { wsm_eq(ctx, radio, antenna) };

    assert_eq!(same, CANONICAL_T, "same symbol id must yield canonical Symbol(\"t\")");
    assert_eq!(diff, NIL, "different symbol ids must yield canonical ()");
    println!("wsm_eq: (eq radio radio) -> t, (eq radio antenna) -> ()  [both correct]");
}
