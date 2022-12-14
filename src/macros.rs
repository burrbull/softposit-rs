macro_rules! impl_math_consts {
    ($T:ty) => {
        impl $crate::MathConsts for $T {
            const E: Self = Self::from_f64(core::f64::consts::E);
            const FRAC_1_PI: Self = Self::from_f64(core::f64::consts::FRAC_1_PI);
            const FRAC_1_SQRT_2: Self = Self::from_f64(core::f64::consts::FRAC_1_SQRT_2);
            const FRAC_2_PI: Self = Self::from_f64(core::f64::consts::FRAC_2_PI);
            const FRAC_2_SQRT_PI: Self = Self::from_f64(core::f64::consts::FRAC_2_SQRT_PI);
            const FRAC_PI_2: Self = Self::from_f64(core::f64::consts::FRAC_PI_2);
            const FRAC_PI_3: Self = Self::from_f64(core::f64::consts::FRAC_PI_3);
            const FRAC_PI_4: Self = Self::from_f64(core::f64::consts::FRAC_PI_4);
            const FRAC_PI_6: Self = Self::from_f64(core::f64::consts::FRAC_PI_6);
            const FRAC_PI_8: Self = Self::from_f64(core::f64::consts::FRAC_PI_8);
            const LN_10: Self = Self::from_f64(core::f64::consts::LN_10);
            const LN_2: Self = Self::from_f64(core::f64::consts::LN_2);
            const LOG10_E: Self = Self::from_f64(core::f64::consts::LOG10_E);
            const LOG2_E: Self = Self::from_f64(core::f64::consts::LOG2_E);
            const PI: Self = Self::from_f64(core::f64::consts::PI);
            const SQRT_2: Self = Self::from_f64(core::f64::consts::SQRT_2);
            const LOG2_10: Self = Self::from_f64(core::f64::consts::LOG2_10);
            const LOG10_2: Self = Self::from_f64(core::f64::consts::LOG10_2);
        }
    };
}
pub(crate) use impl_math_consts;

macro_rules! impl_const_fns {
    ($T:ty) => {
        impl $T {
            #[inline]
            pub const fn abs(self) -> Self {
                if self.is_sign_negative() {
                    self.neg()
                } else {
                    self
                }
            }
            #[inline]
            pub const fn is_zero(self) -> bool {
                self.eq(Self::ZERO)
            }
            #[inline]
            pub const fn is_nar(self) -> bool {
                self.eq(Self::NAR)
            }
            #[inline]
            pub const fn is_nan(self) -> bool {
                self.is_nar()
            }
            #[inline]
            pub const fn is_infinite(self) -> bool {
                self.is_nar()
            }
            #[inline]
            pub const fn is_finite(self) -> bool {
                !self.is_nar()
            }
            #[inline]
            pub const fn is_normal(self) -> bool {
                !self.is_nar()
            }
            #[inline]
            pub const fn clamp(mut self, min: Self, max: Self) -> Self {
                assert!(min.le(max));
                if self.lt(min) {
                    self = min;
                }
                if self.gt(max) {
                    self = max;
                }
                self
            }
            #[inline]
            pub const fn min(self, other: Self) -> Self {
                if self.lt(other) {
                    self
                } else {
                    other
                }
            }
            #[inline]
            pub const fn max(self, other: Self) -> Self {
                if self.gt(other) {
                    self
                } else {
                    other
                }
            }
            #[inline]
            pub const fn classify(self) -> core::num::FpCategory {
                use core::num::FpCategory::*;
                match self {
                    Self::ZERO => Zero,
                    Self::NAR => Nan,
                    _ => Normal,
                }
            }
            #[inline]
            pub const fn is_sign_positive(self) -> bool {
                !self.is_sign_negative()
            }
            #[inline]
            pub const fn is_sign_negative(self) -> bool {
                self.lt(Self::ZERO)
            }
            #[inline]
            pub const fn signum(self) -> Self {
                match self.0 {
                    n if n == Self::NAR.0 => Self::NAR,
                    n if n > 0 => Self::ONE,
                    0 => Self::ZERO,
                    _ => Self::ONE.neg(),
                }
            }
            #[inline]
            pub const fn copysign(self, other: Self) -> Self {
                if ((self.to_bits() ^ other.to_bits()) & Self::SIGN_MASK) != 0 {
                    self.neg()
                } else {
                    self
                }
            }
            #[inline]
            pub const fn eq(self, other: Self) -> bool {
                self.0 == other.0
            }
            #[inline]
            pub const fn cmp(self, other: Self) -> Ordering {
                let a = self.0;
                let b = other.0;
                if a == b {
                    Ordering::Equal
                } else if a < b {
                    Ordering::Less
                } else {
                    Ordering::Greater
                }
            }
            #[inline]
            pub const fn lt(&self, other: Self) -> bool {
                self.0 < other.0
            }
            #[inline]
            pub const fn le(&self, other: Self) -> bool {
                self.0 <= other.0
            }
            #[inline]
            pub const fn ge(&self, other: Self) -> bool {
                self.0 >= other.0
            }
            #[inline]
            pub const fn gt(&self, other: Self) -> bool {
                self.0 > other.0
            }
        }
    };
}
pub(crate) use impl_const_fns;

