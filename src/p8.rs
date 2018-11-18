use core::mem;

mod convert;
mod math;
mod ops;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct P8E0(i8);

impl P8E0 {
    #[inline]
    pub fn new() -> Self {
        Self::from_bits(0)
    }
    #[inline]
    pub fn infinity() -> Self {
        Self::from_bits(0x80)
    }
    #[inline]
    pub fn nan() -> Self {
        Self::from_bits(0x80)
    }
    #[inline]
    pub fn min_value() -> Self {
        Self::from_bits(0x81)
    }
    #[inline]
    pub fn max_value() -> Self {
        Self::from_bits(0x7F)
    }
    #[inline]
    pub fn from_bits(v: u8) -> Self {
        unsafe { mem::transmute(v) }
    }
    #[inline]
    pub fn to_bits(self) -> u8 {
        unsafe { mem::transmute(self) }
    }
    #[inline]
    pub fn abs(self) -> Self {
        let i = self.to_bits() as i8;
        Self::from_bits((if i < 0 { -i } else { i }) as u8)
    }
}

impl P8E0 {
    #[inline]
    pub(crate) fn sign_ui(a: u8) -> bool {
        (a >> 7) != 0
    }

    #[inline]
    fn sign_reg_ui(a: u8) -> bool {
        ((a >> 6) & 0x1) != 0
    }

    #[inline]
    fn pack_to_ui(regime: u8, frac_a: u8) -> u8 {
        regime + frac_a
    }

    #[inline]
    pub(crate) fn separate_bits(bits: u8) -> (i8, u8) {
        let (k, tmp) = Self::separate_bits_tmp(bits);
        (k, 0x80 | tmp)
    }

    #[inline]
    pub(crate) fn separate_bits_tmp(bits: u8) -> (i8, u8) {
        let mut k = 0_i8;
        let mut tmp = bits << 2;
        if Self::sign_reg_ui(bits) {
            while (tmp >> 7) != 0 {
                k += 1;
                tmp <<= 1;
            }
        } else {
            k = -1;
            while (tmp >> 7) == 0 {
                k -= 1;
                tmp <<= 1;
            }
            tmp &= 0x7F;
        }
        (k, tmp)
    }
}

impl PartialOrd for P8E0 {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        (self.to_bits() as i8).partial_cmp(&(other.to_bits() as i8))
    }
}
