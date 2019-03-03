use core::mem;

pub mod consts;
mod convert;
mod math;
mod ops;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Hash)]
pub struct P16E1(i16);

pub const ZERO: P16E1 = P16E1::new(0);

/// Machine epsilon (2.44140625e-4).
pub const EPSILON: P16E1 = P16E1::new(0x_100);

/// Smallest finite value (-268435456).
pub const MIN: P16E1 = P16E1::new(-0x_7FFF);

/// Smallest positive normal value (3.725290298_e-9).
pub const MIN_POSITIVE: P16E1 = P16E1::new(0x_1);

/// Largest finite value (268435456).
pub const MAX: P16E1 = P16E1::new(0x_7FFF);

/// Not a Number (NaN).
pub const NAN: P16E1 = P16E1::new(-0x_8000);

/// Infinity (∞).
pub const INFINITY: P16E1 = P16E1::new(-0x_8000);

impl P16E1 {
    #[inline]
    pub const fn new(i: i16) -> Self {
        P16E1(i)
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
    #[inline]
    pub fn is_nan(self) -> bool {
        self == NAN
    }
    #[inline]
    pub fn is_infinite(self) -> bool {
        self == INFINITY
    }
    #[inline]
    pub fn is_zero(self) -> bool {
        self == ZERO
    }
    #[inline]
    pub fn is_finite(self) -> bool {
        !self.is_nan()
    }
    #[inline]
    pub fn to_degrees(self) -> P16E1 {
        const PIS_IN_180: P16E1 = P16E1::new(0x_7729);
        self * PIS_IN_180
    }
    #[inline]
    pub fn to_radians(self) -> P16E1 {
        const PIS_O_180: P16E1 = P16E1::new(0x_0878);
        self * PIS_O_180
    }
    #[inline]
    pub fn max(self, other: Self) -> Self {
        if self.is_nan() || (self < other) {
            other
        } else {
            self
        }
    }
    #[inline]
    pub fn min(self, other: Self) -> Self {
        if other.is_nan() || (self < other) {
            self
        } else {
            other
        }
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
    fn pack_to_ui(regime: u16, reg_a: u8, exp_a: u16, frac_a: u16) -> u16 {
        regime + (exp_a << (13 - reg_a)) + frac_a
    }

    #[inline]
    pub(crate) fn separate_bits(bits: u16) -> (i8, i8, u16) {
        let (k, tmp) = Self::separate_bits_tmp(bits);
        (k, (tmp >> 14) as i8, (tmp | 0x4000))
    }
    #[inline]
    pub(crate) fn separate_bits_tmp(bits: u16) -> (i8, u16) {
        let mut k = 0;
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

    #[inline]
    fn calculate_scale(mut bits: u16) -> (u16, u16) {
        let mut scale = 0_u16;
        // Decode the posit, left-justifying as we go.
        bits -= 0x4000; // Strip off first regime bit (which is a 1).
        while (0x2000 & bits) != 0 {
            // Increment scale by 2 for each regime sign bit.
            scale += 2; // Regime sign bit is always 1 in this range.
            bits = (bits - 0x2000) << 1; // Remove the bit; line up the next regime bit.
        }
        bits <<= 1; // Skip over termination bit, which is 0.
        if (0x2000 & bits) != 0 {
            scale += 1; // If exponent is 1, increment the scale.
        }
        (scale, bits)
    }

    #[inline]
    fn calculate_regime(k: i8) -> (u16, bool, u8) {
        let reg;
        if k < 0 {
            reg = (-k) as u8;
            (if reg > 15 { 0 } else { 0x4000_u16 >> reg }, false, reg)
        } else {
            reg = (k + 1) as u8;
            (
                if reg > 15 {
                    0x7FFF
                } else {
                    0x7FFF - (0x7FFF >> reg)
                },
                true,
                reg,
            )
        }
    }
}

#[derive(Clone, Copy)]
pub struct Q16E1(i64, u64);

impl Q16E1 {
    #[inline]
    pub const fn new(i: i64, u: u64) -> Self {
        Q16E1(i, u)
    }
    #[inline]
    pub fn from_bits(v: [u64; 2]) -> Self {
        unsafe { mem::transmute(v) }
    }
    #[inline]
    pub fn to_bits(self) -> [u64; 2] {
        unsafe { mem::transmute(self) }
    }
    #[inline]
    pub fn is_zero(self) -> bool {
        self.to_bits() == [0, 0]
    }
    #[inline]
    pub fn is_nan(self) -> bool {
        self.to_bits() == [0x8000_0000, 0]
    }
}
