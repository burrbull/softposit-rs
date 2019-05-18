use super::{P32E2, Q32E2};
use crate::WithSign;
use core::ops;

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
        if (ui_a == 0) || (ui_b == 0) {
            // Not required but put here for speed
            Self::from_bits(ui_a | ui_b)
        } else if (ui_a == 0x8000_0000) || (ui_b == 0x8000_0000) {
            Self::INFINITY
        } else {
            //different signs
            if Self::sign_ui(ui_a ^ ui_b) {
                sub_mags_p32(ui_a, ui_b)
            } else {
                add_mags_p32(ui_a, ui_b)
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
        if (ui_a == 0x8000_0000) || (ui_b == 0x8000_0000) {
            Self::INFINITY
        } else if (ui_a == 0) || (ui_b == 0) {
            //Zero
            Self::from_bits(ui_a | ui_b.wrapping_neg())
        } else {
            //different signs
            if Self::sign_ui(ui_a ^ ui_b) {
                add_mags_p32(ui_a, ui_b.wrapping_neg())
            } else {
                sub_mags_p32(ui_a, ui_b.wrapping_neg())
            }
        }
    }
}

impl ops::Div for P32E2 {
    type Output = Self;
    #[inline]
    fn div(self, other: Self) -> Self {
        let mut u_z: u32;

        let mut ui_a = self.to_bits();
        let mut ui_b = other.to_bits();

        //Zero or infinity
        if (ui_a == 0x8000_0000) || (ui_b == 0x8000_0000) || (ui_b == 0) {
            return Self::INFINITY;
        } else if ui_a == 0 {
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

        let (regime, reg_sa, reg_a) = Self::calculate_regime(k_a);

        if reg_a > 30 {
            //max or min pos. exp and frac does not matter.
            u_z = if reg_sa { 0x7FFF_FFFF } else { 0x1 };
        } else {
            //remove carry and rcarry bits and shift to correct position
            frac64_z &= 0x3FFF_FFFF;

            frac_a = (frac64_z >> (reg_a + 2)) as u32;

            let mut bit_n_plus_one = false;
            let mut bits_more = false;
            if reg_a <= 28 {
                bit_n_plus_one = ((frac64_z >> (reg_a + 1)) & 0x1) != 0;
                exp_a <<= 28 - reg_a;
                if bit_n_plus_one {
                    bits_more = (((1 << (reg_a + 1)) - 1) & frac64_z) != 0;
                }
            } else {
                if reg_a == 30 {
                    bit_n_plus_one = (exp_a & 0x2) != 0;
                    bits_more = (exp_a & 0x1) != 0;
                    exp_a = 0;
                } else if reg_a == 29 {
                    bit_n_plus_one = (exp_a & 0x1) != 0;
                    exp_a >>= 1; //taken care of by the pack algo
                }
                if frac64_z > 0 {
                    frac_a = 0;
                    bits_more = true;
                }
            }
            if rem != 0 {
                bits_more = true;
            }

            u_z = Self::pack_to_ui(regime, exp_a as u32, frac_a);
            if bit_n_plus_one {
                u_z += (u_z & 1) | (bits_more as u32);
            }
        }

        Self::from_bits(u_z.with_sign(sign_z))
    }
}

impl ops::Mul for P32E2 {
    type Output = Self;
    #[inline]
    fn mul(self, other: Self) -> Self {
        let mut ui_a = self.to_bits();
        let mut ui_b = other.to_bits();

        //NaR or Zero
        if (ui_a == 0x8000_0000) || (ui_b == 0x8000_0000) {
            return Self::INFINITY;
        } else if (ui_a == 0) || (ui_b == 0) {
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

        let (k_b, exp_b, frac_b) = Self::separate_bits(ui_b);
        k_a += k_b;
        exp_a += exp_b;
        let mut frac64_z = (frac_a as u64) * (frac_b as u64);

        if exp_a > 3 {
            k_a += 1;
            exp_a &= 0x3; // -=4
        }

        let rcarry = (frac64_z >> 61) != 0; //3rd bit of frac64_z
        if rcarry {
            exp_a += 1;
            if exp_a > 3 {
                k_a += 1;
                exp_a &= 0x3;
            }
            frac64_z >>= 1;
        }
        let (regime, reg_sa, reg_a) = Self::calculate_regime(k_a);

        let u_z = if reg_a > 30 {
            //max or min pos. exp and frac does not matter.
            if reg_sa {
                0x7FFF_FFFF
            } else {
                0x1
            }
        } else {
            //remove carry and rcarry bits and shift to correct position (2 bits exp, so + 1 than 16 bits)
            frac64_z = (frac64_z & 0xFFF_FFFF_FFFF_FFFF) >> reg_a;
            frac_a = (frac64_z >> 32) as u32;

            let mut bit_n_plus_one = false;
            if reg_a <= 28 {
                bit_n_plus_one = (0x8000_0000 & frac64_z) != 0;
                exp_a <<= 28 - reg_a;
            } else {
                if reg_a == 30 {
                    bit_n_plus_one = (exp_a & 0x2) != 0;
                    exp_a = 0;
                } else if reg_a == 29 {
                    bit_n_plus_one = (exp_a & 0x1) != 0;
                    exp_a >>= 1; //taken care of by the pack algo
                }
                if frac_a > 0 {
                    frac_a = 0;
                }
            }
            //sign is always zero
            let mut u_z = Self::pack_to_ui(regime, exp_a as u32, frac_a);
            //n+1 frac bit is 1. Need to check if another bit is 1 too if not round to even
            if bit_n_plus_one {
                let bits_more = (0x7FFF_FFFF & frac64_z) != 0;
                u_z += (u_z & 1) | (bits_more as u32);
            }
            u_z
        };

        Self::from_bits(u_z.with_sign(sign_z))
    }
}

#[inline]
fn add_mags_p32(mut ui_a: u32, mut ui_b: u32) -> P32E2 {
    let sign = P32E2::sign_ui(ui_a);
    if sign {
        ui_a = ui_a.wrapping_neg();
        ui_b = ui_b.wrapping_neg();
    }

    if (ui_a as i32) < (ui_b as i32) {
        ui_a ^= ui_b;
        ui_b ^= ui_a;
        ui_a ^= ui_b;
    }

    let (mut k_a, mut exp_a, frac_a) = P32E2::separate_bits(ui_a);

    let mut frac64_a = (frac_a as u64) << 32;

    let (k_b, exp_b, frac_b) = P32E2::separate_bits(ui_b);

    let mut shift_right = (k_a as i16) - (k_b as i16);
    let mut frac64_b = (frac_b as u64) << 32;

    //This is 4kZ + expZ; (where kZ=k_a-kB and expZ=exp_a-expB)
    shift_right = (shift_right << 2) + (exp_a as i16) - (exp_b as i16);

    //Manage CLANG (LLVM) compiler when shifting right more than number of bits
    if shift_right > 63 {
        frac64_b = 0;
    } else {
        frac64_b >>= shift_right;
    }

    frac64_a += frac64_b;

    let rcarry = (0x8000_0000_0000_0000 & frac64_a) != 0; //first left bit
    if rcarry {
        exp_a += 1;
        if exp_a > 3 {
            k_a += 1;
            exp_a &= 0x3;
        }
        frac64_a >>= 1;
    }
    let (regime, reg_sa, reg_a) = P32E2::calculate_regime(k_a);

    let u_z = if reg_a > 30 {
        //max or min pos. exp and frac does not matter.
        if reg_sa {
            0x7FFF_FFFF
        } else {
            0x1
        }
    } else {
        //remove hidden bits
        frac64_a = (frac64_a & 0x3FFF_FFFF_FFFF_FFFF) >> (reg_a + 2); // 2 bits exp

        let mut frac_a = (frac64_a >> 32) as u32;

        let mut bit_n_plus_one = false;
        if reg_a <= 28 {
            bit_n_plus_one = (0x8000_0000 & frac64_a) != 0;
            exp_a <<= 28 - reg_a;
        } else {
            if reg_a == 30 {
                bit_n_plus_one = (exp_a & 0x2) != 0;
                exp_a = 0;
            } else if reg_a == 29 {
                bit_n_plus_one = (exp_a & 0x1) != 0;
                exp_a >>= 1;
            }
            if frac_a > 0 {
                frac_a = 0;
            }
        }

        let mut u_z = P32E2::pack_to_ui(regime, exp_a as u32, frac_a);
        //n+1 frac bit is 1. Need to check if another bit is 1 too if not round to even
        if bit_n_plus_one {
            let bits_more = (0x7FFF_FFFF & frac64_a) != 0;
            u_z += (u_z & 1) | (bits_more as u32);
        }
        u_z
    };

    P32E2::from_bits(u_z.with_sign(sign))
}

#[inline]
fn sub_mags_p32(mut ui_a: u32, mut ui_b: u32) -> P32E2 {
    let mut sign = P32E2::sign_ui(ui_a);
    if sign {
        ui_a = ui_a.wrapping_neg();
    } else {
        ui_b = ui_b.wrapping_neg();
    }

    if ui_a == ui_b {
        //essential, if not need special handling
        return P32E2::ZERO;
    }
    if (ui_a as i32) < (ui_b as i32) {
        ui_a ^= ui_b;
        ui_b ^= ui_a;
        ui_a ^= ui_b;
        sign = !sign; //A becomes B
    }

    let (mut k_a, mut exp_a, frac_a) = P32E2::separate_bits(ui_a);

    let mut frac64_a = (frac_a as u64) << 32;

    let (k_b, exp_b, frac_b) = P32E2::separate_bits(ui_b);

    let mut shift_right = (k_a as i16) - (k_b as i16);
    let mut frac64_b = (frac_b as u64) << 32;
    //This is 4kZ + expZ; (where kZ=k_a-kB and expZ=exp_a-expB)
    shift_right = (shift_right << 2) + (exp_a as i16) - (exp_b as i16);

    if shift_right > 63 {
        return P32E2::from_bits(if sign { ui_a.wrapping_neg() } else { ui_a });
    } else {
        frac64_b >>= shift_right;
    }

    frac64_a -= frac64_b;

    while (frac64_a >> 59) == 0 {
        k_a -= 1;
        frac64_a <<= 4;
    }
    let mut ecarry = (0x4000_0000_0000_0000 & frac64_a) != 0; //(0x4000_0000_0000_0000 & frac64_a)>>62;
    while !ecarry {
        if exp_a == 0 {
            k_a -= 1;
            exp_a = 3;
        } else {
            exp_a -= 1;
        }
        frac64_a <<= 1;
        ecarry = (0x4000_0000_0000_0000 & frac64_a) != 0;
    }

    let (regime, reg_sa, reg_a) = P32E2::calculate_regime(k_a);

    let u_z = if reg_a > 30 {
        //max or min pos. exp and frac does not matter.
        if reg_sa {
            0x7FFF_FFFF
        } else {
            0x1
        }
    } else {
        //remove hidden bits
        frac64_a = (frac64_a & 0x3FFF_FFFF_FFFF_FFFF) >> (reg_a + 2); // 2 bits exp

        let mut frac_a = (frac64_a >> 32) as u32;

        let mut bit_n_plus_one = false;
        if reg_a <= 28 {
            bit_n_plus_one = (0x8000_0000 & frac64_a) != 0;
            exp_a <<= 28 - reg_a;
        } else {
            if reg_a == 30 {
                bit_n_plus_one = (exp_a & 0x2) != 0;
                exp_a = 0;
            } else if reg_a == 29 {
                bit_n_plus_one = (exp_a & 0x1) != 0;
                exp_a >>= 1;
            }
            if frac_a > 0 {
                frac_a = 0;
            }
        }

        let mut u_z = P32E2::pack_to_ui(regime, exp_a as u32, frac_a);
        //n+1 frac bit is 1. Need to check if another bit is 1 too if not round to even
        if bit_n_plus_one {
            let bits_more = (0x7FFF_FFFF & frac64_a) != 0;
            u_z += (u_z & 1) | (bits_more as u32);
        }
        u_z
    };

    P32E2::from_bits(u_z.with_sign(sign))
}

impl ops::Rem for P32E2 {
    type Output = Self;
    fn rem(self, other: Self) -> Self {
        self - (self / other).trunc() * other
    }
}

impl ops::AddAssign<(P32E2, P32E2)> for Q32E2 {
    #[inline]
    fn add_assign(&mut self, rhs: (P32E2, P32E2)) {
        q32_fdp_add(self, rhs.0, rhs.1);
    }
}

impl ops::SubAssign<(P32E2, P32E2)> for Q32E2 {
    #[inline]
    fn sub_assign(&mut self, rhs: (P32E2, P32E2)) {
        q32_fdp_sub(self, rhs.0, rhs.1);
    }
}

pub(super) fn q32_fdp_add(q: &mut Q32E2, p_a: P32E2, p_b: P32E2) {
    let u_z1 = q.to_bits();

    let mut ui_a = p_a.to_bits();
    let mut ui_b = p_b.to_bits();

    if q.is_nan() || p_a.is_nan() || p_b.is_nan() {
        *q = Q32E2::NAN;
        return;
    } else if (ui_a == 0) || (ui_b == 0) {
        return;
    }

    //max pos (sign plus and minus)
    let sign_a = P32E2::sign_ui(ui_a);
    let sign_b = P32E2::sign_ui(ui_b);
    let sign_z2 = sign_a ^ sign_b;

    if sign_a {
        ui_a = ui_a.wrapping_neg();
    }
    if sign_b {
        ui_b = ui_b.wrapping_neg();
    }

    let (mut k_a, tmp) = P32E2::separate_bits_tmp(ui_a);
    let mut exp_a = (tmp >> 29) as i32; //to get 2 bits
    let frac_a = ((tmp << 2) | 0x_8000_0000) & 0x_ffff_ffff;

    let (k_b, tmp) = P32E2::separate_bits_tmp(ui_b);
    k_a += k_b;
    exp_a += (tmp >> 29) as i32;
    let mut frac64_z = (frac_a as u64) * ((((tmp << 2) | 0x_8000_0000) & 0x_ffff_ffff) as u64);

    if exp_a > 3 {
        k_a += 1;
        exp_a &= 0x3; // -=4
    }
    //Will align frac64_z such that hidden bit is the first bit on the left.
    let rcarry = (frac64_z >> 63) != 0; //1st bit of frac64_z
    if rcarry {
        exp_a += 1;
        if exp_a > 3 {
            k_a += 1;
            exp_a &= 0x3;
        }
    //frac64_z>>=1;
    } else {
        frac64_z <<= 1;
    }

    //default dot is between bit 271 and 272, extreme left bit is bit 0. Last right bit is bit 512.
    //Minpos is 120 position to the right of binary point (dot)
    //Scale = 2^es * k + e  => 2k + e
    let first_pos = 271 - ((k_a << 2) as i32) - exp_a;

    //Moving in chunk of 64. If it is in first chunk, a part might be in the chunk right to it. Simply have to handle that.
    let mut u_z2: [u64; 8] = [0; 8];
    for i in 0usize..8 {
        if first_pos < ((i + 1) * 64) as i32 {
            //Need to check how much of the fraction is in the next 64 bits
            let shift_right = (first_pos - ((i * 64) as i32)) as i16;
            u_z2[i] = frac64_z >> shift_right;
            if (i != 7) && (shift_right != 0) {
                u_z2[i + 1] = frac64_z << (64 - shift_right);
            }
            break;
        }
    }

    if sign_z2 {
        let mut j = u_z2.iter_mut().rev();
        while let Some(u) = j.next() {
            if *u > 0 {
                *u = u.wrapping_neg();
                while let Some(w) = j.next() {
                    *w = !*w;
                }
                break;
            }
        }
    }

    //Subtraction
    let mut u_z: [u64; 8] = [0; 8];
    let mut rcarry_z = false;
    for (i, (u, (u1, u2))) in (0..8).rev().zip(
        u_z.iter_mut()
            .rev()
            .zip(u_z1.iter().rev().zip(u_z2.iter().rev())),
    ) {
        let b1 = (*u1 & 0x1) != 0;
        let b2 = (*u2 & 0x1) != 0;
        if i == 7 {
            let rcarryb = b1 & b2;
            *u = (*u1 >> 1) + (*u2 >> 1) + (rcarryb as u64);
            rcarry_z = *u >> 63 != 0;
            *u = (*u << 1) | ((b1 ^ b2) as u64);
        } else {
            let rcarryb3 = (b1 as i8) + (b2 as i8) + (rcarry_z as i8);
            *u = (*u1 >> 1) + (*u2 >> 1) + ((rcarryb3 >> 1) as u64);
            rcarry_z = *u >> 63 != 0;
            *u = (*u << 1) | ((rcarryb3 & 0x1) as u64);
        }
    }

    //Exception handling
    let q_z = Q32E2::from_bits(u_z);
    *q = if q_z.is_nan() { Q32E2::ZERO } else { q_z }
}

pub(super) fn q32_fdp_sub(q: &mut Q32E2, p_a: P32E2, p_b: P32E2) {
    let u_z1 = q.to_bits();

    let mut ui_a = p_a.to_bits();
    let mut ui_b = p_b.to_bits();

    if q.is_nan() || p_a.is_nan() || p_b.is_nan() {
        *q = Q32E2::NAN;
        return;
    } else if (ui_a == 0) || (ui_b == 0) {
        return;
    }

    //max pos (sign plus and minus)
    let sign_a = P32E2::sign_ui(ui_a);
    let sign_b = P32E2::sign_ui(ui_b);
    let sign_z2 = sign_a ^ sign_b;

    if sign_a {
        ui_a = ui_a.wrapping_neg();
    }
    if sign_b {
        ui_b = ui_b.wrapping_neg();
    }

    let (mut k_a, tmp) = P32E2::separate_bits_tmp(ui_a);
    let mut exp_a = (tmp >> 29) as i32; //to get 2 bits
    let frac_a = ((tmp << 2) | 0x_8000_0000) & 0x_ffff_ffff;

    let (k_b, tmp) = P32E2::separate_bits_tmp(ui_b);
    k_a += k_b;
    exp_a += (tmp >> 29) as i32;
    let mut frac64_z = (frac_a as u64) * ((((tmp << 2) | 0x_8000_0000) & 0x_ffff_ffff) as u64);

    if exp_a > 3 {
        k_a += 1;
        exp_a &= 0x3; // -=4
    }
    //Will align frac64_z such that hidden bit is the first bit on the left.
    let rcarry = (frac64_z >> 63) != 0; //1st bit of frac64_z
    if rcarry {
        exp_a += 1;
        if exp_a > 3 {
            k_a += 1;
            exp_a &= 0x3;
        }
    //frac64_z>>=1;
    } else {
        frac64_z <<= 1;
    }

    //default dot is between bit 271 and 272, extreme left bit is bit 0. Last right bit is bit 512.
    //Minpos is 120 position to the right of binary point (dot)
    //Scale = 2^es * k + e  => 2k + e
    let first_pos = 271 - ((k_a << 2) as i32) - exp_a;

    //Moving in chunk of 64. If it is in first chunk, a part might be in the chunk right to it. Simply have to handle that.
    let mut u_z2: [u64; 8] = [0; 8];
    for i in 0usize..8 {
        if first_pos < ((i + 1) * 64) as i32 {
            //Need to check how much of the fraction is in the next 64 bits
            let shift_right = (first_pos - ((i * 64) as i32)) as i16;
            u_z2[i] = frac64_z >> shift_right;
            if (i != 7) && (shift_right != 0) {
                u_z2[i + 1] = frac64_z << (64 - shift_right);
            }
            break;
        }
    }

    //This is the only difference from ADD (sign_z2) and (!sign_z2)
    if !sign_z2 {
        let mut j = u_z2.iter_mut().rev();
        while let Some(u) = j.next() {
            if *u > 0 {
                *u = u.wrapping_neg();
                while let Some(w) = j.next() {
                    *w = !*w;
                }
                break;
            }
        }
    }

    //Subtraction
    let mut u_z: [u64; 8] = [0; 8];
    let mut rcarry_z = false;
    for (i, (u, (u1, u2))) in (0..8).rev().zip(
        u_z.iter_mut()
            .rev()
            .zip(u_z1.iter().rev().zip(u_z2.iter().rev())),
    ) {
        let b1 = (*u1 & 0x1) != 0;
        let b2 = (*u2 & 0x1) != 0;
        if i == 7 {
            let rcarryb = b1 & b2;
            *u = (*u1 >> 1) + (*u2 >> 1) + (rcarryb as u64);
            rcarry_z = *u >> 63 != 0;
            *u = (*u << 1) | ((b1 ^ b2) as u64);
        } else {
            let rcarryb3 = (b1 as i8) + (b2 as i8) + (rcarry_z as i8);
            *u = (*u1 >> 1) + (*u2 >> 1) + ((rcarryb3 >> 1) as u64);
            rcarry_z = *u >> 63 != 0;
            *u = (*u << 1) | ((rcarryb3 & 0x1) as u64);
        }
    }

    //Exception handling
    let q_z = Q32E2::from_bits(u_z);
    *q = if q_z.is_nan() { Q32E2::ZERO } else { q_z }
}

#[test]
fn test_quire_mul_add() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS32 {
        let p_a: P32E2 = rng.gen();
        let p_b: P32E2 = rng.gen();
        let p_c: P32E2 = rng.gen();
        let f_a = f64::from(p_a);
        let f_b = f64::from(p_b);
        let f_c = f64::from(p_c);
        let mut q = Q32E2::new();
        q += (p_a, p_b);
        q += (p_c, P32E2::ONE);
        let p = q.roundp();
        let f = f_a.mul_add(f_b, f_c);
        assert_eq!(p, P32E2::from(f));
    }
}

#[test]
fn test_quire_mul_sub() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS32 {
        let p_a: P32E2 = rng.gen();
        let p_b: P32E2 = rng.gen();
        let p_c: P32E2 = rng.gen();
        let f_a = f64::from(p_a);
        let f_b = f64::from(p_b);
        let f_c = f64::from(p_c);
        let mut q = Q32E2::new();
        q -= (p_a, p_b);
        q += (p_c, P32E2::ONE);
        let p = q.roundp();
        let f = (-f_a).mul_add(f_b, f_c);
        assert_eq!(p, P32E2::from(f));
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
