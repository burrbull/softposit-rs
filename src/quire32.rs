#[cfg(feature = "nightly")]
use crate::PxE2;
use crate::P32E2;
use core::mem;

mod convert;
mod math;
mod ops;

#[derive(Clone, Debug)]
pub struct Q32E2(i64, u64, u64, u64, u64, u64, u64, u64);

impl Q32E2 {
    pub const ZERO: Self = Self(0, 0, 0, 0, 0, 0, 0, 0);
    pub const ONE: Self = Self(0, 0, 0, 0, 0x_0001_0000_0000_0000, 0, 0, 0);
    pub const NAR: Self = Self(-0x_8000_0000_0000_0000, 0, 0, 0, 0, 0, 0, 0);

    #[inline]
    pub const fn init() -> Self {
        Self::ZERO
    }

    #[inline]
    pub fn from_posit(p: P32E2) -> Self {
        Self::from(p)
    }

    #[inline]
    pub fn from_bits(v: [u64; 8]) -> Self {
        unsafe { mem::transmute(v) }
    }

    #[inline]
    pub fn to_bits(&self) -> [u64; 8] {
        unsafe { mem::transmute(self.clone()) }
    }

    #[inline]
    pub fn is_zero(&self) -> bool {
        self.to_bits() == [0, 0, 0, 0, 0, 0, 0, 0]
    }

    #[inline]
    pub fn is_nar(&self) -> bool {
        self.to_bits() == [0x8000_0000_0000_0000, 0, 0, 0, 0, 0, 0, 0]
    }

    #[inline]
    pub fn add_product(&mut self, p_a: P32E2, p_b: P32E2) {
        let ui_a = p_a.to_bits();
        let ui_b = p_b.to_bits();
        ops::fdp_add(self, ui_a, ui_b);
    }

    #[inline]
    pub fn sub_product(&mut self, p_a: P32E2, p_b: P32E2) {
        let ui_a = p_a.to_bits();
        let ui_b = p_b.to_bits();
        ops::fdp_sub(self, ui_a, ui_b);
    }

    #[inline]
    pub fn to_posit(&self) -> P32E2 {
        P32E2::from(self)
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
    pub fn into_two_posits(mut self) -> (P32E2, P32E2) {
        let p1 = self.to_posit();
        self -= p1;
        (p1, self.to_posit())
    }

    #[inline]
    pub fn into_three_posits(mut self) -> (P32E2, P32E2, P32E2) {
        let p1 = self.to_posit();
        self -= p1;
        let p2 = self.to_posit();
        self -= p2;
        (p1, p2, self.to_posit())
    }
}

impl crate::Quire<P32E2> for Q32E2 {
    type Bits = [u64; 8];
    fn init() -> Self {
        Self::init()
    }
    fn from_posit(p: P32E2) -> Self {
        Self::from_posit(p)
    }
    fn to_posit(&self) -> P32E2 {
        Self::to_posit(self)
    }
    fn from_bits(v: Self::Bits) -> Self {
        Self::from_bits(v)
    }
    fn to_bits(&self) -> Self::Bits {
        Self::to_bits(&self)
    }
    fn is_zero(&self) -> bool {
        Self::is_zero(&self)
    }
    fn is_nar(&self) -> bool {
        Self::is_nar(self)
    }
    fn add_product(&mut self, p_a: P32E2, p_b: P32E2) {
        Self::add_product(self, p_a, p_b)
    }
    fn sub_product(&mut self, p_a: P32E2, p_b: P32E2) {
        Self::sub_product(self, p_a, p_b)
    }
    fn clear(&mut self) {
        Self::clear(self)
    }
    fn neg(&mut self) {
        Self::neg(self)
    }
}

#[cfg(feature = "nightly")]
impl<const N: u32> crate::Quire<PxE2<{ N }>> for Q32E2 {
    type Bits = [u64; 8];
    fn init() -> Self {
        Self::init()
    }
    fn from_posit(p: PxE2<{ N }>) -> Self {
        Self::from(p)
    }
    fn to_posit(&self) -> PxE2<{ N }> {
        PxE2::<{ N }>::from(self)
    }
    fn from_bits(v: Self::Bits) -> Self {
        Self::from_bits(v)
    }
    fn to_bits(&self) -> Self::Bits {
        Self::to_bits(&self)
    }
    fn is_zero(&self) -> bool {
        Self::is_zero(&self)
    }
    fn is_nar(&self) -> bool {
        Self::is_nar(self)
    }
    fn add_product(&mut self, p_a: PxE2<{ N }>, p_b: PxE2<{ N }>) {
        let ui_a = p_a.to_bits();
        let ui_b = p_b.to_bits();
        ops::fdp_add(self, ui_a, ui_b);
    }
    fn sub_product(&mut self, p_a: PxE2<{ N }>, p_b: PxE2<{ N }>) {
        let ui_a = p_a.to_bits();
        let ui_b = p_b.to_bits();
        ops::fdp_sub(self, ui_a, ui_b);
    }
    fn clear(&mut self) {
        Self::clear(self)
    }
    fn neg(&mut self) {
        Self::neg(self)
    }
}

use core::fmt;
impl fmt::Display for Q32E2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", f64::from(self.clone().to_posit()))
    }
}
