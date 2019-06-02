use super::{P16E1, Q16E1};
use crate::WithSign;
use core::ops;

impl ops::Neg for P16E1 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self::new(self.0.wrapping_neg())
    }
}

impl ops::AddAssign for P16E1 {
    #[inline]
    fn add_assign(&mut self, other: Self) {
        *self = *self + other
    }
}

impl ops::SubAssign for P16E1 {
    #[inline]
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other
    }
}

impl ops::MulAssign for P16E1 {
    #[inline]
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other
    }
}

impl ops::DivAssign for P16E1 {
    #[inline]
    fn div_assign(&mut self, other: Self) {
        *self = *self / other
    }
}

impl ops::RemAssign for P16E1 {
    #[inline]
    fn rem_assign(&mut self, other: Self) {
        *self = *self % other
    }
}

#[inline]
pub fn sub_mags_p16(mut ui_a: u16, mut ui_b: u16) -> P16E1 {
    //Both ui_a and ui_b are actually the same signs if ui_b inherits sign of sub
    //Make both positive
    let mut sign = P16E1::sign_ui(ui_a);
    if sign {
        ui_a = ui_a.wrapping_neg();
    } else {
        ui_b = ui_b.wrapping_neg();
    }

    if ui_a == ui_b {
        //essential, if not need special handling
        return P16E1::ZERO;
    }
    if ui_a < ui_b {
        ui_a ^= ui_b;
        ui_b ^= ui_a;
        ui_a ^= ui_b;
        sign = !sign; //A becomes B
    }

    let (mut k_a, mut exp_a, frac_a) = P16E1::separate_bits(ui_a);
    let mut frac32_a = (frac_a as u32) << 16;
    let (k_b, exp_b, frac_b) = P16E1::separate_bits(ui_b);
    let mut frac32_b = (frac_b as u32) << 16;

    let mut shift_right = (k_a as i16) - (k_b as i16);

    //This is 2kZ + expZ; (where kZ=k_a-k_b and expZ=exp_a-expB)

    shift_right = (shift_right << 1) + (exp_a as i16) - (exp_b as i16);

    if shift_right != 0 {
        if shift_right >= 29 {
            let mut u_z: u16 = ui_a;
            if sign {
                u_z = (-(u_z as i16)) as u16/* & 0xFFFF*/;
            }
            return P16E1::from_bits(u_z);
        } else {
            frac32_b >>= shift_right;
        }
    }

    frac32_a -= frac32_b;

    while (frac32_a >> 29) == 0 {
        k_a -= 1;
        frac32_a <<= 2;
    }
    let ecarry = (0x4000_0000 & frac32_a) >> 30 != 0;
    if !ecarry {
        if exp_a == 0 {
            k_a -= 1;
        }
        exp_a ^= 1;
        frac32_a <<= 1;
    }

    let (regime, reg_sa, reg_a) = P16E1::calculate_regime(k_a);

    let u_z = if reg_a > 14 {
        //max or min pos. exp and frac does not matter.
        if reg_sa {
            0x7FFF
        } else {
            0x1
        }
    } else {
        //remove hidden bits
        frac32_a = (frac32_a & 0x3FFF_FFFF) >> (reg_a + 1);
        let mut frac_a = (frac32_a >> 16) as u16;
        let mut bit_n_plus_one = false;
        if reg_a != 14 {
            bit_n_plus_one = ((frac32_a >> 15) & 0x1) != 0;
        } else if frac32_a > 0 {
            frac_a = 0;
        }
        if (reg_a == 14) && (exp_a != 0) {
            bit_n_plus_one = true;
        }
        let mut u_z = P16E1::pack_to_ui(regime, reg_a, exp_a as u16, frac_a);
        if bit_n_plus_one {
            let bits_more = (frac32_a & 0x7FFF) != 0;
            //n+1 frac bit is 1. Need to check if another bit is 1 too if not round to even
            u_z += (u_z & 1) | (bits_more as u16);
        }
        u_z
    };
    P16E1::from_bits(u_z.with_sign(sign))
}

