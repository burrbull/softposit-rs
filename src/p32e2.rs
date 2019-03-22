use core::mem;

pub mod consts;
mod convert;
mod math;
mod ops;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Hash)]
pub struct P32E2(i32);

/// Machine epsilon (7.450580596923828e-9).
pub const EPSILON: P32E2 = P32E2::new(0x_a0_0000);

/// Smallest finite value (-1.329227996_e36).
pub const MIN: P32E2 = P32E2::new(-0x_7FFF_FFFF);

/// Smallest positive normal value (7.523163845_e-37).
pub const MIN_POSITIVE: P32E2 = P32E2::new(0x_1);

/// Largest finite value (1.329227996_e36).
pub const MAX: P32E2 = P32E2::new(0x_7FFF_FFFF);

/// Not a Number (NaN).
pub const NAN: P32E2 = P32E2::new(-0x_8000_0000);

/// Infinity (∞).
pub const INFINITY: P32E2 = P32E2::new(-0x_8000_0000);

impl P32E2 {
    #[inline]
    pub const fn new(i: i32) -> Self {
        P32E2(i)
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
    pub fn to_degrees(self) -> P32E2 {
        const PIS_IN_180: P32E2 = P32E2::new(0x_6729_7707);
        self * PIS_IN_180
    }
    #[inline]
    pub fn to_radians(self) -> P32E2 {
        let value: P32E2 = consts::PI;
        self * (value / P32E2::new(0x_6da0_0000))
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
    pub(crate) fn separate_bits(bits: u32) -> (i8, i32, u32) {
        let (k, tmp) = Self::separate_bits_tmp(bits);
        (
            k,
            (tmp >> 29) as i32,
            ((tmp << 1) | 0x4000_0000) & 0x7FFF_FFFF,
        )
    }

    #[inline]
    pub(crate) fn separate_bits_tmp(bits: u32) -> (i8, u32) {
        let mut k = 0;
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

    /* // Slower
    #[inline]
    pub(crate) fn separate_bits_tmp(bits: u32) -> (i16, u32) {
        let tmp = bits << 1;
        let lz = tmp.leading_zeros() as i16;
        if lz == 0 {
            let lo = (!tmp).leading_zeros() as i16;
            (lo - 1, tmp << lo)
        } else {
            (-lz, (tmp << lz) & 0x7FFF_FFFF)
        }
    }
    */

    #[inline]
    fn calculate_scale(mut bits: u32) -> (u32, u32) {
        let mut scale = 0_u32;
        bits -= 0x4000_0000;
        while (0x2000_0000 & bits) != 0 {
            scale += 4;
            bits = (bits - 0x2000_0000) << 1;
        }
        bits <<= 1; // Skip over termination bit, which is 0.
        if (0x2000_0000 & bits) != 0 {
            scale += 2; // If first exponent bit is 1, increment the scale.
        }
        if (0x1000_0000 & bits) != 0 {
            scale += 1;
        }
        (scale, bits)
    }

    #[inline]
    fn calculate_regime(k: i8) -> (u32, bool, u8) {
        let reg;
        if k < 0 {
            reg = (-k) as u8;
            (
                if reg > 31 { 0 } else { 0x4000_0000_u32 >> reg },
                false,
                reg,
            )
        } else {
            reg = (k + 1) as u8;
            (
                if reg > 31 {
                    0x7FFF_FFFF
                } else {
                    0x7FFF_FFFF - (0x7FFF_FFFF >> reg)
                },
                true,
                reg,
            )
        }
    }
}

#[derive(Clone, Copy)]
pub struct Q32E2(i64, u64, u64, u64, u64, u64, u64, u64);

impl Q32E2 {
    #[inline]
    pub const fn new(
        i: i64,
        u1: u64,
        u2: u64,
        u3: u64,
        u4: u64,
        u5: u64,
        u6: u64,
        u7: u64,
    ) -> Self {
        Q32E2(i, u1, u2, u3, u4, u5, u6, u7)
    }
    #[inline]
    pub fn from_bits(v: [u64; 8]) -> Self {
        unsafe { mem::transmute(v) }
    }
    #[inline]
    pub fn to_bits(self) -> [u64; 8] {
        unsafe { mem::transmute(self) }
    }
    #[inline]
    pub fn is_zero(self) -> bool {
        self.to_bits() == [0, 0, 0, 0, 0, 0, 0, 0]
    }
    #[inline]
    pub fn is_nan(self) -> bool {
        self.to_bits() == [0x8000_0000, 0, 0, 0, 0, 0, 0, 0]
    }
}

impl num_traits::Zero for P32E2 {
    fn zero() -> Self {
        P32E2::new(0)
    }
    fn is_zero(&self) -> bool {
        *self == P32E2::new(0)
    }
}

impl num_traits::One for P32E2 {
    fn one() -> Self {
        P32E2::new(0x_4000_0000)
    }
    fn is_one(&self) -> bool {
        *self == P32E2::new(0x_4000_0000)
    }
}

impl num_traits::Num for P32E2 {
    type FromStrRadixErr = num_traits::ParseFloatError;
    fn from_str_radix(src: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        Ok(Self::from(f64::from_str_radix(src, radix)?))
    }
}

use core::fmt;
impl fmt::Display for P32E2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "p32({})", f32::from(*self))
    }
}
