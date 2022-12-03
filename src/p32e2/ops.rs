use super::P32E2;
use crate::WithSign;
use core::{mem, ops};

impl ops::Neg for P32E2 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self::new(self.0.wrapping_neg())
    }
}

impl ops::AddAssign for P32E2 {
    #[inline]
    fn add_assign(&mut self, other: Self) {
        *self = *self + other
    }
}

impl ops::SubAssign for P32E2 {
    #[inline]
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other
    }
}

impl ops::MulAssign for P32E2 {
    #[inline]
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other
    }
}

impl ops::DivAssign for P32E2 {
    #[inline]
    fn div_assign(&mut self, other: Self) {
        *self = *self / other
    }
}

impl ops::RemAssign for P32E2 {
    #[inline]
    fn rem_assign(&mut self, other: Self) {
        *self = *self % other
    }
}

impl ops::Add for P32E2 {
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

impl ops::Sub for P32E2 {
    type Output = Self;
    #[inline]
    fn sub(self, other: Self) -> Self {
        let ui_a = self.to_bits();
        let ui_b = other.to_bits();

        //infinity
        if self.is_nar() || other.is_nar() {
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

impl ops::Div for P32E2 {
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
            ui_a = ui_a.wrapping_neg()
        };
        if sign_b {
            ui_b = ui_b.wrapping_neg()
        };

        let (mut k_a, mut exp_a, mut frac_a) = Self::separate_bits(ui_a);

        let frac64_a = (frac_a as u64) << 30;

        let (k_b, exp_b, frac_b) = Self::separate_bits(ui_b);
        k_a -= k_b;
        exp_a -= exp_b;

        let (quot, rem) = crate::lldiv(frac64_a as i64, frac_b as i64);
        let mut frac64_z = quot as u64;

        if exp_a < 0 {
            exp_a += 4;
            k_a -= 1;
        }
        if frac64_z != 0 {
            let rcarry = (frac64_z >> 30) != 0; // this is the hidden bit (14th bit) , extreme right bit is bit 0
            if !rcarry {
                if exp_a == 0 {
                    k_a -= 1;
                    exp_a = 3;
                } else {
                    exp_a -= 1;
                }
                frac64_z <<= 1;
            }
        }

        let (regime, reg_s, reg_len) = Self::calculate_regime(k_a);

        let u_z = if reg_len > 30 {
            //max or min pos. exp and frac does not matter.
            if reg_s {
                0x7FFF_FFFF
            } else {
                0x1
            }
        } else {
            //remove carry and rcarry bits and shift to correct position
            frac64_z &= 0x3FFF_FFFF;

            frac_a = (frac64_z >> (reg_len + 2)) as u32;

            let mut bit_n_plus_one = false;
            let mut bits_more = false;
            if reg_len <= 28 {
                bit_n_plus_one = ((frac64_z >> (reg_len + 1)) & 0x1) != 0;
                exp_a <<= 28 - reg_len;
                if bit_n_plus_one {
                    bits_more = (((1 << (reg_len + 1)) - 1) & frac64_z) != 0;
                }
            } else {
                if reg_len == 30 {
                    bit_n_plus_one = (exp_a & 0x2) != 0;
                    bits_more = (exp_a & 0x1) != 0;
                    exp_a = 0;
                } else if reg_len == 29 {
                    bit_n_plus_one = (exp_a & 0x1) != 0;
                    exp_a >>= 1; //taken care of by the pack algo
                }
                if frac64_z > 0 {
                    frac_a = 0;
                    bits_more = true;
                }
            }

            let mut u_z = Self::pack_to_ui(regime, exp_a as u32, frac_a);
            if bit_n_plus_one {
                if rem != 0 {
                    bits_more = true;
                }
                u_z += (u_z & 1) | (bits_more as u32);
            }
            u_z
        };

        Self::from_bits(u_z.with_sign(sign_z))
    }
}

impl P32E2 {
    #[inline]
    fn form_ui(reg_len: u32, regime: u32, mut exp: i32, frac64: u64) -> u32 {
        let mut bit_n_plus_one = false;
        let mut bits_more = false;
        let mut frac = (frac64 >> 32) as u32;
        if reg_len <= 28 {
            bit_n_plus_one = (0x80000000 & frac64) != 0;
            exp <<= 28 - reg_len;
        } else {
            if reg_len == 30 {
                bit_n_plus_one = exp & 0x2 != 0;
                bits_more = exp & 0x1 != 0;
                exp = 0;
            } else if reg_len == 29 {
                bit_n_plus_one = exp & 0x1 != 0;
                exp >>= 1; //taken care of by the pack algo
            }
            if frac > 0 {
                frac = 0;
                bits_more = true;
            }
        }
        //sign is always zero
        let mut u_z = Self::pack_to_ui(regime, exp as u32, frac);
        //n+1 frac bit is 1. Need to check if another bit is 1 too if not round to even
        if bit_n_plus_one {
            if 0x7FFF_FFFF & frac64 != 0 {
                bits_more = true;
            }
            u_z += (u_z & 1) | (bits_more as u32);
        }
        u_z
    }
}

impl ops::Mul for P32E2 {
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
            ui_a = ui_a.wrapping_neg()
        };
        if sign_b {
            ui_b = ui_b.wrapping_neg()
        };

        let (mut k_a, mut exp_a, frac_a) = Self::separate_bits(ui_a);

        let (k_b, exp_b, frac_b) = Self::separate_bits(ui_b);
        k_a += k_b;
        exp_a += exp_b;
        let mut frac64 = (frac_a as u64) * (frac_b as u64);

