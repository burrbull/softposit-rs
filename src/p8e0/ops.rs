use super::P8E0;
use crate::u8_with_sign;
use core::ops;

crate::macros::impl_ops!(P8E0);

impl P8E0 {
    #[inline]
    pub const fn neg(self) -> Self {
        Self::new(self.0.wrapping_neg())
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

    pub const fn div(self, other: Self) -> Self {
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
        let mut frac16 = quot as u16;

        if frac16 != 0 {
            let rcarry = (frac16 >> 7) != 0; // this is the hidden bit (7th bit) , extreme right bit is bit 0
            if !rcarry {
                k_a -= 1;
                frac16 <<= 1;
            }
        }

        let (regime, reg_sa, reg_len) = Self::calculate_regime(k_a);

        let u_z = if reg_len > 6 {
            //max or min pos. exp and frac does not matter.
            if reg_sa {
                0x7F
            } else {
                0x1
            }
        } else {
            //remove carry and rcarry bits and shift to correct position
            frac16 &= 0x7F;
            let frac_a = (frac16 >> (reg_len + 1)) as u8;

            let bit_n_plus_one = (0x1 & (frac16 >> reg_len)) != 0;
            let mut u_z = Self::pack_to_ui(regime, frac_a);

            if bit_n_plus_one {
                let bits_more = if rem != 0 {
                    true
                } else {
                    (((1 << reg_len) - 1) & frac16) != 0
                };
                //n+1 frac bit is 1. Need to check if another bit is 1 too if not round to even
                u_z += (u_z & 1) | (bits_more as u8);
            }
            u_z
        };

        Self::from_bits(u8_with_sign(u_z, sign_z))
    }

    #[inline]
    const fn calc_ui(k: i8, mut frac16: u16) -> u8 {
        let (regime, reg_s, reg_len) = Self::calculate_regime(k);

        if reg_len > 6 {
            //max or min pos. exp and frac does not matter.
            if reg_s {
                0x7F
            } else {
                0x1
            }
        } else {
            frac16 = (frac16 & 0x3FFF) >> reg_len;
            let frac = (frac16 >> 8) as u8;
            let bit_n_plus_one = (frac16 & 0x80) != 0;
            let mut u_z = Self::pack_to_ui(regime, frac);

            //n+1 frac bit is 1. Need to check if another bit is 1 too if not round to even
            if bit_n_plus_one {
                let bits_more = (frac16 & 0x7F) != 0;
                u_z += (u_z & 1) | (bits_more as u8);
            }
            u_z
        }
    }

    #[inline]
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

        let (mut k_a, frac_a) = Self::separate_bits(ui_a);
        let (k_b, frac_b) = Self::separate_bits(ui_b);
        k_a += k_b;

        let mut frac16 = (frac_a as u16) * (frac_b as u16);

        let rcarry = (frac16 & 0x_8000) != 0; //1st bit of frac32Z
        if rcarry {
            k_a += 1;
            frac16 >>= 1;
        }

        let u_z = Self::calc_ui(k_a, frac16);
        Self::from_bits(u8_with_sign(u_z, sign_z))
    }

    #[allow(clippy::manual_swap)]
    const fn add_mags(mut ui_a: u8, mut ui_b: u8) -> Self {
        let sign = Self::sign_ui(ui_a);
        if sign {
            ui_a = ui_a.wrapping_neg();
            ui_b = ui_b.wrapping_neg();
        }

        if (ui_a as i8) < (ui_b as i8) {
            let temp = ui_a;
            ui_a = ui_b;
            ui_b = temp;
        }

        let (mut k_a, frac_a) = Self::separate_bits(ui_a);
        let mut frac16_a = (frac_a as u16) << 7;

        let (k_b, frac_b) = Self::separate_bits(ui_b);
        let shift_right = (k_a as i16) - (k_b as i16);

        frac16_a += if let Some(val) = (frac_b as u16).checked_shl((7 - shift_right) as u32) {
            val
        } else {
            0
        };

        let rcarry = (0x8000 & frac16_a) != 0; //first left bit
        if rcarry {
            k_a += 1;
            frac16_a >>= 1;
        }

        let u_z = Self::calc_ui(k_a, frac16_a);
        Self::from_bits(u8_with_sign(u_z, sign))
    }

    #[allow(clippy::manual_swap)]
    const fn sub_mags(mut ui_a: u8, mut ui_b: u8) -> Self {
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

        let (mut k_a, frac_a) = Self::separate_bits(ui_a);
        let mut frac16_a = (frac_a as u16) << 7;

        let (k_b, frac_b) = Self::separate_bits(ui_b);
        let shift_right = (k_a as i16) - (k_b as i16);

        let mut frac16_b = (frac_b as u16) << 7;

        if shift_right >= 14 {
            return Self::from_bits(u8_with_sign(ui_a, sign));
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

        let u_z = Self::calc_ui(k_a, frac16_a);
        Self::from_bits(u8_with_sign(u_z, sign))
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
