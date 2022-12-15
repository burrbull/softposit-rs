#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::float_cmp)]
#![allow(clippy::many_single_char_names)]
#![allow(clippy::suspicious_arithmetic_impl)]
#![allow(clippy::verbose_bit_mask)]
#![allow(clippy::excessive_precision)]

pub mod p8e0;
pub use self::p8e0::P8E0;
pub type P8 = P8E0;
pub mod quire8;
pub use self::quire8::Q8E0;
pub type Q8 = Q8E0;

pub mod p16e1;
pub use self::p16e1::P16E1;
pub type P16 = P16E1;
pub mod quire16;
pub use self::quire16::Q16E1;
pub type Q16 = Q16E1;

pub mod p32e2;
pub use self::p32e2::P32E2;
pub type P32 = P32E2;
pub mod quire32;
pub use self::quire32::Q32E2;
pub type Q32 = Q32E2;

pub mod pxe1;
pub use pxe1::PxE1;

pub mod pxe2;
pub use pxe2::PxE2;

mod convert;
use convert::convert_fraction_p32;

pub(crate) mod macros;

pub mod polynom;
pub use polynom::Polynom;

macro_rules! with_sign {
    ($($uint:ty: $ws:ident),*) => {
        $(
            const fn $ws(val: $uint, sign: bool) -> $uint {
                if sign {
                    val.wrapping_neg()
                } else {
                    val
                }
            }
        )*
    }
}

with_sign!(u32: u32_with_sign, u64: u64_with_sign);

const fn lldiv(numer: i64, denom: i64) -> (i64, i64) {
    let mut quot = numer / denom;
    let mut rem = numer % denom;
    if (numer >= 0) && (rem < 0) {
        quot += 1;
        rem -= denom;
    }

    (quot, rem)
}

const fn div(numer: i32, denom: i32) -> (i32, i32) {
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

#[derive(Clone, Copy)]
enum MulAddType {
    Add,
    SubC,
    SubProd,
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
#[cfg(debug_assertions)]
const NTESTS32: usize = 1_000_000;
#[cfg(test)]
#[cfg(not(debug_assertions))]
const NTESTS32: usize = 10_000_000;
#[cfg(test)]
#[cfg(debug_assertions)]
const NTESTS16: usize = 100_000;
#[cfg(test)]
#[cfg(not(debug_assertions))]
const NTESTS16: usize = 1000_000;
#[cfg(test)]
#[cfg(debug_assertions)]
const NTESTS8: usize = 1_000;
#[cfg(test)]
#[cfg(not(debug_assertions))]
const NTESTS8: usize = 10_000;

pub trait AssociatedQuire<P> {
    type Q: Quire<P>;
}

pub trait Quire<P> {
    type Bits;
    fn init() -> Self;
    fn from_posit(p: P) -> Self;
    fn to_posit(&self) -> P;
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

trait RawPosit {
    type UInt;
    type Int;
    const ES_MASK: Self::UInt;
}

trait RawFloat {
    type UInt;
    type Int;

    const BITSIZE: u32;

    const EXPONENT_BITS: u32;
    const EXPONENT_MASK: Self::UInt;

    const EXPONENT_BIAS: Self::Int;

    const SIGNIFICAND_BITS: u32;
    const SIGNIFICAND_MASK: Self::UInt;

    const SIGN_MASK: Self::UInt;
}

impl RawFloat for f32 {
    type UInt = u32;
    type Int = i32;

    const BITSIZE: u32 = 32;

    const EXPONENT_BITS: u32 = 8;
    const EXPONENT_MASK: Self::UInt = 0x_7f80_0000;

    const EXPONENT_BIAS: Self::Int = (Self::MAX_EXP - 1) as _;

    const SIGNIFICAND_BITS: u32 = Self::MANTISSA_DIGITS - 1;
    const SIGNIFICAND_MASK: Self::UInt = u32::MAX >> (u32::BITS - Self::SIGNIFICAND_BITS);

    const SIGN_MASK: Self::UInt = 0x8000_0000;
}

impl RawFloat for f64 {
    type UInt = u64;
    type Int = i64;

    const BITSIZE: u32 = 64;

    const EXPONENT_BITS: u32 = 11;
    const EXPONENT_MASK: Self::UInt = 0x_7ff0_0000_0000_0000;

    const EXPONENT_BIAS: Self::Int = (Self::MAX_EXP - 1) as _;

    const SIGNIFICAND_BITS: u32 = Self::MANTISSA_DIGITS - 1;
    const SIGNIFICAND_MASK: Self::UInt = u64::MAX >> (u64::BITS - Self::SIGNIFICAND_BITS);

    const SIGN_MASK: Self::UInt = 0x_8000_0000_0000_0000;
}

const fn u8_zero_shr(val: u8, rhs: u32) -> u8 {
    if rhs > 7 {
        0
    } else {
        val.wrapping_shr(rhs)
    }
}

const fn u16_zero_shr(val: u16, rhs: u32) -> u16 {
    if rhs > 15 {
        0
    } else {
        val.wrapping_shr(rhs)
    }
}

const fn u32_zero_shr(val: u32, rhs: u32) -> u32 {
    if rhs > 31 {
        0
    } else {
        val.wrapping_shr(rhs)
    }
}

const fn u64_zero_shr(val: u64, rhs: u32) -> u64 {
    if rhs > 63 {
        0
    } else {
        val.wrapping_shr(rhs)
    }
}
