#![no_std]

pub mod p8e0;
pub use self::p8e0::{P8E0, Q8E0};
pub type P8 = P8E0;
pub type Q8 = Q8E0;

pub mod p16e1;
pub use self::p16e1::{P16E1, Q16E1};
pub type P16 = P16E1;
pub type Q16 = Q16E1;

pub mod p32e2;
pub use self::p32e2::{P32E2, Q32E2};
pub type P32 = P32E2;
pub type Q32 = Q32E2;

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

pub trait Quire {
    type Q;
}

pub trait Poly: Copy {
    fn poly1k(x: Self, c: &[Self]) -> Self;
    fn poly2k(x: Self, x2: Self, c: &[Self], p: Self) -> Self;
    fn poly3k(x: Self, x2: Self, x3: Self, c: &[Self], p: Self) -> Self;
    fn poly4k(x: Self, x2: Self, x3: Self, x4: Self, c: &[Self], p: Self) -> Self;
    #[inline]
    fn poly5k(x: Self, x2: Self, x3: Self, c: &[Self], p: Self) -> Self {
        let p = Self::poly2k(x, x2, &c[3..5], p);
        Self::poly3k(x, x2, x3, &c[..3], p)
    }
    #[inline]
    fn poly6k(x: Self, x2: Self, x3: Self, c: &[Self], p: Self) -> Self {
        let p = Self::poly3k(x, x2, x3, &c[3..6], p);
        Self::poly3k(x, x2, x3, &c[..3], p)
    }
    #[inline]
    fn poly7k(x: Self, x2: Self, x3: Self, x4: Self, c: &[Self], p: Self) -> Self {
        let p = Self::poly3k(x, x2, x3, &c[4..7], p);
        Self::poly4k(x, x2, x3, x4, &c[..4], p)
    }
    #[inline]
    fn poly8k(x: Self, x2: Self, x3: Self, x4: Self, c: &[Self], p: Self) -> Self {
        let p = Self::poly4k(x, x2, x3, x4, &c[4..8], p);
        Self::poly4k(x, x2, x3, x4, &c[..4], p)
    }
    #[inline]
    fn poly9k(x: Self, x2: Self, x3: Self, x4: Self, c: &[Self], p: Self) -> Self {
        let p = Self::poly5k(x, x2, x3, &c[4..9], p);
        Self::poly4k(x, x2, x3, x4, &c[..4], p)
    }
    #[inline]
    fn poly10k(x: Self, x2: Self, x3: Self, x4: Self, c: &[Self], p: Self) -> Self {
        let p = Self::poly6k(x, x2, x3, &c[4..10], p);
        Self::poly4k(x, x2, x3, x4, &c[..4], p)
    }
    #[inline]
    fn poly11k(x: Self, x2: Self, x3: Self, x4: Self, c: &[Self], p: Self) -> Self {
        let p = Self::poly7k(x, x2, x3, x4, &c[4..11], p);
        Self::poly4k(x, x2, x3, x4, &c[..4], p)
    }
    #[inline]
    fn poly12k(x: Self, x2: Self, x3: Self, x4: Self, c: &[Self], p: Self) -> Self {
        let p = Self::poly8k(x, x2, x3, x4, &c[4..12], p);
        Self::poly4k(x, x2, x3, x4, &c[..4], p)
    }
    #[inline]
    fn poly13k(x: Self, x2: Self, x3: Self, x4: Self, c: &[Self], p: Self) -> Self {
        let p = Self::poly9k(x, x2, x3, x4, &c[4..13], p);
        Self::poly4k(x, x2, x3, x4, &c[..4], p)
    }
    #[inline]
    fn poly14k(x: Self, x2: Self, x3: Self, x4: Self, c: &[Self], p: Self) -> Self {
        let p = Self::poly10k(x, x2, x3, x4, &c[4..14], p);
        Self::poly4k(x, x2, x3, x4, &c[..4], p)
    }
    #[inline]
    fn poly15k(x: Self, x2: Self, x3: Self, x4: Self, c: &[Self], p: Self) -> Self {
        let p = Self::poly11k(x, x2, x3, x4, &c[4..15], p);
        Self::poly4k(x, x2, x3, x4, &c[..4], p)
    }
    #[inline]
    fn poly16k(x: Self, x2: Self, x3: Self, x4: Self, c: &[Self], p: Self) -> Self {
        let p = Self::poly12k(x, x2, x3, x4, &c[4..16], p);
        Self::poly4k(x, x2, x3, x4, &c[..4], p)
    }
    #[inline]
    fn poly17k(x: Self, x2: Self, x3: Self, x4: Self, c: &[Self], p: Self) -> Self {
        let p = Self::poly13k(x, x2, x3, x4, &c[4..17], p);
        Self::poly4k(x, x2, x3, x4, &c[..4], p)
    }
    #[inline]
    fn poly18k(x: Self, x2: Self, x3: Self, x4: Self, c: &[Self], p: Self) -> Self {
        let p = Self::poly14k(x, x2, x3, x4, &c[4..18], p);
        Self::poly4k(x, x2, x3, x4, &c[..4], p)
    }
}

