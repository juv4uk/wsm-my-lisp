core::arch::global_asm!(include_str!("../../asm/nucleus.s"), options(att_syntax));

unsafe extern "C" {
    fn wsm_sid8_bits(context: *mut core::ffi::c_void, value: u64) -> u64;
}

fn main() {
    // Fixnum zero is target word 3, therefore not Boxed.
    let _ = unsafe { wsm_sid8_bits(core::ptr::null_mut(), 3) };
    std::process::exit(98);
}
