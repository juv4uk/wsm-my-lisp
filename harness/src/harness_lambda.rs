//! Parity witness harness for `((lambda (x) x) 7)` (asm/entry-lambda.s,
//! copied from wsm-os/artifacts/nucleus-witness-lambda-fixture.s, cml
//! commit 56ef40b). This fixture happens to call no wsm_* primitive at all
//! (pure identity, stack-only), so it links against asm/nucleus.s only to
//! satisfy the shared build, not because it exercises the nucleus.

core::arch::global_asm!(include_str!("../../asm/nucleus.s"), options(att_syntax));
core::arch::global_asm!(include_str!("../../asm/entry-lambda.s"), options(att_syntax));

unsafe extern "C" {
    fn wsm_entry(context: *mut core::ffi::c_void) -> u64;
}

fn main() {
    let result = unsafe { wsm_entry(core::ptr::null_mut()) };
    // wsm-os-target::encode_fixnum: (value << 3) | Tag::Fixnum(3).
    if result & 0b111 == 3 {
        println!("{}", (result as i64) >> 3);
    } else {
        println!("<unrecognized word {result}>");
    }
}
