//! DLL wrapper for the my-lisp-cyberpunk embedding effort (owner go-ahead
//! 2026-09-10, coordinated with my-lisp/cml/cyberpunk sessions -- see
//! ../asm/nucleus-win64.s's header for the full context).
//!
//! Step 2 of that plan: link ../asm/nucleus-win64.s (Win64 ABI port of the
//! 5 core primitives) into a cdylib, and provide the `wsm_fail_win64` OOM
//! handler that file calls out to instead of nucleus.s's raw Linux
//! syscalls (no stable direct-syscall ABI on Windows). This crate does not
//! yet expose the reader/evaluator/host-primitive-registration surface
//! (wsm_init/wsm_register_primitive/wsm_eval_string/wsm_value_to_string)
//! proposed for the game-facing API -- that is step 3, built on top of
//! this once this step's primitives are confirmed working.
//!
//! Windows-only: nucleus-win64.s uses the Microsoft x64 calling
//! convention, not SysV, so this must not be built for non-Windows
//! targets (see cfg guard below).

#[cfg(not(windows))]
compile_error!(
    "wsm-my-lisp-cyberpunk-dll links asm/nucleus-win64.s, which uses the \
     Microsoft x64 calling convention -- it is Windows-only. Use \
     asm/nucleus.s (SysV) via the harness/ crate for non-Windows builds."
);

pub mod eval;
pub mod ffi;
pub mod printer;
pub mod reader;
pub mod word;

core::arch::global_asm!(include_str!("../../asm/nucleus-win64.s"), options(att_syntax));

unsafe extern "C" {
    #[allow(dead_code)]
    pub fn wsm_cons(context: *mut core::ffi::c_void, car: u64, cdr: u64) -> u64;
    #[allow(dead_code)]
    pub fn wsm_car(context: *mut core::ffi::c_void, pair: u64) -> u64;
    #[allow(dead_code)]
    pub fn wsm_cdr(context: *mut core::ffi::c_void, pair: u64) -> u64;
    #[allow(dead_code)]
    pub fn wsm_eq(context: *mut core::ffi::c_void, left: u64, right: u64) -> u64;
    #[allow(dead_code)]
    pub fn wsm_atom(context: *mut core::ffi::c_void, value: u64) -> u64;
}

/// Called from asm/nucleus-win64.s's `wsm_cons_oom` path (Win64 ABI:
/// code/a/b arrive in ecx/rdx/r8) -- see that file's header comment for
/// why it can't reuse nucleus.s's raw Linux syscalls. Must not return:
/// the asm side has no recovery path after this call.
#[unsafe(no_mangle)]
pub extern "C" fn wsm_fail_win64(code: u32, a: u64, b: u64) -> ! {
    eprintln!("wsm-my-lisp-cyberpunk-dll: unrecoverable condition (code={code}, a={a}, b={b})");
    std::process::exit(97); // matches nucleus.s's own arbitrary exit code
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tag encoding, mirrored from asm/nucleus-win64.s / asm/nucleus.s
    // (wsm-os-target::Tag): Cons=0, Nil=1, Symbol=4.
    const TAG_NIL: u64 = 1;
    const SYM_T_WORD: u64 = 0xFFFF_FFFF_FFFF_FFFC; // (SYMBOL_ID_MAX << 3) | Tag::Symbol

    #[test]
    fn cons_car_cdr_roundtrip() {
        unsafe {
            let pair = wsm_cons(core::ptr::null_mut(), 10, 20);
            assert_eq!(wsm_car(core::ptr::null_mut(), pair), 10);
            assert_eq!(wsm_cdr(core::ptr::null_mut(), pair), 20);
        }
    }

    #[test]
    fn eq_matches_and_distinguishes() {
        unsafe {
            assert_eq!(wsm_eq(core::ptr::null_mut(), 41, 41), SYM_T_WORD);
            assert_eq!(wsm_eq(core::ptr::null_mut(), 41, 42), TAG_NIL);
        }
    }

    #[test]
    fn atom_distinguishes_cons_from_non_cons() {
        unsafe {
            let pair = wsm_cons(core::ptr::null_mut(), 1, 2);
            assert_eq!(wsm_atom(core::ptr::null_mut(), pair), TAG_NIL); // cons -> not atom
            assert_eq!(wsm_atom(core::ptr::null_mut(), SYM_T_WORD), SYM_T_WORD); // symbol -> atom
        }
    }
}
