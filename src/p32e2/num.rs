use super::P32E2;

use num_traits::Zero;
impl Zero for P32E2 {
    fn zero() -> Self {
        Self::ZERO
    }
    fn is_zero(&self) -> bool {
        *self == Self::ZERO
    }
}

use num_traits::One;
impl One for P32E2 {
    #[inline]
    fn one() -> Self {
        Self::ONE
    }
    #[inline]
    fn is_one(&self) -> bool {
        *self == Self::ONE
    }
}

impl num_traits::Num for P32E2 {
    type FromStrRadixErr = num_traits::ParseFloatError;
    fn from_str_radix(src: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        Ok(Self::from(f64::from_str_radix(src, radix)?))
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
            Self::ZERO
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
        Self::NAN
    }
    fn infinity() -> Self {
        Self::INFINITY
    }
    fn neg_infinity() -> Self {
        Self::INFINITY
    }
    fn neg_zero() -> Self {
        Self::ZERO
    }
    fn min_value() -> Self {
        Self::MIN
    }
    fn min_positive_value() -> Self {
        Self::MIN_POSITIVE
    }
    fn max_value() -> Self {
        Self::MAX
    }
    fn is_nan(self) -> bool {
        self == Self::NAN
    }
    fn is_infinite(self) -> bool {
        self == Self::INFINITY
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
        super::math::round(self)
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
        super::math::mul_add(
            self.to_bits(),
            a.to_bits(),
            b.to_bits(),
            crate::MulAddType::Add,
        )
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
        super::math::sqrt(self)
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
        Self::MIN
    }
    fn max_value() -> Self {
        Self::MAX
    }
}