        if exp_a > 3 {
            k_a += 1;
            exp_a &= 0x3; // -=4
        }

        let rcarry = (frac64 >> 61) != 0; //3rd bit of frac64
        if rcarry {
            exp_a += 1;
            if exp_a > 3 {
                k_a += 1;
                exp_a &= 0x3;
            }
            frac64 >>= 1;
        }
        let (regime, reg_s, reg_len) = Self::calculate_regime(k_a);

        let u_z = if reg_len > 30 {
            //max or min pos. exp and frac does not matter.
            if reg_s {
                0x7FFF_FFFF
            } else {
                0x1
            }
        } else {
            //remove carry and rcarry bits and shift to correct position (2 bits exp, so + 1 than 16 bits)
            Self::form_ui(
                reg_len,
                regime,
                exp_a,
                (frac64 & 0x_0FFF_FFFF_FFFF_FFFF) >> reg_len,
            )
        };

        Self::from_bits(u_z.with_sign(sign_z))
    }
}

impl P32E2 {
    #[inline]
    fn add_mags(mut ui_a: u32, mut ui_b: u32) -> Self {
        let sign = Self::sign_ui(ui_a);
        if sign {
            ui_a = ui_a.wrapping_neg();
            ui_b = ui_b.wrapping_neg();
        }

        if (ui_a as i32) < (ui_b as i32) {
            mem::swap(&mut ui_a, &mut ui_b);
        }

        let (mut k_a, mut exp_a, frac_a) = Self::separate_bits(ui_a);

        let mut frac64 = (frac_a as u64) << 32;

        let (k_b, exp_b, frac_b) = Self::separate_bits(ui_b);

        let mut shift_right = (k_a as i16) - (k_b as i16);

        //This is 4kZ + expZ; (where kZ=k_a-kB and expZ=exp_a-expB)
        shift_right = (shift_right << 2) + (exp_a as i16) - (exp_b as i16);

        frac64 += ((frac_b as u64) << 32)
            .checked_shr(shift_right as u32)
            .unwrap_or(0);

        let rcarry = (0x8000_0000_0000_0000 & frac64) != 0; //first left bit
        if rcarry {
            exp_a += 1;
            if exp_a > 3 {
                k_a += 1;
                exp_a &= 0x3;
            }
            frac64 >>= 1;
        }
        let (regime, reg_s, reg_len) = Self::calculate_regime(k_a);

        let u_z = if reg_len > 30 {
            //max or min pos. exp and frac does not matter.
            if reg_s {
                0x7FFF_FFFF
            } else {
                0x1
            }
        } else {
            //remove hidden bits
            Self::form_ui(
                reg_len,
                regime,
                exp_a,
                (frac64 & 0x3FFF_FFFF_FFFF_FFFF) >> (reg_len + 2),
            )
        };

        Self::from_bits(u_z.with_sign(sign))
    }

    #[inline]
    fn sub_mags(mut ui_a: u32, mut ui_b: u32) -> Self {
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
        if (ui_a as i32) < (ui_b as i32) {
            mem::swap(&mut ui_a, &mut ui_b);
            sign = !sign; //A becomes B
        }

        let (mut k_a, mut exp_a, frac_a) = Self::separate_bits(ui_a);

        let mut frac64 = (frac_a as u64) << 32;

        let (k_b, exp_b, frac_b) = Self::separate_bits(ui_b);

        let mut shift_right = (k_a as i16) - (k_b as i16);
        let mut frac64_b = (frac_b as u64) << 32;
        //This is 4kZ + expZ; (where kZ=k_a-kB and expZ=exp_a-expB)
        shift_right = (shift_right << 2) + (exp_a as i16) - (exp_b as i16);

        if shift_right > 63 {
            return Self::from_bits(ui_a.with_sign(sign));
        } else {
            frac64_b >>= shift_right;
        }

        frac64 -= frac64_b;

        while (frac64 >> 59) == 0 {
            k_a -= 1;
            frac64 <<= 4;
        }
        let mut ecarry = (0x4000_0000_0000_0000 & frac64) != 0; //(0x4000_0000_0000_0000 & frac64)>>62;
        while !ecarry {
            if exp_a == 0 {
                k_a -= 1;
                exp_a = 3;
            } else {
                exp_a -= 1;
            }
            frac64 <<= 1;
            ecarry = (0x4000_0000_0000_0000 & frac64) != 0;
        }

        let (regime, reg_s, reg_len) = Self::calculate_regime(k_a);

        let u_z = if reg_len > 30 {
            //max or min pos. exp and frac does not matter.
            if reg_s {
                0x7FFF_FFFF
            } else {
                0x1
            }
        } else {
            //remove hidden bits
            Self::form_ui(
                reg_len,
                regime,
                exp_a,
                (frac64 & 0x3FFF_FFFF_FFFF_FFFF) >> (reg_len + 2),
            )
        };

        Self::from_bits(u_z.with_sign(sign))
    }
}

impl ops::Rem for P32E2 {
    type Output = Self;
    fn rem(self, other: Self) -> Self {
        self - (self / other).trunc() * other
    }
}

#[cfg(test)]
fn test_ops(fun: fn(P32E2, P32E2, f64, f64) -> (P32E2, f64)) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS32 {
        let p_a: P32E2 = rng.gen();
        let p_b: P32E2 = rng.gen();
        let f_a = f64::from(p_a);
        let f_b = f64::from(p_b);
        let (p, f) = fun(p_a, p_b, f_a, f_b);
        assert_eq!(p, P32E2::from(f));
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
