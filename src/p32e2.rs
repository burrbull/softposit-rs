use core::mem;

mod convert;
#[cfg(feature = "linalg")]
mod linalg;
mod math;
#[cfg(feature = "num-traits")]
mod num;
mod ops;

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Hash)]
pub struct P32E2(i32);

impl P32E2 {
    pub const SIZE: usize = 32;
    pub const ES: usize = 2;

    /// Machine epsilon (7.450580596923828e-9).
    pub const EPSILON: Self = Self::new(0x_00a0_0000);

    /// Smallest finite value (-1.329227996_e36).
    pub const MIN: Self = Self::new(-0x_7FFF_FFFF);

    /// Smallest positive normal value (7.523163845_e-37).
    pub const MIN_POSITIVE: Self = Self::new(0x_1);

    /// Largest finite value (1.329227996_e36).
    pub const MAX: Self = Self::new(0x_7FFF_FFFF);

    /// Not a Number (NaN).
    pub const NAN: Self = Self::new(-0x_8000_0000);

    /// Infinity (∞).
    pub const INFINITY: Self = Self::new(-0x_8000_0000);

    /// Zero.
    pub const ZERO: Self = Self::new(0);

    /// Identity.
    pub const ONE: Self = Self::new(0x_4000_0000);

    #[inline]
    pub const fn new(i: i32) -> Self {
        Self(i)
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
        self == Self::NAN
    }
    #[inline]
    pub fn is_infinite(self) -> bool {
        self == Self::INFINITY
    }
    #[inline]
    pub fn is_finite(self) -> bool {
        !self.is_nan()
    }
    #[inline]
    pub fn to_degrees(self) -> Self {
        const PIS_IN_180: P32E2 = P32E2::new(0x_6729_7707);
        self * PIS_IN_180
    }
    #[inline]
    pub fn to_radians(self) -> Self {
        let value: Self = crate::MathConsts::PI;
        self * (value / Self::new(0x_6da0_0000))
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

#[derive(Clone, Debug)]
pub struct Q32E2(i64, u64, u64, u64, u64, u64, u64, u64);

impl Q32E2 {
    pub const ZERO: Self = Self(0, 0, 0, 0, 0, 0, 0, 0);
    pub const NAN: Self = Self(-0x8000_0000_0000_0000, 0, 0, 0, 0, 0, 0, 0);

    #[inline]
    pub const fn new() -> Self {
        Self::ZERO
    }

    #[inline]
    pub fn from_bits(v: [u64; 8]) -> Self {
        unsafe { mem::transmute(v) }
    }

    #[inline]
    pub fn to_bits(&self) -> [u64; 8] {
        unsafe { mem::transmute(self.clone()) }
    }

    #[inline]
    pub fn is_zero(&self) -> bool {
        self.to_bits() == [0, 0, 0, 0, 0, 0, 0, 0]
    }

    #[inline]
    pub fn is_nan(&self) -> bool {
        self.to_bits() == [0x8000_0000, 0, 0, 0, 0, 0, 0, 0]
    }

    #[inline]
    pub fn qma(&mut self, p_a: P32E2, p_b: P32E2) {
        ops::q32_fdp_add(self, p_a, p_b);
    }

    #[inline]
    pub fn qms(&mut self, p_a: P32E2, p_b: P32E2) {
        ops::q32_fdp_sub(self, p_a, p_b);
    }

    #[inline]
    pub fn roundp(self) -> P32E2 {
        P32E2::from(self)
    }

    #[inline]
    pub fn clear(&mut self) {
        *self = Self::ZERO;
    }

    #[inline]
    pub fn neg(&mut self) {
        self.0 = -(self.0);
    }
}

impl core::str::FromStr for P32E2 {
    type Err = core::num::ParseFloatError;
    #[inline]
    fn from_str(src: &str) -> Result<Self, core::num::ParseFloatError> {
        Ok(Self::from(f64::from_str(src)?))
    }
}

use core::fmt;
impl fmt::Display for P32E2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", f64::from(*self))
    }
}

impl fmt::Display for Q32E2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", f64::from(self.clone().roundp()))
    }
}

impl fmt::Debug for P32E2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "P32E2({})", self.0)
    }
}

impl crate::MathConsts for P32E2 {
    const E: Self = Self::new(0x_4adf_8546);
    const FRAC_1_PI: Self = Self::new(0x_322f_9837);
    const FRAC_1_SQRT_2: Self = Self::new(0x_3b50_4f33);
    const FRAC_2_PI: Self = Self::new(0x_3a2f_9837);
    const FRAC_2_SQRT_PI: Self = Self::new(0x_4106_eba8);
    const FRAC_PI_2: Self = Self::new(0x_4490_fdaa);
    const FRAC_PI_3: Self = Self::new(0x_4060_a91c);
    const FRAC_PI_4: Self = Self::new(0x_3c90_fdaa);
    const FRAC_PI_6: Self = Self::new(0x_3860_a91c);
    const FRAC_PI_8: Self = Self::new(0x_3490_fdaa);
    const LN_10: Self = Self::new(0x_4935_d8de);
    const LN_2: Self = Self::new(0x_3b17_217f);
    const LOG10_E: Self = Self::new(0x_35e5_bd8b);
    const LOG2_E: Self = Self::new(0x_438a_a3b3);
    const PI: Self = Self::new(0x_4c90_fdaa);
    const SQRT_2: Self = Self::new(0x_4350_4f33);
    const LOG2_10: Self = Self::new(0x_4d49_a785);
    const LOG10_2: Self = Self::new(0x_31a2_09a8);
}

impl crate::Quire for P32E2 {
    type Q = Q32E2;
}

impl crate::Poly for P32E2 {
    #[inline]
    fn poly1k(x: Self, c0: Self, c1: Self) -> Self {
        let mut q = Q32E2::new(); // QCLR.S
        q += (c0, x); // QMADD.S
        q += (c1, Self::ONE);
        q.into() // QROUND.S
    }
    #[inline]
    fn poly2k(x: Self, x2: Self, c0: Self, c: &[Self]) -> Self {
        let mut q = Q32E2::new();
        q += (c0, x2);
        q += (c[0], x);
        q += (c[1], Self::ONE);
        q.into()
    }
    #[inline]
    fn poly3k(x: Self, x2: Self, x3: Self, c0: Self, c: &[Self]) -> Self {
        let mut q = Q32E2::new();
        q += (c0, x3);
        q += (c[0], x2);
        q += (c[1], x);
        q += (c[2], Self::ONE);
        q.into()
    }
    #[inline]
    fn poly4k(x: Self, x2: Self, x3: Self, x4: Self, c0: Self, c: &[Self]) -> Self {
        let mut q = Q32E2::new();
        q += (c0, x4);
        q += (c[0], x3);
        q += (c[1], x2);
        q += (c[2], x);
        q += (c[3], Self::ONE);
        q.into()
    }
}

impl crate::Polynom for P32E2 {}

impl rand::distributions::Distribution<P32E2> for rand::distributions::Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> P32E2 {
        let s = rng.gen_range(-0x_7fff_ffff_i32, 0x_7fff_ffff);
        P32E2::new(s)
    }
}
