use super::P32E2;

pub mod sleef;

const HALF: P32E2 = P32E2::new(0x_3800_0000);
const TWO: P32E2 = P32E2::new(0x_4800_0000);

impl P32E2 {
    #[inline]
    pub const fn trunc(self) -> Self {
        if self.gt(Self::ZERO) {
            self.floor()
        } else {
            self.ceil()
        }
    }
    #[inline]
    pub const fn fract(self) -> Self {
        self.sub(self.trunc())
    }
    #[inline]
    pub const fn div_euclid(self, rhs: Self) -> Self {
        let q = self.div(rhs).trunc();
        if self.rem(rhs).lt(Self::ZERO) {
            return if rhs.gt(Self::ZERO) {
                q.sub(Self::ONE)
            } else {
                q.add(Self::ONE)
            };
        }
        q
    }
    #[inline]
    pub const fn rem_euclid(self, rhs: Self) -> Self {
        let r = self.rem(rhs);
        if r.lt(Self::ZERO) {
            r.add(rhs.abs())
        } else {
            r
        }
    }
    #[inline]
    pub fn powi(self, _n: i32) -> Self {
        todo!()
    }
    #[inline]
    pub fn powf(self, other: Self) -> Self {
        sleef::pow(self, other)
    }
    #[inline]
    pub fn exp(self) -> Self {
        sleef::exp(self)
    }
    #[inline]
    pub fn exp2(self) -> Self {
        sleef::exp2(self)
    }
    #[inline]
    pub fn exp10(self) -> Self {
        sleef::exp10(self)
    }
    #[inline]
    pub fn ln(self) -> Self {
        sleef::ln(self)
    }
    #[inline]
    pub fn log(self, _base: Self) -> Self {
        todo!()
    }
    #[inline]
    pub fn log2(self) -> Self {
        sleef::log2(self)
    }
    #[inline]
    pub fn log10(self) -> Self {
        todo!()
    }
    #[inline]
    pub fn cbrt(self) -> Self {
        sleef::cbrt(self)
    }
    #[inline]
    pub fn hypot(self, other: Self) -> Self {
        sleef::hypot(self, other)
    }
    #[inline]
    pub fn sin(self) -> Self {
        sleef::sin(self)
    }
    #[inline]
    pub fn cos(self) -> Self {
        sleef::cos(self)
    }
    #[inline]
    pub fn tan(self) -> Self {
        sleef::tan(self)
    }
    #[inline]
    pub fn asin(self) -> Self {
        sleef::asin(self)
    }
    #[inline]
    pub fn acos(self) -> Self {
        sleef::acos(self)
    }
    #[inline]
    pub fn atan(self) -> Self {
        sleef::atan(self)
    }
    #[inline]
    pub fn atan2(self, other: Self) -> Self {
        sleef::atan2(self, other)
    }
    #[inline]
    pub fn sin_cos(self) -> (Self, Self) {
        sleef::sin_cos(self)
    }
    #[inline]
    pub fn exp_m1(self) -> Self {
        todo!()
    }
    #[inline]
    pub const fn ln_1p(self) -> Self {
        todo!()
    }
    #[inline]
    pub fn sinh(self) -> Self {
        sleef::sinh(self)
    }
    #[inline]
    pub fn cosh(self) -> Self {
        sleef::cosh(self)
    }
    #[inline]
    pub fn tanh(self) -> Self {
        sleef::tanh(self)
    }
    #[inline]
    pub fn asinh(self) -> Self {
        if self.is_nar() {
            self
        } else {
            self.add(self.mul(self).add(Self::ONE).sqrt()).ln()
        }
    }
    #[inline]
    pub fn acosh(self) -> Self {
        match self {
            x if x.lt(Self::ONE) => Self::NAR,
            x => x.add(x.mul(x).sub(Self::ONE).sqrt()).ln(),
        }
    }
    #[inline]
    pub const fn atanh(self) -> Self {
        HALF.mul(TWO.mul(self).div(Self::ONE.sub(self)).ln_1p())
    }
}

mod ceil;
mod floor;
mod mul_add;
mod round;
mod sqrt;