#[inline]
pub fn add_mags_p16(mut ui_a: u16, mut ui_b: u16) -> P16E1 {
    let sign = P16E1::sign_ui(ui_a); //sign is always positive.. actually don't have to do this.
    if sign {
        ui_a = ui_a.wrapping_neg();
        ui_b = ui_b.wrapping_neg();
    }

    if (ui_a as i16) < (ui_b as i16) {
        let ui_x = ui_a;
        let ui_y = ui_b;
        ui_a = ui_y;
        ui_b = ui_x;
    }

    let (mut k_a, mut exp_a, frac_a) = P16E1::separate_bits(ui_a);
    let mut frac32_a = (frac_a as u32) << 16;
    let (k_b, exp_b, frac_b) = P16E1::separate_bits(ui_b);
    let mut frac32_b = (frac_b as u32) << 16;

    let mut shift_right = (k_a as i16) - (k_b as i16);

    //This is 2kZ + expZ; (where kZ=k_a-k_b and expZ=exp_a-expB)
    shift_right = (shift_right << 1) + (exp_a as i16) - (exp_b as i16);

    if shift_right == 0 {
        frac32_a += frac32_b;
        //rcarry is one
        if exp_a != 0 {
            k_a += 1;
        }
        exp_a ^= 1;
        frac32_a >>= 1;
    } else {
        //Manage CLANG (LLVM) compiler when shifting right more than number of bits
        if shift_right > 31 {
            frac32_b = 0;
        } else {
            //frac32_b >>= shift_right
            frac32_b >>= shift_right;
        }

        frac32_a += frac32_b;
        let rcarry = (frac32_a & 0x8000_0000) != 0; //first left bit
        if rcarry {
            if exp_a != 0 {
                k_a += 1;
            }
            exp_a ^= 1;
            frac32_a >>= 1;
        }
    }

    let (regime, reg_sa, reg_a) = P16E1::calculate_regime(k_a);

    let u_z = if reg_a > 14 {
        //max or min pos. exp and frac does not matter.
        if reg_sa {
            0x7FFF
        } else {
            0x1
        }
    } else {
        //remove hidden bits
        frac32_a = (frac32_a & 0x3FFF_FFFF) >> (reg_a + 1);
        let mut frac_a = (frac32_a >> 16) as u16;
        let mut bit_n_plus_one = false;
        if reg_a != 14 {
            bit_n_plus_one = ((frac32_a >> 15) & 0x1) != 0;
        } else if frac32_a > 0 {
            frac_a = 0;
        }
        if (reg_a == 14) && (exp_a != 0) {
            bit_n_plus_one = true;
        }
        let mut u_z = P16E1::pack_to_ui(regime, reg_a, exp_a as u16, frac_a);
        if bit_n_plus_one {
            let bits_more = (frac32_a & 0x7FFF) != 0;
            //n+1 frac bit is 1. Need to check if another bit is 1 too if not round to even
            u_z += (u_z & 1) | (bits_more as u16);
        }
        u_z
    };
    P16E1::from_bits(u_z.with_sign(sign))
}

impl ops::Add for P16E1 {
    type Output = Self;
    #[inline]
    fn add(self, other: Self) -> Self {
        let ui_a = self.to_bits();
        let ui_b = other.to_bits();

        //Zero or infinity
        if (ui_a == 0) || (ui_b == 0) {
            // Not required but put here for speed
            Self::from_bits(ui_a | ui_b)
        } else if (ui_a == 0x8000) || (ui_b == 0x8000) {
            Self::NAR
        } else {
            //different signs
            if Self::sign_ui(ui_a ^ ui_b) {
                sub_mags_p16(ui_a, ui_b)
            } else {
                add_mags_p16(ui_a, ui_b)
            }
        }
    }
}

impl ops::Sub for P16E1 {
    type Output = Self;
    #[inline]
    fn sub(self, other: Self) -> Self {
        let ui_a = self.to_bits();
        let ui_b = other.to_bits();

        //infinity
        if (ui_a == 0x8000) || (ui_b == 0x8000) {
            Self::NAR
        } else if (ui_a == 0) || (ui_b == 0) {
            //Zero
            Self::from_bits(ui_a | ui_b.wrapping_neg())
        } else {
            //different signs
            if Self::sign_ui(ui_a ^ ui_b) {
                add_mags_p16(ui_a, ui_b.wrapping_neg())
            } else {
                sub_mags_p16(ui_a, ui_b.wrapping_neg())
            }
        }
    }
}

