use super::{P16E1, Q16E1};
use crate::{MulAddType, WithSign};

const HALF: P16E1 = P16E1::new(0x_3000);
const TWO: P16E1 = P16E1::new(0x_5000);

impl crate::MathConsts for P16E1 {
    const E: Self = Self::new(0x_55bf);
    const FRAC_1_PI: Self = Self::new(0x_245f);
    const FRAC_1_SQRT_2: Self = Self::new(0x_36a1);
    const FRAC_2_PI: Self = Self::new(0x_345f);
    const FRAC_2_SQRT_PI: Self = Self::new(0x_420e);
    const FRAC_PI_2: Self = Self::new(0x_4922);
    const FRAC_PI_3: Self = Self::new(0x_40c1);
    const FRAC_PI_4: Self = Self::new(0x_3922);
    const FRAC_PI_6: Self = Self::new(0x_30c1);
    const FRAC_PI_8: Self = Self::new(0x_2922);
    const LN_10: Self = Self::new(0x_526c);
    const LN_2: Self = Self::new(0x_362e);
    const LOG10_E: Self = Self::new(0x_2bcb);
    const LOG2_E: Self = Self::new(0x_2344);
    const PI: Self = Self::new(0x_5922);
    const SQRT_2: Self = Self::new(0x_46a1);
    const LOG2_10: Self = Self::new(0x_5a93);
    const LOG10_2: Self = Self::new(0x_2344);
}

