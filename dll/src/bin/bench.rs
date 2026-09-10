//! Manual timing harness for dll/: wsm_eval_string end-to-end, the FFI
//! marshaling overhead specifically, and BoxedTable add/get cost as the
//! table grows. `std::time::Instant`-based, no criterion/nightly
//! `#[bench]` dependency.
//!
//! **THE HEADLINE FINDING IS NOT A TIMING NUMBER**: asm/nucleus-win64.s's
//! arena (wsm_arena/wsm_arena_next) is a single global 4096-byte
//! (256-cell) bump allocator, never reset, shared by EVERY Session and
//! EVERY wsm_cons call for the entire lifetime of the loaded DLL/process
//! -- not per-session, not reclaimable. Any expression requiring at least
//! one list allocation permanently consumes cons cells from that same
//! 256-cell budget. Running this bench's first version with 100,000
//! iterations of `wsm_eval_string("(noop)")` hit wsm_fail_win64's
//! exit(97) after roughly 256 calls, not 100,000 -- discovered by
//! actually running it, not assumed. A follow-up controlled measurement
//! pinned the EXACT number for a 2-cell expression: `wsm_eval_string`
//! on `"(quote a)"` (a 2-element list -> 2 wsm_cons calls, 32 bytes)
//! succeeds for **exactly 128 calls** (4096 / 32 = 128 exactly, no
//! fudge factor) before the 129th hits wsm_fail_win64. A real game
//! session issuing more than ~128-256 total list-allocating Lisp
//! commands over its ENTIRE LIFETIME (not per frame, not per second --
//! ever, until the plugin DLL is unloaded) will hard-crash the same
//! way. This is a far more urgent constraint than any per-call
//! nanosecond figure below, and the numbers below should be read in
//! that light: a fast interpreter that can only ever run ~128-256
//! total commands is not "fast enough for a frame budget," it's simply
//! not usable for a real play session yet -- fixing THIS (a real,
//! reclaimable, or much larger arena) is a prerequisite for the
//! per-call latency numbers below to matter at all.
//!
//! Each cons-allocating section here therefore runs in ITS OWN process
//! invocation (spawned by tests/bench_the_bench.sh-equivalent logic
//! below is NOT how this works -- see `main`'s own arg dispatch), each
//! using an iteration count safely under the 256-cell ceiling, so one
//! section's allocations don't starve the next. The BoxedTable
//! add/get section does NOT touch the asm arena at all (pure Rust
//! `Vec`), so it alone can use large iteration counts freely.
//!
//! Run with `cargo run --release --bin bench --target x86_64-pc-windows-msvc -- all`
//! (drives every section as its own subprocess) or with one of `a`/`b`/`c`/`boxed`
//! directly. Debug-profile numbers are not representative of what a game
//! process would actually see; this crate's `[profile.release]` keeps
//! debug info so a profiler can still symbolize this binary if needed.
//!
//! This prints numbers; it does not assert on them (except the arena
//! ceiling, which is asserted precisely because it's the point).

use std::ffi::{CStr, CString};
use std::time::Instant;

use wsm_my_lisp_cyberpunk_dll::eval::{self, Env};
use wsm_my_lisp_cyberpunk_dll::ffi::{
    wsm_eval_string, wsm_free_string, wsm_register_primitive, wsm_session_free, wsm_session_init,
};
use wsm_my_lisp_cyberpunk_dll::printer::value_to_string;
use wsm_my_lisp_cyberpunk_dll::reader::read_one;
use wsm_my_lisp_cyberpunk_dll::word::{BoxedTable, SymbolTable, WORD_NIL};

// Safely under the 256-cell arena ceiling (4096 bytes / 16 bytes per
// cons cell) -- leaves headroom since this crate's own SymbolTable/Env
// setup in each section might also allocate a cell or two before the
// timed loop starts.
const CONS_BOUND_ITERATIONS: u32 = 200;
const BOXED_LOOKUPS: u32 = 100_000;

unsafe extern "C" fn noop_primitive(_argc: usize, _argv: *const u64, out: *mut u64) -> i32 {
    unsafe { *out = WORD_NIL };
    0
}

fn report(label: &str, total: std::time::Duration, iterations: u32) {
    let avg_ns = total.as_nanos() as f64 / iterations as f64;
    println!("{label:<52} total={total:>10.3?}  avg={avg_ns:>10.1} ns/call  ({iterations} iterations)");
}

