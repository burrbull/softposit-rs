use super::*;
use crate::{MulAddType, WithSign};

impl P16E1 {
    #[inline]
    pub fn mul_add(self, b: Self, c: Self) -> Self {
        let ui_a = self.to_bits();
        let ui_b = b.to_bits();
        let ui_c = c.to_bits();
        mul_add(ui_a, ui_b, ui_c, crate::MulAddType::Add)
    }
    #[inline]
    pub fn round(self) -> Self {
        round(self)
    }
    #[inline]
    pub fn sqrt(self) -> Self {
        sqrt(self)
    }
}

impl Q16E1 {
    #[inline]
    pub fn fdp_add(self, p_a: P16E1, p_b: P16E1) -> Self {
        q16_fdp_add(self, p_a, p_b)
    }
    #[inline]
    pub fn fdp_sub(self, p_a: P16E1, p_b: P16E1) -> Self {
        q16_fdp_sub(self, p_a, p_b)
    }
}

//softposit_mulAdd_subC => (ui_a*ui_b)-ui_c
//softposit_mulAdd_subProd => ui_c - (ui_a*ui_b)
//Default is always op==0
fn mul_add(mut ui_a: u16, mut ui_b: u16, mut ui_c: u16, op: MulAddType) -> P16E1 {
    let mut bits_more = false;

    //NaR
    if (ui_a == 0x8000) || (ui_b == 0x8000) || (ui_c == 0x8000) {
        return INFINITY;
    } else if (ui_a == 0) || (ui_b == 0) {
        return match op {
            MulAddType::SubC => P16E1::from_bits(ui_c.wrapping_neg()),
            _ => P16E1::from_bits(ui_c),
        };
    }

    let sign_a = P16E1::sign_ui(ui_a);
    let sign_b = P16E1::sign_ui(ui_b);
    let sign_c = P16E1::sign_ui(ui_c); //^ (op == softposit_mulAdd_subC);
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

    let (mut k_a, tmp) = P16E1::separate_bits_tmp(ui_a);
    let mut exp_a = (tmp >> 14) as i8;
    let frac_a = 0x8000 | (tmp << 1); //use first bit here for hidden bit to get more bits

    let (k_b, tmp) = P16E1::separate_bits_tmp(ui_b);
    k_a += k_b;

    exp_a += (tmp >> 14) as i8;
    let mut frac32_z = (frac_a as u32) * ((0x8000 | (tmp << 1)) as u32); // first bit hidden bit

    if exp_a > 1 {
        k_a += 1;
        exp_a ^= 0x2;
    }

    let rcarry = (frac32_z >> 31) != 0; //1st bit of frac32_z
    if rcarry {
        if exp_a != 0 {
            k_a += 1;
        }
        exp_a ^= 1;
        frac32_z >>= 1;
    }

    let mut k_z: i8;
    let mut exp_z: i8;
    //Add
    if ui_c != 0 {
        let (k_c, exp_c, frac_c) = P16E1::separate_bits(ui_a);
        let mut frac32_c = (frac_c as u32) << 16;

        let mut shift_right: i16 = (((k_a - k_c) as i16) << 1) + ((exp_a - exp_c) as i16); //actually this is the scale

        exp_z = if shift_right < 0 {
            // |ui_c| > |Prod Z|
            if shift_right <= -31 {
                bits_more = true;
                frac32_z = 0;
                shift_right = 0;
            } else if (frac32_z << (32 + shift_right)/*&0xFFFF_FFFF*/) != 0 {
                bits_more = true;
            }
            if sign_z == sign_c {
                frac32_z = frac32_c + (frac32_z >> -shift_right);
            } else {
                //different signs
                frac32_z = frac32_c - (frac32_z >> -shift_right);
                sign_z = sign_c;
                if bits_more {
                    frac32_z -= 1;
                }
            }
            k_z = k_c;
            exp_c
        } else if shift_right > 0 {
            // |ui_c| < |Prod|
            //if (frac32_c&((1<<shift_right)-1)) bits_more = 1;
            if shift_right >= 31 {
                bits_more = true;
                frac32_c = 0;
                shift_right = 0;
            } else if (frac32_c << (32 - shift_right)) != 0 {
                bits_more = true;
            }
            if sign_z == sign_c {
                frac32_z += frac32_c >> shift_right;
            } else {
                frac32_z -= frac32_c >> shift_right;
                if bits_more {
                    frac32_z -= 1;
                }
            }
            k_z = k_a;
            exp_a
        } else {
            if (frac32_c == frac32_z) && (sign_z != sign_c) {
                //check if same number
                return ZERO;
            } else if sign_z == sign_c {
                frac32_z += frac32_c;
            } else if frac32_z < frac32_c {
                frac32_z = frac32_c - frac32_z;
                sign_z = sign_c;
            } else {
                frac32_z -= frac32_c;
            }
            k_z = k_a; // actually can be k_c too, no diff
            exp_a //same here
        };

        let rcarry = (0x8000_0000 & frac32_z) != 0; //first left bit
        if rcarry {
            if exp_z != 0 {
                k_z += 1;
            }
            exp_z ^= 1;
            if (frac32_z & 0x1) != 0 {
                bits_more = true;
            }
            frac32_z >>= 1 /*&0x7FFF_FFFF*/;
        } else {
            //for subtract cases
            if frac32_z != 0 {
                while (frac32_z >> 29) == 0 {
                    k_z -= 1;
                    frac32_z <<= 2;
                }
            }
            let ecarry = ((0x4000_0000 & frac32_z) >> 30) != 0;

            if !ecarry {
                if exp_z == 0 {
                    k_z -= 1;
                }
                exp_z ^= 1;
                frac32_z <<= 1;
            }
        }
    } else {
        k_z = k_a;
        exp_z = exp_a;
    }

    let (regime, reg_sz, reg_z) = P16E1::calculate_regime(k_z);

    let u_z = if reg_z > 14 {
        //max or min pos. exp and frac does not matter.
        if reg_sz {
            0x7FFF
        } else {
            0x1
        }
    } else {
        //remove hidden bits
        frac32_z &= 0x3FFF_FFFF;
        let mut frac_z = (frac32_z >> (reg_z + 17)) as u16;

        let mut bit_n_plus_one = false;
        if reg_z != 14 {
            bit_n_plus_one = ((frac32_z >> reg_z) & 0x10000) != 0;
        } else if frac32_z > 0 {
            frac_z = 0;
            bits_more = true;
        }
        if (reg_z == 14) && (exp_z != 0) {
            bit_n_plus_one = true;
        }
        let mut u_z = P16E1::pack_to_ui(regime, reg_z, exp_z as u16, frac_z);
        if bit_n_plus_one {
            if (frac32_z << (16 - reg_z)) != 0 {
                bits_more = true;
            }
            u_z += (u_z & 1) | (bits_more as u16);
        }
        u_z
    };

    P16E1::from_bits(u_z.with_sign(sign_z))
}

