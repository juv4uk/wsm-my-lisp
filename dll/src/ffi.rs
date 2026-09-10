//! extern "C" surface for the game-facing embedding (RED4ext/CET-style
//! callers): wsm_session_init/wsm_session_free/wsm_register_primitive/
//! wsm_eval_string/wsm_free_string. This is the layer my-lisp-cyberpunk's
//! proposed API (wsm_init/wsm_register_primitive/wsm_eval_string/
//! wsm_value_to_string) maps onto -- named `wsm_session_*` here rather
//! than `wsm_init`/a bare handle type, kept explicit about being a Session
//! (Env + SymbolTable pair) rather than the whole nucleus.
//!
//! Independent of the 3 open questions still pending from my-lisp (cond
//! special-form-vs-macro, bare-symbol bindings convention, string
//! encoding): this plumbing works the same regardless of how those get
//! resolved, since it only marshals C ABI <-> the existing Env/SymbolTable
//! types. Host primitives here operate on raw Word (u64) values, not
//! resolved types -- the C caller is responsible for knowing what tag it
//! is passing/expecting, same as nucleus.s's own callers already are.
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
use crate::word::SymbolTable;

pub struct Session {
    env: Env,
    symbols: SymbolTable,
}

/// C-callable host primitive: receives `argc` already-evaluated Word
/// arguments in `argv`, must write its Word result to `*out`, and returns
/// 0 on success or a nonzero host-defined error code. Matches nucleus.s's
/// own convention of raw Word (u64) values, not a richer marshaled type.
pub type HostPrimitiveFn = unsafe extern "C" fn(argc: usize, argv: *const u64, out: *mut u64) -> i32;

#[unsafe(no_mangle)]
pub extern "C" fn wsm_session_init() -> *mut Session {
    Box::into_raw(Box::new(Session { env: Env::new(), symbols: SymbolTable::new() }))
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

fn eval_str(session: &mut Session, text: &str) -> String {
    let word = match reader::read_one(text, &mut session.symbols) {
        Ok(w) => w,
        Err(ReadError::UnexpectedEof) => return "error: unexpected end of input".to_string(),
        Err(ReadError::UnexpectedCloseParen) => return "error: unexpected ')'".to_string(),
        Err(ReadError::TrailingInput(rest)) => return format!("error: trailing input: {rest}"),
    };
    match eval::eval(word, &session.env, &session.symbols) {
        Ok(result) => value_to_string(result, &session.symbols),
        Err(EvalError::UnknownSymbol(name)) => format!("error: unknown symbol: {name}"),
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
            assert_eq!(err, "error: unknown symbol: undefined-symbol");
            wsm_free_string(err_ptr);

            wsm_session_free(session);
        }
    }

    #[test]
    fn host_primitive_error_code_surfaces_through_eval_string() {
        unsafe {
            let session = wsm_session_init();
            let prim_name = CString::new("give-weapon").unwrap();
            assert_eq!(wsm_register_primitive(session, prim_name.as_ptr(), stub_give_weapon), 0);

            let source = CString::new("(give-weapon)").unwrap();
            let result_ptr = wsm_eval_string(session, source.as_ptr());
            let result = CStr::from_ptr(result_ptr).to_str().unwrap().to_string();
            assert_eq!(result, "error: give-weapon failed: host primitive reported error code 1");
            wsm_free_string(result_ptr);

            wsm_session_free(session);
        }
    }
}
