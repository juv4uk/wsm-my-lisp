//! Parity witness for `(cons (quote A) (quote B))` (asm/entry-cons.s, copied
//! from wsm-os/artifacts/fixture.s -- wsm-os-target::FIRST_FIXTURE_SOURCE /
//! FIRST_FIXTURE_EXPECTED = "(A . B)"), the canonical first cons fixture.
//! This is the one that actually exercises wsm_cons's bump allocator in
//! asm/nucleus.s, unlike the identity-lambda witness. Also calls wsm_car/
//! wsm_cdr on the result here in the host to confirm the stored words
//! round-trip, since CML's generated wsm_entry for this fixture only calls
//! wsm_cons itself.

use wsm_os_target::{decode_symbol, tag, Tag};

core::arch::global_asm!(include_str!("../../asm/nucleus.s"), options(att_syntax));
core::arch::global_asm!(include_str!("../../asm/entry-cons.s"), options(att_syntax));

unsafe extern "C" {
    fn wsm_entry(context: *mut core::ffi::c_void) -> u64;
    fn wsm_car(context: *mut core::ffi::c_void, pair: u64) -> u64;
    fn wsm_cdr(context: *mut core::ffi::c_void, pair: u64) -> u64;
}

fn render_symbol(word: u64) -> String {
    // CML's pinned fixture assigns image-local ids A=1, B=2. Tag decoding
    // itself belongs to the target contract rather than this harness.
    match decode_symbol(word) {
        Some(1) => "A".to_string(),
        Some(2) => "B".to_string(),
        Some(other) => format!("<symbol {other}>"),
        None => format!("<non-symbol word {word}>"),
    }
}

fn main() {
    let ctx = core::ptr::null_mut();
    let pair = unsafe { wsm_entry(ctx) };
    assert_eq!(tag(pair), Tag::Cons as u64, "result must be a target-contract Cons word");
    let car = unsafe { wsm_car(ctx, pair) };
    let cdr = unsafe { wsm_cdr(ctx, pair) };
    println!("({} . {})", render_symbol(car), render_symbol(cdr));
}
