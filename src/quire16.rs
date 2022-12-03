use crate::P16E1;
use core::mem;

mod convert;
mod math;
mod ops;

#[derive(Clone, Debug)]
pub struct Q16E1(i128);

impl Q16E1 {
    pub const ZERO: Self = Self(0);
    pub const ONE: Self = Self(0x_0100_0000_0000_0000_0000_0000_0000_0000);
    pub const NAR: Self = Self(-0x_8000_0000_0000_0000_0000_0000_0000_0000);

    #[inline]
    pub const fn init() -> Self {
        Self::ZERO
    }

    #[inline]
    pub fn from_posit(p: P16E1) -> Self {
        Self::from(p)
    }

    #[inline]
    pub fn from_bits(v: u128) -> Self {
        unsafe { mem::transmute(v) }
    }

    #[inline]
    pub fn to_bits(&self) -> u128 {
        unsafe { mem::transmute(self.clone()) }
    }

    #[inline]
    pub fn is_zero(&self) -> bool {
        self.to_bits() == Self::ZERO.to_bits()
    }

    #[inline]
    pub fn is_nar(&self) -> bool {
        self.to_bits() == Self::NAR.to_bits()
    }

    #[inline]
    pub fn add_product(&mut self, p_a: P16E1, p_b: P16E1) {
        let ui_a = p_a.to_bits();
        let ui_b = p_b.to_bits();
        ops::fdp(self, ui_a, ui_b, true);
    }

    #[inline]
    pub fn sub_product(&mut self, p_a: P16E1, p_b: P16E1) {
        let ui_a = p_a.to_bits();
        let ui_b = p_b.to_bits();
        ops::fdp(self, ui_a, ui_b, false);
    }

    #[inline]
    pub fn to_posit(&self) -> P16E1 {
        P16E1::from(self)
    }

    #[inline]
    pub fn clear(&mut self) {
        *self = Self::ZERO;
    }

    #[inline]
    pub fn neg(&mut self) {
        self.0 = self.0.wrapping_neg();
    }

    #[inline]
    pub fn into_two_posits(mut self) -> (P16E1, P16E1) {
        let p1 = self.to_posit();
        self -= p1;
        (p1, self.to_posit())
    }

    #[inline]
    pub fn into_three_posits(mut self) -> (P16E1, P16E1, P16E1) {
        let p1 = self.to_posit();
        self -= p1;
        let p2 = self.to_posit();
        self -= p2;
        (p1, p2, self.to_posit())
    }
}

impl crate::Quire<P16E1> for Q16E1 {
    type Bits = u128;
    fn init() -> Self {
        Self::init()
    }
    fn from_posit(p: P16E1) -> Self {
        Self::from_posit(p)
    }
    fn to_posit(&self) -> P16E1 {
        Self::to_posit(self)
    }
    fn from_bits(v: Self::Bits) -> Self {
        Self::from_bits(v)
    }
    fn to_bits(&self) -> Self::Bits {
        Self::to_bits(self)
    }
    fn is_zero(&self) -> bool {
        Self::is_zero(self)
    }
    fn is_nar(&self) -> bool {
        Self::is_nar(self)
    }
    fn add_product(&mut self, p_a: P16E1, p_b: P16E1) {
        Self::add_product(self, p_a, p_b)
    }
    fn sub_product(&mut self, p_a: P16E1, p_b: P16E1) {
        Self::sub_product(self, p_a, p_b)
    }
    fn clear(&mut self) {
        Self::clear(self)
    }
    fn neg(&mut self) {
        Self::neg(self)
    }
}

use core::fmt;
impl fmt::Display for Q16E1 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", f64::from(self.clone().to_posit()))
    }
}