macro_rules! impl_ops {
    ($T:ty) => {
        impl ops::Neg for $T {
            type Output = Self;
            #[inline]
            fn neg(self) -> Self {
                self.neg()
            }
        }

        impl ops::Add for $T {
            type Output = Self;
            #[inline]
            fn add(self, other: Self) -> Self {
                self.add(other)
            }
        }

        impl ops::Sub for $T {
            type Output = Self;
            #[inline]
            fn sub(self, other: Self) -> Self {
                self.sub(other)
            }
        }

        impl ops::Div for $T {
            type Output = Self;
            #[inline]
            fn div(self, other: Self) -> Self {
                self.div(other)
            }
        }

        impl ops::Mul for $T {
            type Output = Self;
            #[inline]
            fn mul(self, other: Self) -> Self {
                self.mul(other)
            }
        }

        impl ops::Rem for $T {
            type Output = Self;
            fn rem(self, other: Self) -> Self {
                self.rem(other)
            }
        }

        impl ops::AddAssign for $T {
            #[inline]
            fn add_assign(&mut self, other: Self) {
                *self = *self + other
            }
        }

        impl ops::SubAssign for $T {
            #[inline]
            fn sub_assign(&mut self, other: Self) {
                *self = *self - other
            }
        }

        impl ops::MulAssign for $T {
            #[inline]
            fn mul_assign(&mut self, other: Self) {
                *self = *self * other
            }
        }

        impl ops::DivAssign for $T {
            #[inline]
            fn div_assign(&mut self, other: Self) {
                *self = *self / other
            }
        }

        impl ops::RemAssign for $T {
            #[inline]
            fn rem_assign(&mut self, other: Self) {
                *self = *self % other
            }
        }
    };
}
pub(crate) use impl_ops;

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
                Some((*self).to_i64())
            }
            fn to_u64(&self) -> Option<u64> {
                Some((*self).to_u64())
            }
            fn to_f64(&self) -> Option<f64> {
                Some((*self).to_f64())
            }
        }

        impl num_traits::NumCast for $posit {
            fn from<N: num_traits::ToPrimitive>(n: N) -> Option<Self> {
                n.to_f64().map(|x| x.into())
            }
        }

        impl num_traits::FromPrimitive for $posit {
            #[inline]
            fn from_i8(n: i8) -> Option<$posit> {
                Some(Self::from_i8(n))
            }
            #[inline]
            fn from_i16(n: i16) -> Option<$posit> {
                Some(Self::from_i16(n))
            }
            #[inline]
            fn from_i32(n: i32) -> Option<$posit> {
                Some(Self::from_i32(n))
            }
            #[inline]
            fn from_i64(n: i64) -> Option<$posit> {
                Some(Self::from_i64(n))
            }

            #[inline]
            fn from_u8(n: u8) -> Option<$posit> {
                Some(Self::from_u8(n))
            }
            #[inline]
            fn from_u16(n: u16) -> Option<$posit> {
                Some(Self::from_u16(n))
            }
            #[inline]
            fn from_u32(n: u32) -> Option<$posit> {
                Some(Self::from_u32(n))
            }
            #[inline]
            fn from_u64(n: u64) -> Option<$posit> {
                Some(Self::from_u64(n))
            }

            #[inline]
            fn from_f32(f: f32) -> Option<$posit> {
                Some(Self::from_f32(f))
            }
            #[inline]
            fn from_f64(f: f64) -> Option<$posit> {
                Some(Self::from_f64(f))
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
                Self::NAR
            }
            fn infinity() -> Self {
                Self::NAR
            }
            fn neg_infinity() -> Self {
                Self::NAR
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
                self == Self::NAR
            }
            fn is_infinite(self) -> bool {
                self == Self::NAR
            }
            fn is_finite(self) -> bool {
                !self.is_nar()
            }
            fn is_normal(self) -> bool {
                !self.is_nar()
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
                todo!()
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
                todo!()
            }
        }

        use $crate::MathConsts;
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
pub(crate) use impl_num_traits;

