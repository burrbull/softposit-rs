use super::*;
use num_traits::Zero;
use crate::{MulAddType, WithSign};

impl Q32E2 {
    #[inline]
    pub fn fdp_add(self, p_a: P32E2, p_b: P32E2) -> Self {
        q32_fdp_add(self, p_a, p_b)
    }
    #[inline]
    pub fn fdp_sub(self, p_a: P32E2, p_b: P32E2) -> Self {
        q32_fdp_sub(self, p_a, p_b)
    }
}

pub(super) fn mul_add(mut ui_a: u32, mut ui_b: u32, mut ui_c: u32, op: MulAddType) -> P32E2 {
    let mut bits_more = false;
    //NaR
    if (ui_a == 0x8000_0000) || (ui_b == 0x8000_0000) || (ui_c == 0x8000_0000) {
        return INFINITY;
    } else if (ui_a == 0) || (ui_b == 0) {
        return match op {
            MulAddType::SubC => P32E2::from_bits(ui_c.wrapping_neg()),
            _ => P32E2::from_bits(ui_c),
        };
    }

    let sign_a = P32E2::sign_ui(ui_a);
    let sign_b = P32E2::sign_ui(ui_b);
    let sign_c = P32E2::sign_ui(ui_c); //^ (op == softposit_mulAdd_subC);
    let mut sign_z = sign_a ^ sign_b; // ^ (op == softposit_mulAdd_subProd);

    if sign_a {
        ui_a = ui_a.wrapping_neg();
    }
    if sign_b {
        ui_b = ui_b.wrapping_neg();
    }
    if sign_c {
        ui_c = ui_c.wrapping_neg();
    }

    let (mut k_a, tmp) = P32E2::separate_bits_tmp(ui_a);
    let mut exp_a = (tmp >> 29) as i32; //to get 2 bits
    let frac_a = (tmp << 2) | 0x8000_0000;

    let (k_b, tmp) = P32E2::separate_bits_tmp(ui_b);
    k_a += k_b;
    exp_a += (tmp >> 29) as i32;
    let mut frac64_z = (frac_a as u64) * (((tmp << 2) | 0x8000_0000) as u64);

    if exp_a > 3 {
        k_a += 1;
        exp_a &= 0x3; // -=4
    }

    let rcarry = (frac64_z & 0x_8000_0000_0000_0000) != 0; //1st bit of frac64_z
    if rcarry {
        exp_a += 1;
        if exp_a > 3 {
            k_a += 1;
            exp_a &= 0x3;
        }
        frac64_z >>= 1;
    }

    let mut k_z;
    let mut exp_z: i32;
    if ui_c != 0 {
        let (k_c, exp_c, frac_c) = P32E2::separate_bits(ui_c);
        let mut frac64_c = (frac_c as u64) << 32;
        let mut shift_right = (((k_a - k_c) as i16) << 2) + (exp_a - exp_c) as i16;

        exp_z = if shift_right < 0 {
            // |ui_c| > |Prod|
            if shift_right <= -63 {
                bits_more = true;
                frac64_z = 0;
                shift_right = 0;
            //set bits_more to one?
            } else if (frac64_z << (64 + shift_right)) != 0 {
                bits_more = true;
            }
            if sign_z == sign_c {
                frac64_z = frac64_c + (frac64_z >> -shift_right);
            } else {
                //different signs
                frac64_z = frac64_c - (frac64_z >> -shift_right);
                sign_z = sign_c;
                if bits_more {
                    frac64_z -= 1;
                }
            }
            k_z = k_c;
            exp_c
        } else if shift_right > 0 {
            // |ui_c| < |Prod|
            //if frac32C&((1<<shift_right)-1) {bits_more = true;}
            if shift_right >= 63 {
                bits_more = true;
                frac64_c = 0;
                shift_right = 0;
            } else if (frac64_c << (64 - shift_right)) != 0 {
                bits_more = true;
            }
            if sign_z == sign_c {
                frac64_z += frac64_c >> shift_right;
            } else {
                frac64_z -= frac64_c >> shift_right;
                if bits_more {
                    frac64_z -= 1;
                }
            }
            k_z = k_a;
            exp_a
        } else {
            if (frac64_c == frac64_z) && (sign_z != sign_c) {
                //check if same number
                return P32E2::zero();
            } else if sign_z == sign_c {
                frac64_z += frac64_c;
            } else if frac64_z < frac64_c {
                frac64_z = frac64_c - frac64_z;
                sign_z = sign_c;
            } else {
                frac64_z -= frac64_c;
            }
            k_z = k_a; // actually can be k_c too, no diff
            exp_a //same here
        };
        let rcarry = (frac64_z & 0x_8000_0000_0000_0000) != 0; //first left bit

        if rcarry {
            exp_z += 1;
            if exp_z > 3 {
                k_z += 1;
                exp_z &= 0x3;
            }
            frac64_z = (frac64_z >> 1) & 0x7FFF_FFFF_FFFF_FFFF;
        } else {
            //for subtract cases
            if frac64_z != 0 {
                while (frac64_z >> 59) == 0 {
                    k_z -= 1;
                    frac64_z <<= 4;
                }
                while (frac64_z >> 62) == 0 {
                    exp_z -= 1;
                    frac64_z <<= 1;
                    if exp_z < 0 {
                        k_z -= 1;
                        exp_z = 3;
                    }
                }
            }
        }
    } else {
        k_z = k_a;
        exp_z = exp_a;
    }

    let (regime, reg_sz, reg_z) = P32E2::calculate_regime(k_z);

    let u_z = if reg_z > 30 {
        //max or min pos. exp and frac does not matter.
        if reg_sz {
            0x7FFF_FFFF
        } else {
            0x1
        }
    } else {
        let mut bit_n_plus_one = false;
        let mut frac_z: u32 = 0; // possibly uninitialized
        if reg_z <= 28 {
            //remove hidden bits
            frac64_z &= 0x3FFF_FFFF_FFFF_FFFF;
            frac_z = (frac64_z >> (reg_z + 34)) as u32; //frac32Z>>16;
            bit_n_plus_one = (0x2_0000_0000 & (frac64_z >> reg_z)) != 0;
            exp_z <<= 28 - reg_z;
        } else {
            if reg_z == 30 {
                bit_n_plus_one = (exp_z & 0x2) != 0;
                bits_more = (exp_z & 0x1) != 0;
                exp_z = 0;
            } else if reg_z == 29 {
                bit_n_plus_one = (exp_z & 0x1) != 0;
                exp_z >>= 1;
            }
            if frac_z > 0 {
                frac_z = 0;
                bits_more = true;
            }
        }
        let mut u_z = P32E2::pack_to_ui(regime, exp_z as u32, frac_z);

        if bit_n_plus_one {
            if (frac64_z << (32 - reg_z)) != 0
            /* &0xFFFF_FFFF_FFFF_FFFF */
            {
                bits_more = true;
            }
            u_z += (u_z & 1) | (bits_more as u32);
        }
        u_z
    };
    P32E2::from_bits(u_z.with_sign(sign_z))
}

