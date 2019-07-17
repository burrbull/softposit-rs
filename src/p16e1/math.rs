use super::P16E1;
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
        exp2(self)
    }
    #[inline]
    pub fn ln(self) -> Self {
        ln(self)
    }
    #[inline]
    pub fn log(self, _base: Self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn log2(self) -> Self {
        log2(self)
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
    #[inline]
    pub fn sin_pi(self) -> Self {
        sin_pi(self)
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
    let ui_a = p_a.to_bits();

    let mut f = ui_a as u64;

    // Calculate the exponential for given posit pA
    if ui_a < 28846 {
        // result does not round up to maxpos
        if ui_a < 192 {
            // small positive values that round to 1
            return P16E1::ONE;
        }

        let mut s: i32;
        if (f & 0x4000) != 0 {
            // decode regime
            s = 8;
            while (f & 0x2000) != 0 {
                f <<= 1;
                s += 2;
            }
        } else {
            s = 6;
            while (f & 0x2000) == 0 {
                f <<= 1;
                s -= 2;
            }
        }

        if (f & 0x1000) != 0 {
            s += 1; // decode exponent
        }
        f = (f & 0x0FFF) | 0x1000; // decode fraction
        f = ((if s < 0 { f >> -s } else { f << s }) * 48_408_813) >> 20;
        let mut s = f >> 25; // s now stores floor(x)
        f = poly::exp(f & 0x_01FF_FFFF); // 37 fraction bits of exp(x)
        let mut bit = (s & 1) << 37; // exponent bit of exp(x)
        s >>= 1; // regime length of exp(x)
        f |= ((0x_0100_0000_0000 << s) - 0x_0080_0000_0000) | bit;

        bit = 1_u64 << (24 + s); // location of bit n-plus-1
        if ((f & bit) != 0) && (((f & (bit - 1)) != 0) || ((f & (bit << 1)) != 0)) {
            f += bit;
        }
        return P16E1::from_bits((f >> (25 + s)) as u16); // return rounded exp(x) as posit
    } else if ui_a > 36690 {
        // result does not round up to minpos
        if ui_a > 65407 {
            // small negative values that round to 1
            return P16E1::ONE;
        }

        let mut s: i32;
        if (f & 0x4000) != 0 {
            // decode regime
            s = 7;
            while (f & 0x2000) != 0 {
                f <<= 1;
                s -= 2;
            }
        } else {
            s = 9;
            while (f & 0x2000) == 0 {
                f <<= 1;
                s += 2;
            }
        }

        if (f & 0x1000) != 0 {
            s -= 1; // decode exponent
        }
        f = (f & 0x0FFF) | 0x_01FF_E000; // decode fraction
        f = if s < 0 {
            (f >> -s) | (0x_0200_0000 - (1 << (13 + s)))
        } else {
            (f << s) & 0x_01ff_ffff
        };
        f = (0x_0004_0000_0000_0000 - ((0x_0200_0000 - f) * 48_408_813)) >> 20;

        let mut s = (f >> 25).wrapping_sub(32); // s now stores floor(x)
        f = poly::exp(f & 0x_01FF_FFFF); // 37 fraction bits of exp(x)
        let mut bit = (s & 1) << 37; // exponent bit of exp(x)
        s = ((-1 - (s as i64)) >> 1) as u64;
        f |= 0x_0040_0000_0000 | bit; // Install regime end bit

        bit = 1_u64 << (24 + s); // location of bit n-plus-1
        if ((f & bit) != 0) && (((f & (bit - 1)) != 0) || ((f & (bit << 1)) != 0)) {
            f += bit;
        }
        return P16E1::from_bits((f >> (25 + s)) as u16); // return rounded exp(x) as posit
    }

    // Section for exception cases
    if ui_a < 0x8000 {
        P16E1::MAX // return maxpos
    } else if ui_a > 0x8000 {
        P16E1::MIN_POSITIVE // return minpos
    } else {
        P16E1::NAR // return NaR
    }
}

fn exp2(p_a: P16E1) -> P16E1 {
    let ui_a = p_a.to_bits();

    let mut f = ui_a as u64;

    // Calculate the exponential for given posit pA
    if ui_a < 29377 {
        // result does not round up to maxpos

        if ui_a < 221 {
            // cases that round down to 1.
            return P16E1::ONE;
        }

        let mut s: i32;
        if (f & 0x4000) != 0 {
            // decode regime
            s = 8;
            while (f & 0x2000) != 0 {
                f <<= 1;
                s += 2;
            }
        } else {
            s = 6;
            while (f & 0x2000) == 0 {
                f <<= 1;
                s -= 2;
            }
        }

        if (f & 0x1000) != 0 {
            s += 1; // decode exponent
        }
        f = (f & 0x0FFF) | 0x1000; // decode fraction
        f = if s < 0 { f >> -s } else { f << s };
        let mut s = f >> 20; // s now stores floor(x)
        f = poly::exp2(f & 0x_000F_FFFF); // fraction bits of exp2(x)
        let mut bit = (s & 1) << 26; // exponent bit of exp2(x)
        s >>= 1; // regime length of exp2(x)
        f |= ((0x_2000_0000_u64 << s) - 0x_1000_0000) | bit;

        bit = 1_u64 << (13 + s); // location of bit n-plus-1
        if ((f & bit) != 0) && (((f & (bit - 1)) != 0) || ((f & (bit << 1)) != 0)) {
            f += bit;
        }
        return P16E1::from_bits((f >> (14 + s)) as u16); // return rounded exp2(x) as posit
    } else if ui_a > 36159 {
        if ui_a > 65379 {
            // cases that round up to 1.
            return P16E1::ONE;
        }

        let mut s: i32;
        if (f & 0x4000) != 0 {
            // decode regime
            s = 7;
            while (f & 0x2000) != 0 {
                f <<= 1;
                s -= 2;
            }
        } else {
            s = 9;
            while (f & 0x2000) == 0 {
                f <<= 1;
                s += 2;
            }
        }

        if (f & 0x1000) != 0 {
            s -= 1; // decode exponent
        }
        f = (f & 0x0FFF) | 0x_01FF_E000; // decode fraction
        f = if s < 0 {
            (f >> -s) | (0x_0200_0000 - (1 << (13 + s)))
        } else {
            (f << s) & 0x_01ff_ffff
        };
        let mut s = (f >> 20).wrapping_sub(32); // s now stores floor(x)
        f = poly::exp2(f & 0x_000F_FFFF); // fraction bits of exp2(x)
        let mut bit = (s & 1) << 26; // exponent bit of exp2(x)
        s = ((-1 - (s as i64)) >> 1) as u64;
        f |= 0x_0800_0000 | bit; // Install regime end bit

        bit = 1_u64 << (13 + s); // location of bit n-plus-1
        if ((f & bit) != 0) && (((f & (bit - 1)) != 0) || ((f & (bit << 1)) != 0)) {
            f += bit;
        }
        return P16E1::from_bits((f >> (14 + s)) as u16); // return rounded exp2(x) as posit
    }

    // Section for exception cases
    if ui_a < 0x8000 {
        P16E1::MAX // return maxpos
    } else if ui_a > 0x8000 {
        P16E1::MIN_POSITIVE // return minpos
    } else {
        P16E1::NAR // return NaR
    }
}

fn ln(p_a: P16E1) -> P16E1 {
    let ui_a = p_a.to_bits();

    let mut f = ui_a as u64;

    if (f > 0x7FFF) || (f == 0) {
        // if input is 0, or greater than maxpos, return NaR
        return P16E1::NAR;
    }

    let mut s: i32;
    if (f & 0x4000) != 0 {
        // decode regime
        s = 0;
        while (f & 0x2000) != 0 {
            f <<= 1;
            s += 2;
        }
    } else {
        s = -2;
        while (f & 0x2000) == 0 {
            f <<= 1;
            s -= 2;
        }
    }

    if (f & 0x1000) != 0 {
        s += 1; // decode exponent
    }
    f &= 0x0FFF; // get 12-bit fraction, without hidden bit
    if f != 0 {
        f = poly::ln(f); // turn fraction into mantissa of logarithm
    }
    f |= ((if s < 0 { 64 + s } else { s }) as u64) << 30;

    f = if s < 0 {
        0x_0010_0000_0000 - (((0x_0010_0000_0000 - f) * 186_065_280) >> 28)
    } else {
        (f * 186_065_279) >> 28
    };

    let sign = (f & 0x_0008_0000_0000) != 0;
    if sign {
        f = 0x_0010_0000_0000 - f; // take absolute value of fixed-point result
    }
    if f < 0x_4000_0000 {
        // turn fixed-point into posit format
        if f != 0 {
            s = 34;
            while (f & 0x_2000_0000) == 0 {
                f <<= 1;
                s += 1;
            }
            f = (f ^ 0x_6000_0000) | (((1 ^ (s & 1)) as u64) << 29);
            s >>= 1;
            let bit = 1_u64 << (s - 1);
            if ((f & bit) != 0) && (((f & (bit - 1)) != 0) || ((f & (bit << 1)) != 0)) {
                f += bit;
            }
            f >>= s;
        }
    } else {
        s = 0;
        while f > 0x_7FFF_FFFF {
            f = (f & 1) | (f >> 1);
            s += 1;
        }
        f &= 0x_3FFF_FFFF;
        if (s & 1) != 0 {
            f |= 0x_4000_0000;
        }
        s >>= 1;
        f |= (0x_0002_0000_0000_u64 << s) - 0x_0001_0000_0000;
        let bit = 0x_0002_0000_u64 << s;
        if ((f & bit) != 0) && (((f & (bit - 1)) != 0) || ((f & (bit << 1)) != 0)) {
            f += bit;
        }
        f >>= s + 18;
    }
    if sign {
        f = 0x_0001_0000 - f; // restore sign
    }
    P16E1::from_bits(f as u16)
}

fn log2(p_a: P16E1) -> P16E1 {
    let ui_a = p_a.to_bits();

    let mut f = ui_a as u64;

    if (f > 0x7FFF) || (f == 0) {
        // if input is 0, or greater than maxpos, return NaR
        return P16E1::NAR;
    }

    let mut s: i32;
    if (f & 0x4000) != 0 {
        // decode regime
        s = 0;
        while (f & 0x2000) != 0 {
            f <<= 1;
            s += 2;
        }
    } else {
        s = -2;
        while (f & 0x2000) == 0 {
            f <<= 1;
            s -= 2;
        }
    }

    if (f & 0x1000) != 0 {
        s += 1; // decode exponent
    }
    f &= 0x0FFF; // get 12-bit fraction, without hidden bit
    if f != 0 {
        f = poly::log2(f); // turn fraction into mantissa of logarithm
    }
    f |= ((if s < 0 { 64 + s } else { s }) as u64) << 28;
    let sign = (f & 0x_0002_0000_0000) != 0;
    if sign {
        f = 0x_0004_0000_0000 - f; // take absolute value of fixed-point result
    }
    if f < 0x_1000_0000 {
        // turn fixed-point into posit format
        if f != 0 {
            s = 30;
            while (f & 0x_0800_0000) == 0 {
                f <<= 1;
                s += 1;
            }
            f = (f ^ 0x_1800_0000) | (((1 ^ (s & 1)) as u64) << 27);
            s >>= 1;
            let bit = 1_u64 << (s - 1);
            if ((f & bit) != 0) && (((f & (bit - 1)) != 0) || ((f & (bit << 1)) != 0)) {
                f += bit;
            }
            f >>= s;
        }
    } else {
        s = 0;
        while f > 0x_1FFF_FFFF {
            f = (f & 1) | (f >> 1);
            s += 1;
        }
        f &= 0x_0FFF_FFFF;
        if (s & 1) != 0 {
            f |= 0x_1000_0000;
        }
        s >>= 1;
        f |= (0x_8000_0000_u64 << s) - 0x_4000_0000;
        let bit = 0x8000_u64 << s;
        if ((f & bit) != 0) && (((f & (bit - 1)) != 0) || ((f & (bit << 1)) != 0)) {
            f += bit;
        }
        f >>= s + 16;
    }
    if sign {
        f = 0x_0001_0000 - f; // restore sign
    }
    P16E1::from_bits(f as u16)
}

fn sin_pi(p_a: P16E1) -> P16E1 {
    let ui_a = p_a.to_bits();

    let mut f = ui_a as u64;

    let mut sign = f & 0x8000;
    if sign != 0 {
        f = 0x10000 - f; // 2's complement if negative
    }
    if f > 31743 {
        // input value is an integer?
        if f == 0x8000 {
            return P16E1::NAR; // sinpi(NaR) is NaR
        } else {
            return P16E1::ZERO; // sinpi of an integer is zero
        }
    }
    if f == 0 {
        // sinpi(0) = 0
        return P16E1::ZERO;
    }
    let mut s: i32;
    if (f & 0x4000) != 0 {
        // decode regime
        s = 16;
        while (f & 0x2000) != 0 {
            f <<= 1;
            s += 2;
        }
    } else {
        s = 14;
        while (f & 0x2000) == 0 {
            f <<= 1;
            s -= 2;
        }
    }
    if (f & 0x1000) != 0 {
        s += 1; // decode exponent
    }
    f = (f & 0x0FFF) | 0x1000; // get 12-bit fraction and restore hidden bit
    f = if s < 0 { f >> -s } else { f << s };
    f &= 0x_1FFF_FFFF; // fixed-point with 28-bit fraction
    let mut s = f >> 27; // the quadrant is the multiple of 1/2
    f &= 0x_07FF_FFFF; // input value modulo 1/2
    if (s & 2) != 0 {
        sign ^= 0x8000; // quadrants 2 and 3 flip the sign
    }
    if f == 0 {
        return P16E1::from_bits(if (s & 1) != 0 {
            (sign as u16) | 0x4000
        } else {
            0
        });
    }
    if (s & 1) != 0 {
        f = 0x_0800_0000 - f;
    }
    f = poly::sin_pi(f);
    s = 1; // convert 28-bit fixed-point to a posit
    while (f & 0x_0800_0000) == 0 {
        f <<= 1;
        s += 1;
    }
    let bit = s & 1;
    s = (s >> 1) + 14 + bit;
    if bit == 0 {
        f &= 0x_07FF_FFFF; // encode exponent bit
    }
    f |= 0x_1000_0000; // encode regime termination bit
    let bit = 1_u64 << (s - 1);
    if ((f & bit) != 0) && (((f & (bit - 1)) != 0) || ((f & (bit << 1)) != 0)) {
        // round to nearest, tie to even
        f += bit;
    }
    f >>= s;
    P16E1::from_bits((if sign != 0 { 0x10000 - f } else { f }) as u16)
}

mod poly {
    #[inline]
    pub fn exp(f: u64) -> u64 {
        let mut s = (f * 7_529) >> 26;
        s = (f * (20_487 + s)) >> 20;
        s = (f * (0x_004F_8300 + s)) >> 24;
        s = (f * (0x_038C_C980 + s)) >> 20;
        s = (f * (0x_0001_EBFF_C800 + s)) >> 26;
        ((f * (0x_0002_C5C8_3600 + s)) >> 22) + 2048
    }

    #[inline]
    pub fn exp2(f: u64) -> u64 {
        let mut s = (f * (0x_9BA0_0000 + (f * 491))) >> 34;
        s = (f * (0x_0013_F840 + s)) >> 20;
        s = (f * (0x_0071_8A80 + s)) >> 16;
        s = (f * (0x_1EC0_4000 + s)) >> 21;
        ((f * (0x_2C5C_8000 + s)) >> 24)
    }

    #[inline]
    pub fn ln(f: u64) -> u64 {
        let z = ((f << 31) + 2) / (f + 8192); // fixed-point divide; discard remainder
        let zsq = (z * z) >> 30; // fixed-point squaring
        let mut s = (zsq * 1_584) >> 28;
        s = (zsq * (26_661 + s)) >> 29;
        s = (zsq * (302_676 + s)) >> 27;
        s = (zsq * (16_136_153 + s)) >> 30;
        (z * (193_635_259 + s)) >> 27
    }

    #[inline]
    pub fn log2(f: u64) -> u64 {
        let z = (f << 29) / (f + 8_192); // fixed-point divide; discard remainder
        let zsq = (z * z) >> 30; // fixed-point squaring
        let mut s = (zsq * 1_661) >> 25;
        s = (zsq * (13_209 + s)) >> 26;
        s = (zsq * (75_694 + s)) >> 24;
        s = (zsq * (2_017_019 + s)) >> 24;
        (z * (96_817_627 + s)) >> 26
    }

    #[inline]
    pub fn sin_pi(f: u64) -> u64 {
        if f < 0x_000A_5801 {
            return (f * 102_943) >> 15; // linear approximation suffices
        }
        let fs = f >> 11;
        let fsq = (fs * fs) >> 8;
        let mut s = (fsq * 650) >> 25;
        s = (fsq * (9_813 - s)) >> 23;
        s = (fsq * (334_253 - s)) >> 23;
        s = (fsq * (5_418_741 - s)) >> 22;
        (fs * (52_707_180 - s)) >> 13
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
fn test_exp() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS16 {
        let p_a: P16E1 = rng.gen();
        let f_a = f64::from(p_a);
        let p = p_a.exp();
        let f = f_a.exp();
        let expected = P16E1::from(f);
        if expected.is_zero() || expected.is_nar() {
            continue;
        }
        assert_eq!(p, expected);
    }
}

#[test]
fn test_exp2() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS16 {
        let p_a: P16E1 = rng.gen();
        let f_a = f64::from(p_a);
        let p = p_a.exp2();
        let f = f_a.exp2();
        let expected = P16E1::from(f);
        if expected.is_zero() || expected.is_nar() {
            continue;
        }
        assert_eq!(p, expected);
    }
}

#[test]
fn test_ln() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS16 {
        let p_a: P16E1 = rng.gen();
        let f_a = f64::from(p_a);
        let p = p_a.ln();
        let f = f_a.ln();
        let expected = P16E1::from(f);
        assert_eq!(p, expected);
    }
}

#[test]
fn test_log2() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS16 {
        let p_a: P16E1 = rng.gen();
        let f_a = f64::from(p_a);
        let p = p_a.log2();
        let f = f_a.log2();
        let expected = P16E1::from(f);
        assert_eq!(p, expected);
    }
}

#[test]
fn test_sin_pi() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS16 {
        let p_a: P16E1 = rng.gen();
        let f_a = f64::from(p_a);
        let p = p_a.sin_pi();
        let f = (f_a * core::f64::consts::PI).sin();
        let expected = P16E1::from(f);
        if p.is_zero() {
            continue;
        }
        assert_eq!(p, expected);
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