macro_rules! impl_convert {
    ($posit:ty) => {
        impl From<i8> for $posit {
            #[inline]
            fn from(a: i8) -> Self {
                Self::from_i8(a)
            }
        }

        impl From<$posit> for i8 {
            #[inline]
            fn from(a: $posit) -> Self {
                a.to_i8()
            }
        }

        impl From<i16> for $posit {
            #[inline]
            fn from(a: i16) -> Self {
                Self::from_i16(a)
            }
        }

        impl From<$posit> for i16 {
            #[inline]
            fn from(a: $posit) -> Self {
                a.to_i16()
            }
        }

        impl From<isize> for $posit {
            #[inline]
            fn from(a: isize) -> Self {
                Self::from_isize(a)
            }
        }

        impl From<$posit> for isize {
            #[inline]
            fn from(a: $posit) -> Self {
                a.to_isize()
            }
        }

        impl From<u8> for $posit {
            #[inline]
            fn from(a: u8) -> Self {
                Self::from_u8(a)
            }
        }

        impl From<$posit> for u8 {
            #[inline]
            fn from(a: $posit) -> Self {
                a.to_u8()
            }
        }

        impl From<u16> for $posit {
            #[inline]
            fn from(a: u16) -> Self {
                Self::from_u16(a)
            }
        }

        impl From<$posit> for u16 {
            #[inline]
            fn from(a: $posit) -> Self {
                a.to_u16()
            }
        }

        impl From<usize> for $posit {
            #[inline]
            fn from(a: usize) -> Self {
                Self::from_usize(a)
            }
        }

        impl From<$posit> for usize {
            #[inline]
            fn from(a: $posit) -> Self {
                a.to_usize()
            }
        }

        impl $posit {
            #[inline]
            pub const fn from_i8(a: i8) -> Self {
                Self::from_i32(a as i32)
            }

            #[inline]
            pub const fn to_i8(self) -> i8 {
                self.to_i32() as i8
            }

            #[inline]
            pub const fn from_i16(a: i16) -> Self {
                Self::from_i32(a as i32)
            }

            #[inline]
            pub const fn to_i16(self) -> i16 {
                self.to_i32() as i16
            }

            #[inline]
            pub const fn from_isize(a: isize) -> Self {
                Self::from_i64(a as i64)
            }

            #[inline]
            pub const fn to_isize(self) -> isize {
                self.to_i64() as isize
            }

            #[inline]
            pub const fn from_u8(a: u8) -> Self {
                Self::from_u32(a as u32)
            }

            #[inline]
            pub const fn to_u8(self) -> u8 {
                self.to_u32() as u8
            }

            #[inline]
            pub const fn from_u16(a: u16) -> Self {
                Self::from_u32(a as u32)
            }

            #[inline]
            pub const fn to_u16(self) -> u16 {
                self.to_u32() as u16
            }

            #[inline]
            pub const fn from_usize(a: usize) -> Self {
                Self::from_u64(a as u64)
            }

            #[inline]
            pub const fn to_usize(self) -> usize {
                self.to_u64() as usize
            }
        }

        impl From<f32> for $posit {
            #[inline]
            fn from(float: f32) -> Self {
                Self::from_f32(float)
            }
        }

        impl From<f64> for $posit {
            #[inline]
            fn from(float: f64) -> Self {
                Self::from_f64(float)
            }
        }

        impl From<$posit> for f32 {
            #[inline]
            fn from(p_a: $posit) -> Self {
                p_a.to_f32()
            }
        }

        impl From<$posit> for f64 {
            #[inline]
            fn from(p_a: $posit) -> Self {
                p_a.to_f64()
            }
        }

        impl From<$posit> for i32 {
            #[inline]
            fn from(p_a: $posit) -> Self {
                p_a.to_i32()
            }
        }

        impl From<$posit> for i64 {
            #[inline]
            fn from(p_a: $posit) -> Self {
                p_a.to_i64()
            }
        }

        impl From<u32> for $posit {
            #[inline]
            fn from(a: u32) -> Self {
                Self::from_u32(a)
            }
        }

        impl From<i32> for $posit {
            #[inline]
            fn from(i_a: i32) -> Self {
                Self::from_i32(i_a)
            }
        }

        impl From<u64> for $posit {
            #[inline]
            fn from(a: u64) -> Self {
                Self::from_u64(a)
            }
        }

        impl From<i64> for $posit {
            #[inline]
            fn from(i_a: i64) -> Self {
                Self::from_i64(i_a)
            }
        }

        impl From<$posit> for u64 {
            #[inline]
            fn from(p_a: $posit) -> Self {
                p_a.to_u64()
            }
        }

        impl From<$posit> for u32 {
            #[inline]
            fn from(p_a: $posit) -> Self {
                p_a.to_u32()
            }
        }
    };
}
pub(crate) use impl_convert;