pub(super) fn round(p_a: P32E2) -> P32E2 {
    let mut mask = 0x2000_0000_u32;
    let mut scale = 0_u32;

    let u_a: u32;

    let mut ui_a = p_a.to_bits();
    let sign = (ui_a & 0x8000_0000) != 0;

    // sign is True if pA > NaR.
    if sign {
        ui_a = ui_a.wrapping_neg();
    } // A is now |A|.
    if ui_a <= 0x3800_0000 {
        // 0 <= |pA| <= 1/2 rounds to zero.
        return P32E2::zero();
    } else if ui_a < 0x4400_0000 {
        // 1/2 < x < 3/2 rounds to 1.
        u_a = 0x4000_0000;
    } else if ui_a <= 0x4A00_0000 {
        // 3/2 <= x <= 5/2 rounds to 2.
        u_a = 0x4800_0000;
    } else if ui_a >= 0x7E80_0000 {
        // If |A| is 0x7E80_0000 (posit is pure integer value), leave it unchanged.
        return p_a; // This also takes care of the NaR case, 0x8000_0000.
    } else {
        // 34% of the cases, we have to decode the posit.

        while (mask & ui_a) != 0 {
            scale += 4;
            mask >>= 1;
        }
        mask >>= 1;

        //Exponential (2 bits)
        if (mask & ui_a) != 0 {
            scale += 2;
        }
        mask >>= 1;
        if (mask & ui_a) != 0 {
            scale += 1;
        }
        mask >>= scale;

        //the rest of the bits
        let bit_last = (ui_a & mask) != 0;
        mask >>= 1;
        let mut tmp = ui_a & mask;
        let bit_n_plus_one = tmp != 0;
        ui_a ^= tmp; // Erase the bit, if it was set.
        tmp = ui_a & (mask - 1); // this is actually bits_more

        ui_a ^= tmp;

        if bit_n_plus_one && (((bit_last as u32) | tmp) != 0) {
            ui_a += mask << 1;
        }
        u_a = ui_a;
    }
    P32E2::from_bits(u_a.with_sign(sign))
}

