use core::mem;

mod convert;
mod math;
#[cfg(feature = "num-traits")]
mod num;
mod ops;

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Hash)]
pub struct P8E0(i8);

impl P8E0 {
    pub const SIZE: usize = 8;
    pub const ES: usize = 0;

    /// Machine epsilon (3.125e-2).
    pub const EPSILON: Self = Self::new(0x_2);

    /// Smallest finite value (-64).
    pub const MIN: Self = Self::new(-0x_7F);

    /// Smallest positive normal value (0.015625).
    pub const MIN_POSITIVE: Self = Self::new(0x_1);

    /// Largest finite value (64).
    pub const MAX: Self = Self::new(0x_7F);

    /// Not a Number (NaN).
    pub const NAN: Self = Self::new(-0x_80);

    /// Infinity (∞).
    pub const INFINITY: Self = Self::new(-0x_80);

    /// Zero.
    pub const ZERO: Self = Self::new(0);

    /// Identity.
    pub const ONE: Self = Self::new(0x_40);

    #[inline]
    pub const fn new(i: i8) -> Self {
        Self(i)
    }
    #[inline]
    pub fn from_bits(v: u8) -> Self {
        unsafe { mem::transmute(v) }
    }
    #[inline]
    pub fn to_bits(self) -> u8 {
        unsafe { mem::transmute(self) }
    }
    #[inline]
    pub fn abs(self) -> Self {
        let i = self.to_bits() as i8;
        Self::from_bits((if i < 0 { -i } else { i }) as u8)
    }
    #[inline]
    pub fn is_nan(self) -> bool {
        self == Self::NAN
    }
    #[inline]
    pub fn is_infinite(self) -> bool {
        self == Self::INFINITY
    }
    #[inline]
    pub fn is_finite(self) -> bool {
        !self.is_nan()
    }
    #[inline]
    pub fn max(self, other: Self) -> Self {
        if self.is_nan() || (self < other) {
            other
        } else {
            self
        }
    }
    #[inline]
    pub fn min(self, other: Self) -> Self {
        if other.is_nan() || (self < other) {
            self
        } else {
            other
        }
    }
}

impl P8E0 {
    pub const SIGN_MASK: u8 = 0x_80;
    pub const REGIME_SIGN_MASK: u8 = 0x_40;

    #[inline]
    pub(crate) fn sign_ui(a: u8) -> bool {
        (a & Self::SIGN_MASK) != 0
    }

    #[inline]
    fn sign_reg_ui(a: u8) -> bool {
        (a & Self::REGIME_SIGN_MASK) != 0
    }

    #[inline]
    fn pack_to_ui(regime: u8, frac_a: u8) -> u8 {
        regime + frac_a
    }

    #[inline]
    pub(crate) fn separate_bits(bits: u8) -> (i8, u8) {
        let (k, tmp) = Self::separate_bits_tmp(bits);
        (k, 0x80 | tmp)
    }

    #[inline]
    pub(crate) fn separate_bits_tmp(bits: u8) -> (i8, u8) {
        let mut k = 0;
        let mut tmp = bits << 2;
        if Self::sign_reg_ui(bits) {
            while (tmp & 0x_80) != 0 {
                k += 1;
                tmp <<= 1;
            }
        } else {
            k = -1;
            while (tmp & 0x_80) == 0 {
                k -= 1;
                tmp <<= 1;
            }
            tmp &= 0x7F;
        }
        (k, tmp)
    }

    #[inline]
    fn calculate_scale(mut bits: u8) -> (u8, u8) {
        let mut scale = 0_u8;
        // Decode the posit, left-justifying as we go.
        bits -= 0x40; // Strip off first regime bit (which is a 1).
        while (0x20 & bits) != 0 {
            // Increment scale one for each regime sign bit.
            scale += 1; // Regime sign bit is always 1 in this range.
            bits = (bits - 0x20) << 1; // Remove the bit; line up the next regime bit.
        }
        bits <<= 1; // Skip over termination bit, which is 0.
        (scale, bits)
    }

