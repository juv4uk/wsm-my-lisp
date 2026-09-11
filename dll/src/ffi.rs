//! extern "C" surface for the game-facing embedding (RED4ext/CET-style
//! callers): wsm_session_init/wsm_session_free/wsm_register_primitive/
//! wsm_eval_string/wsm_free_string. This is the layer my-lisp-cyberpunk's
//! proposed API (wsm_init/wsm_register_primitive/wsm_eval_string/
//! wsm_value_to_string) maps onto -- named `wsm_session_*` here rather
//! than `wsm_init`/a bare handle type, kept explicit about being a Session
//! (Env + SymbolTable pair) rather than the whole nucleus.
//!
//! cond and bare-symbol bindings were both confirmed by my-lisp (see
//! eval.rs's module doc); string encoding is implemented here too now
//! (TAG_BOXED + BoxedTable, see word.rs's module doc) but remains
//! TENTATIVE -- not yet reserved in wsm-target-contract. Host primitives
//! here operate on raw Word (u64) values, not resolved types -- the C
//! caller is responsible for knowing what tag it is passing/expecting,
//! same as nucleus.s's own callers already are.
//!
//! Safety: every `#[no_mangle] pub extern "C" fn` here is a boundary
//! function -- it trusts its raw-pointer arguments are valid for the
//! duration of the call (as any C ABI does) but does not otherwise assume
//! anything about the caller beyond that contract.
//!
//! Unwind safety, and its CONFIRMED LIMIT (tested, not assumed): every
//! function here wraps its body in `std::panic::catch_unwind` so a panic
//! inside this crate's own Rust logic (e.g. eval.rs's `.expect()` calls,
//! `String`/`CStr` conversions) turns into an ordinary error return
//! instead of unwinding further. This does NOT protect against a panic
//! *inside a host-registered `HostPrimitiveFn` callback itself*: that
//! callback is declared `extern "C"` (plain "C" ABI, not "C-unwind"), and
//! modern rustc inserts an abort right at THAT boundary the moment a
//! panic tries to cross it -- before it ever reaches this file's
//! `catch_unwind`, which sits one call further up the stack. Confirmed by
//! deliberately triggering this during development: the test process hit
//! `STATUS_STACK_BUFFER_OVERRUN` / "thread caused non-unwinding panic.
//! aborting." immediately, never reaching wsm_eval_string's catch_unwind
//! at all. A real host-side callback (C++ in the actual RED4ext/CET case)
//! can't "panic" in the Rust sense, so this specific failure mode may not
//! be reachable in practice -- but if `HostPrimitiveFn` is ever
//! implemented on the Rust side too (e.g. in tests, or a future in-process
//! stub), a panic there aborts the whole process, full stop; this crate
//! does not currently attempt to change that (would need the callback
//! type to be `extern "C-unwind"`, a bigger design change, not done here).

use std::ffi::{c_char, CStr, CString};
use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::eval::{self, Env, EvalError};
use crate::printer::value_to_string;
use crate::reader::{self, ReadError};
use crate::word::{BoxedTable, SymbolTable};

pub struct Session {
    env: Env,
    symbols: SymbolTable,
    /// Ratified -- see word.rs's module doc. Renamed from `strings` to
    /// `boxed` (2026-09-10) once it stopped being String-only:
    /// `BoxedValue::GameHandle` is now the second kind it carries.
    /// Threaded through here so both string literals in
    /// `wsm_eval_string`'s input and `wsm_wrap_game_handle`'s handles
    /// work end to end, not just in unit tests.
    boxed: BoxedTable,
}

/// C-callable host primitive: receives `argc` already-evaluated Word
/// arguments in `argv`, must write its Word result to `*out`, and returns
/// 0 on success or a nonzero host-defined error code. Matches nucleus.s's
/// own convention of raw Word (u64) values, not a richer marshaled type.
pub type HostPrimitiveFn = unsafe extern "C" fn(argc: usize, argv: *const u64, out: *mut u64) -> i32;