#[inline]
pub(super) fn sqrt(p_a: P32E2) -> P32E2 {
    let mut ui_a = p_a.to_bits();

    // If NaR or a negative number, return NaR.
    if (ui_a & 0x8000_0000) != 0 {
        return INFINITY;
    }
    // If the argument is zero, return zero.
    else if ui_a == 0 {
        return p_a;
    }
    // Compute the square root; shift_z is the power-of-2 scaling of the result.
    // Decode regime and exponent; scale the input to be in the range 1 to 4:
    let mut shift_z: i32;
    if (ui_a & 0x4000_0000) != 0 {
        shift_z = -2;
        while (ui_a & 0x4000_0000) != 0 {
            shift_z += 2;
            ui_a <<= 1 /*() & 0xFFFF_FFFF*/;
        }
    } else {
        shift_z = 0;
        while (ui_a & 0x4000_0000) == 0 {
            shift_z -= 2;
            ui_a <<= 1 /*90 & 0xFFFF_FFFF*/;
        }
    }

    ui_a &= 0x3FFF_FFFF;
    let mut exp_a = ui_a >> 28;
    shift_z += (exp_a >> 1) as i32;
    exp_a = 0x1 ^ (exp_a & 0x1);
    ui_a &= 0x0FFF_FFFF;
    let frac_a = ui_a | 0x1000_0000;

    // Use table look-up of first 4 bits for piecewise linear approx. of 1/sqrt:
    let index = (((frac_a >> 24) & 0xE) + exp_a) as usize;
    let eps = ((frac_a >> 9) & 0xFFFF) as i32;
    let r0: u32 = (crate::APPROX_RECIP_SQRT0[index] as u32)
        - (((crate::APPROX_RECIP_SQRT1[index] as u32) * (eps as u32)) >> 20);

    // Use Newton-Raphson refinement to get 33 bits of accuracy for 1/sqrt:
    let mut e_sqr_r0 = (r0 as u64) * (r0 as u64);
    if exp_a == 0 {
        e_sqr_r0 <<= 1;
    }
    let sigma0: u64 = 0xFFFF_FFFF & (0xFFFF_FFFF ^ ((e_sqr_r0 * (frac_a as u64)) >> 20));
    let mut recip_sqrt: u64 = ((r0 as u64) << 20) + (((r0 as u64) * sigma0) >> 21);

    let sqr_sigma0 = (sigma0 * sigma0) >> 35;
    recip_sqrt += ((recip_sqrt + (recip_sqrt >> 2) - ((r0 as u64) << 19)) * sqr_sigma0) >> 46;

    let mut frac_z = ((frac_a as u64) * recip_sqrt) >> 31;
    if exp_a != 0 {
        frac_z >>= 1;
    }

    // Find the exponent of Z and encode the regime bits.
    let exp_z = (shift_z as u32) & 0x3;
    let shift: u32;
    let ui_z: u32 = if shift_z < 0 {
        shift = ((-1 - shift_z) >> 2) as u32;
        0x2000_0000 >> shift
    } else {
        shift = (shift_z >> 2) as u32;
        0x7FFF_FFFF - (0x3FFF_FFFF >> shift)
    };

    // Trick for eliminating off-by-one cases that only uses one multiply:
    frac_z += 1;

    if (frac_z & 0xF) == 0 {
        let shifted_frac_z = frac_z >> 1;
        let neg_rem = (shifted_frac_z * shifted_frac_z) & 0x1_FFFF_FFFF;
        if (neg_rem & 0x1_0000_0000) != 0 {
            frac_z |= 1;
        } else if neg_rem != 0 {
            frac_z -= 1;
        }
    }
    // Strip off the hidden bit and round-to-nearest using last shift+5 bits.
    frac_z &= 0xFFFF_FFFF;
    let mask = 1 << (4 + shift);
    if ((mask & frac_z) != 0) && ((((mask - 1) & frac_z) | ((mask << 1) & frac_z)) != 0) {
        frac_z += mask << 1;
    }
    // Assemble the result and return it.
    P32E2::from_bits(ui_z | (exp_z << (27 - shift)) | (frac_z >> (5 + shift)) as u32)
}

