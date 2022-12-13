use super::P8E0;

const HALF: P8E0 = P8E0::new(0x_20);
const TWO: P8E0 = P8E0::new(0x_60);

impl P8E0 {
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
    pub fn powf(self, _n: Self) -> Self {
        todo!()
    }
    #[inline]
    pub fn exp2(self) -> Self {
        todo!()
    }
    #[inline]
    pub fn log(self, _base: Self) -> Self {
        todo!()
    }
    #[inline]
    pub fn log2(self) -> Self {
        todo!()
    }
    #[inline]
    pub fn log10(self) -> Self {
        todo!()
    }
    #[inline]
    pub fn cbrt(self) -> Self {
        todo!()
    }
    #[inline]
    pub fn hypot(self, _other: Self) -> Self {
        todo!()
    }
    #[inline]
    pub fn sin(self) -> Self {
        todo!()
    }
    #[inline]
    pub fn cos(self) -> Self {
        todo!()
    }
    #[inline]
    pub fn tan(self) -> Self {
        todo!()
    }
    #[inline]
    pub fn asin(self) -> Self {
        todo!()
    }
    #[inline]
    pub fn acos(self) -> Self {
        todo!()
    }
    #[inline]
    pub fn atan(self) -> Self {
        todo!()
    }
    #[inline]
    pub fn atan2(self, _other: Self) -> Self {
        todo!()
    }
    #[inline]
    pub fn sin_cos(self) -> (Self, Self) {
        (self.sin(), self.cos())
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
        todo!()
    }
    #[inline]
    pub fn cosh(self) -> Self {
        todo!()
    }
    #[inline]
    pub fn tanh(self) -> Self {
        todo!()
    }
    #[inline]
    pub const fn asinh(self) -> Self {
        if self.is_nar() {
            self
        } else {
            self.add(self.mul(self).add(Self::ONE).sqrt()).ln()
        }
    }
    #[inline]
    pub const fn acosh(self) -> Self {
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
mod exp;
mod floor;
mod ln;
mod mul_add;
mod round;
mod sqrt;
