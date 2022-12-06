use crate::Q16E1;

mod convert;
mod math;
mod ops;
crate::macros::impl_num_traits!(P16E1);
#[cfg(feature = "approx")]
mod impl_approx {
    pub use super::*;
    use approx::AbsDiffEq;
    crate::macros::approx::impl_ulps_eq!(P16E1, i16);
    crate::macros::approx::impl_signed_abs_diff_eq!(P16E1, P16E1::ZERO);
    //crate::impl_signed_abs_diff_eq!(P16E1, P16E1::EPSILON);
    crate::macros::approx::impl_relative_eq!(P16E1, i16);
}

#[cfg(feature = "simba")]
mod impl_simba {
    pub use super::*;
    crate::macros::simba::impl_real!(P16E1);
    crate::macros::simba::impl_complex!(P16E1);
    crate::macros::simba::impl_primitive_simd_value_for_scalar!(P16E1);
    impl simba::scalar::Field for P16E1 {}
}

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct P16E1(i16);

impl P16E1 {
    pub const SIZE: usize = 16;
    pub const ES: usize = 1;
    pub const USEED: usize = 4;

    /// Machine epsilon (2.44140625e-4).
    pub const EPSILON: Self = Self::new(0x_0100);

    /// Smallest finite value (-268435456).
    pub const MIN: Self = Self::new(-0x_7FFF);

    /// Smallest positive normal value (3.725290298_e-9).
    pub const MIN_POSITIVE: Self = Self::new(0x_0001);

    /// Largest finite value (268435456).
    pub const MAX: Self = Self::new(0x_7FFF);

    /// Not a Real (NaR).
    pub const NAR: Self = Self::new(-0x_8000);

    /// Not a Number (NaN).
    pub const NAN: Self = Self::NAR;

    /// Infinity (∞).
    pub const INFINITY: Self = Self::NAR;

    /// Zero.
    pub const ZERO: Self = Self::new(0);

    /// Identity.
    pub const ONE: Self = Self::new(0x_4000);

    #[inline]
    pub const fn new(i: i16) -> Self {
        Self(i)
    }
    #[inline]
    pub const fn from_bits(v: u16) -> Self {
        Self(v as _)
    }
    #[inline]
    pub const fn to_bits(self) -> u16 {
        self.0 as _
    }
    // TODO: optimize
    #[inline]
    pub const fn recip(self) -> Self {
        Self::ONE.div(self)
    }
    #[inline]
    pub const fn to_degrees(self) -> Self {
        const PIS_IN_180: P16E1 = P16E1::new(0x_7729);
        self.mul(PIS_IN_180)
    }
    #[inline]
    pub const fn to_radians(self) -> Self {
        const PIS_O_180: P16E1 = P16E1::new(0x_0878);
        self.mul(PIS_O_180)
    }
}

crate::macros::impl_const_fns!(P16E1);

impl P16E1 {
    pub const SIGN_MASK: u16 = 0x_8000;
    pub const REGIME_SIGN_MASK: u16 = 0x_4000;

    #[inline]
    pub(crate) const fn sign_ui(a: u16) -> bool {
        (a & Self::SIGN_MASK) != 0
    }

    #[inline]
    const fn sign_reg_ui(a: u16) -> bool {
        (a & Self::REGIME_SIGN_MASK) != 0
    }

    #[inline]
    pub(crate) const fn pack_to_ui(regime: u16, reg: u32, exp: u16, frac: u16) -> u16 {
        regime + (if reg == 14 { 0 } else { exp << (13 - reg) }) + frac
    }

