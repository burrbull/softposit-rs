use core::mem;

mod convert;
mod math;
mod ops;
crate::impl_num_traits!(P8E0);
#[cfg(feature = "approx")]
crate::impl_ulps_eq!(P8E0, i8);
#[cfg(feature = "approx")]
use approx::AbsDiffEq;
#[cfg(feature = "approx")]
crate::impl_signed_abs_diff_eq!(P8E0, P8E0::ZERO);
//crate::impl_signed_abs_diff_eq!(P8E0, P8E0::EPSILON);
#[cfg(feature = "approx")]
crate::impl_relative_eq!(P8E0, i8);

#[cfg(feature = "alga")]
crate::impl_lattice!(P8E0);
#[cfg(feature = "alga")]
crate::impl_real!(P8E0);
#[cfg(feature = "alga")]
crate::impl_complex!(P8E0);
#[cfg(feature = "alga")]
crate::impl_alga!(P8E0);
#[cfg(feature = "alga")]
use alga::general::{Additive, Multiplicative};

#[cfg_attr(feature = "alga", derive(alga_derive::Alga))]
#[cfg_attr(feature = "alga", alga_traits(Field(Additive, Multiplicative)))]
#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct P8E0(i8);

impl P8E0 {
    pub const SIZE: usize = 8;
    pub const ES: usize = 0;
    pub const USEED: usize = 2;

    /// Machine epsilon (3.125e-2).
    pub const EPSILON: Self = Self::new(0x_2);

    /// Smallest finite value (-64).
    pub const MIN: Self = Self::new(-0x_7F);

    /// Smallest positive normal value (0.015625).
    pub const MIN_POSITIVE: Self = Self::new(0x_1);

    /// Largest finite value (64).
    pub const MAX: Self = Self::new(0x_7F);

    /// Not a Real (NaR).
    pub const NAR: Self = Self::new(-0x_80);

    /// Not a Number (NaN).
    pub const NAN: Self = Self::NAR;

    /// Infinity (∞).
    pub const INFINITY: Self = Self::NAR;

    /// Zero.
    pub const ZERO: Self = Self::new(0);

    /// Identity.
    pub const ONE: Self = Self::new(0x_40);

    #[inline]
    pub const fn new(i: i8) -> Self {
        Self(i)
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
        if self.is_sign_negative() {
            -self
        } else {
            self
        }
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
            Self::NAR => Nan,
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
            n if n == Self::NAR.0 => Self::NAR,
            n if n > 0 => Self::ONE,
            0 => Self::ZERO,
            _ => -Self::ONE,
        }
    }
    #[inline]
    // TODO: optimize
    pub fn recip(self) -> Self {
        Self::ONE / self
    }
}

impl P8E0 {
    pub const SIGN_MASK: u8 = 0x_80;
    pub const REGIME_SIGN_MASK: u8 = 0x_40;

    #[inline]
    pub(crate) fn sign_ui(a: u8) -> bool {
        (a & Self::SIGN_MASK) != 0
    }

    #[inline]
    fn sign_reg_ui(a: u8) -> bool {
        (a & Self::REGIME_SIGN_MASK) != 0
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
            while (tmp & 0x_80) != 0 {
                k += 1;
                tmp <<= 1;
            }
        } else {
            k = -1;
            while (tmp & 0x_80) == 0 {
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
            (0x40_u8.checked_shr(reg as u32).unwrap_or(0), false, reg)
        } else {
            reg = (k + 1) as u8;
            (
                0x7f - 0x7f_u8.checked_shr(reg as u32).unwrap_or(0),
                true,
                reg,
            )
        }
    }
}

#[derive(Clone, Debug)]
pub struct Q8E0(i32);

impl Q8E0 {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(0x_0000_1000);
    pub const NAR: Self = Self(-0x_8000_0000);

    #[inline]
    pub const fn init() -> Self {
        Self::ZERO
    }

    #[inline]
    pub fn from_posit(p: P8E0) -> Self {
        Self::from(p)
    }

    #[inline]
    pub fn from_bits(v: u32) -> Self {
        unsafe { mem::transmute(v) }
    }

    #[inline]
    pub fn to_bits(&self) -> u32 {
        unsafe { mem::transmute(self.clone()) }
    }

    #[inline]
    pub fn is_zero(&self) -> bool {
        self.to_bits() == 0
    }

    #[inline]
    pub fn is_nar(&self) -> bool {
        self.to_bits() == 0x8000_0000
    }

    #[inline]
    pub fn add_product(&mut self, p_a: P8E0, p_b: P8E0) {
        ops::fdp_add(self, p_a, p_b);
    }

    #[inline]
    pub fn sub_product(&mut self, p_a: P8E0, p_b: P8E0) {
        ops::fdp_sub(self, p_a, p_b);
    }

    #[inline]
    pub fn to_posit(&self) -> P8E0 {
        P8E0::from(self)
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

impl crate::Quire<P8E0> for Q8E0 {
    type Bits = u32;
    fn init() -> Self {
        Self::init()
    }
    fn from_posit(p: P8E0) -> Self {
        Self::from_posit(p)
    }
    fn to_posit(&self) -> P8E0 {
        Self::to_posit(self)
    }
    fn from_bits(v: Self::Bits) -> Self {
        Self::from_bits(v)
    }
    fn to_bits(&self) -> Self::Bits {
        Self::to_bits(&self)
    }
    fn is_zero(&self) -> bool {
        Self::is_zero(&self)
    }
    fn is_nar(&self) -> bool {
        Self::is_nar(self)
    }
    fn add_product(&mut self, p_a: P8E0, p_b: P8E0) {
        Self::add_product(self, p_a, p_b)
    }
    fn sub_product(&mut self, p_a: P8E0, p_b: P8E0) {
        Self::sub_product(self, p_a, p_b)
    }
    fn clear(&mut self) {
        Self::clear(self)
    }
    fn neg(&mut self) {
        Self::neg(self)
    }
}

impl core::str::FromStr for P8E0 {
    type Err = core::num::ParseFloatError;
    #[inline]
    fn from_str(src: &str) -> Result<Self, core::num::ParseFloatError> {
        Ok(Self::from(f64::from_str(src)?))
    }
}

use core::fmt;
impl fmt::Display for P8E0 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", f64::from(*self))
    }
}

impl fmt::Display for Q8E0 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", f64::from(self.clone().to_posit()))
    }
}

impl fmt::Debug for P8E0 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "P8E0({})", self.0)
    }
}

impl crate::AssociatedQuire<Self> for P8E0 {
    type Q = Q8E0;
}

impl crate::polynom::poly::Poly<Self> for P8E0 {}
impl crate::Polynom<Self> for P8E0 {}

impl crate::polynom::poly::Poly<[Self; 1]> for P8E0 {}
impl crate::Polynom<[Self; 1]> for P8E0 {}
impl crate::polynom::poly::Poly<[Self; 2]> for P8E0 {}
impl crate::Polynom<[Self; 2]> for P8E0 {}
impl crate::polynom::poly::Poly<[Self; 3]> for P8E0 {}
impl crate::Polynom<[Self; 3]> for P8E0 {}
impl crate::polynom::poly::Poly<[Self; 4]> for P8E0 {}
impl crate::Polynom<[Self; 4]> for P8E0 {}

#[cfg(any(feature = "rand", test))]
impl rand::distributions::Distribution<P8E0> for rand::distributions::Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> P8E0 {
        let s = rng.gen_range(-0x_7f_i8, 0x_7f);
        P8E0::new(s)
    }
}