macro_rules! quire_add_sub_array {
    ($posit:ty, $quire:ty, $($i:literal),*) => {$(
        impl ops::AddAssign<($posit, [$posit; $i])> for $quire {
            #[inline]
            fn add_assign(&mut self, rhs: ($posit, [$posit; $i])) {
                for p in &rhs.1 {
                    *self += (rhs.0, *p);
                }
            }
        }

        impl ops::SubAssign<($posit, [$posit; $i])> for $quire {
            #[inline]
            fn sub_assign(&mut self, rhs: ($posit, [$posit; $i])) {
                for p in &rhs.1 {
                    *self -= (rhs.0, *p);
                }
            }
        }
    )*}
}
pub(crate) use quire_add_sub_array;

macro_rules! quire_add_sub {
    ($posit:ty, $quire:ty) => {
        impl ops::AddAssign<($posit, $posit)> for $quire {
            #[inline]
            fn add_assign(&mut self, rhs: ($posit, $posit)) {
                let ui_a = (rhs.0).to_bits();
                let ui_b = (rhs.1).to_bits();
                fdp(self, ui_a, ui_b, true);
            }
        }

        impl ops::AddAssign<($posit, ($posit, $posit))> for $quire {
            #[inline]
            fn add_assign(&mut self, rhs: ($posit, ($posit, $posit))) {
                *self += (rhs.0, (rhs.1).0);
                *self += (rhs.0, (rhs.1).1);
            }
        }

        impl ops::AddAssign<($posit, ($posit, $posit, $posit))> for $quire {
            #[inline]
            fn add_assign(&mut self, rhs: ($posit, ($posit, $posit, $posit))) {
                *self += (rhs.0, (rhs.1).0);
                *self += (rhs.0, (rhs.1).1);
                *self += (rhs.0, (rhs.1).2);
            }
        }

        impl ops::AddAssign<$posit> for $quire {
            #[inline]
            fn add_assign(&mut self, rhs: $posit) {
                let ui = rhs.to_bits();
                fdp_one(self, ui, true);
            }
        }

        impl ops::AddAssign<(($posit, $posit), ($posit, $posit))> for $quire {
            #[inline]
            fn add_assign(&mut self, rhs: (($posit, $posit), ($posit, $posit))) {
                *self += ((rhs.0).0, (rhs.1).0);
                *self += ((rhs.0).0, (rhs.1).1);
                *self += ((rhs.0).1, (rhs.1).0);
                *self += ((rhs.0).1, (rhs.1).1);
            }
        }

        impl ops::SubAssign<($posit, $posit)> for $quire {
            #[inline]
            fn sub_assign(&mut self, rhs: ($posit, $posit)) {
                let ui_a = (rhs.0).to_bits();
                let ui_b = (rhs.1).to_bits();
                fdp(self, ui_a, ui_b, false);
            }
        }

        impl ops::SubAssign<$posit> for $quire {
            #[inline]
            fn sub_assign(&mut self, rhs: $posit) {
                let ui = rhs.to_bits();
                fdp_one(self, ui, false);
            }
        }

        impl ops::SubAssign<($posit, ($posit, $posit))> for $quire {
            #[inline]
            fn sub_assign(&mut self, rhs: ($posit, ($posit, $posit))) {
                *self -= (rhs.0, (rhs.1).0);
                *self -= (rhs.0, (rhs.1).1);
            }
        }

        impl ops::SubAssign<(($posit, $posit), ($posit, $posit))> for $quire {
            #[inline]
            fn sub_assign(&mut self, rhs: (($posit, $posit), ($posit, $posit))) {
                *self -= ((rhs.0).0, (rhs.1).0);
                *self -= ((rhs.0).0, (rhs.1).1);
                *self -= ((rhs.0).1, (rhs.1).0);
                *self -= ((rhs.0).1, (rhs.1).1);
            }
        }
    };
}
pub(crate) use quire_add_sub;