fn q32_fdp_add(q: Q32E2, p_a: P32E2, p_b: P32E2) -> Q32E2 {
    let u_z1 = q.to_bits();

    let mut ui_a = p_a.to_bits();
    let mut ui_b = p_b.to_bits();

    if q.is_nan() || p_a.is_nan() || p_b.is_nan() {
        return Q32E2::new(-0x8000_0000_0000_0000, 0, 0, 0, 0, 0, 0, 0);
    } else if (ui_a == 0) || (ui_b == 0) {
        return q;
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

    let (mut k_a, mut exp_a, frac_a) = P32E2::separate_bits(ui_a);

    let (k_b, exp_b, frac_b) = P32E2::separate_bits(ui_b);
    k_a += k_b;
    exp_a += exp_b;
    let mut frac64_z = (frac_a as u64) * (frac_b as u64);

    if exp_a>3 {
        k_a += 1;
        exp_a&=0x3; // -=4
    }
    //Will align frac64_z such that hidden bit is the first bit on the left.
    let rcarry = (frac64_z>>63) != 0;//1st bit of frac64_z
    if rcarry {
        exp_a += 1;
        if exp_a>3 {
            k_a += 1;
            exp_a&=0x3;
        }
        //frac64_z>>=1;
    }
    else {
        frac64_z<<=1;
    }

    //default dot is between bit 271 and 272, extreme left bit is bit 0. Last right bit is bit 512.
    //Minpos is 120 position to the right of binary point (dot)
    //Scale = 2^es * k + e  => 2k + e
    let first_pos = 271 - ((k_a<<2) as i32) - exp_a;

    //Moving in chunk of 64. If it is in first chunk, a part might be in the chunk right to it. Simply have to handle that.
    let mut u_z2: [u64; 8] = [0; 8];
    for i in 0usize..8 {
        if first_pos < ((i+1)*64) as i32 {
            //Need to check how much of the fraction is in the next 64 bits
            let shift_right = (first_pos - ((i*64) as i32)) as i16;
            u_z2[i] = frac64_z >> shift_right;
            if (i != 7) && (shift_right != 0) {
                u_z2[i+1] = frac64_z << (64 - shift_right);
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
    for (i, (u, (u1, u2))) in (0..8).rev().zip(u_z.iter_mut().rev()
                                 .zip(u_z1.iter().rev().zip(u_z2.iter().rev())))
    {
        let b1 = (*u1 & 0x1) != 0;
        let b2 = (*u2 & 0x1) != 0;
        if i==7 {
            let rcarryb = b1 & b2;
            *u = (*u1>>1) + (*u2>>1) + (rcarryb as u64);
            rcarry_z = *u>>63 != 0;
            *u = (*u<<1) | ((b1^b2) as u64);
        }
        else{
            let rcarryb3 =  (b1 as i8) + (b2 as i8) + (rcarry_z as i8);
            *u = (*u1>>1) + (*u2>>1) + ((rcarryb3>>1) as u64);
            rcarry_z = *u>>63 != 0;
            *u = (*u<<1) | ((rcarryb3 & 0x1) as u64);
        }
    }

    //Exception handling
    let q_z = Q32E2::from_bits(u_z);
    if q_z.is_nan() {
        Q32E2::new(0, 0, 0, 0, 0, 0, 0, 0)
    } else {
        q_z
    }
}


fn q32_fdp_sub(q: Q32E2, p_a: P32E2, p_b: P32E2) -> Q32E2 {
    let u_z1 = q.to_bits();

    let mut ui_a = p_a.to_bits();
    let mut ui_b = p_b.to_bits();

    if q.is_nan() || p_a.is_nan() || p_b.is_nan() {
        return Q32E2::new(-0x8000_0000_0000_0000, 0, 0, 0, 0, 0, 0, 0);
    } else if (ui_a == 0) || (ui_b == 0) {
        return q;
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

    let (mut k_a, mut exp_a, frac_a) = P32E2::separate_bits(ui_a);

    let (k_b, exp_b, frac_b) = P32E2::separate_bits(ui_b);
    k_a += k_b;
    exp_a += exp_b;
    let mut frac64_z = (frac_a as u64) * (frac_b as u64);

    if exp_a>3 {
        k_a += 1;
        exp_a&=0x3; // -=4
    }
    //Will align frac64_z such that hidden bit is the first bit on the left.
    let rcarry = (frac64_z>>63) != 0;//1st bit of frac64_z
    if rcarry {
        exp_a += 1;
        if exp_a>3 {
            k_a += 1;
            exp_a&=0x3;
        }
        //frac64_z>>=1;
    }
    else {
        frac64_z<<=1;
    }

    //default dot is between bit 271 and 272, extreme left bit is bit 0. Last right bit is bit 512.
    //Minpos is 120 position to the right of binary point (dot)
    //Scale = 2^es * k + e  => 2k + e
    let first_pos = 271 - ((k_a<<2) as i32) - exp_a;

    //Moving in chunk of 64. If it is in first chunk, a part might be in the chunk right to it. Simply have to handle that.
    let mut u_z2: [u64; 8] = [0; 8];
    for i in 0usize..8 {
        if first_pos < ((i+1)*64) as i32 {
            //Need to check how much of the fraction is in the next 64 bits
            let shift_right = (first_pos - ((i*64) as i32)) as i16;
            u_z2[i] = frac64_z >> shift_right;
            if (i != 7) && (shift_right != 0) {
                u_z2[i+1] = frac64_z << (64 - shift_right);
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
    for (i, (u, (u1, u2))) in (0..8).rev().zip(u_z.iter_mut().rev()
                                 .zip(u_z1.iter().rev().zip(u_z2.iter().rev())))
    {
        let b1 = (*u1 & 0x1) != 0;
        let b2 = (*u2 & 0x1) != 0;
        if i==7 {
            let rcarryb = b1 & b2;
            *u = (*u1>>1) + (*u2>>1) + (rcarryb as u64);
            rcarry_z = *u>>63 != 0;
            *u = (*u<<1) | ((b1^b2) as u64);
        }
        else{
            let rcarryb3 =  (b1 as i8) + (b2 as i8) + (rcarry_z as i8);
            *u = (*u1>>1) + (*u2>>1) + ((rcarryb3>>1) as u64);
            rcarry_z = *u>>63 != 0;
            *u = (*u<<1) | ((rcarryb3 & 0x1) as u64);
        }
    }

    //Exception handling
    let q_z = Q32E2::from_bits(u_z);
    if q_z.is_nan() {
        Q32E2::new(0, 0, 0, 0, 0, 0, 0, 0)
    } else {
        q_z
    }
}
