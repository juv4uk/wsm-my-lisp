//! Parity witness for `(cons (quote A) (quote B))` (asm/entry-cons.s, copied
//! from wsm-os/artifacts/fixture.s -- wsm-os-target::FIRST_FIXTURE_SOURCE /
//! FIRST_FIXTURE_EXPECTED = "(A . B)"), the canonical first cons fixture.
//! This is the one that actually exercises wsm_cons's bump allocator in
//! asm/nucleus.s, unlike the identity-lambda witness. Also calls wsm_car/
//! wsm_cdr on the result here in the host to confirm the stored words
//! round-trip, since CML's generated wsm_entry for this fixture only calls
//! wsm_cons itself.

core::arch::global_asm!(include_str!("../../asm/nucleus.s"), options(att_syntax));
core::arch::global_asm!(include_str!("../../asm/entry-cons.s"), options(att_syntax));

unsafe extern "C" {
    fn wsm_entry(context: *mut core::ffi::c_void) -> u64;
    fn wsm_car(context: *mut core::ffi::c_void, pair: u64) -> u64;
    fn wsm_cdr(context: *mut core::ffi::c_void, pair: u64) -> u64;
}

fn render_symbol(word: u64) -> String {
    // Tag::Symbol = 4; image-local ids assigned by CML for this fixture: A=1, B=2
    // (word 12 = (1<<3)|4, word 20 = (2<<3)|4), matching wsm-os-hosted's own
    // render()'s hardcoded 1=>"A", 2=>"B" for the same fixture family.
    match word >> 3 {
        1 => "A".to_string(),
        2 => "B".to_string(),
        other => format!("<symbol {other}>"),
    }
}

fn main() {
    let ctx = core::ptr::null_mut();
    let pair = unsafe { wsm_entry(ctx) };
    assert_eq!(pair & 0b111, 0, "result must be a Tag::Cons word (tag 0)");
    let car = unsafe { wsm_car(ctx, pair) };
    let cdr = unsafe { wsm_cdr(ctx, pair) };
    println!("({} . {})", render_symbol(car), render_symbol(cdr));
}
