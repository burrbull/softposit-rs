use super::P16E1;
use crate::u32_zero_shr;
use core::ops;

crate::macros::impl_ops!(P16E1);

impl P16E1 {
    #[inline]
    pub const fn neg(self) -> Self {
        Self::new(self.0.wrapping_neg())
    }

    pub(crate) const fn form_ui(reg_len: u32, regime: u16, exp: i8, frac32: u32) -> u16 {
        let mut frac = (frac32 >> 16) as u16;

        let mut bits_more = false;
        let bit_n_plus_one = if reg_len != 14 {
            (0x8000 & frac32) != 0
        } else {
            if frac32 > 0 {
                frac = 0;
                bits_more = true;
            }
            exp != 0
        };

        //sign is always zero
        let mut u_z = Self::pack_to_ui(regime, reg_len, exp as u16, frac);
        //n+1 frac bit is 1. Need to check if another bit is 1 too if not round to even
        if bit_n_plus_one {
            if (0x7FFF & frac32) != 0 {
                bits_more = true;
            }
            u_z += (u_z & 1) | (bits_more as u16);
        }
        u_z
    }

    #[allow(clippy::manual_swap)]
    const fn sub_mags(mut ui_a: u16, mut ui_b: u16) -> Self {
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
            let temp = ui_a;
            ui_a = ui_b;
            ui_b = temp;
            sign = !sign; //A becomes B
        }

        let (mut k_a, mut exp_a, frac_a) = Self::separate_bits(ui_a);
        let mut frac32 = (frac_a as u32) << 16;
        let (k_b, exp_b, frac_b) = Self::separate_bits(ui_b);
        let mut frac32_b = (frac_b as u32) << 16;

        let mut shift_right = (k_a as i16) - (k_b as i16);

        //This is 2kZ + expZ; (where kZ=k_a-k_b and expZ=exp_a-expB)

        shift_right = (shift_right << 1) + (exp_a as i16) - (exp_b as i16);

        if shift_right != 0 {
            if shift_right >= 29 {
                return Self::from_bits(ui_a).with_sign(sign);
            } else {
                frac32_b >>= shift_right;
            }
        }

        frac32 -= frac32_b;

        while (frac32 >> 29) == 0 {
            k_a -= 1;
            frac32 <<= 2;
        }
        let ecarry = (0x4000_0000 & frac32) != 0;
        if !ecarry {
            if exp_a == 0 {
                k_a -= 1;
            }
            exp_a ^= 1;
            frac32 <<= 1;
        }

        let (regime, reg_s, reg_len) = Self::calculate_regime(k_a);