#[unsafe(no_mangle)]
pub extern "C" fn wsm_session_init() -> *mut Session {
    Box::into_raw(Box::new(Session { env: Env::new(), symbols: SymbolTable::new(), boxed: BoxedTable::new() }))
}

/// # Safety
/// `session` must be a pointer previously returned by `wsm_session_init`
/// and not already freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wsm_session_free(session: *mut Session) {
    if session.is_null() {
        return;
    }
    drop(unsafe { Box::from_raw(session) });
}

/// # Safety
/// `session` must be a live pointer from `wsm_session_init`. `name` must
/// be a valid, NUL-terminated, UTF-8 C string for the duration of this
/// call (it is copied, not retained past return). `f` must remain valid
/// for as long as `session` is alive and this primitive can still be
/// dispatched to.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wsm_register_primitive(
    session: *mut Session,
    name: *const c_char,
    f: HostPrimitiveFn,
) -> i32 {
    if session.is_null() || name.is_null() {
        return -1;
    }
    let result = catch_unwind(AssertUnwindSafe(|| {
        let session = unsafe { &mut *session };
        let name = match unsafe { CStr::from_ptr(name) }.to_str() {
            Ok(s) => s.to_string(),
            Err(_) => return -1, // not valid UTF-8
        };
        session.env.register_primitive(
            &name,
            Box::new(move |args: &[u64]| {
                let mut out: u64 = 0;
                let rc = unsafe { f(args.len(), args.as_ptr(), &mut out as *mut u64) };
                if rc != 0 {
                    Err(format!("host primitive reported error code {rc}"))
                } else {
                    Ok(out)
                }
            }),
        );
        0
    }));
    result.unwrap_or(-2) // -2: registration panicked, distinct from -1 (bad args)
}

/// # Safety
/// `session` must be a live pointer from `wsm_session_init`. `name` must
/// be a valid NUL-terminated UTF-8 C string for the duration of this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wsm_bind(session: *mut Session, name: *const c_char, value: u64) -> i32 {
    if session.is_null() || name.is_null() {
        return -1;
    }
    let result = catch_unwind(AssertUnwindSafe(|| {
        let session = unsafe { &mut *session };
        let name = match unsafe { CStr::from_ptr(name) }.to_str() {
            Ok(s) => s,
            Err(_) => return -1,
        };
        session.env.bind(name, value);
        0
    }));
    result.unwrap_or(-2)
}

/// Evaluates one form read from `source`. Returns an owned, NUL-terminated
/// C string the caller must free with `wsm_free_string`: either the
/// printed result on success, or a `"error: ..."`-prefixed message on
/// read/eval failure. A string return (rather than an out-param Word) is
/// used here so read/eval errors have somewhere to go without a second
/// out-parameter -- this is this crate's own convention, not something
/// confirmed against a wider API contract.
///
/// # Safety
/// `session` must be a live pointer from `wsm_session_init`. `source` must
/// be a valid NUL-terminated UTF-8 C string for the duration of this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wsm_eval_string(session: *mut Session, source: *const c_char) -> *mut c_char {
    let message = if session.is_null() || source.is_null() {
        "error: null session or source pointer".to_string()
    } else {
        let result = catch_unwind(AssertUnwindSafe(|| {
            let session = unsafe { &mut *session };
            match unsafe { CStr::from_ptr(source) }.to_str() {
                Err(_) => "error: source is not valid UTF-8".to_string(),
                Ok(text) => eval_str(session, text),
            }
        }));
        result.unwrap_or_else(|_| "error: internal panic during eval".to_string())
    };
    CString::new(message)
        .unwrap_or_else(|_| CString::new("error: result contained an embedded NUL").unwrap())
        .into_raw()
}

