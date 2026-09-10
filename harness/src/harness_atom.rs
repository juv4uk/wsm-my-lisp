//! Parity witness harness: links this repo's hand-written asm/nucleus.s
//! (wsm_cons/wsm_car/wsm_cdr/wsm_eq/wsm_atom) against CML's generated
//! wsm_entry for `(atom (quote ()))` (asm/entry-atom.s, copied from
//! wsm-os/artifacts/nucleus-witness-fixture.s, produced by cml commit
//! 6d74b8d) -- the same fixture already run once against wsm-os-runtime's
//! Rust implementation (wsm-os commit a2aae35, printed "t"). This harness
//! does not depend on wsm-os-runtime or wsm-os-hosted at all: the context
//! pointer wsm_entry receives is unused, since this nucleus keeps its own
//! static arena (see asm/nucleus.s's own header comment for why).

use wsm_os_target::{CANONICAL_T, NIL};

core::arch::global_asm!(include_str!("../../asm/nucleus.s"), options(att_syntax));
core::arch::global_asm!(include_str!("../../asm/entry-atom.s"), options(att_syntax));

unsafe extern "C" {
    fn wsm_entry(context: *mut core::ffi::c_void) -> u64;
}

fn main() {
    let result = unsafe { wsm_entry(core::ptr::null_mut()) };
    // Witness не має власної копії бітового коду істини: canonical `t` і
    // `()` приходять з ратифікованого wsm-target-contract.
    let rendered = match result {
        CANONICAL_T => "t".to_string(),
        NIL => "()".to_string(),
        other => format!("<unrecognized word {other}>"),
    };
    println!("{rendered}");
}
