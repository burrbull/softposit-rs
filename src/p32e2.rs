mod convert;
mod math;
mod ops;
crate::macros::impl_num_traits!(P32E2);
crate::macros::impl_math_consts!(P32E2);

#[cfg(feature = "approx")]
mod impl_approx {
    use super::*;
    use approx::AbsDiffEq;
    crate::macros::approx::impl_ulps_eq!(P32E2, i32);
    crate::macros::approx::impl_signed_abs_diff_eq!(P32E2, P32E2::ZERO);
    //crate::impl_signed_abs_diff_eq!(P32E2, P32E2::EPSILON);
    crate::macros::approx::impl_relative_eq!(P32E2, i32);
}

#[cfg(feature = "simba")]
mod impl_simba {
    pub use super::*;
    crate::macros::simba::impl_real!(P32E2);
    crate::macros::simba::impl_complex!(P32E2);
    crate::macros::simba::impl_primitive_simd_value_for_scalar!(P32E2);
    impl simba::scalar::Field for P32E2 {}
}

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct P32E2(i32);

impl P32E2 {
    pub const BITS: u32 = 32;
    pub const ES: u32 = 2;
    pub const USEED: u32 = 2u32.pow(2u32.pow(Self::ES));

    /// Machine epsilon (7.450580596923828e-9).
    pub const EPSILON: Self = Self::new(0x_00a0_0000);

    /// Smallest finite value (-1.329227996_e36).
    pub const MIN: Self = Self::new(-0x_7FFF_FFFF);

    /// Smallest positive normal value (7.523163845_e-37).
    pub const MIN_POSITIVE: Self = Self::new(0x_0001);

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
    pub const fn from_bits(v: u32) -> Self {
        Self(v as _)
    }
    #[inline]
    pub const fn to_bits(self) -> u32 {
        self.0 as _
    }
    // TODO: optimize
    #[inline]
    pub const fn recip(self) -> Self {
        Self::ONE.div(self)
    }
    #[inline]
    pub const fn to_degrees(self) -> Self {
        const PIS_IN_180: P32E2 = P32E2::new(0x_6729_7707);
        self.mul(PIS_IN_180)
    }
    #[inline]
    pub const fn to_radians(self) -> Self {
        const PIS_O_180: P32E2 = P32E2::PI.div(P32E2::new(0x_6da0_0000));
        self.mul(PIS_O_180)
    }
}

crate::macros::impl_const_fns!(P32E2);

impl P32E2 {
    /*pub(crate) const fn mask() -> u32 {
        u32::MAX
    }*/
    pub const SIGN_MASK: u32 = 0x_8000_0000;
    pub const REGIME_SIGN_MASK: u32 = 0x_4000_0000;

    #[inline]
    pub(crate) const fn sign_ui(a: u32) -> bool {
        (a & Self::SIGN_MASK) != 0
    }

    #[inline]
    const fn sign_reg_ui(a: u32) -> bool {
        (a & Self::REGIME_SIGN_MASK) != 0
    }

    #[inline]
    pub(crate) const fn pack_to_ui(regime: u32, exp_a: u32, frac_a: u32) -> u32 {
        regime + exp_a + frac_a
    }

    #[inline]
    pub(crate) const fn separate_bits(bits: u32) -> (i8, i32, u32) {
        let (k, tmp) = Self::separate_bits_tmp(bits);
        (
            k,
            (tmp >> (Self::BITS - 1 - Self::ES)) as i32,
            ((tmp << 1) | 0x4000_0000) & 0x7FFF_FFFF,
        )
    }

    #[inline]
    pub(crate) const fn separate_bits_tmp(bits: u32) -> (i8, u32) {
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
    const fn calculate_scale(mut bits: u32) -> (u32, u32) {
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
    pub(crate) const fn calculate_regime(k: i8) -> (u32, bool, u32) {
        let len;
        if k < 0 {
            len = (-k) as u32;
            (u32_zero_shr(0x4000_0000, len), false, len)
        } else {
            len = (k + 1) as u32;
            (0x7fff_ffff - u32_zero_shr(0x7fff_ffff, len), true, len)
        }
    }
}

impl core::str::FromStr for P32E2 {
    type Err = core::num::ParseFloatError;
    #[inline]
    fn from_str(src: &str) -> Result<Self, core::num::ParseFloatError> {
        Ok(Self::from(f64::from_str(src)?))
    }
}

use core::{cmp::Ordering, fmt};

use crate::u32_zero_shr;
impl fmt::Display for P32E2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", f64::from(*self))
    }
}

impl fmt::Debug for P32E2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "P32E2({})", self.0)
    }
}

impl crate::AssociatedQuire<Self> for P32E2 {
    type Q = crate::Q32E2;
}

impl crate::polynom::poly::Poly<Self> for P32E2 {}
impl crate::Polynom<Self> for P32E2 {}

impl crate::polynom::poly::Poly<[Self; 1]> for P32E2 {}
impl crate::Polynom<[Self; 1]> for P32E2 {}
impl crate::polynom::poly::Poly<[Self; 2]> for P32E2 {}
impl crate::Polynom<[Self; 2]> for P32E2 {}
impl crate::polynom::poly::Poly<[Self; 3]> for P32E2 {}
impl crate::Polynom<[Self; 3]> for P32E2 {}
impl crate::polynom::poly::Poly<[Self; 4]> for P32E2 {}
impl crate::Polynom<[Self; 4]> for P32E2 {}

#[cfg(any(feature = "rand", test))]
impl rand::distributions::Distribution<P32E2> for rand::distributions::Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> P32E2 {
        let s = rng.gen_range(0x_4000_0000_u32..0x_4800_0000);
        let s2 = rng.gen_range(0_u32..4);
        P32E2::from_bits((P32E2::from_bits(s) - P32E2::ONE).to_bits() ^ s2)
    }
}

impl crate::RawPosit for P32E2 {
    type UInt = u32;
    type Int = i32;
    const ES_MASK: Self::UInt = u32::MAX >> (u32::BITS - Self::ES);
}

#[cfg(test)]
fn test21_exact(fun: fn(P32E2, P32E2, f64, f64) -> (P32E2, f64)) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS32 {
        let i: i32 = rng.gen();
        let p_a = P32E2::new(i);
        let i: i32 = rng.gen();
        let p_b = P32E2::new(i);
        let f_a = f64::from(p_a);
        let f_b = f64::from(p_b);
        let (answer, f) = fun(p_a, p_b, f_a, f_b);
        let expected = P32E2::from_f64(f);
        assert_eq!(
            answer,
            expected,
            "\n\tinput: ({p_a:?}, {p_b:?})\n\tor: {f_a}, {f_b}\n\tanswer: {}, expected {f}",
            answer.to_f64()
        );
    }
}