macro_rules! quire_add_sub_array_x {
    ($posit:ty, $quire:ty, $($i:literal),*) => {$(
        impl<const N: u32> ops::AddAssign<($posit, [$posit; $i])> for $quire {
            #[inline]
            fn add_assign(&mut self, rhs: ($posit, [$posit; $i])) {
                for p in &rhs.1 {
                    *self += (rhs.0, *p);
                }
            }
        }

        impl<const N: u32> ops::SubAssign<($posit, [$posit; $i])> for $quire {
            #[inline]
            fn sub_assign(&mut self, rhs: ($posit, [$posit; $i])) {
                for p in &rhs.1 {
                    *self -= (rhs.0, *p);
                }
            }
        }
    )*}
}
pub(crate) use quire_add_sub_array_x;

macro_rules! quire_add_sub_x {
    ($posit:ty, $quire:ty) => {
        impl<const N: u32> ops::AddAssign<($posit, $posit)> for $quire {
            #[inline]
            fn add_assign(&mut self, rhs: ($posit, $posit)) {
                let ui_a = (rhs.0).to_bits();
                let ui_b = (rhs.1).to_bits();
                fdp(self, ui_a, ui_b, true);
            }
        }

        impl<const N: u32> ops::AddAssign<($posit, ($posit, $posit))> for $quire {
            #[inline]
            fn add_assign(&mut self, rhs: ($posit, ($posit, $posit))) {
                *self += (rhs.0, (rhs.1).0);
                *self += (rhs.0, (rhs.1).1);
            }
        }

        impl<const N: u32> ops::AddAssign<($posit, ($posit, $posit, $posit))> for $quire {
            #[inline]
            fn add_assign(&mut self, rhs: ($posit, ($posit, $posit, $posit))) {
                *self += (rhs.0, (rhs.1).0);
                *self += (rhs.0, (rhs.1).1);
                *self += (rhs.0, (rhs.1).2);
            }
        }

        impl<const N: u32> ops::AddAssign<$posit> for $quire {
            #[inline]
            fn add_assign(&mut self, rhs: $posit) {
                let ui = rhs.to_bits();
                fdp_one(self, ui, true);
            }
        }

        impl<const N: u32> ops::AddAssign<(($posit, $posit), ($posit, $posit))> for $quire {
            #[inline]
            fn add_assign(&mut self, rhs: (($posit, $posit), ($posit, $posit))) {
                *self += ((rhs.0).0, (rhs.1).0);
                *self += ((rhs.0).0, (rhs.1).1);
                *self += ((rhs.0).1, (rhs.1).0);
                *self += ((rhs.0).1, (rhs.1).1);
            }
        }

        impl<const N: u32> ops::SubAssign<($posit, $posit)> for $quire {
            #[inline]
            fn sub_assign(&mut self, rhs: ($posit, $posit)) {
                let ui_a = (rhs.0).to_bits();
                let ui_b = (rhs.1).to_bits();
                fdp(self, ui_a, ui_b, false);
            }
        }

        impl<const N: u32> ops::SubAssign<$posit> for $quire {
            #[inline]
            fn sub_assign(&mut self, rhs: $posit) {
                let ui = rhs.to_bits();
                fdp_one(self, ui, false);
            }
        }

        impl<const N: u32> ops::SubAssign<($posit, ($posit, $posit))> for $quire {
            #[inline]
            fn sub_assign(&mut self, rhs: ($posit, ($posit, $posit))) {
                *self -= (rhs.0, (rhs.1).0);
                *self -= (rhs.0, (rhs.1).1);
            }
        }

        impl<const N: u32> ops::SubAssign<(($posit, $posit), ($posit, $posit))> for $quire {
            #[inline]
            fn sub_assign(&mut self, rhs: (($posit, $posit), ($posit, $posit))) {
                *self -= ((rhs.0).0, (rhs.1).0);
                *self -= ((rhs.0).0, (rhs.1).1);
                *self -= ((rhs.0).1, (rhs.1).0);
                *self -= ((rhs.0).1, (rhs.1).1);
            }
        }
    };
}
pub(crate) use quire_add_sub_x;

