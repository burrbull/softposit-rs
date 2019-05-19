use core::mem;

mod convert;
mod math;
mod ops;
#[cfg(feature = "num-traits")]
crate::impl_num_traits!(P32E2);
#[cfg(feature = "linalg")]
crate::impl_quire_dot!(P32E2, Q32E2);

#[cfg(feature = "approx")]
crate::impl_ulps_eq!(P32E2, i32);
#[cfg(feature = "approx")]
use approx::AbsDiffEq;
#[cfg(feature = "approx")]
crate::impl_signed_abs_diff_eq!(P32E2, P32E2::ZERO);
//crate::impl_signed_abs_diff_eq!(P32E2, P32E2::EPSILON);
#[cfg(feature = "approx")]
crate::impl_relative_eq!(P32E2, i32);

#[cfg(feature = "alga")]
crate::impl_lattice!(P32E2);
#[cfg(feature = "alga")]
crate::impl_real!(P32E2);
#[cfg(feature = "alga")]
crate::impl_complex!(P32E2);
#[cfg(feature = "alga")]
crate::impl_alga!(P32E2);
#[cfg(feature = "alga")]
use alga::general::{Additive, Multiplicative};

#[cfg_attr(feature = "alga", derive(alga_derive::Alga))]
#[cfg_attr(feature = "alga", alga_traits(Field(Additive, Multiplicative)))]
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

    /// Not a Real (NaR).
    pub const NAR: Self = Self::new(-0x_8000_0000);

    /// Not a Number (NaN).
    pub const NAN: Self = Self::NAR;

    /// Infinity (∞).
    pub const INFINITY: Self = Self::NAR;

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
        if self.is_sign_negative() {
            -self
        } else {
            self
        }
    }
    #[inline]
    pub fn is_nar(self) -> bool {
        self == Self::NAR
    }
    #[inline]
    pub fn is_nan(self) -> bool {
        self.is_nar()
    }
    #[inline]
    pub fn is_infinite(self) -> bool {
        self.is_nar()
    }
    #[inline]
    pub fn is_finite(self) -> bool {
        !self.is_nar()
    }
    #[inline]
    pub fn is_normal(self) -> bool {
        !self.is_nar()
    }
    #[inline]
    pub fn classify(self) -> core::num::FpCategory {
        use core::num::FpCategory::*;
        match self {
            Self::ZERO => Zero,
            Self::NAN => Nan,
            _ => Normal,
        }
    }
    #[inline]
    pub fn is_sign_positive(self) -> bool {
        !self.is_sign_negative()
    }
    #[inline]
    pub fn is_sign_negative(self) -> bool {
        self < Self::ZERO
    }
    #[inline]
    pub fn copysign(self, other: Self) -> Self {
        if ((self.to_bits() ^ other.to_bits()) & Self::SIGN_MASK) != 0 {
            -self
        } else {
            self
        }
    }
    #[inline]
    pub fn signum(self) -> Self {
        match self.0 {
            n if n == Self::NAN.0 => Self::NAN,
            n if n > 0 => Self::ONE,
            0 => Self::ZERO,
            _ => -Self::ONE,
        }
    }
    // TODO: optimize
    #[inline]
    pub fn recip(self) -> Self {
        Self::ONE / self
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
        self.0 = self.0.wrapping_neg();
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

#[cfg(any(feature = "rand", test))]
impl rand::distributions::Distribution<P32E2> for rand::distributions::Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> P32E2 {
        let s = rng.gen_range(-0x_7fff_ffff_i32, 0x_7fff_ffff);
        P32E2::new(s)
    }
}
