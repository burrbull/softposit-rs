use super::{P8E0, Q8E0};
use crate::WithSign;
use core::ops;

impl ops::Neg for P8E0 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self::new(self.0.wrapping_neg())
    }
}

impl ops::AddAssign for P8E0 {
    #[inline]
    fn add_assign(&mut self, other: Self) {
        *self = *self + other
    }
}

impl ops::SubAssign for P8E0 {
    #[inline]
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other
    }
}

impl ops::MulAssign for P8E0 {
    #[inline]
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other
    }
}

impl ops::DivAssign for P8E0 {
    #[inline]
    fn div_assign(&mut self, other: Self) {
        *self = *self / other
    }
}

impl ops::RemAssign for P8E0 {
    #[inline]
    fn rem_assign(&mut self, other: Self) {
        *self = *self % other
    }
}

impl ops::Add for P8E0 {
    type Output = Self;
    #[inline]
    fn add(self, other: Self) -> Self {
        let ui_a = self.to_bits();
        let ui_b = other.to_bits();

        //Zero or infinity
        if self.is_zero() || other.is_zero() {
            // Not required but put here for speed
            Self::from_bits(ui_a | ui_b)
        } else if self.is_nar() || other.is_nar() {
            Self::NAR
        } else {
            //different signs
            if Self::sign_ui(ui_a ^ ui_b) {
                Self::sub_mags(ui_a, ui_b)
            } else {
                Self::add_mags(ui_a, ui_b)
            }
        }
    }
}

impl ops::Sub for P8E0 {
    type Output = Self;
    #[inline]
    fn sub(self, other: Self) -> Self {
        let ui_a = self.to_bits();
        let ui_b = other.to_bits();

        if self.is_nar() || other.is_nar() {
            //infinity
            Self::NAR
        } else if self.is_zero() || other.is_zero() {
            //Zero
            Self::from_bits(ui_a | ui_b.wrapping_neg())
        } else {
            //different signs
            if Self::sign_ui(ui_a ^ ui_b) {
                Self::add_mags(ui_a, ui_b.wrapping_neg())
            } else {
                Self::sub_mags(ui_a, ui_b.wrapping_neg())
            }
        }
    }
}

impl ops::Div for P8E0 {
    type Output = Self;
    #[inline]
    fn div(self, other: Self) -> Self {
        let mut ui_a = self.to_bits();
        let mut ui_b = other.to_bits();

        //Zero or infinity
        if self.is_nar() || other.is_nar() || other.is_zero() {
            return Self::NAR;
        } else if self.is_zero() {
            return Self::ZERO;
        }

        let sign_a = Self::sign_ui(ui_a);
        let sign_b = Self::sign_ui(ui_b);
        let sign_z = sign_a ^ sign_b;
        if sign_a {
            ui_a = ui_a.wrapping_neg();
        }
        if sign_b {
            ui_b = ui_b.wrapping_neg();
        }

        let (mut k_a, frac_a) = Self::separate_bits(ui_a);
        let (k_b, frac_b) = Self::separate_bits(ui_b);
        k_a -= k_b;

        let frac16_a = (frac_a as u16) << 7; //hidden bit 2nd bit

        let (quot, rem) = crate::div(frac16_a as i32, frac_b as i32);
        let mut frac16_z = quot as u16;

        if frac16_z != 0 {
            let rcarry = (frac16_z >> 7) != 0; // this is the hidden bit (7th bit) , extreme right bit is bit 0
            if !rcarry {
                k_a -= 1;
                frac16_z <<= 1;
            }
        }

        let (regime, reg_sa, reg_a) = Self::calculate_regime(k_a);

        let u_z = if reg_a > 6 {
            //max or min pos. exp and frac does not matter.
            if reg_sa {
                0x7F
            } else {
                0x1
            }
        } else {
            //remove carry and rcarry bits and shift to correct position
            frac16_z &= 0x7F;
            let frac_a = (frac16_z >> (reg_a + 1)) as u8;

            let bit_n_plus_one = (0x1 & (frac16_z >> reg_a)) != 0;
            let mut u_z = Self::pack_to_ui(regime, frac_a);

            if bit_n_plus_one {
                let bits_more = if rem != 0 {
                    true
                } else {
                    (((1 << reg_a) - 1) & frac16_z) != 0
                };
                //n+1 frac bit is 1. Need to check if another bit is 1 too if not round to even
                u_z += (u_z & 1) | (bits_more as u8);
            }
            u_z
        };

        Self::from_bits(u_z.with_sign(sign_z))
    }
}

