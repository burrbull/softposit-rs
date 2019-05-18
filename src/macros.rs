#[cfg(feature = "num-traits")]
#[macro_export]
macro_rules! impl_num_traits {
    ($posit:ty) => {
        impl num_traits::Zero for $posit {
            fn zero() -> Self {
                Self::ZERO
            }
            fn is_zero(&self) -> bool {
                *self == Self::ZERO
            }
        }

        impl num_traits::One for $posit {
            #[inline]
            fn one() -> Self {
                Self::ONE
            }
            #[inline]
            fn is_one(&self) -> bool {
                *self == Self::ONE
            }
        }

        impl num_traits::Num for $posit {
            type FromStrRadixErr = num_traits::ParseFloatError;
            fn from_str_radix(src: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
                Ok(Self::from(f64::from_str_radix(src, radix)?))
            }
        }

        impl num_traits::ToPrimitive for $posit {
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

        impl num_traits::NumCast for $posit {
            fn from<N: num_traits::ToPrimitive>(n: N) -> Option<Self> {
                n.to_f64().map(|x| x.into())
            }
        }

        impl num_traits::Signed for $posit {
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
                Self::signum(*self)
            }
            fn is_positive(&self) -> bool {
                !self.is_negative()
            }
            fn is_negative(&self) -> bool {
                self.0 < 0
            }
        }

        impl num_traits::Float for $posit {
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
                !self.is_nan()
            }
            fn classify(self) -> core::num::FpCategory {
                Self::classify(self)
            }
            fn floor(self) -> Self {
                Self::floor(self)
            }
            fn ceil(self) -> Self {
                Self::ceil(self)
            }
            fn round(self) -> Self {
                Self::round(self)
            }
            fn trunc(self) -> Self {
                Self::trunc(self)
            }
            fn fract(self) -> Self {
                Self::fract(self)
            }
            fn abs(self) -> Self {
                Self::abs(self)
            }
            fn signum(self) -> Self {
                Self::signum(self)
            }
            fn is_sign_positive(self) -> bool {
                Self::is_sign_positive(self)
            }
            fn is_sign_negative(self) -> bool {
                Self::is_sign_negative(self)
            }
            fn mul_add(self, a: Self, b: Self) -> Self {
                Self::mul_add(self, a, b)
            }
            fn recip(self) -> Self {
                Self::recip(self)
            }
            fn powi(self, n: i32) -> Self {
                Self::powi(self, n)
            }
            fn powf(self, n: Self) -> Self {
                Self::powf(self, n)
            }
            fn sqrt(self) -> Self {
                Self::sqrt(self)
            }
            fn exp(self) -> Self {
                Self::exp(self)
            }
            fn exp2(self) -> Self {
                Self::exp2(self)
            }
            fn ln(self) -> Self {
                Self::ln(self)
            }
            fn log(self, base: Self) -> Self {
                Self::log(self, base)
            }
            fn log2(self) -> Self {
                Self::log2(self)
            }
            fn log10(self) -> Self {
                Self::log10(self)
            }
            fn max(self, other: Self) -> Self {
                core::cmp::Ord::max(self, other)
            }
            fn min(self, other: Self) -> Self {
                core::cmp::Ord::min(self, other)
            }
            fn abs_sub(self, _other: Self) -> Self {
                unimplemented!()
            }
            fn cbrt(self) -> Self {
                Self::cbrt(self)
            }
            fn hypot(self, other: Self) -> Self {
                Self::hypot(self, other)
            }
            fn sin(self) -> Self {
                Self::sin(self)
            }
            fn cos(self) -> Self {
                Self::cos(self)
            }
            fn tan(self) -> Self {
                Self::tan(self)
            }
            fn asin(self) -> Self {
                Self::asin(self)
            }
            fn acos(self) -> Self {
                Self::acos(self)
            }
            fn atan(self) -> Self {
                Self::atan(self)
            }
            fn atan2(self, other: Self) -> Self {
                Self::atan2(self, other)
            }
            fn sin_cos(self) -> (Self, Self) {
                Self::sin_cos(self)
            }
            fn exp_m1(self) -> Self {
                Self::exp_m1(self)
            }
            fn ln_1p(self) -> Self {
                Self::ln_1p(self)
            }
            fn sinh(self) -> Self {
                Self::sinh(self)
            }
            fn cosh(self) -> Self {
                Self::cosh(self)
            }
            fn tanh(self) -> Self {
                Self::tanh(self)
            }
            fn asinh(self) -> Self {
                Self::asinh(self)
            }
            fn acosh(self) -> Self {
                Self::acosh(self)
            }
            fn atanh(self) -> Self {
                Self::atanh(self)
            }
            fn integer_decode(self) -> (u64, i16, i8) {
                unimplemented!()
            }
        }

        use crate::MathConsts;
        impl num_traits::FloatConst for $posit {
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

        impl num_traits::Bounded for $posit {
            fn min_value() -> Self {
                Self::MIN
            }
            fn max_value() -> Self {
                Self::MAX
            }
        }
    };
}

