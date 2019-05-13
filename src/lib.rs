#![no_std]

pub mod p8e0;
pub use self::p8e0::{P8E0, Q8E0};
pub type P8 = P8E0;

pub mod p16e1;
pub use self::p16e1::P16E1;
pub type P16 = P16E1;

pub mod p32e2;
pub use self::p32e2::P32E2;
pub type P32 = P32E2;

mod convert;

trait WithSign {
    fn with_sign(self, sign: bool) -> Self;
}
impl WithSign for u8 {
    #[inline]
    fn with_sign(self, sign: bool) -> Self {
        if sign {
            self.wrapping_neg()
        } else {
            self
        }
    }
}
impl WithSign for u16 {
    #[inline]
    fn with_sign(self, sign: bool) -> Self {
        if sign {
            self.wrapping_neg()
        } else {
            self
        }
    }
}
impl WithSign for u32 {
    #[inline]
    fn with_sign(self, sign: bool) -> Self {
        if sign {
            self.wrapping_neg()
        } else {
            self
        }
    }
}

fn lldiv(numer: i64, denom: i64) -> (i64, i64) {
    let mut quot = numer / denom;
    let mut rem = numer % denom;
    if (numer >= 0) && (rem < 0) {
        quot += 1;
        rem -= denom;
    }

    (quot, rem)
}

fn div(numer: i32, denom: i32) -> (i32, i32) {
    let mut quot = numer / denom;
    let mut rem = numer % denom;
    if (numer >= 0) && (rem < 0) {
        quot += 1;
        rem -= denom;
    }

    (quot, rem)
}

const APPROX_RECIP_SQRT0: [u16; 16] = [
    0xb4c9, 0xffab, 0xaa7d, 0xf11c, 0xa1c5, 0xe4c7, 0x9a43, 0xda29, 0x93b5, 0xd0e5, 0x8ded, 0xc8b7,
    0x88c6, 0xc16d, 0x8424, 0xbae1,
];
const APPROX_RECIP_SQRT1: [u16; 16] = [
    0xa5a5, 0xea42, 0x8c21, 0xc62d, 0x788f, 0xaa7f, 0x6928, 0x94b6, 0x5cc7, 0x8335, 0x52a6, 0x74e2,
    0x4a3e, 0x68fe, 0x432b, 0x5efd,
];

#[allow(dead_code)]
#[derive(Clone, Copy)]
enum MulAddType {
    Add = 0,
    SubC = 1,
    SubProd = 2,
}

pub trait MathConsts {
    /// Euler's number (e)
    const E: Self;
    /// 1/π
    const FRAC_1_PI: Self;
    /// 1/sqrt(2)
    const FRAC_1_SQRT_2: Self;
    /// 2/π
    const FRAC_2_PI: Self;
    /// 2/sqrt(π)
    const FRAC_2_SQRT_PI: Self;
    /// π/2
    const FRAC_PI_2: Self;
    /// π/3
    const FRAC_PI_3: Self;
    /// π/4
    const FRAC_PI_4: Self;
    /// π/6
    const FRAC_PI_6: Self;
    /// π/8
    const FRAC_PI_8: Self;
    /// ln(10)
    const LN_10: Self;
    /// ln(2)
    const LN_2: Self;
    /// log<sub>10</sub>(e)
    const LOG10_E: Self;
    /// log<sub>2</sub>(e)
    const LOG2_E: Self;
    /// Archimedes' constant (π)
    const PI: Self;
    /// sqrt(2)
    const SQRT_2: Self;
    /// log<sub>2</sub>(10)
    const LOG2_10: Self;
    /// log<sub>10</sub>(2)
    const LOG10_2: Self;
}

#[cfg(test)]
const NTESTS32: usize = 1000_000;
#[cfg(test)]
const NTESTS16: usize = 100_000;
#[cfg(test)]
const NTESTS8: usize = 1_000;
