use super::P16E1;

impl num_traits::Zero for P16E1 {
    fn zero() -> Self {
        Self::ZERO
    }
    fn is_zero(&self) -> bool {
        *self == Self::ZERO
    }
}

impl num_traits::One for P16E1 {
    fn one() -> Self {
        Self::ONE
    }
    fn is_one(&self) -> bool {
        *self == Self::ONE
    }
}

#[cfg(feature = "num-traits")]
impl num_traits::Num for P16E1 {
    type FromStrRadixErr = num_traits::ParseFloatError;
    fn from_str_radix(src: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        Ok(Self::from(f64::from_str_radix(src, radix)?))
    }
}

impl num_traits::ToPrimitive for P16E1 {
    fn to_i64(&self) -> Option<i64> {
        Some(i64::from(*self))
    }
    fn to_u64(&self) -> Option<u64> {
        Some(u64::from(*self))
    }
    fn to_f64(&self) -> Option<f64> {
        Some(f64::from(*self))
    }
}

impl num_traits::NumCast for P16E1 {
    fn from<N: num_traits::ToPrimitive>(n: N) -> Option<Self> {
        n.to_f64().map(|x| x.into())
    }
}
