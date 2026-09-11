//! Stage2 executable witness for the ratified closure ABI in `asm/nucleus.s`.
//!
//! Two descriptors with identical definition/environment payloads must still
//! be distinct closure identities. That is the concrete machine property the
//! current `meta-eval.my` result-token provenance relies on: a program cannot
//! forge token identity merely by reconstructing equal-looking Lisp data.

use wsm_os_target::{CANONICAL_T, NIL, decode_closure_pointer, encode_fixnum};

core::arch::global_asm!(include_str!("../../asm/nucleus.s"), options(att_syntax));

unsafe extern "C" {
    fn wsm_closure_new(
        context: *mut core::ffi::c_void,
        definition_id: u32,
        environment_ref: u64,
    ) -> u64;
    fn wsm_closure_definition(context: *mut core::ffi::c_void, closure: u64) -> u32;
    fn wsm_closure_environment(context: *mut core::ffi::c_void, closure: u64) -> u64;
    fn wsm_eq(context: *mut core::ffi::c_void, left: u64, right: u64) -> u64;
}

fn main() {
    let context = core::ptr::null_mut();
    let environment = encode_fixnum(42).expect("42 must fit the target fixnum domain");

    let first = unsafe { wsm_closure_new(context, 17, environment) };
    let second = unsafe { wsm_closure_new(context, 17, environment) };

    assert!(
        decode_closure_pointer(first).is_some(),
        "first closure must carry the ratified Closure tag"
    );
    assert!(
        decode_closure_pointer(second).is_some(),
        "second closure must carry the ratified Closure tag"
    );
    assert_ne!(
        first, second,
        "separate closure allocations must have separate identity"
    );

    assert_eq!(unsafe { wsm_closure_definition(context, first) }, 17);
    assert_eq!(
        unsafe { wsm_closure_environment(context, first) },
        environment
    );
    assert_eq!(unsafe { wsm_eq(context, first, first) }, CANONICAL_T);
    assert_eq!(unsafe { wsm_eq(context, first, second) }, NIL);

    println!("closure identity: definition=17 env=42 self-eq=t peer-eq=()");
}