fn round(p_a: P16E1) -> P16E1 {
    let mut mask = 0x2000_u16;
    let mut scale = 0_u16;

    let mut u_a = p_a.to_bits();
    let mut ui_a = u_a; // Copy of the input.
    let sign = ui_a > 0x8000;

    // sign is True if p_a > NaR.
    if sign {
        ui_a = ui_a.wrapping_neg() // A is now |A|.
    };
    if ui_a <= 0x3000 {
        // 0 <= |p_a| <= 1/2 rounds to zero.
        return ZERO;
    } else if ui_a < 0x4800 {
        // 1/2 < x < 3/2 rounds to 1.
        u_a = 0x4000;
    } else if ui_a <= 0x5400 {
        // 3/2 <= x <= 5/2 rounds to 2.
        u_a = 0x5000;
    } else if ui_a >= 0x7C00 {
        // If |A| is 256 or greater, leave it unchanged.
        return P16E1::from_bits(u_a); // This also takes care of the NaR case, 0x8000.
    } else {
        // 34% of the cases, we have to decode the posit.
        while (mask & ui_a) != 0 {
            // Increment scale by 2 for each regime sign bit.
            scale += 2; // Regime sign bit is always 1 in this range.
            mask >>= 1; // Move the mask right, to the next bit.
        }
        mask >>= 1; // Skip over termination bit.
        if (mask & ui_a) != 0 {
            scale += 1; // If exponent is 1, increment the scale.
        }
        mask >>= scale; // Point to the last bit of the integer part.
        let bit_last = (ui_a & mask) != 0; // Extract the bit, without shifting it.

        mask >>= 1;
        let mut tmp = ui_a & mask;
        let bit_n_plus_one = tmp != 0; // "True" if nonzero.
        ui_a ^= tmp; // Erase the bit, if it was set.
        tmp = ui_a & (mask - 1); // tmp has any remaining bits.
        ui_a ^= tmp; // Erase those bits, if any were set.

        if bit_n_plus_one {
            // logic for round to nearest, tie to even
            if (bit_last as u16 | tmp) != 0 {
                ui_a += mask << 1;
            }
        }
        u_a = ui_a;
    }
    P16E1::from_bits(u_a.with_sign(sign))
}

