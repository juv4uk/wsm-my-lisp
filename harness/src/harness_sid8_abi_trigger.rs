use wsm_os_target::Tag;

core::arch::global_asm!(include_str!("../../asm/nucleus.s"), options(att_syntax));

unsafe extern "C" {
    fn wsm_sid8_bits(context: *mut core::ffi::c_void, value: u64) -> u64;
}

fn main() {
    // SID8 canonical handles are exactly 1..=256. 257 is forged/out of range.
    let forged = (257_u64 << 3) | Tag::Boxed as u64;
    let _ = unsafe { wsm_sid8_bits(core::ptr::null_mut(), forged) };
    std::process::exit(98);
}