#[cfg(feature = "simba")]
pub mod simba {
    macro_rules! impl_real {
        ($T:ty) => {
            impl simba::scalar::RealField for $T {
                #[inline]
                fn is_sign_positive(&self) -> bool {
                    Self::is_sign_positive(*self)
                }

                #[inline]
                fn is_sign_negative(&self) -> bool {
                    Self::is_sign_negative(*self)
                }

                #[inline]
                fn copysign(self, sign: Self) -> Self {
                    Self::copysign(self, sign)
                }

                #[inline]
                fn max(self, other: Self) -> Self {
                    core::cmp::Ord::max(self, other)
                }

                #[inline]
                fn min(self, other: Self) -> Self {
                    core::cmp::Ord::min(self, other)
                }

                #[inline]
                fn clamp(self, min: Self, max: Self) -> Self {
                    Self::clamp(self, min, max)
                }

                #[inline]
                fn atan2(self, other: Self) -> Self {
                    Self::atan2(self, other)
                }

                #[inline]
                fn min_value() -> Option<Self> {
                    Some(Self::MIN)
                }

                #[inline]
                fn max_value() -> Option<Self> {
                    Some(Self::MAX)
                }

                /// Archimedes' constant.
                #[inline]
                fn pi() -> Self {
                    MathConsts::PI
                }

                /// 2.0 * pi.
                #[inline]
                fn two_pi() -> Self {
                    <Self as MathConsts>::PI + <Self as MathConsts>::PI
                }

                /// pi / 2.0.
                #[inline]
                fn frac_pi_2() -> Self {
                    MathConsts::FRAC_PI_2
                }

                /// pi / 3.0.
                #[inline]
                fn frac_pi_3() -> Self {
                    MathConsts::FRAC_PI_3
                }

                /// pi / 4.0.
                #[inline]
                fn frac_pi_4() -> Self {
                    MathConsts::FRAC_PI_4
                }

                /// pi / 6.0.
                #[inline]
                fn frac_pi_6() -> Self {
                    MathConsts::FRAC_PI_6
                }

                /// pi / 8.0.
                #[inline]
                fn frac_pi_8() -> Self {
                    MathConsts::FRAC_PI_8
                }

                /// 1.0 / pi.
                #[inline]
                fn frac_1_pi() -> Self {
                    MathConsts::FRAC_1_PI
                }

                /// 2.0 / pi.
                #[inline]
                fn frac_2_pi() -> Self {
                    MathConsts::FRAC_2_PI
                }

                /// 2.0 / sqrt(pi).
                #[inline]
                fn frac_2_sqrt_pi() -> Self {
                    MathConsts::FRAC_2_SQRT_PI
                }

                /// Euler's number.
                #[inline]
                fn e() -> Self {
                    MathConsts::E
                }

                /// log2(e).
                #[inline]
                fn log2_e() -> Self {
                    MathConsts::LOG2_E
                }

                /// log10(e).
                #[inline]
                fn log10_e() -> Self {
                    MathConsts::LOG10_E
                }

                /// ln(2.0).
                #[inline]
                fn ln_2() -> Self {
                    MathConsts::LN_2
                }

                /// ln(10.0).
                #[inline]
                fn ln_10() -> Self {
                    MathConsts::LN_10
                }
            }
        };
    }
    pub(crate) use impl_real;