impl P16E1 {
    #[inline]
    pub fn mul_add(self, b: Self, c: Self) -> Self {
        let ui_a = self.to_bits();
        let ui_b = b.to_bits();
        let ui_c = c.to_bits();
        mul_add(ui_a, ui_b, ui_c, crate::MulAddType::Add)
    }
    #[inline]
    pub fn floor(self) -> Self {
        floor(self)
    }
    #[inline]
    pub fn ceil(self) -> Self {
        ceil(self)
    }
    #[inline]
    pub fn round(self) -> Self {
        round(self)
    }
    // TODO: optimize
    #[inline]
    pub fn trunc(self) -> Self {
        if self > Self::ZERO {
            self.floor()
        } else {
            self.ceil()
        }
    }
    #[inline]
    pub fn fract(self) -> Self {
        self - self.trunc()
    }
    #[inline]
    pub fn div_euclid(self, rhs: Self) -> Self {
        let q = (self / rhs).trunc();
        if self % rhs < Self::ZERO {
            return if rhs > Self::ZERO {
                q - Self::ONE
            } else {
                q + Self::ONE
            };
        }
        q
    }
    #[inline]
    pub fn rem_euclid(self, rhs: Self) -> Self {
        let r = self % rhs;
        if r < Self::ZERO {
            r + rhs.abs()
        } else {
            r
        }
    }
    #[inline]
    pub fn powi(self, _n: i32) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn powf(self, _n: Self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn sqrt(self) -> Self {
        sqrt(self)
    }
    #[inline]
    pub fn exp(self) -> Self {
        exp(self)
    }
    #[inline]
    pub fn exp2(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn ln(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn log(self, _base: Self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn log2(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn log10(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn cbrt(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn hypot(self, _other: Self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn sin(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn cos(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn tan(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn asin(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn acos(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn atan(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn atan2(self, _other: Self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn sin_cos(self) -> (Self, Self) {
        (self.sin(), self.cos())
    }
    #[inline]
    pub fn exp_m1(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn ln_1p(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn sinh(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn cosh(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn tanh(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn asinh(self) -> Self {
        if self.is_nar() {
            self
        } else {
            (self + ((self * self) + Self::ONE).sqrt()).ln()
        }
    }
    #[inline]
    pub fn acosh(self) -> Self {
        match self {
            x if x < Self::ONE => Self::NAR,
            x => (x + ((x * x) - Self::ONE).sqrt()).ln(),
        }
    }
    #[inline]
    pub fn atanh(self) -> Self {
        HALF * ((TWO * self) / (Self::ONE - self)).ln_1p()
    }
}

//softposit_mulAdd_subC => (ui_a*ui_b)-ui_c
//softposit_mulAdd_subProd => ui_c - (ui_a*ui_b)
//Default is always op==0
#[allow(clippy::cognitive_complexity)]
fn mul_add(mut ui_a: u16, mut ui_b: u16, mut ui_c: u16, op: MulAddType) -> P16E1 {
    let mut bits_more = false;

    //NaR
    if (ui_a == 0x8000) || (ui_b == 0x8000) || (ui_c == 0x8000) {
        return P16E1::NAR;
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

    let rcarry = (frac32_z & 0x8000_0000) != 0; //1st bit of frac32_z
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
        let (k_c, exp_c, frac_c) = P16E1::separate_bits(ui_c);
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
                return P16E1::ZERO;
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

        let rcarry = (frac32_z & 0x8000_0000) != 0; //first left bit
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

    let mut ui_a = p_a.to_bits();
    let sign = ui_a > 0x8000;

    // sign is True if p_a > NaR.
    if sign {
        ui_a = ui_a.wrapping_neg() // A is now |A|.
    };
    let u_a = if ui_a <= 0x3000 {
        // 0 <= |p_a| <= 1/2 rounds to zero.
        return P16E1::ZERO;
    } else if ui_a < 0x4800 {
        // 1/2 < x < 3/2 rounds to 1.
        0x4000
    } else if ui_a <= 0x5400 {
        // 3/2 <= x <= 5/2 rounds to 2.
        0x5000
    } else if ui_a >= 0x7C00 {
        // If |A| is 256 or greater, leave it unchanged.
        return p_a; // This also takes care of the NaR case, 0x8000.
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
        ui_a
    };
    P16E1::from_bits(u_a.with_sign(sign))
}

fn sqrt(p_a: P16E1) -> P16E1 {
    let mut ui_a = p_a.to_bits();

    // If sign bit is set, return NaR.
    if (ui_a & 0x_8000) != 0 {
        return P16E1::NAR;
    }
    // If the argument is zero, return zero.
    if ui_a == 0 {
        return P16E1::ZERO;
    }
    // Compute the square root. Here, k_z is the net power-of-2 scaling of the result.
    // Decode the regime and exponent bit; scale the input to be in the range 1 to 4:
    let mut k_z: i16;
    if (ui_a >> 14) != 0 {
        k_z = -1;
        while (ui_a & 0x4000) != 0 {
            k_z += 1;
            ui_a <<= 1;
        }
    } else {
        k_z = 0;
        while (ui_a & 0x4000) == 0 {
            k_z -= 1;
            ui_a <<= 1;
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
    let mut ui_z: u16 = if k_z < 0 {
        shift = ((-1 - k_z) >> 1) as u16;
        0x2000 >> shift
    } else {
        shift = (k_z >> 1) as u16;
        0x7fff - (0x7FFF >> (shift + 1))
    };
    // Set the exponent bit in the answer, if it is nonzero:
    if (k_z & 1) != 0 {
        ui_z |= 0x1000 >> shift;
    }

    // Right-shift fraction bits, accounting for 1 <= a < 2 versus 2 <= a < 4:
    frac_z >>= exp_a + shift;

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

fn ceil(p_a: P16E1) -> P16E1 {
    let mut mask = 0x2000_u16;
    let mut scale = 0_u16;

    let mut ui_a = p_a.to_bits();
    let sign = ui_a > 0x8000;

    // sign is True if p_a > NaR.
    if sign {
        ui_a = ui_a.wrapping_neg() // A is now |A|.
    };

    let u_a = if ui_a == 0 {
        return p_a;
    } else if ui_a <= 0x4000 {
        // 0 <= |pA| < 1 ceiling to zero.(if not negative and whole number)
        if sign && (ui_a != 0x4000) {
            0x0
        } else {
            0x4000
        }
    } else if ui_a <= 0x5000 {
        // 1 <= x < 2 ceiling to 1 (if not negative and whole number)
        if sign && (ui_a != 0x5000) {
            0x4000
        } else {
            0x5000
        }
    } else if ui_a <= 0x5800 {
        // 2 <= x < 3 ceiling to 2 (if not negative and whole number)
        if sign & (ui_a != 0x5800) {
            0x5000
        } else {
            0x5800
        }
    } else if ui_a >= 0x7C00 {
        // If |A| is 256 or greater, leave it unchanged.
        return p_a; // This also takes care of the NaR case, 0x8000.
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

        mask >>= 1;
        let mut tmp = ui_a & mask;
        let bit_n_plus_one = tmp; // "True" if nonzero.
        ui_a ^= tmp; // Erase the bit, if it was set.
        tmp = ui_a & (mask - 1); // tmp has any remaining bits = bitsMore
        ui_a ^= tmp; // Erase those bits, if any were set.

        if !sign && (bit_n_plus_one | tmp) != 0 {
            ui_a += mask << 1;
        }
        ui_a
    };
    P16E1::from_bits(u_a.with_sign(sign))
}

fn floor(p_a: P16E1) -> P16E1 {
    let mut mask = 0x2000_u16;
    let mut scale = 0_u16;

    let mut ui_a = p_a.to_bits();
    let sign = ui_a > 0x8000;

    // sign is True if p_a > NaR.
    if sign {
        ui_a = ui_a.wrapping_neg() // A is now |A|.
    };

    let u_a = if ui_a < 0x4000 {
        // 0 <= |pA| < 1 floor to zero.(if not negative and whole number)
        if sign && (ui_a != 0x0) {
            0x4000
        } else {
            0x0
        }
    } else if ui_a < 0x5000 {
        // 1 <= x < 2 floor to 1 (if not negative and whole number)
        if sign && (ui_a != 0x4000) {
            0x5000
        } else {
            0x4000
        }
    } else if ui_a < 0x5800 {
        // 2 <= x < 3 floor to 2 (if not negative and whole number)
        if sign & (ui_a != 0x5000) {
            0x5800
        } else {
            0x5000
        }
    } else if ui_a >= 0x7C00 {
        // If |A| is 256 or greater, leave it unchanged.
        return p_a; // This also takes care of the NaR case, 0x8000.
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

        mask >>= 1;
        let mut tmp = ui_a & mask;
        let bit_n_plus_one = tmp; // "True" if nonzero.
        ui_a ^= tmp; // Erase the bit, if it was set.
        tmp = ui_a & (mask - 1); // tmp has any remaining bits = bitsMore
        ui_a ^= tmp; // Erase those bits, if any were set.

        if sign && ((bit_n_plus_one | tmp) != 0) {
            ui_a += mask << 1;
        }
        ui_a
    };
    P16E1::from_bits(u_a.with_sign(sign))
}

fn exp(p_a: P16E1) -> P16E1 {
    // Use names for commonly-used posits.
    const LARGE: P16E1 = P16E1::new(0x7FFE);
    const SMALL: P16E1 = P16E1::new(0x0002);

    // If input is NaR, return NaR.
    if p_a.is_nar() {
        P16E1::NAR
    }
    // If input is large enough that result is maxpos, return maxpos.
    else if P16E1::new(0x70AE) <= p_a {
        P16E1::MAX
    }
    // If input is negative enough that result is minpos, return minpos.
    else if p_a <= P16E1::new(-0x_70ae) {
        P16E1::MIN_POSITIVE
    }
    // This range rounds to maxpos/4, the next-to-largest posit16.
    else if P16E1::new(0x706F) < p_a {
        LARGE
    }
    // This range rounds to minpos*4, the next-to-smallest posit16.
    else if p_a < P16E1::new(-0x_7067) {
        SMALL
    } else {
        // Scale input by 1/log(2) to trio precision, using the quire.
        let mut q = Q16E1::init();

        q += (
            p_a,
            (P16E1::new(0x4715), P16E1::new(0x0087), P16E1::new(0x000A)),
        );

        let mut p_n = q.to_posit().round();
        q -= p_n; // Reduce the argument range.
        let (t1, t2, t3) = q.into_three_posits();

        // Evaluate 6th-degree polynomial in t using Horner's rule.
        let mut q = Q16E1::init();
        q += (t1, P16E1::new(0x00D1));
        q += (P16E1::new(0x0A20), P16E1::new(0x0F28));
        let p1 = q.to_posit(); // c6 * t + c5, solo precision

        q.clear();
        q += (p1, t1);
        q += (P16E1::new(0x0A60), P16E1::new(0x28B8));
        let (p1, p2) = q.into_two_posits(); // (...) * t + c4, duo precision

        let mut q = Q16E1::init();
        q += (t1, (p1, p2));
        q += (p1, t2);
        q += (P16E1::new(0x0B80), P16E1::new(0x4E50));
        let (p1, p2) = q.into_two_posits(); // (...) * t + c3, duo precision

        let mut q = Q16E1::init();
        q += (t1, (p1, p2));
        q += (p1, t2);
        q += (P16E1::new(0x11F0), P16E1::new(0x58C1));
        let (p1, p2, p3) = q.into_three_posits(); // (...) * t + c2, trio precision

        let mut q = Q16E1::init();
        q += (t1, (p1, p2, p3));
        q += (p1, t2);
        q += (P16E1::new(0x2D78), P16E1::new(0x4816));
        q += (P16E1::new(0x0180), P16E1::new(0x0180));
        let (p1, p2, p3) = q.into_three_posits(); // c1 term, trio precision

        let mut q = Q16E1::init();
        q += (t1, (p1, p2, p3));
        q += (p1, t2);
        q += (p1, t3);
        q += P16E1::ONE; // (...) * t + c0, (where c0 = 1).
        let (p1, p2, p3) = q.into_three_posits(); // polynomial for exp(x), trio precision

        // Convert `p_n` to an integer and use to compute `2` to the power `n`.
        let mut n = i32::from(p_n);

        n = if n < 0 {
            ((n & 0x1) | 0x0002) << (12 - ((-n) >> 1))
        } else {
            0x7FFF & (((n & 0x1) | 0x7FFC) << (12 - (n >> 1)))
        };
        p_n = P16E1::new(n as i16);

        // Scale the result by that power.
        let mut q = Q16E1::init();
        q += (p_n, (p1, p2, p3)); // trio precision; round it and we're done.

        q.to_posit()
    }
}

#[test]
fn test_mul_add() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS16 {
        let p_a: P16E1 = rng.gen();
        let p_b: P16E1 = rng.gen();
        let p_c: P16E1 = rng.gen();
        let f_a = f64::from(p_a);
        let f_b = f64::from(p_b);
        let f_c = f64::from(p_c);
        let p = p_a.mul_add(p_b, p_c);
        let f = f_a.mul_add(f_b, f_c);
        assert_eq!(p, P16E1::from(f));
    }
}

#[test]
fn test_sqrt() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS16 {
        let p_a: P16E1 = rng.gen();
        let f_a = f64::from(p_a);
        let p = p_a.sqrt();
        let f = f_a.sqrt();
        assert_eq!(p, P16E1::from(f));
    }
}

#[test]
fn test_round() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS16 {
        let p_a: P16E1 = rng.gen();
        let f_a = f64::from(p_a);
        let p = p_a.round();
        let f = f_a.round();
        if (f - f_a).abs() == 0.5 {
            continue;
        }
        assert_eq!(p, P16E1::from(f));
    }
}
