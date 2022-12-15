use core::cmp::Ordering;

use crate::u32_zero_shr;

mod convert;
mod math;
mod ops;

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct PxE1<const N: u32>(i32);

impl<const N: u32> PxE1<{ N }> {
    pub const BITS: u32 = N;
    pub const ES: u32 = 1;
    pub const USEED: u32 = 2u32.pow(2u32.pow(Self::ES));

    /// Not a Real (NaR).
    pub const NAR: Self = Self::new(-0x_8000_0000);

    /// Zero.
    pub const ZERO: Self = Self::new(0);

    /// Identity.
    pub const ONE: Self = Self::new(0x_4000_0000);

    #[inline]
    pub const fn new(i: i32) -> Self {
        Self(i)
    }
    #[inline]
    pub const fn is_zero(self) -> bool {
        self.eq(Self::ZERO)
    }
    #[inline]
    pub const fn is_nar(self) -> bool {
        self.eq(Self::NAR)
    }
    #[inline]
    pub const fn from_bits(v: u32) -> Self {
        Self(v as _)
    }
    #[inline]
    pub const fn to_bits(self) -> u32 {
        self.0 as _
    }

    #[inline]
    pub const fn eq(self, other: Self) -> bool {
        self.0 == other.0
    }
    #[inline]
    pub const fn cmp(self, other: Self) -> Ordering {
        let a = self.0;
        let b = other.0;
        if a == b {
            Ordering::Equal
        } else if a < b {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }
    #[inline]
    pub const fn lt(&self, other: Self) -> bool {
        self.0 < other.0
    }
    #[inline]
    pub const fn le(&self, other: Self) -> bool {
        self.0 <= other.0
    }
    #[inline]
    pub const fn ge(&self, other: Self) -> bool {
        self.0 >= other.0
    }
    #[inline]
    pub const fn gt(&self, other: Self) -> bool {
        self.0 > other.0
    }
}

impl<const N: u32> PxE1<{ N }> {
    pub(crate) const fn mask() -> u32 {
        ((-0x_8000_0000_i32) >> (N - 1)) as u32
    }
    pub const SIGN_MASK: u32 = 0x_8000_0000;
    pub const REGIME_SIGN_MASK: u32 = 0x_4000_0000;

    #[inline]
    pub(crate) const fn sign_ui(a: u32) -> bool {
        (a & Self::SIGN_MASK) != 0
    }

    #[inline]
    const fn sign_reg_ui(a: u32) -> bool {
        (a & Self::REGIME_SIGN_MASK) != 0
    }

    #[inline]
    const fn pack_to_ui(regime: u32, exp_a: u32, frac_a: u32) -> u32 {
        regime + exp_a + frac_a
    }

    #[inline]
    pub(crate) const fn separate_bits(bits: u32) -> (i8, i32, u32) {
        let (k, tmp) = Self::separate_bits_tmp(bits);
        (
            k,
            (tmp >> (N - 1 - Self::ES)) as i32,
            (tmp | 0x4000_0000) & 0x7FFF_FFFF,
        )
    }

    #[inline]
    pub(crate) const fn separate_bits_tmp(bits: u32) -> (i8, u32) {
        let mut k = 0;
        let mut tmp = bits << 2;
        if Self::sign_reg_ui(bits) {
            while (tmp & 0x8000_0000) != 0 {
                k += 1;
                tmp <<= 1;
            }
        } else {
            k = -1;
            while (tmp & 0x8000_0000) == 0 {
                k -= 1;
                tmp <<= 1;
            }
            tmp &= 0x7FFF_FFFF;
        }
        (k, tmp)
    }

    #[inline]
    const fn calculate_regime(k: i8) -> (u32, bool, u32) {
        let reg;
        if k < 0 {
            reg = (-k) as u32;
            (u32_zero_shr(0x_4000_0000, reg), false, reg)
        } else {
            reg = (k + 1) as u32;
            (0x_7fff_ffff - u32_zero_shr(0x_7fff_ffff, reg), true, reg)
        }
    }
}

impl<const N: u32> crate::RawPosit for PxE1<{ N }> {
    type UInt = u32;
    type Int = i32;
    const ES_MASK: Self::UInt = u32::MAX >> (u32::BITS - Self::ES);
}