#[cfg(feature = "linalg")]
#[macro_export]
macro_rules! impl_quire_dot {
    ($posit:ty, $quire:ty) => {
        use nalgebra::{
            base::{
                allocator::Allocator,
                constraint::{AreMultipliable, ShapeConstraint},
                storage::Storage,
            },
            DefaultAllocator, Dim, Matrix, MatrixMN,
        };

        impl<'b, R1: Dim, C1: Dim, R2: Dim, C2: Dim, SA, SB> crate::QuireDot<&'b Matrix<$posit, R2, C2, SB>>
            for Matrix<$posit, R1, C1, SA>
        where
            SB: Storage<$posit, R2, C2>,
            SA: Storage<$posit, R1, C1>,
            DefaultAllocator: Allocator<$posit, R1, C2>,
            ShapeConstraint: AreMultipliable<R1, C1, R2, C2>,
        {
            type Output = MatrixMN<$posit, R1, C2>;
            fn quire_dot(&self, rhs: &'b Matrix<$posit, R2, C2, SB>) -> Self::Output {
                let mut out =
                    unsafe { Matrix::new_uninitialized_generic(self.data.shape().0, rhs.data.shape().1) };
                for (i, mut row) in out.row_iter_mut().enumerate() {
                    for (j, elem) in row.iter_mut().enumerate() {
                        let mut quire = <$quire>::new();
                        for (a, b) in self.row(i).iter().zip(rhs.column(j).iter()) {
                            quire += (*a, *b);
                        }
                        *elem = quire.into()
                    }
                }
                out
            }
        }
    };
}

#[cfg(feature = "alga")]
#[macro_export]
macro_rules! impl_lattice(
    ($($T:ident),*) => {$(
        impl alga::general::MeetSemilattice for $T {
            #[inline]
            fn meet(&self, other: &Self) -> Self {
                if *self <= *other {
                    *self
                }
                else {
                    *other
                }
            }
        }

        impl alga::general::JoinSemilattice for $T {
            #[inline]
            fn join(&self, other: &Self) -> Self {
                if *self >= *other {
                    *self
                }
                else {
                    *other
                }
            }
        }

        impl alga::general::Lattice for $T {
            #[inline]
            fn meet_join(&self, other: &Self) -> (Self, Self) {
                if *self >= *other {
                    (*other, *self)
                }
                else {
                    (*self, *other)
                }
            }
        }
    )*}
);


#[cfg(feature = "approx")]
#[macro_export]
macro_rules! impl_ulps_eq {
    ($T:ident, $U:ident) => {
        impl approx::UlpsEq for $T {
            #[inline]
            fn default_max_ulps() -> u32 {
                4
            }

            #[inline]
            fn ulps_eq(&self, other: &$T, epsilon: $T, max_ulps: u32) -> bool {
                // For when the numbers are really close together
                if $T::abs_diff_eq(self, other, epsilon) {
                    return true;
                }

                // Trivial negative sign check
                if self.signum() != other.signum() {
                    return false;
                }

                // ULPS difference comparison
                let int_self: $U = unsafe { core::mem::transmute(*self) };
                let int_other: $U = unsafe { core::mem::transmute(*other) };

                $U::abs(int_self - int_other) <= max_ulps as $U
            }
        }
    };
}

#[cfg(feature = "approx")]
#[macro_export]
macro_rules! impl_signed_abs_diff_eq {
    ($T:ident, $default_epsilon:expr) => {
        impl approx::AbsDiffEq for $T {
            type Epsilon = $T;

            #[inline]
            fn default_epsilon() -> $T {
                $default_epsilon
            }

            #[inline]
            fn abs_diff_eq(&self, other: &$T, epsilon: $T) -> bool {
                $T::abs(*self - *other) <= epsilon
            }
        }
    };
}

#[cfg(feature = "approx")]
#[macro_export]
macro_rules! impl_relative_eq {
    ($T:ident, $U:ident) => {
        impl approx::RelativeEq for $T {
            #[inline]
            fn default_max_relative() -> $T {
                $T::EPSILON
            }

            #[inline]
            fn relative_eq(&self, other: &$T, epsilon: $T, max_relative: $T) -> bool {
                // Handle same infinities
                if self == other {
                    return true;
                }

                // Handle remaining infinities
                if $T::is_infinite(*self) || $T::is_infinite(*other) {
                    return false;
                }

                let abs_diff = $T::abs(*self - *other);

                // For when the numbers are really close together
                if abs_diff <= epsilon {
                    return true;
                }

                let abs_self = $T::abs(*self);
                let abs_other = $T::abs(*other);

                let largest = if abs_other > abs_self {
                    abs_other
                } else {
                    abs_self
                };

                // Use a relative difference comparison
                abs_diff <= largest * max_relative
            }
        }
    };
}
