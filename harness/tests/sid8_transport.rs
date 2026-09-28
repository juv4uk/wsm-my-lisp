use wsm_os_target::{BoxedKind, Tag, CANONICAL_T};

core::arch::global_asm!(include_str!("../../asm/nucleus.s"), options(att_syntax));

unsafe extern "C" {
    fn wsm_sid8_new(context: *mut core::ffi::c_void, bits: u64) -> u64;
    fn wsm_sid8_bits(context: *mut core::ffi::c_void, value: u64) -> u64;
    fn wsm_eq(context: *mut core::ffi::c_void, left: u64, right: u64) -> u64;
}

#[test]
fn all_256_sid8_values_round_trip_through_real_nucleus() {
    for raw in 0_u64..=255 {
        let first = unsafe { wsm_sid8_new(core::ptr::null_mut(), raw) };
        let second = unsafe { wsm_sid8_new(core::ptr::null_mut(), raw) };

        assert_eq!(first & 7, Tag::Boxed as u64);
        assert_eq!(first, second, "same exact SID8 must have canonical Word");
        assert_eq!(
            first,
            ((raw + 1) << 3) | Tag::Boxed as u64,
            "boxed SID8 handle must be canonical bits+1"
        );
        assert_eq!(unsafe { wsm_sid8_bits(core::ptr::null_mut(), first) }, raw);
        assert_eq!(
            unsafe { wsm_eq(core::ptr::null_mut(), first, second) },
            CANONICAL_T
        );
    }
}

#[test]
fn sid8_boxed_kind_number_is_pinned_by_target_contract() {
    assert_eq!(BoxedKind::Sid8 as u8, 4);
}
