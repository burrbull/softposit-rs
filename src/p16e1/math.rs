use super::P16E1;

const HALF: P16E1 = P16E1::new(0x_3000);
const TWO: P16E1 = P16E1::new(0x_5000);

impl crate::MathConsts for P16E1 {
    const E: Self = Self::new(0x_55bf);
    const FRAC_1_PI: Self = Self::new(0x_245f);
    const FRAC_1_SQRT_2: Self = Self::new(0x_36a1);
    const FRAC_2_PI: Self = Self::new(0x_345f);
    const FRAC_2_SQRT_PI: Self = Self::new(0x_420e);
    const FRAC_PI_2: Self = Self::new(0x_4922);
    const FRAC_PI_3: Self = Self::new(0x_40c1);
    const FRAC_PI_4: Self = Self::new(0x_3922);
    const FRAC_PI_6: Self = Self::new(0x_30c1);
    const FRAC_PI_8: Self = Self::new(0x_2922);
    const LN_10: Self = Self::new(0x_526c);
    const LN_2: Self = Self::new(0x_362e);
    const LOG10_E: Self = Self::new(0x_2bcb);
    const LOG2_E: Self = Self::new(0x_2344);
    const PI: Self = Self::new(0x_5922);
    const SQRT_2: Self = Self::new(0x_46a1);
    const LOG2_10: Self = Self::new(0x_5a93);
    const LOG10_2: Self = Self::new(0x_2344);
}

impl P16E1 {
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
        unimplemented!()
    }
    #[inline]
    pub fn powf(self, _n: Self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn log(self, _base: Self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn log10(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn cbrt(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn hypot(self, _other: Self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn sin(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn cos(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn tan(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn asin(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn acos(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn atan(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn atan2(self, _other: Self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn sin_cos(self) -> (Self, Self) {
        (self.sin(), self.cos())
    }
    #[inline]
    pub fn exp_m1(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub const fn ln_1p(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn sinh(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn cosh(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn tanh(self) -> Self {
        unimplemented!()
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

mod acos_pi;
mod asin_pi;
mod atan_pi;
mod ceil;
mod cos_pi;
mod exp;
mod exp2;
mod floor;
mod ln;
mod log2;
mod mul_add;
mod round;
mod sin_pi;
mod sqrt;
mod tan_pi;

mod kernel {
    #[inline]
    pub const fn isqrt(f: u64) -> u64 {
        let mut bit = 0x_0040_0000_0000_0000_u64;
        let mut res = 0_u64;

        let mut n = f;
        while bit > n {
            bit >>= 2;
        }
        while bit != 0 {
            if n >= res + bit {
                n -= res + bit;
                res = (res >> 1) + bit;
            } else {
                res >>= 1;
            }
            bit >>= 2;
        }
        res
    }
}