impl ops::Mul for P8E0 {
    type Output = Self;
    #[inline]
    fn mul(self, other: Self) -> Self {
        let mut ui_a = self.to_bits();
        let mut ui_b = other.to_bits();

        //NaR or Zero
        if self.is_nar() || other.is_nar() {
            return Self::NAR;
        } else if self.is_zero() || other.is_zero() {
            return Self::ZERO;
        }

        let sign_a = Self::sign_ui(ui_a);
        let sign_b = Self::sign_ui(ui_b);
        let sign_z = sign_a ^ sign_b;

        if sign_a {
            ui_a = ui_a.wrapping_neg();
        }
        if sign_b {
            ui_b = ui_b.wrapping_neg();
        }

        let (mut k_a, frac_a) = Self::separate_bits(ui_a);
        let (k_b, frac_b) = Self::separate_bits(ui_b);
        k_a += k_b;

        let mut frac16_z = (frac_a as u16) * (frac_b as u16);

        let rcarry = (frac16_z & 0x_8000) != 0; //1st bit of frac32Z
        if rcarry {
            k_a += 1;
            frac16_z >>= 1;
        }

        let (regime, reg_sa, reg_a) = Self::calculate_regime(k_a);

        let u_z = if reg_a > 6 {
            //max or min pos. exp and frac does not matter.
            if reg_sa {
                0x7F
            } else {
                0x1
            }
        } else {
            //remove carry and rcarry bits and shift to correct position
            frac16_z = (frac16_z & 0x3FFF) >> reg_a;
            let frac_a = (frac16_z >> 8) as u8;
            let bit_n_plus_one = (frac16_z & 0x80) != 0;
            let mut u_z = Self::pack_to_ui(regime, frac_a);

            //n+1 frac bit is 1. Need to check if another bit is 1 too if not round to even
            if bit_n_plus_one {
                let bits_more = (frac16_z & 0x7F) != 0;
                u_z += (u_z & 1) | (bits_more as u8);
            }
            u_z
        };

        Self::from_bits(u_z.with_sign(sign_z))
    }
}

impl P8E0 {
    #[inline]
    fn add_mags(mut ui_a: u8, mut ui_b: u8) -> Self {
        let sign = Self::sign_ui(ui_a);
        if sign {
            ui_a = ui_a.wrapping_neg();
            ui_b = ui_b.wrapping_neg();
        }

        if (ui_a as i8) < (ui_b as i8) {
            ui_a ^= ui_b;
            ui_b ^= ui_a;
            ui_a ^= ui_b;
        }

        let (mut k_a, frac_a) = Self::separate_bits(ui_a);
        let mut frac16_a = (frac_a as u16) << 7;

        let (k_b, frac_b) = Self::separate_bits(ui_b);
        let shift_right = (k_a as i16) - (k_b as i16);

        //Manage CLANG (LLVM) compiler when shifting right more than number of bits
        let frac16_b = if shift_right > 7 {
            0
        } else {
            (frac_b as u16) << (7 - shift_right)
        };

        frac16_a += frac16_b;

        let rcarry = (0x8000 & frac16_a) != 0; //first left bit
        if rcarry {
            k_a += 1;
            frac16_a >>= 1;
        }

        let (regime, reg_sa, reg_a) = Self::calculate_regime(k_a);

        let u_z = if reg_a > 6 {
            //max or min pos. exp and frac does not matter.
            if reg_sa {
                0x7F
            } else {
                0x1
            }
        } else {
            frac16_a = (frac16_a & 0x3FFF) >> reg_a;
            let frac_a = (frac16_a >> 8) as u8;
            let bit_n_plus_one = (0x80 & frac16_a) != 0;
            let mut u_z = Self::pack_to_ui(regime, frac_a);

            //n+1 frac bit is 1. Need to check if another bit is 1 too if not round to even
            if bit_n_plus_one {
                let bits_more = (0x7F & frac16_a) != 0;
                u_z += (u_z & 1) | (bits_more as u8);
            }
            u_z
        };
        Self::from_bits(u_z.with_sign(sign))
    }

