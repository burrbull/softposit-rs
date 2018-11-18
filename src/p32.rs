use core::mem;

mod convert;
mod math;
mod ops;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct P32E2(u32);

impl P32E2 {
    #[inline]
    pub fn new() -> Self {
        Self::from_bits(0)
    }
    #[inline]
    pub fn infinity() -> Self {
        Self::from_bits(0x8000_0000)
    }
    #[inline]
    pub fn nan() -> Self {
        Self::from_bits(0x8000_0000)
    }
    #[inline]
    pub fn min_value() -> Self {
        Self::from_bits(0x8000_0001)
    }
    #[inline]
    pub fn max_value() -> Self {
        Self::from_bits(0x7FFF_FFFF)
    }
    #[inline]
    pub fn from_bits(v: u32) -> Self {
        unsafe { mem::transmute(v) }
    }
    #[inline]
    pub fn to_bits(self) -> u32 {
        unsafe { mem::transmute(self) }
    }
    #[inline]
    pub fn abs(self) -> Self {
        let i = self.to_bits() as i32;
        Self::from_bits((if i < 0 { -i } else { i }) as u32)
    }
}

impl P32E2 {
    #[inline]
    pub(crate) fn sign_ui(a: u32) -> bool {
        (a >> 31) != 0
    }

    #[inline]
    fn sign_reg_ui(a: u32) -> bool {
        ((a >> 30) & 0x1) != 0
    }

    #[inline]
    fn pack_to_ui(regime: u32, exp_a: u32, frac_a: u32) -> u32 {
        regime + exp_a + frac_a
    }

    #[inline]
    pub(crate) fn separate_bits(bits: u32) -> (i16, i32, u32) {
        let (k, tmp) = Self::separate_bits_tmp(bits);
        (
            k,
            (tmp >> 29) as i32,
            ((tmp << 1) | 0x4000_0000) & 0x7FFF_FFFF,
        )
    }

    #[inline]
    pub(crate) fn separate_bits_tmp(bits: u32) -> (i16, u32) {
        let mut k = 0_i16;
        let mut tmp = bits << 2;
        if Self::sign_reg_ui(bits) {
            while (tmp >> 31) != 0 {
                k += 1;
                tmp <<= 1;
            }
        } else {
            k = -1;
            while (tmp >> 31) == 0 {
                k -= 1;
                tmp <<= 1;
            }
            tmp &= 0x7FFF_FFFF;
        }
        (k, tmp)
    }
}

impl PartialOrd for P32E2 {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        (self.to_bits() as i32).partial_cmp(&(other.to_bits() as i32))
    }
}
