use crate::{u8_zero_shr, Q8E0};

mod convert;
mod math;
mod ops;
crate::macros::impl_num_traits!(P8E0);
crate::macros::impl_math_consts!(P8E0);
crate::macros::impl_const_fns!(P8E0);

#[cfg(feature = "approx")]
mod impl_approx {
    use super::*;
    use approx::AbsDiffEq;
    crate::macros::approx::impl_ulps_eq!(P8E0, i8);
    crate::macros::approx::impl_signed_abs_diff_eq!(P8E0, P8E0::ZERO);
    //crate::impl_signed_abs_diff_eq!(P8E0, P8E0::EPSILON);
    crate::macros::approx::impl_relative_eq!(P8E0, i8);
}

#[cfg(feature = "simba")]
mod impl_simba {
    pub use super::*;
    crate::macros::simba::impl_real!(P8E0);
    crate::macros::simba::impl_complex!(P8E0);
    crate::macros::simba::impl_primitive_simd_value_for_scalar!(P8E0);
    impl simba::scalar::Field for P8E0 {}
}

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct P8E0(i8);

impl P8E0 {
    pub const BITS: u32 = 8;
    pub const ES: u32 = 0;
    pub const USEED: u32 = 2u32.pow(2u32.pow(Self::ES));

    /// Machine epsilon (3.125e-2).
    pub const EPSILON: Self = Self::new(0x_0002);

    /// Smallest finite value (-64).
    pub const MIN: Self = Self::new(-0x_7F);

    /// Smallest positive normal value (0.015625).
    pub const MIN_POSITIVE: Self = Self::new(0x_0001);

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
    pub const fn from_bits(v: u8) -> Self {
        Self(v as _)
    }
    #[inline]
    pub const fn to_bits(self) -> u8 {
        self.0 as _
    }
    #[inline]
    // TODO: optimize
    pub const fn recip(self) -> Self {
        Self::ONE.div(self)
    }

    #[inline]
    pub(crate) const fn with_sign(self, sign: bool) -> Self {
        if sign {
            self.neg()
        } else {
            self
        }
    }
    /*pub(crate) const fn mask() -> u8 {
        u8::MAX
    }*/
    pub const SIGN_MASK: u8 = 0x_80;
    pub const REGIME_SIGN_MASK: u8 = 0x_40;

    #[inline]
    pub(crate) const fn sign_ui(a: u8) -> bool {
        (a & Self::SIGN_MASK) != 0
    }

    #[inline]
    const fn sign_reg_ui(a: u8) -> bool {
        (a & Self::REGIME_SIGN_MASK) != 0
    }

    #[inline]
    pub(crate) const fn pack_to_ui(regime: u8, frac: u8) -> u8 {
        regime + frac
    }

    #[inline]
    pub(crate) const fn separate_bits(bits: u8) -> (i8, u8) {
        let (k, tmp) = Self::separate_bits_tmp(bits);
        (k, 0x80 | tmp)
    }

    #[inline]
    pub(crate) const fn separate_bits_tmp(bits: u8) -> (i8, u8) {
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
    const fn calculate_scale(mut bits: u8) -> (u8, u8) {
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
    pub(crate) const fn calculate_regime(k: i8) -> (u8, bool, u32) {
        let len;
        if k < 0 {
            len = (-k) as u32;
            (u8_zero_shr(0x40, len), false, len)
        } else {
            len = (k + 1) as u32;
            (0x7f - u8_zero_shr(0x7f, len), true, len)
        }
    }
}

impl core::str::FromStr for P8E0 {
    type Err = core::num::ParseFloatError;
    #[inline]
    fn from_str(src: &str) -> Result<Self, core::num::ParseFloatError> {
        Ok(Self::from(f64::from_str(src)?))
    }
}

use core::{cmp::Ordering, fmt};
impl fmt::Display for P8E0 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", f64::from(*self))
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
        let s = rng.gen_range(0_u8..0x_40);
        P8E0::new(s as i8)
    }
}

impl crate::RawPosit for P8E0 {
    type UInt = u8;
    type Int = i8;
    const ES_MASK: Self::UInt = u8_zero_shr(u8::MAX, u8::BITS - Self::ES);
}

#[cfg(test)]
fn test21_exact(fun: fn(P8E0, P8E0, f64, f64) -> (P8E0, f64)) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS8 {
        let i: i8 = rng.gen();
        let p_a = P8E0::new(i);
        let i: i8 = rng.gen();
        let p_b = P8E0::new(i);
        let f_a = f64::from(p_a);
        let f_b = f64::from(p_b);
        let (answer, f) = fun(p_a, p_b, f_a, f_b);
        let expected = P8E0::from_f64(f);
        assert_eq!(
            answer,
            expected,
            "\n\tinput: ({p_a:?}, {p_b:?})\n\tor: {f_a}, {f_b}\n\tanswer: {}, expected {f}",
            answer.to_f64()
        );
    }
}
