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

mod macros;

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
    /// Euler's number (e) = 2.7182818284590452353602874713526625
    const E: Self;
    /// 1/π = 0.318309886183790671537767526745028724
    const FRAC_1_PI: Self;
    /// 1/sqrt(2) = 0.707106781186547524400844362104849039
    const FRAC_1_SQRT_2: Self;
    /// 2/π = 0.636619772367581343075535053490057448
    const FRAC_2_PI: Self;
    /// 2/sqrt(π) = 1.12837916709551257389615890312154517
    const FRAC_2_SQRT_PI: Self;
    /// π/2 = 1.57079632679489661923132169163975144
    const FRAC_PI_2: Self;
    /// π/3 = 1.04719755119659774615421446109316763
    const FRAC_PI_3: Self;
    /// π/4 = 0.785398163397448309615660845819875721
    const FRAC_PI_4: Self;
    /// π/6 = 0.52359877559829887307710723054658381
    const FRAC_PI_6: Self;
    /// π/8 = 0.39269908169872415480783042290993786
    const FRAC_PI_8: Self;
    /// ln(10) = 2.30258509299404568401799145468436421
    const LN_10: Self;
    /// ln(2) = 0.693147180559945309417232121458176568
    const LN_2: Self;
    /// log<sub>10</sub>(e) = 0.434294481903251827651128918916605082
    const LOG10_E: Self;
    /// log<sub>2</sub>(e) = 1.44269504088896340735992468100189214
    const LOG2_E: Self;
    /// Archimedes' constant (π) = 3.14159265358979323846264338327950288
    const PI: Self;
    /// sqrt(2) = 1.41421356237309504880168872420969808
    const SQRT_2: Self;
    /// log<sub>2</sub>(10) = 3.32192809488736234787031942948939018
    const LOG2_10: Self;
    /// log<sub>10</sub>(2) = 0.301029995663981195213738894724493027
    const LOG10_2: Self;
}

#[cfg(test)]
const NTESTS32: usize = 1000_000;
#[cfg(test)]
const NTESTS16: usize = 100_000;
#[cfg(test)]
const NTESTS8: usize = 1_000;

pub trait AssociatedQuire<P> {
    type Q: Quire<P>;
}

pub trait Quire<P> {
    type Bits;
    fn init() -> Self;
    fn from_posit(p: P) -> Self;
    fn to_posit(self) -> P;
    fn from_bits(v: Self::Bits) -> Self;
    fn to_bits(&self) -> Self::Bits;
    fn is_zero(&self) -> bool;
    fn is_nar(&self) -> bool;
    fn add_product(&mut self, p_a: P, p_b: P);
    fn sub_product(&mut self, p_a: P, p_b: P);
    fn clear(&mut self);
    fn neg(&mut self);
}

