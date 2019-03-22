use core::mem;

mod convert;
mod math;
mod ops;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Hash)]
pub struct P8E0(i8);

/// Machine epsilon (3.125e-2).
pub const EPSILON: P8E0 = P8E0::new(0x_2);

/// Smallest finite value (-64).
pub const MIN: P8E0 = P8E0::new(-0x_7F);

/// Smallest positive normal value (0.015625).
pub const MIN_POSITIVE: P8E0 = P8E0::new(0x_1);

/// Largest finite value (64).
pub const MAX: P8E0 = P8E0::new(0x_7F);

/// Not a Number (NaN).
pub const NAN: P8E0 = P8E0::new(-0x_80);

/// Infinity (∞).
pub const INFINITY: P8E0 = P8E0::new(-0x_80);

impl P8E0 {
    #[inline]
    pub const fn new(i: i8) -> Self {
        P8E0(i)
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
    #[inline]
    pub fn is_nan(self) -> bool {
        self == NAN
    }
    #[inline]
    pub fn is_infinite(self) -> bool {
        self == INFINITY
    }
    #[inline]
    pub fn is_finite(self) -> bool {
        !self.is_nan()
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
        let mut k = 0;
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

    #[inline]
    fn calculate_scale(mut bits: u8) -> (u8, u8) {
        let mut scale = 0_u8;
        // Decode the posit, left-justifying as we go.
        bits -= 0x40; // Strip off first regime bit (which is a 1).
        while (0x20 & bits) != 0 {
            // Increment scale one for each regime sign bit.
            scale += 1; // Regime sign bit is always 1 in this range.
            bits = (bits - 0x20) << 1; // Remove the bit; line up the next regime bit.
        }
        bits <<= 1; // Skip over termination bit, which is 0.
        (scale, bits)
    }

    #[inline]
    fn calculate_regime(k: i8) -> (u8, bool, u8) {
        let reg;
        if k < 0 {
            reg = (-k) as u8;
            (if reg > 7 { 0 } else { 0x40 >> reg }, false, reg)
        } else {
            reg = (k + 1) as u8;
            (if reg > 7 { 0x7F } else { 0x7F - (0x7F >> reg) }, true, reg)
        }
    }
}

#[derive(Clone, Copy)]
pub struct Q8E0(i32);

impl Q8E0 {
    #[inline]
    pub const fn new(i: i32) -> Self {
        Q8E0(i)
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
    pub fn is_zero(self) -> bool {
        self.to_bits() == 0
    }
    #[inline]
    pub fn is_nan(self) -> bool {
        self.to_bits() == 0x8000_0000
    }
}

impl num_traits::Zero for P8E0 {
    fn zero() -> Self {
        P8E0::new(0)
    }
    fn is_zero(&self) -> bool {
        *self == P8E0::new(0)
    }
}

impl num_traits::Num for P8E0 {
    type FromStrRadixErr = num_traits::ParseFloatError;
    fn from_str_radix(src: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        Ok(Self::from(f64::from_str_radix(src, radix)?))
    }
}



impl num_traits::One for P8E0 {
    fn one() -> Self {
        P8E0::new(0x_40)
    }
    fn is_one(&self) -> bool {
        *self == P8E0::new(0x_40)
    }
}
