use super::PxE2;
use core::convert::From;
use core::f64;

impl From<P32E2> for f32 {
    #[inline]
    fn from(a: P32E2) -> Self {
        f64::from(a) as f32
    }
}

impl<const N: u32> From<PxE2<{ N }>> for f64 {
    #[inline]
    fn from(p_a: PxE2<{ N }>) -> Self {
        let mut ui_a = p_a.to_bits();

        if p_a.is_zero() {
            0.
        } else if p_a.is_nar() {
            f64::NAN
        } else {
            let sign_a = PxE2::<{ N }>::sign_ui(ui_a);
            if sign_a {
                ui_a = ui_a.wrapping_neg();
            }
            let (k_a, tmp) = PxE2::<{ N }>::separate_bits_tmp(ui_a);

            let frac_a = ((tmp << 3) as u64) << 20;
            let exp_a = (((k_a as u64) << 2) + ((tmp >> 29) as u64)).wrapping_add(1023) << 52;

            f64::from_bits(exp_a + frac_a + (((sign_a as u64) & 0x1) << 63))
        }
    }
}
