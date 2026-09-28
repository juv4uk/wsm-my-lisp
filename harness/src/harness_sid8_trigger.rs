//! Subprocess trigger for SID8 ABI rejection paths.
//! Each invocation starts with a fresh nucleus table and must terminate through
//! wsm_fail rather than returning a coerced/truncated value.

use wsm_os_target::{encode_boxed, encode_fixnum};

core::arch::global_asm!(include_str!("../../asm/nucleus.s"), options(att_syntax));

unsafe extern "C" {
    fn wsm_sid8_new(context: *mut core::ffi::c_void, bits: u64) -> u64;
    fn wsm_sid8_bits(context: *mut core::ffi::c_void, value: u64) -> u64;
}

fn main() {
    let context = core::ptr::null_mut();
    let mode = std::env::args().nth(1).expect("trigger mode");

    unsafe {
        match mode.as_str() {
            "out-of-range" => {
                let _ = wsm_sid8_new(context, 256);
            }
            "fixnum" => {
                let five = encode_fixnum(5).expect("5 is a valid fixnum");
                let _ = wsm_sid8_bits(context, five);
            }
            "foreign-boxed" => {
                let forged = encode_boxed(1).expect("boxed handle 1 has valid wire shape");
                let _ = wsm_sid8_bits(context, forged);
            }
            other => panic!("unknown SID8 trigger mode {other:?}"),
        }
    }

    eprintln!("BUG: SID8 rejection path returned instead of failing closed");
    std::process::exit(1);
}
