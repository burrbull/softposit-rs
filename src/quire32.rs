use crate::{PxE2, P32E2};

mod convert;
mod math;
mod ops;

#[derive(Debug)]
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
    pub const fn from_bits(v: [u64; 8]) -> Self {
        Self(v[0] as _, v[1], v[2], v[3], v[4], v[5], v[6], v[7])
    }

    #[inline]
    pub const fn to_bits(&self) -> [u64; 8] {
        [
            self.0 as _,
            self.1,
            self.2,
            self.3,
            self.4,
            self.5,
            self.6,
            self.7,
        ]
    }

    #[inline]
    pub const fn is_zero(&self) -> bool {
        self.0 == 0
            && self.1 == 0
            && self.2 == 0
            && self.3 == 0
            && self.4 == 0
            && self.5 == 0
            && self.7 == 0
    }

    #[inline]
    pub const fn is_nar(&self) -> bool {
        self.0 as u64 == 0x8000_0000_0000_0000
            && self.1 == 0
            && self.2 == 0
            && self.3 == 0
            && self.4 == 0
            && self.5 == 0
            && self.7 == 0
    }

    #[inline]
    pub fn add_product(&mut self, p_a: P32E2, p_b: P32E2) {
        let ui_a = p_a.to_bits();
        let ui_b = p_b.to_bits();
        ops::fdp(self, ui_a, ui_b, true);
    }

    #[inline]
    pub fn sub_product(&mut self, p_a: P32E2, p_b: P32E2) {
        let ui_a = p_a.to_bits();
        let ui_b = p_b.to_bits();
        ops::fdp(self, ui_a, ui_b, false);
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
        Self::to_bits(self)
    }
    fn is_zero(&self) -> bool {
        Self::is_zero(self)
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
        Self::to_bits(self)
    }
    fn is_zero(&self) -> bool {
        Self::is_zero(self)
    }
    fn is_nar(&self) -> bool {
        Self::is_nar(self)
    }
    fn add_product(&mut self, p_a: PxE2<{ N }>, p_b: PxE2<{ N }>) {
        let ui_a = p_a.to_bits();
        let ui_b = p_b.to_bits();
        ops::fdp(self, ui_a, ui_b, true);
    }
    fn sub_product(&mut self, p_a: PxE2<{ N }>, p_b: PxE2<{ N }>) {
        let ui_a = p_a.to_bits();
        let ui_b = p_b.to_bits();
        ops::fdp(self, ui_a, ui_b, false);
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
        write!(f, "{}", f64::from(self.to_posit()))
    }
}