/// Resets the shared asm arena at the start of every top-level eval, so a
/// long-lived session (a REPL, or any adapter issuing many `wsm_eval_string`
/// calls over time) doesn't exhaust the fixed 256-cell arena after ~128
/// calls -- see `dll/README.md`'s "Performance" section for the exact,
/// empirically-confirmed ceiling this fixes and `asm/nucleus-win64.s`'s
/// `wsm_arena_reset` doc comment for the underlying mechanism.
///
/// **Why this is safe under this crate's CURRENT language capabilities,
/// and where that stops being true**: nothing in `eval.rs` lets a Lisp
/// expression persist a cons-containing value past the end of the
/// top-level form that produced it -- there is no `def`/`let`/closures,
/// only `Env::bindings` (set exclusively by the *host* via `wsm_bind`,
/// never by evaluated Lisp code) and `BoxedTable` (Rust-heap-owned, not
/// arena memory). By the time `eval_str` returns, the result has already
/// been fully printed to an owned `String` (walking any cons cells via
/// `wsm_car`/`wsm_cdr` during printing, before this function returns) --
/// so nothing outstanding still needs THIS call's arena allocations once
/// the next call's reset runs. This stops being safe the moment either
/// changes: (1) if `eval.rs` ever gains a form that lets Lisp code stash
/// a cons value somewhere expected to outlive one top-level eval (a
/// future `def`, for instance), or (2) if a host ever calls `wsm_bind`
/// with a `Word` whose tag is `Cons` (bindings today are always Fixnum/
/// Symbol/Boxed handles in every adapter usage seen so far, but nothing
/// in this crate's types currently prevents a `Cons`-tagged bind) --
/// either would leave a dangling reference after the very next reset.
/// Neither is exercised by this crate's own test suite; revisit this
/// comment (and likely make the reset conditional or session-scoped for
/// real) before either becomes true.
///
/// **Also true, unrelated to this reset specifically but sharpened by
/// it**: the arena itself is one global static in `asm/nucleus-win64.s`,
/// not per-`Session` -- `dll/README.md`'s embed contract already
/// documents "single thread only," but this reset makes an additional,
/// previously-latent assumption load-bearing: only ONE `Session` should
/// be mid-eval-lifetime at a time, full stop, even across threads taking
/// turns. If two `Session`s are ever alive concurrently and BOTH call
/// `wsm_eval_string`, each call's reset discards the other session's
/// in-flight cons allocations too -- there is no per-session isolation
/// at the arena level, only at the Rust-struct level (`Env`/
/// `SymbolTable`/`BoxedTable`). Not exercised by this crate's tests
/// (which use one `Session` at a time); a real per-session arena would
/// need the arena itself to move out of static storage and into
/// something the `context` parameter (currently ignored everywhere)
/// actually threads through -- a bigger change than this fix, not
/// attempted here.
fn eval_str(session: &mut Session, text: &str) -> String {
    unsafe { crate::wsm_arena_reset(core::ptr::null_mut()) };
    let word = match reader::read_one(text, &mut session.symbols, &mut session.boxed) {
        Ok(w) => w,
        Err(ReadError::UnexpectedEof) => return "error: unexpected end of input".to_string(),
        Err(ReadError::UnexpectedCloseParen) => return "error: unexpected ')'".to_string(),
        Err(ReadError::UnterminatedString) => return "error: unterminated string literal".to_string(),
        Err(ReadError::TrailingInput(rest)) => return format!("error: trailing input: {rest}"),
    };
    match eval::eval(word, &session.env, &session.symbols) {
        Ok(result) => value_to_string(result, &session.symbols, &session.boxed),
        // Matches my-lisp's own exact trilingual UnknownSymbol text
        // verbatim (docs/cyberpunk-host-dispatch-fixtures.md §4, quoting
        // crates/my-lisp/src/eval/mod.rs, confirmed against a real run of
        // their CLI, not from memory) -- their fixture doc explicitly
        // offers matching on the `: <name>` suffix as an acceptable
        // alternative for a minimal implementation, but matching verbatim
        // costs nothing here and keeps this crate's error text directly
        // comparable to my-lisp's own oracle output.
        Err(EvalError::UnknownSymbol(name)) => {
            format!("error: unknown symbol · nevidomyi symvol · unbekanntes Symbol: {name}")
        }
        Err(EvalError::NotCallable) => "error: not callable".to_string(),
        Err(EvalError::CondFallthrough) => "error: cond: no clause matched".to_string(),
        Err(EvalError::HostPrimitiveFailed { name, message }) => {
            format!("error: {name} failed: {message}")
        }
    }
}