pub trait Poly: Copy {
    fn poly1k(x: Self, c0: Self, c1: Self) -> Self;
    fn poly2k(x: Self, x2: Self, p: Self, c: &[Self]) -> Self;
    fn poly3k(x: Self, x2: Self, x3: Self, p: Self, c: &[Self]) -> Self;
    fn poly4k(x: Self, x2: Self, x3: Self, x4: Self, p: Self, c: &[Self]) -> Self;
    #[inline]
    fn poly5k(x: Self, x2: Self, x3: Self, p: Self, c: &[Self]) -> Self {
        let p = Self::poly2k(x, x2, p, &c[..2]);
        Self::poly3k(x, x2, x3, p, &c[2..])
    }
    #[inline]
    fn poly6k(x: Self, x2: Self, x3: Self, p: Self, c: &[Self]) -> Self {
        let p = Self::poly3k(x, x2, x3, p, &c[..3]);
        Self::poly3k(x, x2, x3, p, &c[3..])
    }
    #[inline]
    fn poly7k(x: Self, x2: Self, x3: Self, x4: Self, p: Self, c: &[Self]) -> Self {
        let p = Self::poly3k(x, x2, x3, p, &c[..3]);
        Self::poly4k(x, x2, x3, x4, p, &c[3..])
    }
    #[inline]
    fn poly8k(x: Self, x2: Self, x3: Self, x4: Self, p: Self, c: &[Self]) -> Self {
        let p = Self::poly4k(x, x2, x3, x4, p, &c[..4]);
        Self::poly4k(x, x2, x3, x4, p, &c[4..])
    }
    #[inline]
    fn poly9k(x: Self, x2: Self, x3: Self, x4: Self, p: Self, c: &[Self]) -> Self {
        let p = Self::poly5k(x, x2, x3, p, &c[..5]);
        Self::poly4k(x, x2, x3, x4, p, &c[5..])
    }
    #[inline]
    fn poly10k(x: Self, x2: Self, x3: Self, x4: Self, p: Self, c: &[Self]) -> Self {
        let p = Self::poly6k(x, x2, x3, p, &c[..6]);
        Self::poly4k(x, x2, x3, x4, p, &c[6..])
    }
    #[inline]
    fn poly11k(x: Self, x2: Self, x3: Self, x4: Self, p: Self, c: &[Self]) -> Self {
        let p = Self::poly7k(x, x2, x3, x4, p, &c[..7]);
        Self::poly4k(x, x2, x3, x4, p, &c[7..])
    }
    #[inline]
    fn poly12k(x: Self, x2: Self, x3: Self, x4: Self, p: Self, c: &[Self]) -> Self {
        let p = Self::poly8k(x, x2, x3, x4, p, &c[..8]);
        Self::poly4k(x, x2, x3, x4, p, &c[8..])
    }
    #[inline]
    fn poly13k(x: Self, x2: Self, x3: Self, x4: Self, p: Self, c: &[Self]) -> Self {
        let p = Self::poly9k(x, x2, x3, x4, p, &c[..9]);
        Self::poly4k(x, x2, x3, x4, p, &c[9..])
    }
    #[inline]
    fn poly14k(x: Self, x2: Self, x3: Self, x4: Self, p: Self, c: &[Self]) -> Self {
        let p = Self::poly10k(x, x2, x3, x4, p, &c[..10]);
        Self::poly4k(x, x2, x3, x4, p, &c[10..])
    }
    #[inline]
    fn poly15k(x: Self, x2: Self, x3: Self, x4: Self, p: Self, c: &[Self]) -> Self {
        let p = Self::poly11k(x, x2, x3, x4, p, &c[..11]);
        Self::poly4k(x, x2, x3, x4, p, &c[11..])
    }
    #[inline]
    fn poly16k(x: Self, x2: Self, x3: Self, x4: Self, p: Self, c: &[Self]) -> Self {
        let p = Self::poly12k(x, x2, x3, x4, p, &c[..12]);
        Self::poly4k(x, x2, x3, x4, p, &c[12..])
    }
    #[inline]
    fn poly17k(x: Self, x2: Self, x3: Self, x4: Self, p: Self, c: &[Self]) -> Self {
        let p = Self::poly13k(x, x2, x3, x4, p, &c[..13]);
        Self::poly4k(x, x2, x3, x4, p, &c[13..])
    }
    #[inline]
    fn poly18k(x: Self, x2: Self, x3: Self, x4: Self, p: Self, c: &[Self]) -> Self {
        let p = Self::poly14k(x, x2, x3, x4, p, &c[..14]);
        Self::poly4k(x, x2, x3, x4, p, &c[14..])
    }
}