    #[inline]
    fn sub_mags(mut ui_a: u8, mut ui_b: u8) -> Self {
        //Both ui_a and ui_b are actually the same signs if ui_b inherits sign of sub
        //Make both positive
        let mut sign = Self::sign_ui(ui_a);
        if sign {
            ui_a = ui_a.wrapping_neg();
        } else {
            ui_b = ui_b.wrapping_neg();
        }
        if ui_a == ui_b {
            //essential, if not need special handling
            return Self::ZERO;
        }
        if ui_a < ui_b {
            ui_a ^= ui_b;
            ui_b ^= ui_a;
            ui_a ^= ui_b;
            sign = !sign; //A becomes B
        }

        let (mut k_a, frac_a) = Self::separate_bits(ui_a);
        let mut frac16_a = (frac_a as u16) << 7;

        let (k_b, frac_b) = Self::separate_bits(ui_b);
        let shift_right = (k_a as i16) - (k_b as i16);

        let mut frac16_b = (frac_b as u16) << 7;

        if shift_right >= 14 {
            return Self::from_bits(ui_a.with_sign(sign));
        } else {
            frac16_b >>= shift_right;
        }
        frac16_a -= frac16_b;

        while (frac16_a >> 14) == 0 {
            k_a -= 1;
            frac16_a <<= 1;
        }
        let ecarry = ((0x4000 & frac16_a) >> 14) != 0;
        if !ecarry {
            k_a -= 1;
            frac16_a <<= 1;
        }

        let (regime, reg_sa, reg_a) = Self::calculate_regime(k_a);

        let u_z = if reg_a > 6 {
            //max or min pos. exp and frac does not matter.
            if reg_sa {
                0x7F
            } else {
                0x1
            }
        } else {
            frac16_a = (frac16_a & 0x3FFF) >> reg_a;
            let frac_a = (frac16_a >> 8) as u8;
            let bit_n_plus_one = (0x80 & frac16_a) != 0;
            let mut u_z = Self::pack_to_ui(regime, frac_a);

            if bit_n_plus_one {
                let bits_more = (0x7F & frac16_a) != 0;
                u_z += (u_z & 1) | (bits_more as u8);
            }
            u_z
        };
        Self::from_bits(u_z.with_sign(sign))
    }
}

impl ops::Rem for P8E0 {
    type Output = Self;
    fn rem(self, _other: Self) -> Self {
        unimplemented!()
    }
}

impl ops::AddAssign<(P8E0, P8E0)> for Q8E0 {
    #[inline]
    fn add_assign(&mut self, rhs: (P8E0, P8E0)) {
        q8_fdp_add(self, rhs.0, rhs.1);
    }
}

impl ops::SubAssign<(P8E0, P8E0)> for Q8E0 {
    #[inline]
    fn sub_assign(&mut self, rhs: (P8E0, P8E0)) {
        q8_fdp_sub(self, rhs.0, rhs.1);
    }
}

crate::add_sub_array!(P8E0, Q8E0, 1, 2, 3, 4);

pub(super) fn q8_fdp_add(q: &mut Q8E0, p_a: P8E0, p_b: P8E0) {
    let uq_z1 = q.to_bits();

    let mut ui_a = p_a.to_bits();
    let mut ui_b = p_b.to_bits();

    if q.is_nar() || p_a.is_nar() || p_b.is_nar() {
        *q = Q8E0::NAR;
        return;
    } else if p_a.is_zero() || p_b.is_zero() {
        return;
    }

    //max pos (sign plus and minus)
    let sign_a = P8E0::sign_ui(ui_a);
    let sign_b = P8E0::sign_ui(ui_b);
    let sign_z2 = sign_a ^ sign_b;

    if sign_a {
        ui_a = ui_a.wrapping_neg();
    }
    if sign_b {
        ui_b = ui_b.wrapping_neg();
    }

    let (mut k_a, frac_a) = P8E0::separate_bits(ui_a);

    let (k_b, frac_b) = P8E0::separate_bits(ui_b);
    k_a += k_b;

    let mut frac32_z = ((frac_a as u32) * (frac_b as u32)) << 16;

    let rcarry = (frac32_z & 0x8000_0000) != 0; //1st bit (position 2) of frac32_z, hidden bit is 4th bit (position 3)
    if rcarry {
        k_a += 1;
        frac32_z >>= 1;
    }

    //default dot is between bit 19 and 20, extreme left bit is bit 0. Last right bit is bit 31.
    //Scale = 2^es * k + e  => 2k + e // firstPost = 19-k_a, shift = firstPos -1 (because frac32_z start from 2nd bit)
    //int firstPos = 19 - k_a;
    let shift_right = 18 - k_a;

    let mut uq_z2 = frac32_z >> shift_right;

    if sign_z2 {
        uq_z2 = uq_z2.wrapping_neg();
    }

    //Addition
    let uq_z = uq_z2.wrapping_add(uq_z1);

    //Exception handling
    let q_z = Q8E0::from_bits(uq_z);
    *q = if q_z.is_nar() { Q8E0::ZERO } else { q_z }
}