/// # Safety
/// `s` must be a pointer previously returned by `wsm_eval_string` (or
/// null, which is a no-op) and not already freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wsm_free_string(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    drop(unsafe { CString::from_raw(s) });
}

/// Wraps an opaque caller-defined token into a `Boxed` Word a Lisp
/// expression can carry and pass back to a later host primitive -- per
/// `wsm-target-contract#2`'s ratification (contract v4,
/// `docs/migration-2026-09-10-game-handle-boxed-kind.md`):
/// `BoxedValue::GameHandle`, session-local, never dereferenced on this
/// side. `handle` is stored and returned verbatim by
/// `wsm_unwrap_game_handle` -- this function does not validate,
/// dereference, or interpret it in any way.
///
/// **Do NOT pass a raw refcounted engine pointer directly** (e.g. a
/// `RED4ext::Handle<T>`'s underlying `T*`, extracted from a
/// stack-local `Handle<T>` that then goes out of scope). This crate
/// holds no reference of its own, so if the only thing keeping the
/// referenced object alive was that local `Handle<T>`, the pointer
/// dangles the moment it does, and a later `wsm_unwrap_game_handle`
/// hands the caller a use-after-free. The correct pattern: the caller
/// keeps its own table of real, refcount-holding `Handle<T>` objects
/// (owning their lifetime for as long as needed) and passes THIS
/// function an opaque token identifying a row in that table (an index
/// cast to `*mut c_void`, or any other caller-chosen bit pattern) --
/// never the engine object's own address. This function's contract was
/// always "store and return an opaque bit pattern verbatim," which
/// already supports that pattern with no signature change; this
/// paragraph exists to make the *safe* usage explicit, not to change
/// behavior. The caller remains solely responsible for the token's
/// validity/meaning for as long as the returned Word might still be
/// unwrapped -- this crate does not (and structurally cannot) enforce
/// or check that.
///
/// Writes the encoded Word to `*out` and returns 0 on success. Returns
/// -1 for a null `session` or `out` pointer, -2 if a panic was caught
/// (see this module's own unwind-safety doc).
///
/// # Safety
/// `session` must be a live pointer from `wsm_session_init`. `out` must
/// be a valid, writable `u64` for the duration of this call. `handle` is
/// opaque to this function and imposes no safety requirement of its own
/// here (it is never dereferenced) -- but see the ownership note above
/// for what a caller must arrange for `wsm_unwrap_game_handle` to later
/// return something safe to use.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wsm_wrap_game_handle(
    session: *mut Session,
    handle: *mut core::ffi::c_void,
    out: *mut u64,
) -> i32 {
    if session.is_null() || out.is_null() {
        return -1;
    }
    let result = catch_unwind(AssertUnwindSafe(|| {
        let session = unsafe { &mut *session };
        let word = session.boxed.add_game_handle(handle);
        unsafe { *out = word };
        0
    }));
    result.unwrap_or(-2)
}

/// Recovers the opaque host handle a `Boxed` Word (produced by
/// `wsm_wrap_game_handle`) carries. Writes the handle to `*out` and
/// returns 0 on success; returns 1 (not 0/-1/-2, deliberately distinct
/// from this file's other error codes) if `word` is not a `GameHandle`
/// -- e.g. it decodes to a `Str` Boxed value instead, or isn't a `Boxed`
/// word at all -- a caller-diagnosable "wrong kind" outcome, not a
/// crash. Returns -1 for a null `session`/`out` pointer, -2 on a caught
/// panic.
///
/// # Safety
/// `session` must be a live pointer from `wsm_session_init`. `out` must
/// be a valid, writable pointer for a `*mut c_void` for the duration of
/// this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wsm_unwrap_game_handle(
    session: *mut Session,
    word: u64,
    out: *mut *mut core::ffi::c_void,
) -> i32 {
    if session.is_null() || out.is_null() {
        return -1;
    }
    let result = catch_unwind(AssertUnwindSafe(|| {
        let session = unsafe { &mut *session };
        match session.boxed.get_game_handle(word) {
            Some(handle) => {
                unsafe { *out = handle };
                0
            }
            None => 1,
        }
    }));
    result.unwrap_or(-2)
}