    macro_rules! impl_complex {
        ($T:ty) => {
            impl simba::scalar::ComplexField for $T {
                type RealField = $T;

                #[inline]
                fn from_real(re: Self::RealField) -> Self {
                    re
                }

                #[inline]
                fn real(self) -> Self::RealField {
                    self
                }

                #[inline]
                fn imaginary(self) -> Self::RealField {
                    Self::ZERO
                }

                #[inline]
                fn norm1(self) -> Self::RealField {
                    Self::abs(self)
                }

                #[inline]
                fn modulus(self) -> Self::RealField {
                    Self::abs(self)
                }

                #[inline]
                fn modulus_squared(self) -> Self::RealField {
                    self * self
                }

                #[inline]
                fn argument(self) -> Self::RealField {
                    if self >= Self::ZERO {
                        Self::ZERO
                    } else {
                        MathConsts::PI
                    }
                }

                #[inline]
                fn to_exp(self) -> (Self, Self) {
                    if self >= Self::ZERO {
                        (self, Self::ONE)
                    } else {
                        (-self, -Self::ONE)
                    }
                }

                #[inline]
                fn recip(self) -> Self {
                    Self::recip(self)
                }

                #[inline]
                fn conjugate(self) -> Self {
                    self
                }

                #[inline]
                fn scale(self, factor: Self::RealField) -> Self {
                    self * factor
                }

                #[inline]
                fn unscale(self, factor: Self::RealField) -> Self {
                    self / factor
                }

                #[inline]
                fn floor(self) -> Self {
                    Self::floor(self)
                }

                #[inline]
                fn ceil(self) -> Self {
                    Self::ceil(self)
                }

                #[inline]
                fn round(self) -> Self {
                    Self::round(self)
                }

                #[inline]
                fn trunc(self) -> Self {
                    Self::trunc(self)
                }

                #[inline]
                fn fract(self) -> Self {
                    Self::fract(self)
                }

                #[inline]
                fn abs(self) -> Self {
                    Self::abs(self)
                }

                #[inline]
                fn signum(self) -> Self {
                    Self::signum(self)
                }

                #[inline]
                fn mul_add(self, a: Self, b: Self) -> Self {
                    Self::mul_add(self, a, b)
                }

                #[inline]
                fn powi(self, n: i32) -> Self {
                    Self::powi(self, n)
                }

                #[inline]
                fn powf(self, n: Self) -> Self {
                    Self::powf(self, n)
                }

                #[inline]
                fn powc(self, n: Self) -> Self {
                    // Same as powf.
                    Self::powf(self, n)
                }

                #[inline]
                fn sqrt(self) -> Self {
                    Self::sqrt(self)
                }

                #[inline]
                fn try_sqrt(self) -> Option<Self> {
                    if self >= Self::ZERO {
                        Some(Self::sqrt(self))
                    } else {
                        None
                    }
                }

                #[inline]
                fn exp(self) -> Self {
                    Self::exp(self)
                }

                #[inline]
                fn exp2(self) -> Self {
                    Self::exp2(self)
                }

                #[inline]
                fn exp_m1(self) -> Self {
                    Self::exp_m1(self)
                }

                #[inline]
                fn ln_1p(self) -> Self {
                    Self::ln_1p(self)
                }

                #[inline]
                fn ln(self) -> Self {
                    Self::ln(self)
                }

                #[inline]
                fn log(self, base: Self) -> Self {
                    Self::log(self, base)
                }

                #[inline]
                fn log2(self) -> Self {
                    Self::log2(self)
                }

                #[inline]
                fn log10(self) -> Self {
                    Self::log10(self)
                }

                #[inline]
                fn cbrt(self) -> Self {
                    Self::cbrt(self)
                }

                #[inline]
                fn hypot(self, other: Self) -> Self::RealField {
                    Self::hypot(self, other)
                }

                #[inline]
                fn sin(self) -> Self {
                    Self::sin(self)
                }

                #[inline]
                fn cos(self) -> Self {
                    Self::cos(self)
                }

                #[inline]
                fn tan(self) -> Self {
                    Self::tan(self)
                }

                #[inline]
                fn asin(self) -> Self {
                    Self::asin(self)
                }

                #[inline]
                fn acos(self) -> Self {
                    Self::acos(self)
                }

                #[inline]
                fn atan(self) -> Self {
                    Self::atan(self)
                }

                #[inline]
                fn sin_cos(self) -> (Self, Self) {
                    Self::sin_cos(self)
                }

                #[inline]
                fn sinh(self) -> Self {
                    Self::sinh(self)
                }

                #[inline]
                fn cosh(self) -> Self {
                    Self::cosh(self)
                }

                #[inline]
                fn tanh(self) -> Self {
                    Self::tanh(self)
                }

                #[inline]
                fn asinh(self) -> Self {
                    Self::asinh(self)
                }

                #[inline]
                fn acosh(self) -> Self {
                    Self::acosh(self)
                }

                #[inline]
                fn atanh(self) -> Self {
                    Self::atanh(self)
                }

                #[inline]
                fn is_finite(&self) -> bool {
                    Self::is_finite(*self)
                }
            }
        };
    }
    pub(crate) use impl_complex;

