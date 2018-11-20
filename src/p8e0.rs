use core::mem;

mod convert;
mod math;
mod ops;

#[derive(Clone, Copy, Debug, PartialEq)]
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
}

impl PartialOrd for P8E0 {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        (self.to_bits() as i8).partial_cmp(&(other.to_bits() as i8))
    }
}