/// Wraps an exact fraction into a `Boxed` Word carrying
/// `BoxedValue::Rational` -- per my-lisp's confirmation (2026-09-11,
/// coordinating on a future `(позиція-гравця)` capability) that a
/// `Rational` must be its own distinct, identity-bearing value, not a
/// plain `(numerator . denominator)` cons pair (which would be
/// indistinguishable from an arbitrary cons a Lisp expression could
/// construct itself, and would print wrong -- `(5 . 336)` instead of
/// their own oracle's `5/336`). Reduces to lowest terms with a positive
/// denominator at construction (`word.rs`'s `add_rational`/`reduce`),
/// matching my-lisp's own `Rational` invariant, so the printed result is
/// byte-identical to their oracle automatically.
///
/// Writes the encoded Word to `*out` and returns 0 on success. Returns
/// -1 for a null `session`/`out` pointer or a zero `denominator` (this
/// crate treats a zero denominator as a caller error to reject, not
/// something to silently coerce), -2 if a panic was caught.
///
/// # Safety
/// `session` must be a live pointer from `wsm_session_init`. `out` must
/// be a valid, writable `u64` for the duration of this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wsm_wrap_rational(
    session: *mut Session,
    numerator: i64,
    denominator: i64,
    out: *mut u64,
) -> i32 {
    if session.is_null() || out.is_null() || denominator == 0 {
        return -1;
    }
    let result = catch_unwind(AssertUnwindSafe(|| {
        let session = unsafe { &mut *session };
        let word = session.boxed.add_rational(numerator, denominator);
        unsafe { *out = word };
        0
    }));
    result.unwrap_or(-2)
}

/// Recovers the `(numerator, denominator)` a `Boxed` Word (produced by
/// `wsm_wrap_rational`) carries -- already reduced, denominator always
/// positive. Writes them to `*out_numerator`/`*out_denominator` and
/// returns 0 on success; returns 1 (matching `wsm_unwrap_game_handle`'s
/// own "wrong kind" convention) if `word` isn't a `Rational` Boxed word.
/// Returns -1 for a null `session`/output pointer, -2 on a caught panic.
///
/// # Safety
/// `session` must be a live pointer from `wsm_session_init`.
/// `out_numerator`/`out_denominator` must each be a valid, writable
/// `i64` for the duration of this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wsm_unwrap_rational(
    session: *mut Session,
    word: u64,
    out_numerator: *mut i64,
    out_denominator: *mut i64,
) -> i32 {
    if session.is_null() || out_numerator.is_null() || out_denominator.is_null() {
        return -1;
    }
    let result = catch_unwind(AssertUnwindSafe(|| {
        let session = unsafe { &mut *session };
        match session.boxed.get_rational(word) {
            Some((n, d)) => {
                unsafe {
                    *out_numerator = n;
                    *out_denominator = d;
                }
                0
            }
            None => 1,
        }
    }));
    result.unwrap_or(-2)
}

#[cfg(test)]
mod tests {
    use super::*;

    unsafe extern "C" fn stub_teleport(argc: usize, argv: *const u64, out: *mut u64) -> i32 {
        assert_eq!(argc, 4);
        let args = unsafe { std::slice::from_raw_parts(argv, argc) };
        // args[0] is the `player` binding's word, args[1..] are x/y/z.
        unsafe { *out = args[1] };
        0
    }