pub trait Polynom: Poly + core::ops::Mul<Output = Self> {
    // Quire1 = 1
    fn poly1(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 2);
        Self::poly1k(self, c)
    }
    // Quire1 + (x2=x*x) = 2
    fn poly2(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 3);
        let x2 = self * self;
        Self::poly2k(self, x2, &c[..2], c[2])
    }
    // Quire1 + (x2, x3=x2*x) = 3, faster
    fn poly3(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 4);
        let x2 = self * self;
        let x3 = x2 * self;
        Self::poly3k(self, x2, x3, &c[..3], c[3])
    }
    // Quire1 + (x2, x3, x4=x2*x2) = 4, faster
    #[inline]
    fn poly4(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 5);
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly4k(self, x2, x3, x4, &c[..4], c[4])
    }
    // Quire2 + (x2, x3) = 4
    #[inline]
    fn poly5(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 6);
        let x2 = self * self;
        let x3 = x2 * self;
        Self::poly5k(self, x2, x3, &c[..5], c[5])
    }
    // Quire2 + (x2, x3) = 4
    #[inline]
    fn poly6(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 7);
        let x2 = self * self;
        let x3 = x2 * self;
        Self::poly6k(self, x2, x3, &c[..6], c[6])
    }
    // Quire2 + (x2, x3, x4) = 5
    #[inline]
    fn poly7(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 8);
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly7k(self, x2, x3, x4, &c[..7], c[7])
    }
    // Quire2 + (x2, x3, x4) = 5
    #[inline]
    fn poly8(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 9);
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly8k(self, x2, x3, x4, &c[..8], c[8])
    }
    // Quire3 + (x2, x3, x4) = 6
    #[inline]
    fn poly9(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 10);
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly9k(self, x2, x3, x4, &c[..9], c[9])
    }
    // Quire3 + (x2, x3, x4) = 6
    #[inline]
    fn poly10(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 11);
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly10k(self, x2, x3, x4, &c[..10], c[10])
    }
    // Quire3 + (x2, x3, x4) = 6
    #[inline]
    fn poly11(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 12);
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly11k(self, x2, x3, x4, &c[..11], c[11])
    }
    // Quire3 + (x2, x3, x4) = 6
    #[inline]
    fn poly12(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 13);
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly12k(self, x2, x3, x4, &c[..12], c[12])
    }
    // Quire4 + (x2, x3, x4) = 7
    #[inline]
    fn poly13(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 14);
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly13k(self, x2, x3, x4, &c[..13], c[13])
    }
    // Quire4 + (x2, x3, x4) = 7
    #[inline]
    fn poly14(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 15);
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly14k(self, x2, x3, x4, &c[..14], c[14])
    }
    // Quire4 + (x2, x3, x4) = 7
    #[inline]
    fn poly15(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 16);
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly15k(self, x2, x3, x4, &c[..15], c[15])
    }
    // Quire4 + (x2, x3, x4) = 7
    #[inline]
    fn poly16(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 17);
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly16k(self, x2, x3, x4, &c[..16], c[16])
    }
    // Quire5 + (x2, x3, x4) = 8
    #[inline]
    fn poly17(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 18);
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly17k(self, x2, x3, x4, &c[..17], c[17])
    }
    // Quire5 + (x2, x3, x4) = 8
    #[inline]
    fn poly18(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 19);
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly18k(self, x2, x3, x4, &c[..18], c[18])
    }
    // Quire2 + (x2) = 3, more accurate
    #[inline]
    fn poly3a(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 4);
        let x2 = self * self;
        let p = Self::poly1k(self, &c[2..]);
        Self::poly2k(self, x2, &c[..2], p)
    }
    // Quire2 + (x2) = 3, more accurate
    #[inline]
    fn poly4a(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 5);
        let x2 = self * self;
        let p = Self::poly2k(self, x2, &c[2..4], c[4]);
        Self::poly2k(self, x2, &c[..2], p)
    }
}
