//! Parity witness harness: links this repo's hand-written asm/nucleus.s
//! (wsm_cons/wsm_car/wsm_cdr/wsm_eq/wsm_atom) against CML's generated
//! wsm_entry for `(atom (quote ()))` (asm/entry-atom.s, copied from
//! wsm-os/artifacts/nucleus-witness-fixture.s, produced by cml commit
//! 6d74b8d) -- the same fixture already run once against wsm-os-runtime's
//! Rust implementation (wsm-os commit a2aae35, printed "t"). This harness
//! does not depend on wsm-os-runtime or wsm-os-hosted at all: the context
//! pointer wsm_entry receives is unused, since this nucleus keeps its own
//! static arena (see asm/nucleus.s's own header comment for why).

core::arch::global_asm!(include_str!("../../asm/nucleus.s"), options(att_syntax));
core::arch::global_asm!(include_str!("../../asm/entry-atom.s"), options(att_syntax));

unsafe extern "C" {
    fn wsm_entry(context: *mut core::ffi::c_void) -> u64;
}

// 2026-09-02: asm/nucleus.s no longer emits Tag::True (a manufactured
// primitive canonical WSM never had) -- ATOM/EQ's positive result is now
// canonical Symbol("t"), encoded as (SYMBOL_ID_MAX << 3) | Tag::Symbol(4)
// per asm/nucleus.s's own SYM_T_WORD constant and comment on why this
// sentinel id, not a proven-unique one, is used.
const SYM_T_WORD: u64 = 0xFFFF_FFFF_FFFF_FFFC;

fn main() {
    let result = unsafe { wsm_entry(core::ptr::null_mut()) };
    // Word encoding: Tag::Nil = 1 (wsm-os-target::Tag).
    let rendered = match result {
        SYM_T_WORD => "t".to_string(),
        1 => "()".to_string(),
        other => format!("<unrecognized word {other}>"),
    };
    println!("{rendered}");
}