    unsafe extern "C" fn stub_give_weapon(_argc: usize, _argv: *const u64, _out: *mut u64) -> i32 {
        1 // nonzero: host-reported failure
    }

    #[test]
    fn end_to_end_register_bind_eval_print_free() {
        unsafe {
            let session = wsm_session_init();
            assert!(!session.is_null());

            let name = CString::new("player").unwrap();
            assert_eq!(wsm_bind(session, name.as_ptr(), crate::word::encode_fixnum(7)), 0);

            let prim_name = CString::new("teleport").unwrap();
            assert_eq!(wsm_register_primitive(session, prim_name.as_ptr(), stub_teleport), 0);

            let source = CString::new("(teleport player 100 200 50)").unwrap();
            let result_ptr = wsm_eval_string(session, source.as_ptr());
            let result = CStr::from_ptr(result_ptr).to_str().unwrap().to_string();
            assert_eq!(result, "100"); // stub echoes args[1] (x = 100)
            wsm_free_string(result_ptr);

            let bad_source = CString::new("(undefined-symbol)").unwrap();
            let err_ptr = wsm_eval_string(session, bad_source.as_ptr());
            let err = CStr::from_ptr(err_ptr).to_str().unwrap().to_string();
            assert_eq!(
                err,
                "error: unknown symbol · nevidomyi symvol · unbekanntes Symbol: undefined-symbol"
            );
            wsm_free_string(err_ptr);

            wsm_session_free(session);
        }
    }