    macro_rules! impl_primitive_simd_value_for_scalar(
        ($($t: ty),*) => {$(
            impl simba::simd::PrimitiveSimdValue for $t {}
            impl simba::simd::SimdValue for $t {
                type Element = $t;
                type SimdBool = bool;

                #[inline(always)]
                fn lanes() -> usize {
                    1
                }

                #[inline(always)]
                fn splat(val: Self::Element) -> Self {
                    val
                }

                #[inline(always)]
                fn extract(&self, _: usize) -> Self::Element {
                    *self
                }

                #[inline(always)]
                unsafe fn extract_unchecked(&self, _: usize) -> Self::Element {
                    *self
                }

                #[inline(always)]
                fn replace(&mut self, _: usize, val: Self::Element) {
                    *self = val
                }

                #[inline(always)]
                unsafe fn replace_unchecked(&mut self, _: usize, val: Self::Element) {
                    *self = val
                }

                #[inline(always)]
                fn select(self, cond: Self::SimdBool, other: Self) -> Self {
                    if cond {
                        self
                    } else {
                        other
                    }
                }
            }
        )*}
    );
    pub(crate) use impl_primitive_simd_value_for_scalar;

    macro_rules! impl_subset_into(
    ($($subset: ty as $( $superset: ty),+ );* $(;)*) => {
        $($(
            impl simba::scalar::SubsetOf<$superset> for $subset {
                #[inline]
                fn to_superset(&self) -> $superset {
                    (*self).into()
                }

                #[inline]
                fn from_superset_unchecked(element: &$superset) -> $subset {
                    (*element).into()
                }

                #[inline]
                fn is_in_subset(_: &$superset) -> bool {
                    true
                }
            }
        )+)*
    });
    pub(crate) use impl_subset_into;
}

#[cfg(feature = "approx")]
pub mod approx {
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
    pub(crate) use impl_ulps_eq;

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
    pub(crate) use impl_signed_abs_diff_eq;

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
    pub(crate) use impl_relative_eq;
}