fn sqrt(p_a: P16E1) -> P16E1 {
    let mut ui_a = p_a.to_bits();

    // If sign bit is set, return NaR.
    if (ui_a >> 15) != 0 {
        return INFINITY;
    }
    // If the argument is zero, return zero.
    if ui_a == 0 {
        return ZERO;
    }
    // Compute the square root. Here, k_z is the net power-of-2 scaling of the result.
    // Decode the regime and exponent bit; scale the input to be in the range 1 to 4:
    let mut k_z: i16;
    if (ui_a >> 14) != 0 {
        k_z = -1;
        while (ui_a & 0x4000) != 0 {
            k_z += 1;
            ui_a= ui_a<<1 /* & 0xFFFF*/;
        }
    } else {
        k_z = 0;
        while (ui_a & 0x4000) == 0 {
            k_z -= 1;
            ui_a= ui_a<<1 /* & 0xFFFF*/;
        }
    }
    ui_a &= 0x3fff;
    let exp_a = 1 - (ui_a >> 13);
    let frac_a = (ui_a | 0x2000) >> 1;

    // Use table look-up of first four bits for piecewise linear approx. of 1/sqrt:
    let index = (((frac_a >> 8) & 0xE) + exp_a) as usize;

    let r0 = (crate::APPROX_RECIP_SQRT0[index] as u32
        - (((crate::APPROX_RECIP_SQRT1[index] as u32) * ((frac_a & 0x1FF) as u32)) >> 13))
        as u16 as u32;
    // Use Newton-Raphson refinement to get more accuracy for 1/sqrt:
    let mut e_sqr_r0 = (r0 * r0) >> 1;

    if exp_a != 0 {
        e_sqr_r0 >>= 1;
    }
    let sigma0 = 0xFFFF ^ ((0xFFFF & (((e_sqr_r0 as u64) * (frac_a as u64)) >> 18)) as u16); //~(u16) ((e_sqr_r0 * frac_a) >> 18);
    let recip_sqrt = (r0 << 2) + ((r0 * (sigma0 as u32)) >> 23);

    // We need 17 bits of accuracy for posit16 square root approximation.
    // Multiplying 16 bits and 18 bits needs 64-bit scratch before the right shift:
    let mut frac_z = (((frac_a as u64) * (recip_sqrt as u64)) >> 13) as u32;

    // Figure out the regime and the resulting right shift of the fraction:
    let shift: u16;
    let mut ui_z: u16;
    if k_z < 0 {
        shift = ((-1 - k_z) >> 1) as u16;
        ui_z = 0x2000 >> shift;
    } else {
        shift = (k_z >> 1) as u16;
        ui_z = 0x7fff - (0x7FFF >> (shift + 1));
    }
    // Set the exponent bit in the answer, if it is nonzero:
    if (k_z & 1) != 0 {
        ui_z |= 0x1000 >> shift;
    }

    // Right-shift fraction bits, accounting for 1 <= a < 2 versus 2 <= a < 4:
    frac_z = frac_z >> (exp_a + shift);

    // Trick for eliminating off-by-one cases that only uses one multiply:
    frac_z += 1;
    if (frac_z & 7) == 0 {
        let shifted_frac_z = frac_z >> 1;
        let neg_rem = (shifted_frac_z * shifted_frac_z) & 0x3_FFFF;
        if (neg_rem & 0x2_0000) != 0 {
            frac_z |= 1;
        } else if neg_rem != 0 {
            frac_z -= 1;
        }
    }
    // Strip off the hidden bit and round-to-nearest using last 4 bits.
    frac_z -= 0x1_0000 >> shift;
    let bit_n_plus_one = ((frac_z >> 3) & 1) != 0;
    if bit_n_plus_one && ((((frac_z >> 4) & 1) | (frac_z & 7)) != 0) {
        frac_z += 0x10;
    }
    // Assemble the result and return it.
    P16E1::from_bits(ui_z | ((frac_z >> 4) as u16))
}

fn q16_fdp_add(q: Q16E1, p_a: P16E1, p_b: P16E1) -> Q16E1 {
    let u_z1 = q.to_bits();

    let mut ui_a = p_a.to_bits();
    let mut ui_b = p_b.to_bits();

    if q.is_nan() || p_a.is_nan() || p_b.is_nan() {
        return Q16E1::new(-0x8000_0000_0000_0000, 0);
    } else if (ui_a == 0) || (ui_b == 0) {
        return q;
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
    let first_pos = 71 - (k_a << 1) as i16 - (exp_a as i16);

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
            u_z2[1] = (frac32_z as u64) << (64 - shift_right);
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

    let rcarry_z = (u_z[1] >> 63) != 0;

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
    if q_z.is_nan() {
        Q16E1::new(0, 0)
    } else {
        q_z
    }
}

fn q16_fdp_sub(q: Q16E1, p_a: P16E1, p_b: P16E1) -> Q16E1 {
    let u_z1 = q.to_bits();

    let mut ui_a = p_a.to_bits();
    let mut ui_b = p_b.to_bits();

    if q.is_nan() || p_a.is_nan() || p_b.is_nan() {
        return Q16E1::new(-0x8000_0000_0000_0000, 0);
    } else if (ui_a == 0) || (ui_b == 0) {
        return q;
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
    let first_pos = 71 - (k_a << 1) as i16 - (exp_a as i16);

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
            u_z2[1] = (frac32_z as u64) << (64 - shift_right);
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

    let rcarry_z = (u_z[1] >> 63) != 0;

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
    if q_z.is_nan() {
        Q16E1::new(0, 0)
    } else {
        q_z
    }
}
