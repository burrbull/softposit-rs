#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(feature = "nightly", feature(const_generics))]
#![allow(clippy::cast_lossless)]
#![allow(clippy::float_cmp)]
#![allow(clippy::many_single_char_names)]
#![allow(clippy::suspicious_arithmetic_impl)]

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

#[cfg(feature = "nightly")]
pub mod pxe2;

mod convert;

mod macros;

pub mod polynom;
pub use polynom::Polynom;

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

#[allow(clippy::declare_interior_mutable_const)]
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

#[cfg(feature = "linalg")]
pub trait QuireDot<T> {
    type Output;
    fn quire_dot(&self, rhs: T) -> Self::Output;
}

#[cfg(feature = "linalg")]
mod linalg;
