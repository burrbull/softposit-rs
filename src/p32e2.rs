use core::mem;

mod convert;
mod math;
mod ops;

const UP_SIGN: u32 = 0x_8000_0000;
const UP_REGSIGN: u32 = 0x_4000_0000;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Hash)]
pub struct P32E2(i32);

/// Machine epsilon (7.450580596923828e-9).
pub const EPSILON: P32E2 = P32E2::new(0x_00a0_0000);

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
        let value: P32E2 = Self::PI;
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
        (a & UP_SIGN) != 0
    }

    #[inline]
    fn sign_reg_ui(a: u32) -> bool {
        (a & UP_REGSIGN) != 0
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

use num_traits::Zero;
impl Zero for P32E2 {
    fn zero() -> Self {
        P32E2::new(0)
    }
    fn is_zero(&self) -> bool {
        *self == P32E2::new(0)
    }
}

use num_traits::One;
impl One for P32E2 {
    #[inline]
    fn one() -> Self {
        P32E2::new(0x_4000_0000)
    }
    #[inline]
    fn is_one(&self) -> bool {
        *self == Self::one()
    }
}

impl num_traits::Num for P32E2 {
    type FromStrRadixErr = num_traits::ParseFloatError;
    fn from_str_radix(src: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        Ok(Self::from(f64::from_str_radix(src, radix)?))
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
        write!(f, "p32({})", f32::from(*self))
    }
}

impl num_traits::ToPrimitive for P32E2 {
    fn to_i64(&self) -> Option<i64> {
        Some(i64::from(*self))
    }
    fn to_u64(&self) -> Option<u64> {
        Some(u64::from(*self))
    }
    fn to_f64(&self) -> Option<f64> {
        Some(f64::from(*self))
    }
}

impl num_traits::NumCast for P32E2 {
    fn from<N: num_traits::ToPrimitive>(n: N) -> Option<Self> {
        n.to_f64().map(|x| x.into())
    }
}

impl num_traits::Signed for P32E2 {
    fn abs(&self) -> Self {
        Self::abs(*self)
    }
    fn abs_sub(&self, other: &Self) -> Self {
        if *self <= *other {
            Self::zero()
        } else {
            *self - *other
        }
    }
    fn signum(&self) -> Self {
        unimplemented!()
    }
    fn is_positive(&self) -> bool {
        !self.is_negative()
    }
    fn is_negative(&self) -> bool {
        unimplemented!()
    }
}

impl num_traits::Float for P32E2 {
    fn nan() -> Self {
        NAN
    }
    fn infinity() -> Self {
        INFINITY
    }
    fn neg_infinity() -> Self {
        INFINITY
    }
    fn neg_zero() -> Self {
        Self::zero()
    }
    fn min_value() -> Self {
        MIN
    }
    fn min_positive_value() -> Self {
        MIN_POSITIVE
    }
    fn max_value() -> Self {
        MAX
    }
    fn is_nan(self) -> bool {
        self == NAN
    }
    fn is_infinite(self) -> bool {
        self == INFINITY
    }
    fn is_finite(self) -> bool {
        !self.is_nan()
    }
    fn is_normal(self) -> bool {
        unimplemented!()
    }
    fn classify(self) -> core::num::FpCategory {
        unimplemented!()
    }
    fn floor(self) -> Self {
        unimplemented!()
    }
    fn ceil(self) -> Self {
        unimplemented!()
    }
    fn round(self) -> Self {
        math::round(self)
    }
    fn trunc(self) -> Self {
        unimplemented!()
    }
    fn fract(self) -> Self {
        unimplemented!()
    }
    fn abs(self) -> Self {
        Self::abs(self)
    }
    fn signum(self) -> Self {
        if self.is_nan() {
            Self::nan()
        } else if self.is_sign_negative() {
            -Self::one()
        } else {
            Self::one()
        }
    }
    fn is_sign_positive(self) -> bool {
        !self.is_sign_negative()
    }
    fn is_sign_negative(self) -> bool {
        self.to_bits() >> 31 != 0
    }
    fn mul_add(self, a: Self, b: Self) -> Self {
        math::mul_add(self.to_bits(), a.to_bits(), b.to_bits(), crate::MulAddType::Add)
    }
    fn recip(self) -> Self {
        unimplemented!()
    }
    fn powi(self, _n: i32) -> Self {
        unimplemented!()
    }
    fn powf(self, _n: Self) -> Self {
        unimplemented!()
    }
    fn sqrt(self) -> Self {
        math::sqrt(self)
    }
    fn exp(self) -> Self {
        unimplemented!()
    }
    fn exp2(self) -> Self {
        unimplemented!()
    }
    fn ln(self) -> Self {
        unimplemented!()
    }
    fn log(self, _base: Self) -> Self {
        unimplemented!()
    }
    fn log2(self) -> Self {
        unimplemented!()
    }
    fn log10(self) -> Self {
        unimplemented!()
    }
    fn max(self, other: Self) -> Self {
        self.max(other)
    }
    fn min(self, other: Self) -> Self {
        self.min(other)
    }
    fn abs_sub(self, _other: Self) -> Self {
        unimplemented!()
    }
    fn cbrt(self) -> Self {
        unimplemented!()
    }
    fn hypot(self, _other: Self) -> Self {
        unimplemented!()
    }
    fn sin(self) -> Self {
        unimplemented!()
    }
    fn cos(self) -> Self {
        unimplemented!()
    }
    fn tan(self) -> Self {
        unimplemented!()
    }
    fn asin(self) -> Self {
        unimplemented!()
    }
    fn acos(self) -> Self {
        unimplemented!()
    }
    fn atan(self) -> Self {
        unimplemented!()
    }
    fn atan2(self, _other: Self) -> Self {
        unimplemented!()
    }
    fn sin_cos(self) -> (Self, Self) {
        unimplemented!()
    }
    fn exp_m1(self) -> Self {
        unimplemented!()
    }
    fn ln_1p(self) -> Self {
        unimplemented!()
    }
    fn sinh(self) -> Self {
        unimplemented!()
    }
    fn cosh(self) -> Self {
        unimplemented!()
    }
    fn tanh(self) -> Self {
        unimplemented!()
    }
    fn asinh(self) -> Self {
        unimplemented!()
    }
    fn acosh(self) -> Self {
        unimplemented!()
    }
    fn atanh(self) -> Self {
        unimplemented!()
    }
    fn integer_decode(self) -> (u64, i16, i8) {
        unimplemented!()
    }
}

use crate::MathConsts;
impl MathConsts for P32E2 {
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

impl num_traits::FloatConst for P32E2 {
    fn E() -> Self {
        MathConsts::E
    }
    fn FRAC_1_PI() -> Self {
        MathConsts::FRAC_1_PI
    }
    fn FRAC_1_SQRT_2() -> Self {
        MathConsts::FRAC_1_SQRT_2
    }
    fn FRAC_2_PI() -> Self {
        MathConsts::FRAC_2_PI
    }
    fn FRAC_2_SQRT_PI() -> Self {
        MathConsts::FRAC_2_SQRT_PI
    }
    fn FRAC_PI_2() -> Self {
        MathConsts::FRAC_PI_2
    }
    fn FRAC_PI_3() -> Self {
        MathConsts::FRAC_PI_3
    }
    fn FRAC_PI_4() -> Self {
        MathConsts::FRAC_PI_4
    }
    fn FRAC_PI_6() -> Self {
        MathConsts::FRAC_PI_6
    }
    fn FRAC_PI_8() -> Self {
        MathConsts::FRAC_PI_8
    }
    fn LN_10() -> Self {
        MathConsts::LN_10
    }
    fn LN_2() -> Self {
        MathConsts::LN_2
    }
    fn LOG10_E() -> Self {
        MathConsts::LOG10_E
    }
    fn LOG2_E() -> Self {
        MathConsts::LOG2_E
    }
    fn PI() -> Self {
        MathConsts::PI
    }
    fn SQRT_2() -> Self {
        MathConsts::SQRT_2
    }
}

impl num_traits::Bounded for P32E2 {
    fn min_value() -> Self {
        MIN
    }
    fn max_value() -> Self {
        MAX
    }
}