impl ops::Mul for P16E1 {
    type Output = Self;
    #[inline]
    fn mul(self, other: Self) -> Self {
        let mut ui_a = self.to_bits();
        let mut ui_b = other.to_bits();

        //NaR or Zero
        if (ui_a == 0x8000) || (ui_b == 0x8000) {
            return Self::NAR;
        } else if (ui_a == 0) || (ui_b == 0) {
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

        let (mut k_a, mut exp_a, mut frac_a) = P16E1::separate_bits(ui_a);
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

        let (regime, reg_sa, reg_a) = Self::calculate_regime(k_a);

        let u_z = if reg_a > 14 {
            //max or min pos. exp and frac does not matter.
            if reg_sa {
                0x7FFF
            } else {
                0x1
            }
        } else {
            //remove carry and rcarry bits and shift to correct position
            frac32_z = (frac32_z & 0xFFF_FFFF) >> (reg_a - 1);
            frac_a = (frac32_z >> 16) as u16;

            let mut bit_n_plus_one = false;
            if reg_a != 14 {
                bit_n_plus_one = (0x8000 & frac32_z) != 0;
            } else if frac_a > 0 {
                frac_a = 0;
            }
            if (reg_a == 14) && (exp_a != 0) {
                bit_n_plus_one = true;
            }

            //sign is always zero
            let mut u_z = Self::pack_to_ui(regime, reg_a, exp_a as u16, frac_a);
            //n+1 frac bit is 1. Need to check if another bit is 1 too if not round to even
            if bit_n_plus_one {
                let bits_more = (0x7FFF & frac32_z) != 0;
                u_z += (u_z & 1) | (bits_more as u16);
            }
            u_z
        };

        Self::from_bits(u_z.with_sign(sign_z))
    }
}

impl ops::Div for P16E1 {
    type Output = Self;
    #[inline]
    fn div(self, other: Self) -> Self {
        let mut ui_a = self.to_bits();
        let mut ui_b = other.to_bits();

        //NaR or Zero
        if (ui_a == 0x8000) || (ui_b == 0x8000) || (ui_b == 0) {
            return Self::NAR;
        } else if ui_a == 0 {
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

        let (mut k_a, mut exp_a, mut frac_a) = Self::separate_bits(ui_a);
        let frac32_a = (frac_a as u32) << 14;
        let (k_b, exp_b, frac_b) = Self::separate_bits(ui_b);
        k_a -= k_b;
        exp_a -= exp_b;

        let (quot, rem) = crate::div(frac32_a as i32, frac_b as i32);
        let mut frac32_z = quot as u32;

        if exp_a < 0 {
            exp_a = 1;
            k_a -= 1;
        }
        if frac32_z != 0 {
            let rcarry = (frac32_z >> 14) != 0; // this is the hidden bit (14th bit) , extreme right bit is bit 0
            if !rcarry {
                if exp_a == 0 {
                    k_a -= 1;
                }
                exp_a ^= 1;
                frac32_z <<= 1;
            }
        }

        let (regime, reg_sa, reg_a) = Self::calculate_regime(k_a);

        let u_z = if reg_a > 14 {
            //max or min pos. exp and frac does not matter.
            if reg_sa {
                0x7FFF
            } else {
                0x1
            }
        } else {
            //remove carry and rcarry bits and shift to correct position
            frac32_z &= 0x3FFF;
            frac_a = (frac32_z >> (reg_a + 1)) as u16;

            let mut bit_n_plus_one = false;
            if reg_a != 14 {
                bit_n_plus_one = ((frac32_z >> reg_a) & 0x1) != 0;
            } else if frac_a > 0 {
                frac_a = 0;
            }
            if (reg_a == 14) && (exp_a != 0) {
                bit_n_plus_one = true;
            }

            //sign is always zero
            let mut u_z = Self::pack_to_ui(regime, reg_a, exp_a as u16, frac_a);

            if bit_n_plus_one {
                let bits_more = if rem != 0 {
                    true
                } else {
                    (((1 << reg_a) - 1) & frac32_z) != 0
                };
                //n+1 frac bit is 1. Need to check if another bit is 1 too if not round to even
                u_z += (u_z & 1) | (bits_more as u16);
            }
            u_z
        };

        Self::from_bits(u_z.with_sign(sign_z))
    }
}

impl ops::Rem for P16E1 {
    type Output = Self;
    fn rem(self, _other: Self) -> Self {
        unimplemented!()
    }
}

impl ops::AddAssign<(P16E1, P16E1)> for Q16E1 {
    #[inline]
    fn add_assign(&mut self, rhs: (P16E1, P16E1)) {
        q16_fdp_add(self, rhs.0, rhs.1);
    }
}

impl ops::SubAssign<(P16E1, P16E1)> for Q16E1 {
    #[inline]
    fn sub_assign(&mut self, rhs: (P16E1, P16E1)) {
        q16_fdp_sub(self, rhs.0, rhs.1);
    }
}

pub(super) fn q16_fdp_add(q: &mut Q16E1, p_a: P16E1, p_b: P16E1) {
    let u_z1 = q.to_bits();

    let mut ui_a = p_a.to_bits();
    let mut ui_b = p_b.to_bits();

    if q.is_nar() || p_a.is_nar() || p_b.is_nar() {
        *q = Q16E1::NAR;
        return;
    } else if (ui_a == 0) || (ui_b == 0) {
        return;
    }

    //max pos (sign plus and minus)
    let sign_a = P16E1::sign_ui(ui_a);
    let sign_b = P16E1::sign_ui(ui_b);
    let sign_z2 = sign_a ^ sign_b;

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

    let rcarry = (frac32_z >> 29) != 0; //3rd bit (position 2) of frac32_z, hidden bit is 4th bit (position 3)
    if rcarry {
        if exp_a != 0 {
            k_a += 1;
        }
        exp_a ^= 1;
        frac32_z >>= 1;
    }

    //default dot is between bit 71 and 72, extreme left bit is bit 0. Last right bit is bit 127.
    //Scale = 2^es * k + e  => 2k + e
    let first_pos = 71 - ((k_a as i16) << 1) - (exp_a as i16);

    //No worries about hidden bit moving before position 4 because fraction is right aligned so
    //there are 16 spare bits
    let mut u_z2: [u64; 2] = [0, 0];
    if first_pos > 63 {
        //This means entire fraction is in right 64 bits
        u_z2[0] = 0;
        let shift_right = first_pos - 99; //99 = 63+ 4+ 32
        if shift_right < 0 {
            //shiftLeft
            u_z2[1] = (frac32_z as u64) << -shift_right;
        } else {
            u_z2[1] = (frac32_z as u64) >> shift_right;
        }
    } else {
        //frac32_z can be in both left64 and right64
        let shift_right = first_pos - 35; // -35= -3-32
        if shift_right < 0 {
            u_z2[0] = (frac32_z as u64) << -shift_right;
        } else {
            u_z2[0] = (frac32_z as u64) >> shift_right;
            u_z2[1] = (frac32_z as u64)
                .checked_shl((64 - shift_right) as u32)
                .unwrap_or(0);
        }
    }

    if sign_z2 {
        if u_z2[1] > 0 {
            u_z2[1] = u_z2[1].wrapping_neg();
            u_z2[0] = !u_z2[0];
        } else {
            u_z2[0] = u_z2[0].wrapping_neg();
        }
    }

    //Addition
    let b1 = u_z1[0] & 0x1 != 0;
    let b2 = u_z2[0] & 0x1 != 0;
    let rcarryb = b1 & b2;
    let mut u_z: [u64; 2] = [0, (u_z1[1] >> 1) + (u_z2[1] >> 1) + (rcarryb as u64)];

    let rcarry_z = (u_z[1] & 0x_8000_0000_0000_0000) != 0;

    u_z[1] = u_z[1] << 1 | ((b1 ^ b2) as u64);

    let b1 = u_z1[0] & 0x1 != 0;
    let b2 = u_z2[0] & 0x1 != 0;
    //rcarryb = b1 & b2 ;
    let rcarryb3 = (b1 as i8) + (b2 as i8) + (rcarry_z as i8);

    u_z[0] = (u_z1[0] >> 1) + (u_z2[0] >> 1) + (((rcarryb3 >> 1) & 0x1) as u64);
    //rcarrySignZ = u_z[0]>>63;

    u_z[0] = u_z[0] << 1 | ((rcarryb3 & 0x1) as u64);

    //Exception handling for NaR
    let q_z = Q16E1::from_bits(u_z);
    *q = if q_z.is_nar() { Q16E1::ZERO } else { q_z }
}

pub(super) fn q16_fdp_sub(q: &mut Q16E1, p_a: P16E1, p_b: P16E1) {
    let u_z1 = q.to_bits();

    let mut ui_a = p_a.to_bits();
    let mut ui_b = p_b.to_bits();

    if q.is_nar() || p_a.is_nar() || p_b.is_nar() {
        *q = Q16E1::NAR;
        return;
    } else if (ui_a == 0) || (ui_b == 0) {
        return;
    }

    //max pos (sign plus and minus)
    let sign_a = P16E1::sign_ui(ui_a);
    let sign_b = P16E1::sign_ui(ui_b);
    let sign_z2 = sign_a ^ sign_b;

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

    let rcarry = (frac32_z >> 29) != 0; //3rd bit (position 2) of frac32_z, hidden bit is 4th bit (position 3)
    if rcarry {
        if exp_a != 0 {
            k_a += 1;
        }
        exp_a ^= 1;
        frac32_z >>= 1;
    }

    //default dot is between bit 71 and 72, extreme left bit is bit 0. Last right bit is bit 127.
    //Scale = 2^es * k + e  => 2k + e
    let first_pos = 71 - ((k_a as i16) << 1) - (exp_a as i16);

    //No worries about hidden bit moving before position 4 because fraction is right aligned so
    //there are 16 spare bits
    let mut u_z2: [u64; 2] = [0, 0];
    if first_pos > 63 {
        //This means entire fraction is in right 64 bits
        u_z2[0] = 0;
        let shift_right = first_pos - 99; //99 = 63+ 4+ 32
        if shift_right < 0 {
            //shiftLeft
            u_z2[1] = (frac32_z as u64) << -shift_right;
        } else {
            u_z2[1] = (frac32_z as u64) >> shift_right;
        }
    } else {
        //frac32_z can be in both left64 and right64
        let shift_right = first_pos - 35; // -35= -3-32
        if shift_right < 0 {
            u_z2[0] = (frac32_z as u64) << -shift_right;
        } else {
            u_z2[0] = (frac32_z as u64) >> shift_right;
            u_z2[1] = (frac32_z as u64)
                .checked_shl((64 - shift_right) as u32)
                .unwrap_or(0);
        }
    }

    //This is the only difference from ADD (sign_z2) and (!sign_z2)
    if !sign_z2 {
        if u_z2[1] > 0 {
            u_z2[1] = u_z2[1].wrapping_neg();
            u_z2[0] = !u_z2[0];
        } else {
            u_z2[0] = u_z2[0].wrapping_neg();
        }
    }

    //Subtraction
    let b1 = u_z1[1] & 0x1 != 0;
    let b2 = u_z2[1] & 0x1 != 0;
    let rcarryb = b1 & b2;
    let mut u_z: [u64; 2] = [0, (u_z1[1] >> 1) + (u_z2[1] >> 1) + (rcarryb as u64)];

    let rcarry_z = (u_z[1] & 0x_8000_0000_0000_0000) != 0;

    u_z[1] = u_z[1] << 1 | ((b1 ^ b2) as u64);

    let b1 = u_z1[0] & 0x1 != 0;
    let b2 = u_z2[0] & 0x1 != 0;
    //let rcarryb = b1 & b2;
    let rcarryb3 = (b1 as i8) + (b2 as i8) + (rcarry_z as i8);

    u_z[0] = (u_z1[0] >> 1) + (u_z2[0] >> 1) + (((rcarryb3 >> 1) & 0x1) as u64);
    //rcarrySignZ = u_z[0]>>63;

    u_z[0] = u_z[0] << 1 | ((rcarryb3 & 0x1) as u64);

    //Exception handling
    let q_z = Q16E1::from_bits(u_z);
    *q = if q_z.is_nar() { Q16E1::ZERO } else { q_z }
}

#[cfg(test)]
fn ulp(x: P16E1, y: P16E1) -> i16 {
    let xi = x.to_bits() as i16;
    let yi = y.to_bits() as i16;
    (xi - yi).abs()
}

#[test]
fn test_quire_mul_add() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS16 {
        let p_a: P16E1 = rng.gen();
        let p_b: P16E1 = rng.gen();
        let p_c: P16E1 = rng.gen();
        let f_a = f64::from(p_a);
        let f_b = f64::from(p_b);
        let f_c = f64::from(p_c);
        let mut q = Q16E1::init();
        q += (p_a, p_b);
        q += (p_c, P16E1::ONE);
        let p = q.to_posit();
        let f = f_a.mul_add(f_b, f_c);
        assert!(ulp(p, P16E1::from(f)) <= 1);
    }
}

#[test]
fn test_quire_mul_sub() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS16 {
        let p_a: P16E1 = rng.gen();
        let p_b: P16E1 = rng.gen();
        let p_c: P16E1 = rng.gen();
        let f_a = f64::from(p_a);
        let f_b = f64::from(p_b);
        let f_c = f64::from(p_c);
        let mut q = Q16E1::init();
        q -= (p_a, p_b);
        q += (p_c, P16E1::ONE);
        let p = q.to_posit();
        let f = (-f_a).mul_add(f_b, f_c);
        assert!(
            ulp(p, P16E1::from(f)) <= 1 /*, "p_a = {}\tp_b = {}\tp_c = {}\tp = {}\tf = {}", p_a, p_b, p_c, p, f*/
        );
    }
}

#[cfg(test)]
fn test_ops(fun: fn(P16E1, P16E1, f64, f64) -> (P16E1, f64)) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS16 {
        let p_a: P16E1 = rng.gen();
        let p_b: P16E1 = rng.gen();
        let f_a = f64::from(p_a);
        let f_b = f64::from(p_b);
        let (p, f) = fun(p_a, p_b, f_a, f_b);
        assert_eq!(p, P16E1::from(f));
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