pub trait Polynom: Poly + core::ops::Mul<Output = Self> {
    // Quire1 = 1
    #[inline]
    fn poly1(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 2);
        Self::poly1k(self, c[0], c[1])
    }
    // Quire1 + (x2=x*x) = 2
    #[inline]
    fn poly2(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 3);
        let x2 = self * self;
        Self::poly2k(self, x2, c[0], &c[1..])
    }
    // Quire1 + (x2, x3=x2*x) = 3, faster
    #[inline]
    fn poly3(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 4);
        let x2 = self * self;
        let x3 = x2 * self;
        Self::poly3k(self, x2, x3, c[0], &c[1..])
    }
    // Quire1 + (x2, x3, x4=x2*x2) = 4, faster
    #[inline]
    fn poly4(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 5);
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly4k(self, x2, x3, x4, c[0], &c[1..])
    }
    // Quire2 + (x2, x3) = 4
    #[inline]
    fn poly5(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 6);
        let x2 = self * self;
        let x3 = x2 * self;
        Self::poly5k(self, x2, x3, c[0], &c[1..])
    }
    // Quire2 + (x2, x3) = 4
    #[inline]
    fn poly6(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 7);
        let x2 = self * self;
        let x3 = x2 * self;
        Self::poly6k(self, x2, x3, c[0], &c[1..])
    }
    // Quire2 + (x2, x3, x4) = 5
    #[inline]
    fn poly7(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 8);
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly7k(self, x2, x3, x4, c[0], &c[1..])
    }
    // Quire2 + (x2, x3, x4) = 5
    #[inline]
    fn poly8(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 9);
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly8k(self, x2, x3, x4, c[0], &c[1..])
    }
    // Quire3 + (x2, x3, x4) = 6
    #[inline]
    fn poly9(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 10);
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly9k(self, x2, x3, x4, c[0], &c[1..])
    }
    // Quire3 + (x2, x3, x4) = 6
    #[inline]
    fn poly10(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 11);
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly10k(self, x2, x3, x4, c[0], &c[1..])
    }
    // Quire3 + (x2, x3, x4) = 6
    #[inline]
    fn poly11(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 12);
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly11k(self, x2, x3, x4, c[0], &c[1..])
    }
    // Quire3 + (x2, x3, x4) = 6
    #[inline]
    fn poly12(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 13);
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly12k(self, x2, x3, x4, c[0], &c[1..])
    }
    // Quire4 + (x2, x3, x4) = 7
    #[inline]
    fn poly13(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 14);
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly13k(self, x2, x3, x4, c[0], &c[1..])
    }
    // Quire4 + (x2, x3, x4) = 7
    #[inline]
    fn poly14(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 15);
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly14k(self, x2, x3, x4, c[0], &c[1..])
    }
    // Quire4 + (x2, x3, x4) = 7
    #[inline]
    fn poly15(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 16);
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly15k(self, x2, x3, x4, c[0], &c[1..])
    }
    // Quire4 + (x2, x3, x4) = 7
    #[inline]
    fn poly16(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 17);
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly16k(self, x2, x3, x4, c[0], &c[1..])
    }
    // Quire5 + (x2, x3, x4) = 8
    #[inline]
    fn poly17(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 18);
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly17k(self, x2, x3, x4, c[0], &c[1..])
    }
    // Quire5 + (x2, x3, x4) = 8
    #[inline]
    fn poly18(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 19);
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly18k(self, x2, x3, x4, c[0], &c[1..])
    }
    // Quire2 + (x2) = 3, more accurate
    #[inline]
    fn poly3a(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 4);
        let x2 = self * self;
        let p = Self::poly1k(self, c[0], c[1]);
        Self::poly2k(self, x2, p, &c[2..])
    }
    // Quire2 + (x2) = 3, more accurate
    #[inline]
    fn poly4a(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 5);
        let x2 = self * self;
        let p = Self::poly2k(self, x2, c[0], &c[1..3]);
        Self::poly2k(self, x2, p, &c[3..])
    }
}

#[cfg(feature = "linalg")]
pub trait QuireDot<T> {
    type Output;
    fn quire_dot(&self, rhs: T) -> Self::Output;
}

#[cfg(feature = "linalg")]
mod linalg;
