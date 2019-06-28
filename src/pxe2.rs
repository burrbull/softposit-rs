use core::mem;

mod convert;
mod math;
mod ops;

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PxE2<const N: u32>(i32);

impl<const N: u32> PxE2<{ N }> {
    pub const SIZE: usize = N as usize;
    pub const ES: usize = 2;
    pub const USEED: usize = 16;

    /// Not a Real (NaR).
    pub const NAR: Self = Self::new(-0x_8000_0000);

    /// Zero.
    pub const ZERO: Self = Self::new(0);

    /// Identity.
    pub const ONE: Self = Self::new(0x_4000_0000);

    #[inline]
    pub const fn new(i: i32) -> Self {
        Self(i)
    }
    #[inline]
    pub fn is_zero(self) -> bool {
        self == Self::ZERO
    }
    #[inline]
    pub fn is_nar(self) -> bool {
        self == Self::NAR
    }
    #[inline]
    pub fn from_bits(v: u32) -> Self {
        unsafe { mem::transmute(v) }
    }
    #[inline]
    pub fn to_bits(self) -> u32 {
        unsafe { mem::transmute(self) }
    }
}

impl<const N: u32> PxE2<{ N }> {
    pub(crate) const MASK: u32 = (((-0x_8000_0000_i32) >> (N - 1)) as u32);
    pub const SIGN_MASK: u32 = 0x_8000_0000;
    pub const REGIME_SIGN_MASK: u32 = 0x_4000_0000;

    #[inline]
    pub(crate) fn sign_ui(a: u32) -> bool {
        (a & Self::SIGN_MASK) != 0
    }

    #[inline]
    fn sign_reg_ui(a: u32) -> bool {
        (a & Self::REGIME_SIGN_MASK) != 0
    }

    #[inline]
    pub(crate) fn pack_to_ui(regime: u32, exp_a: u32, frac_a: u32) -> u32 {
        regime + exp_a + frac_a
    }

    #[inline]
    pub(crate) fn separate_bits(bits: u32) -> (i8, i32, u32) {
        let (k, tmp) = Self::separate_bits_tmp(bits);
        (
            k,
            (tmp >> (Self::SIZE - 1 - Self::ES)) as i32,
            ((tmp << 1) | 0x4000_0000) & 0x7FFF_FFFF,
        )
    }

    #[inline]
    pub(crate) fn separate_bits_tmp(bits: u32) -> (i8, u32) {
        let mut k = 0;
        let mut tmp = bits << 2;
        if Self::sign_reg_ui(bits) {
            while (tmp & 0x8000_0000) != 0 {
                k += 1;
                tmp <<= 1;
            }
        } else {
            k = -1;
            while (tmp & 0x8000_0000) == 0 {
                k -= 1;
                tmp <<= 1;
            }
            tmp &= 0x7FFF_FFFF;
        }
        (k, tmp)
    }

    #[inline]
    pub(crate) fn calculate_regime(k: i8) -> (u32, bool, u8) {
        let reg;
        if k < 0 {
            reg = (-k) as u8;
            (
                0x4000_0000_u32.checked_shr(reg as u32).unwrap_or(0),
                false,
                reg,
            )
        } else {
            reg = (k + 1) as u8;
            (
                0x7fff_ffff - 0x7fff_ffff_u32.checked_shr(reg as u32).unwrap_or(0),
                true,
                reg,
            )
        }
    }
}