    #[test]
    fn string_literal_round_trips_through_eval_string() {
        unsafe {
            let session = wsm_session_init();
            let source = CString::new(r#""пістолет""#).unwrap();
            let result_ptr = wsm_eval_string(session, source.as_ptr());
            let result = CStr::from_ptr(result_ptr).to_str().unwrap().to_string();
            assert_eq!(result, r#""пістолет""#); // self-evaluates, prints with quotes
            wsm_free_string(result_ptr);
            wsm_session_free(session);
        }
    }

    #[test]
    fn unknown_symbol_error_text_is_not_mangled_for_cyrillic() {
        // my-lisp confirmed (docs/cyberpunk-host-dispatch-fixtures.md,
        // commit 5a6bb90) that their own UnknownSymbol error text carries
        // the Cyrillic identifier verbatim, no mangling. Same expectation
        // here, through the full FFI round-trip (CStr -> read -> eval ->
        // CString), not just the in-memory Rust value.
        unsafe {
            let session = wsm_session_init();
            let source = CString::new("(збережи-гру)").unwrap();
            let result_ptr = wsm_eval_string(session, source.as_ptr());
            let result = CStr::from_ptr(result_ptr).to_str().unwrap().to_string();
            assert_eq!(
                result,
                "error: unknown symbol · nevidomyi symvol · unbekanntes Symbol: збережи-гру"
            );
            wsm_free_string(result_ptr);
            wsm_session_free(session);
        }
    }

    #[test]
    fn game_handle_wraps_and_unwraps_through_ffi() {
        unsafe {
            let session = wsm_session_init();
            let fake_handle = 0x5555_usize as *mut core::ffi::c_void;

            let mut word: u64 = 0;
            assert_eq!(wsm_wrap_game_handle(session, fake_handle, &mut word as *mut u64), 0);

            let mut recovered: *mut core::ffi::c_void = core::ptr::null_mut();
            assert_eq!(wsm_unwrap_game_handle(session, word, &mut recovered as *mut _), 0);
            assert_eq!(recovered, fake_handle);

            // Wrong-kind word (a Fixnum, not a GameHandle-carrying Boxed word)
            // reports 1, not a crash.
            let mut out: *mut core::ffi::c_void = core::ptr::null_mut();
            assert_eq!(
                wsm_unwrap_game_handle(session, crate::word::encode_fixnum(42), &mut out as *mut _),
                1
            );

            wsm_session_free(session);
        }
    }

    #[test]
    fn rational_wraps_unwraps_and_reduces_through_ffi() {
        unsafe {
            let session = wsm_session_init();

            // Already-reduced fraction round-trips unchanged.
            let mut word: u64 = 0;
            assert_eq!(wsm_wrap_rational(session, 5, 336, &mut word as *mut u64), 0);
            let (mut n, mut d) = (0i64, 0i64);
            assert_eq!(wsm_unwrap_rational(session, word, &mut n as *mut i64, &mut d as *mut i64), 0);
            assert_eq!((n, d), (5, 336));

            // Reduction happens at construction, per my-lisp's confirmed
            // invariant -- 10/20 comes back as 2/4 reduced, not stored raw.
            let mut word2: u64 = 0;
            assert_eq!(wsm_wrap_rational(session, 10, 20, &mut word2 as *mut u64), 0);
            let (mut n2, mut d2) = (0i64, 0i64);
            assert_eq!(wsm_unwrap_rational(session, word2, &mut n2 as *mut i64, &mut d2 as *mut i64), 0);
            assert_eq!((n2, d2), (1, 2));

            // Negative denominator: sign moves to the numerator, denominator
            // stays positive, per my-lisp's own stated invariant.
            let mut word3: u64 = 0;
            assert_eq!(wsm_wrap_rational(session, 3, -4, &mut word3 as *mut u64), 0);
            let (mut n3, mut d3) = (0i64, 0i64);
            assert_eq!(wsm_unwrap_rational(session, word3, &mut n3 as *mut i64, &mut d3 as *mut i64), 0);
            assert_eq!((n3, d3), (-3, 4));

            // Zero denominator is rejected, not silently accepted.
            let mut bad_word: u64 = 0;
            assert_eq!(wsm_wrap_rational(session, 1, 0, &mut bad_word as *mut u64), -1);

            // Wrong-kind word (a Fixnum, not a Rational-carrying Boxed word)
            // reports 1, not a crash.
            let (mut wn, mut wd) = (0i64, 0i64);
            assert_eq!(
                wsm_unwrap_rational(session, crate::word::encode_fixnum(42), &mut wn as *mut i64, &mut wd as *mut i64),
                1
            );

            wsm_session_free(session);
        }
    }

    #[test]
    fn rational_prints_as_n_slash_d_matching_my_lisp_oracle_format() {
        // Matches conformance.my's own oracle output verbatim:
        // `(/ 5 6 8 7)` -> "5/336".
        unsafe {
            let session = wsm_session_init();
            unsafe extern "C" fn stub_position(_argc: usize, _argv: *const u64, out: *mut u64) -> i32 {
                RATIONAL_SESSION.with(|s| wsm_wrap_rational(*s.borrow(), 5, 336, out))
            }
            thread_local! {
                static RATIONAL_SESSION: std::cell::RefCell<*mut Session> = std::cell::RefCell::new(core::ptr::null_mut());
            }
            RATIONAL_SESSION.with(|s| *s.borrow_mut() = session);

            let name = CString::new("позиція-x").unwrap();
            assert_eq!(wsm_register_primitive(session, name.as_ptr(), stub_position), 0);
            let source = CString::new("(позиція-x)").unwrap();
            let result_ptr = wsm_eval_string(session, source.as_ptr());
            let result = CStr::from_ptr(result_ptr).to_str().unwrap().to_string();
            assert_eq!(result, "5/336");
            wsm_free_string(result_ptr);

            wsm_session_free(session);
        }
    }

    #[test]
    fn game_handle_round_trips_through_a_host_primitive_pair() {
        // Simulates the real shape: one primitive wraps a handle it got
        // from the host (here, a fixed test pointer standing in for
        // ExecuteGlobalFunction("GetPlayer;GameInstance", ...)'s result),
        // a second receives that Word back and unwraps it -- proving the
        // Word survives a round trip through wsm_eval_string, not just a
        // direct Rust-level call.
        unsafe extern "C" fn stub_get_player(_argc: usize, _argv: *const u64, out: *mut u64) -> i32 {
            // In this stub, the "session" isn't reachable from a plain
            // HostPrimitiveFn -- real adapter code would close over its
            // own session pointer. Here we just prove the *shape* works
            // by wrapping inline via a thread-local session pointer set
            // by the test below.
            SESSION_FOR_TEST.with(|s| {
                let session = *s.borrow();
                let fake_handle = 0x1234_usize as *mut core::ffi::c_void;
                wsm_wrap_game_handle(session, fake_handle, out)
            })
        }

        unsafe extern "C" fn stub_check_player(argc: usize, argv: *const u64, out: *mut u64) -> i32 {
            assert_eq!(argc, 1);
            let word = *argv;
            SESSION_FOR_TEST.with(|s| {
                let session = *s.borrow();
                let mut recovered: *mut core::ffi::c_void = core::ptr::null_mut();
                let rc = wsm_unwrap_game_handle(session, word, &mut recovered as *mut _);
                if rc != 0 {
                    return rc;
                }
                *out = if recovered as usize == 0x1234 {
                    crate::word::SYM_T_WORD
                } else {
                    crate::word::WORD_NIL
                };
                0
            })
        }

        thread_local! {
            static SESSION_FOR_TEST: std::cell::RefCell<*mut Session> = std::cell::RefCell::new(core::ptr::null_mut());
        }

        unsafe {
            let session = wsm_session_init();
            SESSION_FOR_TEST.with(|s| *s.borrow_mut() = session);

            let get_player_name = CString::new("гравець-handle").unwrap();
            assert_eq!(wsm_register_primitive(session, get_player_name.as_ptr(), stub_get_player), 0);
            let check_name = CString::new("перевір-гравця").unwrap();
            assert_eq!(wsm_register_primitive(session, check_name.as_ptr(), stub_check_player), 0);

            let source = CString::new("(перевір-гравця (гравець-handle))").unwrap();
            let result_ptr = wsm_eval_string(session, source.as_ptr());
            let result = CStr::from_ptr(result_ptr).to_str().unwrap().to_string();
            assert_eq!(result, "t");
            wsm_free_string(result_ptr);

            wsm_session_free(session);
        }
    }

    #[test]
    fn repeated_eval_string_calls_survive_past_the_old_arena_ceiling() {
        // Before the wsm_arena_reset fix, dll/src/bin/bench.rs found
        // wsm_eval_string("(quote a)") hard-crashing the whole process
        // (exit 97) on exactly the 129th call, since the shared asm
        // arena (4096 bytes / 16 bytes per cons cell = 256 cells; this
        // 2-cons-cell expression = 128 calls) was never reset. This test
        // runs well past that old ceiling (500 > 128) and must still
        // succeed -- if the reset regresses, this test would crash the
        // whole test process, not just fail an assertion (same failure
        // mode `tests/oom_path.rs` exists to catch for the raw wsm_cons
        // path; this one covers the wsm_eval_string path specifically).
        unsafe {
            let session = wsm_session_init();
            let source = CString::new("(quote a)").unwrap();
            for i in 0..500 {
                let result_ptr = wsm_eval_string(session, source.as_ptr());
                let result = CStr::from_ptr(result_ptr).to_str().unwrap().to_string();
                assert_eq!(result, "a", "call {i} produced an unexpected result");
                wsm_free_string(result_ptr);
            }
            wsm_session_free(session);
        }
    }

    #[test]
    fn host_primitive_error_code_surfaces_through_eval_string() {
        unsafe {
            let session = wsm_session_init();
            let prim_name = CString::new("дай-зброю").unwrap();
            assert_eq!(wsm_register_primitive(session, prim_name.as_ptr(), stub_give_weapon), 0);

            let source = CString::new("(дай-зброю)").unwrap();
            let result_ptr = wsm_eval_string(session, source.as_ptr());
            let result = CStr::from_ptr(result_ptr).to_str().unwrap().to_string();
            assert_eq!(result, "error: дай-зброю failed: host primitive reported error code 1");
            wsm_free_string(result_ptr);

            wsm_session_free(session);
        }
    }
}