    #[inline]
    fn calculate_regime(k: i8) -> (u8, bool, u8) {
        let reg;
        if k < 0 {
            reg = (-k) as u8;
            (if reg > 7 { 0 } else { 0x40 >> reg }, false, reg)
        } else {
            reg = (k + 1) as u8;
            (if reg > 7 { 0x7F } else { 0x7F - (0x7F >> reg) }, true, reg)
        }
    }
}

#[derive(Clone, Debug)]
pub struct Q8E0(i32);

impl Q8E0 {
    pub const ZERO: Self = Self(0);
    pub const NAN: Self = Self(-0x8000_0000);

    #[inline]
    pub const fn new() -> Self {
        Self::ZERO
    }

    #[inline]
    pub fn from_bits(v: u32) -> Self {
        unsafe { mem::transmute(v) }
    }

    #[inline]
    pub fn to_bits(&self) -> u32 {
        unsafe { mem::transmute(self.clone()) }
    }

    #[inline]
    pub fn is_zero(&self) -> bool {
        self.to_bits() == 0
    }

    #[inline]
    pub fn is_nan(&self) -> bool {
        self.to_bits() == 0x8000_0000
    }

    #[inline]
    pub fn qma(&mut self, p_a: P8E0, p_b: P8E0) {
        ops::q8_fdp_add(self, p_a, p_b);
    }

    #[inline]
    pub fn qms(&mut self, p_a: P8E0, p_b: P8E0) {
        ops::q8_fdp_sub(self, p_a, p_b);
    }

    #[inline]
    pub fn roundp(self) -> P8E0 {
        P8E0::from(self)
    }

    #[inline]
    pub fn clear(&mut self) {
        *self = Self::ZERO;
    }

    #[inline]
    pub fn neg(&mut self) {
        self.0 = -(self.0);
    }
}

impl core::str::FromStr for P8E0 {
    type Err = core::num::ParseFloatError;
    #[inline]
    fn from_str(src: &str) -> Result<Self, core::num::ParseFloatError> {
        Ok(Self::from(f64::from_str(src)?))
    }
}

use core::fmt;
impl fmt::Display for P8E0 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", f64::from(*self))
    }
}

impl fmt::Display for Q8E0 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", f64::from(self.clone().roundp()))
    }
}

impl fmt::Debug for P8E0 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "P8E0({})", self.0)
    }
}

impl crate::Quire for P8E0 {
    type Q = Q8E0;
}

impl crate::Poly for P8E0 {
    #[inline]
    fn poly1k(x: Self, c: &[Self]) -> Self {
        let mut q = Q8E0::new(); // QCLR.S
        q += (c[1], x); // QMADD.S
        q += (c[0], Self::ONE);
        q.into() // QROUND.S
    }
    #[inline]
    fn poly2k(x: Self, x2: Self, c: &[Self], p: Self) -> Self {
        let mut q = Q8E0::new();
        q += (p, x2);
        q += (c[1], x);
        q += (c[0], Self::ONE);
        q.into()
    }
    #[inline]
    fn poly3k(x: Self, x2: Self, x3: Self, c: &[Self], p: Self) -> Self {
        let mut q = Q8E0::new();
        q += (p, x3);
        q += (c[2], x2);
        q += (c[1], x);
        q += (c[0], Self::ONE);
        q.into()
    }
    #[inline]
    fn poly4k(x: Self, x2: Self, x3: Self, x4: Self, c: &[Self], p: Self) -> Self {
        let mut q = Q8E0::new();
        q += (p, x4);
        q += (c[3], x3);
        q += (c[2], x2);
        q += (c[1], x);
        q += (c[0], Self::ONE);
        q.into()
    }
}

impl crate::Polynom for P8E0 {}
