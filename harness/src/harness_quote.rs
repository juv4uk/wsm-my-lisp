//! Parity witness for `(quote radio)` (`asm/entry-quote.s`) --
//! `conformance.my`'s `compiler-corpus` row 1
//! (`docs/compiler-oracle-corpus-parity-2026-09-11.md`), the last
//! remaining `pending` fixture that turned out to already be within
//! `cml`'s existing capability -- same discovery pattern as
//! `harness-cond`: check what already compiles before waiting on new
//! compiler work.
//!
//! Generated via `cml`'s own CLI (`cml x86-asm <file>`, the real
//! `parser::parse` -> `lower::lower_program` -> `X86FreestandingBackend`
//! pipeline `main.rs` wires together for exactly this subcommand) on a
//! file containing the literal source `(quote radio)` -- not a
//! hand-built IR node. Reproduce with:
//! `cargo run --bin cml -- x86-asm <file containing "(quote radio)">`
//! against any `cml` checkout.

use wsm_os_target::decode_symbol;

core::arch::global_asm!(include_str!("../../asm/nucleus.s"), options(att_syntax));
core::arch::global_asm!(include_str!("../../asm/entry-quote.s"), options(att_syntax));

unsafe extern "C" {
    fn wsm_entry(context: *mut core::ffi::c_void) -> u64;
}

fn render_symbol(word: u64) -> String {
    // Only one symbol ("radio") appears in this fixture's source, so
    // cml's interner assigns it id 1 -- same convention as
    // harness-cons's own pinned-fixture render_symbol.
    match decode_symbol(word) {
        Some(1) => "radio".to_string(),
        Some(other) => format!("<symbol {other}>"),
        None => format!("<non-symbol word {word}>"),
    }
}

fn main() {
    let result = unsafe { wsm_entry(core::ptr::null_mut()) };
    let symbol = render_symbol(result);
    assert_eq!(symbol, "radio", "result must match oracle: radio");
    println!("quote: (quote radio) -> {symbol}");
}
