use core::mem;

mod convert;
mod math;
mod ops;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct P16E1(i16);

impl P16E1 {
    #[inline]
    pub fn new() -> Self {
        Self::from_bits(0)
    }
    #[inline]
    pub fn infinity() -> Self {
        Self::from_bits(0x8000)
    }
    #[inline]
    pub fn nan() -> Self {
        Self::from_bits(0x8000)
    }
    #[inline]
    pub fn min_value() -> Self {
        Self::from_bits(0x8001)
    }
    #[inline]
    pub fn max_value() -> Self {
        Self::from_bits(0x7FFF)
    }
    #[inline]
    pub fn from_bits(v: u16) -> Self {
        unsafe { mem::transmute(v) }
    }
    #[inline]
    pub fn to_bits(self) -> u16 {
        unsafe { mem::transmute(self) }
    }
    #[inline]
    pub fn abs(self) -> Self {
        let i = self.to_bits() as i16;
        Self::from_bits((if i < 0 { -i } else { i }) as u16)
    }
}

impl P16E1 {
    #[inline]
    pub(crate) fn sign_ui(a: u16) -> bool {
        (a >> 15) != 0
    }

    #[inline]
    fn sign_reg_ui(a: u16) -> bool {
        ((a >> 14) & 0x1) != 0
    }

    #[inline]
    fn pack_to_ui(regime: u16, reg_a: u16, exp_a: u16, frac_a: u16) -> u16 {
        regime + (exp_a << (13 - reg_a)) + frac_a
    }

    #[inline]
    pub(crate) fn separate_bits(bits: u16) -> (i8, i8, u16) {
        let (k, tmp) = Self::separate_bits_tmp(bits);
        (k, (tmp >> 14) as i8, (tmp | 0x4000))
    }
    #[inline]
    pub(crate) fn separate_bits_tmp(bits: u16) -> (i8, u16) {
        let mut k = 0_i8;
        let mut tmp = bits << 2;
        if Self::sign_reg_ui(bits) {
            while (tmp >> 15) != 0 {
                k += 1;
                tmp <<= 1;
            }
        } else {
            k = -1;
            while (tmp >> 15) == 0 {
                k -= 1;
                tmp <<= 1;
            }
            tmp &= 0x7FFF;
        }
        (k, tmp)
    }
}

impl PartialOrd for P16E1 {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        (self.to_bits() as i16).partial_cmp(&(other.to_bits() as i16))
    }
}