        let u_z = if reg_len > 14 {
            //max or min pos. exp and frac does not matter.
            if reg_s {
                0x7FFF
            } else {
                0x1
            }
        } else {
            //remove hidden bits
            Self::form_ui(
                reg_len,
                regime,
                exp_a,
                (frac32 & 0x3FFF_FFFF) >> (reg_len + 1),
            )
        };
        Self::from_bits(u_z).with_sign(sign)
    }

    #[allow(clippy::manual_swap)]
    const fn add_mags(mut ui_a: u16, mut ui_b: u16) -> Self {
        let sign = Self::sign_ui(ui_a); //sign is always positive.. actually don't have to do this.
        if sign {
            ui_a = ui_a.wrapping_neg();
            ui_b = ui_b.wrapping_neg();
        }

        if (ui_a as i16) < (ui_b as i16) {
            let temp = ui_a;
            ui_a = ui_b;
            ui_b = temp;
        }

        let (mut k_a, mut exp_a, frac_a) = Self::separate_bits(ui_a);
        let mut frac32 = (frac_a as u32) << 16;
        let (k_b, exp_b, frac_b) = Self::separate_bits(ui_b);
        let frac32_b = (frac_b as u32) << 16;

        let mut shift_right = (k_a as i16) - (k_b as i16);

        //This is 2kZ + expZ; (where kZ=k_a-k_b and expZ=exp_a-expB)
        shift_right = (shift_right << 1) + (exp_a as i16) - (exp_b as i16);

        if shift_right == 0 {
            frac32 += frac32_b;
            //rcarry is one
            if exp_a != 0 {
                k_a += 1;
            }
            exp_a ^= 1;
            frac32 >>= 1;
        } else {
            frac32 += u32_zero_shr(frac32_b, shift_right as u32);

            let rcarry = (frac32 & 0x8000_0000) != 0; //first left bit
            if rcarry {
                if exp_a != 0 {
                    k_a += 1;
                }
                exp_a ^= 1;
                frac32 >>= 1;
            }
        }

        let (regime, reg_s, reg_len) = Self::calculate_regime(k_a);

        let u_z = if reg_len > 14 {
            //max or min pos. exp and frac does not matter.
            if reg_s {
                0x7FFF
            } else {
                0x1
            }
        } else {
            //remove hidden bits
            Self::form_ui(
                reg_len,
                regime,
                exp_a,
                (frac32 & 0x3FFF_FFFF) >> (reg_len + 1),
            )
        };
        Self::from_bits(u_z).with_sign(sign)
    }

    pub const fn add(self, other: Self) -> Self {
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

    pub const fn sub(self, other: Self) -> Self {
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

    pub const fn mul(self, other: Self) -> Self {
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

        let (mut k_a, mut exp_a, frac_a) = P16E1::separate_bits(ui_a);
        let (k_b, exp_b, frac_b) = P16E1::separate_bits(ui_b);
        k_a += k_b;
        exp_a += exp_b;
        let mut frac32_z = (frac_a as u32) * (frac_b as u32);

        if exp_a > 1 {
            k_a += 1;
            exp_a ^= 0x2;
        }

        let rcarry = (frac32_z >> 29) != 0; //3rd bit of frac32_z
        if rcarry {
            if exp_a != 0 {
                k_a += 1;
            }
            exp_a ^= 1;
            frac32_z >>= 1;
        }

        let (regime, reg_s, reg_len) = Self::calculate_regime(k_a);

        let u_z = if reg_len > 14 {
            //max or min pos. exp and frac does not matter.
            if reg_s {
                0x7FFF
            } else {
                0x1
            }
        } else {
            //remove carry and rcarry bits and shift to correct position
            Self::form_ui(
                reg_len,
                regime,
                exp_a,
                (frac32_z & 0x_0FFF_FFFF) >> (reg_len - 1),
            )
        };

        Self::from_bits(u_z).with_sign(sign_z)
    }

    pub const fn div(self, other: Self) -> Self {
        let mut ui_a = self.to_bits();
        let mut ui_b = other.to_bits();

        //NaR or Zero
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

        let (mut k_a, mut exp_a, frac_a) = Self::separate_bits(ui_a);
        let frac32_a = (frac_a as u32) << 14;
        let (k_b, exp_b, frac_b) = Self::separate_bits(ui_b);
        k_a -= k_b;
        exp_a -= exp_b;

        let (quot, rem) = crate::div(frac32_a as i32, frac_b as i32);
        let mut frac32 = quot as u32;

        if exp_a < 0 {
            exp_a = 1;
            k_a -= 1;
        }
        if frac32 != 0 {
            let rcarry = (frac32 >> 14) != 0; // this is the hidden bit (14th bit) , extreme right bit is bit 0
            if !rcarry {
                if exp_a == 0 {
                    k_a -= 1;
                }
                exp_a ^= 1;
                frac32 <<= 1;
            }
        }

        let (regime, reg_s, reg_len) = Self::calculate_regime(k_a);

        let u_z = if reg_len > 14 {
            //max or min pos. exp and frac does not matter.
            if reg_s {
                0x7FFF
            } else {
                0x1
            }
        } else {
            //remove carry and rcarry bits and shift to correct position
            frac32 &= 0x3FFF;

            let (bit_n_plus_one, frac) = if reg_len != 14 {
                (
                    ((frac32 >> reg_len) & 0x1) != 0,
                    (frac32 >> (reg_len + 1)) as u16,
                )
            } else {
                (exp_a != 0, 0)
            };

            //sign is always zero
            let mut u_z = Self::pack_to_ui(regime, reg_len, exp_a as u16, frac);

            if bit_n_plus_one {
                let bits_more = if rem != 0 {
                    true
                } else {
                    (((1 << reg_len) - 1) & frac32) != 0
                };
                //n+1 frac bit is 1. Need to check if another bit is 1 too if not round to even
                u_z += (u_z & 1) | (bits_more as u16);
            }
            u_z
        };

        Self::from_bits(u_z).with_sign(sign_z)
    }

    #[inline]
    pub const fn rem(self, other: Self) -> Self {
        self.sub((self.div(other)).trunc().mul(other))
    }
}

#[test]
fn add() {
    super::test21_exact(|p_a, p_b, f_a, f_b| (p_a + p_b, f_a + f_b));
}

#[test]
fn sub() {
    super::test21_exact(|p_a, p_b, f_a, f_b| (p_a - p_b, f_a - f_b));
}

#[test]
fn mul() {
    super::test21_exact(|p_a, p_b, f_a, f_b| (p_a * p_b, f_a * f_b));
}

#[test]
fn div() {
    super::test21_exact(|p_a, p_b, f_a, f_b| (p_a / p_b, f_a / f_b));
}