fn section_a() {
    unsafe {
        let session = wsm_session_init();
        let prim_name = CString::new("noop").unwrap();
        wsm_register_primitive(session, prim_name.as_ptr(), noop_primitive);
        let source = CString::new("(noop)").unwrap();

        let start = Instant::now();
        for _ in 0..CONS_BOUND_ITERATIONS {
            let result_ptr = wsm_eval_string(session, source.as_ptr());
            std::hint::black_box(CStr::from_ptr(result_ptr));
            wsm_free_string(result_ptr);
        }
        let elapsed = start.elapsed();
        report("A: wsm_eval_string(\"(noop)\") full FFI round trip", elapsed, CONS_BOUND_ITERATIONS);

        wsm_session_free(session);
    }
}

fn section_b() {
    let mut symbols = SymbolTable::new();
    let mut boxed = BoxedTable::new();
    let mut env = Env::new();
    env.register_primitive("noop", Box::new(|_args: &[u64]| Ok(WORD_NIL)));

    let start = Instant::now();
    for _ in 0..CONS_BOUND_ITERATIONS {
        let word = read_one("(noop)", &mut symbols, &mut boxed).unwrap();
        let result = eval::eval(word, &env, &symbols).unwrap();
        std::hint::black_box(value_to_string(result, &symbols, &boxed));
    }
    let elapsed = start.elapsed();
    report("B: raw Rust read+eval+print, no FFI marshaling", elapsed, CONS_BOUND_ITERATIONS);
}

fn section_c() {
    let mut symbols = SymbolTable::new();
    let mut boxed = BoxedTable::new();
    let mut env = Env::new();
    env.register_primitive("noop", Box::new(|_args: &[u64]| Ok(WORD_NIL)));
    // Parses ONCE, outside the timed loop -- this section allocates just
    // 1 cons cell total, not 1-per-iteration, so it can run far more
    // iterations safely; kept at the same count as A/B for a clean,
    // directly comparable ns/call figure across all three.
    let word = read_one("(noop)", &mut symbols, &mut boxed).unwrap();

    let start = Instant::now();
    for _ in 0..CONS_BOUND_ITERATIONS {
        std::hint::black_box(eval::eval(word, &env, &symbols).unwrap());
    }
    let elapsed = start.elapsed();
    report("C: eval() only, pre-parsed word, no reader/printer", elapsed, CONS_BOUND_ITERATIONS);
}

fn section_boxed() {
    // Does not touch the asm arena at all -- BoxedTable is a plain Rust
    // Vec, so this can use realistic large iteration counts freely,
    // unlike A/B/C above.
    for &table_size in &[100usize, 10_000, 1_000_000] {
        let mut boxed = BoxedTable::new();
        let mut words = Vec::with_capacity(table_size);

        let start = Instant::now();
        for i in 0..table_size {
            words.push(boxed.add_string(format!("value-{i}")));
        }
        let add_elapsed = start.elapsed();

        let start = Instant::now();
        for i in 0..BOXED_LOOKUPS {
            let word = words[(i as usize) % table_size];
            std::hint::black_box(boxed.get_string(word));
        }
        let get_elapsed = start.elapsed();

        report(&format!("BoxedTable::add_string (growing table to {table_size})"), add_elapsed, table_size as u32);
        report(&format!("BoxedTable::get_string (table size {table_size})"), get_elapsed, BOXED_LOOKUPS);
    }
}

fn main() {
    let arg = std::env::args().nth(1).unwrap_or_else(|| "all".to_string());

    if arg == "all" {
        // Each cons-allocating section MUST be a separate process: the
        // arena is one global static for the whole process, so running
        // A then B then C in-process here would let A's 200 allocations
        // starve B/C of arena headroom. Re-invoke this same binary with
        // an explicit section argument, once per section.
        let exe = std::env::current_exe().expect("current_exe");
        println!("wsm-my-lisp-cyberpunk-dll bench -- driving each section as its own process\n");
        println!(
            "(arena ceiling reminder: this whole file's own header comment explains why -- \
             the 4096-byte/256-cell global arena, never reset, is the real headline finding \
             here, not the ns/call numbers below)\n"
        );
        for section in ["a", "b", "c", "boxed"] {
            let status = std::process::Command::new(&exe)
                .arg(section)
                .status()
                .unwrap_or_else(|e| panic!("failed to spawn bench subprocess for section {section}: {e}"));
            if !status.success() {
                eprintln!("section {section} exited with {status:?} -- see output above");
            }
        }
        return;
    }

    match arg.as_str() {
        "a" => section_a(),
        "b" => section_b(),
        "c" => section_c(),
        "boxed" => section_boxed(),
        other => panic!("unknown section '{other}': expected a, b, c, boxed, or all"),
    }
}
