//! Deliberately exhausts asm/nucleus-win64.s's fixed 4096-byte arena (256
//! cons cells, 16 bytes each) to exercise the wsm_cons_oom -> wsm_fail_win64
//! path for real. Run only as a subprocess (see tests/oom_path.rs) -- it
//! calls process::exit(97) on success, which would kill an in-process test
//! runner if called directly from a #[test].

fn main() {
    unsafe {
        let mut last = 1u64; // WORD_NIL, an arbitrary valid cdr to chain onto
        // 256 cells exactly fill the arena; the 257th must hit wsm_cons_oom.
        for i in 0..300u64 {
            last = wsm_my_lisp_cyberpunk_dll::wsm_cons(core::ptr::null_mut(), i, last);
        }
        // Unreached if the arena is exhausted before 300 allocations, as
        // expected -- wsm_fail_win64 exits the process first. If this
        // prints, the arena is bigger than assumed and the OOM path was
        // never exercised.
        println!("did not exhaust the arena after 300 allocations: {last}");
    }
}
