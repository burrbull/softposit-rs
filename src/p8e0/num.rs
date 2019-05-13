use super::P8E0;

impl num_traits::Zero for P8E0 {
    fn zero() -> Self {
        Self::ZERO
    }
    fn is_zero(&self) -> bool {
        *self == Self::ZERO
    }
}

impl num_traits::Num for P8E0 {
    type FromStrRadixErr = num_traits::ParseFloatError;
    fn from_str_radix(src: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        Ok(Self::from(f64::from_str_radix(src, radix)?))
    }
}

impl num_traits::One for P8E0 {
    fn one() -> Self {
        Self::new(0x_40)
    }
    fn is_one(&self) -> bool {
        *self == Self::new(0x_40)
    }
}

impl num_traits::ToPrimitive for P8E0 {
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

impl num_traits::NumCast for P8E0 {
    fn from<N: num_traits::ToPrimitive>(n: N) -> Option<Self> {
        n.to_f64().map(|x| x.into())
    }
}