    #[inline]
    pub(crate) const fn separate_bits(bits: u16) -> (i8, i8, u16) {
        let (k, tmp) = Self::separate_bits_tmp(bits);
        (
            k,
            (tmp >> (Self::SIZE - 1 - Self::ES)) as i8,
            (tmp | 0x4000),
        )
    }
    #[inline]
    pub(crate) const fn separate_bits_tmp(bits: u16) -> (i8, u16) {
        let mut k = 0;
        let mut tmp = bits << 2;
        if Self::sign_reg_ui(bits) {
            while (tmp & 0x_8000) != 0 {
                k += 1;
                tmp <<= 1;
            }
        } else {
            k = -1;
            while (tmp & 0x_8000) == 0 {
                k -= 1;
                tmp <<= 1;
            }
            tmp &= 0x7FFF;
        }
        (k, tmp)
    }

    #[inline]
    const fn calculate_scale(mut bits: u16) -> (u16, u16) {
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
    pub(crate) const fn calculate_regime(k: i8) -> (u16, bool, u32) {
        let len;
        if k < 0 {
            len = (-k) as u32;
            (0x4000_u16.wrapping_shr(len), false, len)
        } else {
            len = (k + 1) as u32;
            (0x7fff - 0x7fff_u16.wrapping_shr(len), true, len)
        }
    }
}

impl core::str::FromStr for P16E1 {
    type Err = core::num::ParseFloatError;
    #[inline]
    fn from_str(src: &str) -> Result<Self, core::num::ParseFloatError> {
        Ok(Self::from(f64::from_str(src)?))
    }
}

use core::{cmp::Ordering, fmt};
impl fmt::Display for P16E1 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", f64::from(*self))
    }
}

impl fmt::Debug for P16E1 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "P16E1({})", self.0)
    }
}

impl crate::AssociatedQuire<Self> for P16E1 {
    type Q = Q16E1;
}

impl crate::polynom::poly::Poly<Self> for P16E1 {}
impl crate::Polynom<Self> for P16E1 {}

impl crate::polynom::poly::Poly<[Self; 1]> for P16E1 {}
impl crate::Polynom<[Self; 1]> for P16E1 {}
impl crate::polynom::poly::Poly<[Self; 2]> for P16E1 {}
impl crate::Polynom<[Self; 2]> for P16E1 {}
impl crate::polynom::poly::Poly<[Self; 3]> for P16E1 {}
impl crate::Polynom<[Self; 3]> for P16E1 {}
impl crate::polynom::poly::Poly<[Self; 4]> for P16E1 {}
impl crate::Polynom<[Self; 4]> for P16E1 {}

#[cfg(any(feature = "rand", test))]
impl rand::distributions::Distribution<P16E1> for rand::distributions::Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> P16E1 {
        P16E1::sub_one(rng.gen_range(0_u32..0x_4_0000))
        /*let s = rng.gen_range(0_u16, 0x_1000) | 0x4000;
        let s2 = rng.gen_range(0_u16, 4);
        let b = (P16E1::from_bits(s) - P16E1::ONE).to_bits();
        P16E1::from_bits(b & 0xfffc | s2)*/
    }
}

#[cfg(any(feature = "rand", test))]
impl P16E1 {
    fn sub_one(ui_a: u32) -> Self {
        if ui_a & 0x_f_fff8 == 0 {
            return Self::ZERO;
        }

        let mut frac32 = ui_a << 12;

        let mut reg_len = 0;
        while (frac32 >> 29) == 0 {
            reg_len += 1;
            frac32 <<= 2;
        }

        let ecarry = (0x4000_0000 & frac32) != 0;
        let mut exp_a = 0;
        if !ecarry {
            reg_len += 1;
            exp_a = 1;
            frac32 <<= 1;
        }

        let regime = 0x4000_u16.wrapping_shr(reg_len);

        let u_z = if reg_len > 14 {
            0x1
        } else {
            //remove hidden bits
            Self::form_ui(
                reg_len,
                regime,
                exp_a,
                (frac32 & 0x3FFF_FFFF) >> (reg_len + 1),
            )
        };
        Self::from_bits(u_z)
    }
}

impl crate::RawPosit for P16E1 {
    type UInt = u16;
    type Int = i16;

    const BITSIZE: usize = 16;

    const EXPONENT_BITS: usize = 1;
    const EXPONENT_MASK: Self::UInt = 0x1;
}