//q - (p_a*p_b)

pub(super) fn q8_fdp_sub(q: &mut Q8E0, p_a: P8E0, p_b: P8E0) {
    let uq_z1 = q.to_bits();

    let mut ui_a = p_a.to_bits();
    let mut ui_b = p_b.to_bits();

    if q.is_nar() || p_a.is_nar() || p_b.is_nar() {
        *q = Q8E0::NAR;
        return;
    } else if p_a.is_zero() || p_b.is_zero() {
        return;
    }

    //max pos (sign plus and minus)
    let sign_a = P8E0::sign_ui(ui_a);
    let sign_b = P8E0::sign_ui(ui_b);
    let sign_z2 = sign_a ^ sign_b;

    if sign_a {
        ui_a = ui_a.wrapping_neg();
    }
    if sign_b {
        ui_b = ui_b.wrapping_neg();
    }

    let (mut k_a, frac_a) = P8E0::separate_bits(ui_a);

    let (k_b, frac_b) = P8E0::separate_bits(ui_b);
    k_a += k_b;

    let mut frac32_z = ((frac_a as u32) * (frac_b as u32)) << 16;

    let rcarry = (frac32_z & 0x8000_0000) != 0; //1st bit (position 2) of frac32_z, hidden bit is 4th bit (position 3)
    if rcarry {
        k_a += 1;
        frac32_z >>= 1;
    }

    //default dot is between bit 19 and 20, extreme left bit is bit 0. Last right bit is bit 31.
    //Scale = 2^es * k + e  => 2k + e // firstPost = 19-k_a, shift = firstPos -1 (because frac32_z start from 2nd bit)
    //int firstPos = 19 - k_a;
    let shift_right = 18 - k_a;

    let mut uq_z2 = frac32_z >> shift_right;

    //This is the only difference from ADD (sign_z2) and (!sign_z2)
    if !sign_z2 {
        uq_z2 = uq_z2.wrapping_neg();
    }

    //Addition
    let uq_z = uq_z2.wrapping_add(uq_z1);

    //Exception handling
    let q_z = Q8E0::from_bits(uq_z);
    *q = if q_z.is_nar() { Q8E0::ZERO } else { q_z }
}

#[test]
fn test_quire_mul_add() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS8 {
        let p_a: P8E0 = rng.gen();
        let p_b: P8E0 = rng.gen();
        let p_c: P8E0 = rng.gen();
        let f_a = f64::from(p_a);
        let f_b = f64::from(p_b);
        let f_c = f64::from(p_c);
        let mut q = Q8E0::init();
        q += (p_a, p_b);
        q += (p_c, P8E0::ONE);
        let p = q.to_posit();
        let f = f_a.mul_add(f_b, f_c);
        assert_eq!(p, P8E0::from(f));
    }
}

#[test]
fn test_quire_mul_sub() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS8 {
        let p_a: P8E0 = rng.gen();
        let p_b: P8E0 = rng.gen();
        let p_c: P8E0 = rng.gen();
        let f_a = f64::from(p_a);
        let f_b = f64::from(p_b);
        let f_c = f64::from(p_c);
        let mut q = Q8E0::init();
        q -= (p_a, p_b);
        q += (p_c, P8E0::ONE);
        let p = q.to_posit();
        let f = (-f_a).mul_add(f_b, f_c);
        assert_eq!(p, P8E0::from(f));
    }
}

#[cfg(test)]
fn test_ops(fun: fn(P8E0, P8E0, f64, f64) -> (P8E0, f64)) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS8 {
        let p_a: P8E0 = rng.gen();
        let p_b: P8E0 = rng.gen();
        let f_a = f64::from(p_a);
        let f_b = f64::from(p_b);
        let (p, f) = fun(p_a, p_b, f_a, f_b);
        assert_eq!(p, P8E0::from(f));
    }
}

#[test]
fn add() {
    test_ops(|p_a, p_b, f_a, f_b| (p_a + p_b, f_a + f_b));
}

#[test]
fn sub() {
    test_ops(|p_a, p_b, f_a, f_b| (p_a - p_b, f_a - f_b));
}

#[test]
fn mul() {
    test_ops(|p_a, p_b, f_a, f_b| (p_a * p_b, f_a * f_b));
}

#[test]
fn div() {
    test_ops(|p_a, p_b, f_a, f_b| (p_a / p_b, f_a / f_b));
}
