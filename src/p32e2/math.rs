use super::P32E2;
use crate::{MulAddType, WithSign};

const HALF: P32E2 = P32E2::new(0x_3800_0000);
const TWO: P32E2 = P32E2::new(0x_4800_0000);

impl crate::MathConsts for P32E2 {
    const E: Self = Self::new(0x_4adf_8546);
    const FRAC_1_PI: Self = Self::new(0x_322f_9837);
    const FRAC_1_SQRT_2: Self = Self::new(0x_3b50_4f33);
    const FRAC_2_PI: Self = Self::new(0x_3a2f_9837);
    const FRAC_2_SQRT_PI: Self = Self::new(0x_4106_eba8);
    const FRAC_PI_2: Self = Self::new(0x_4490_fdaa);
    const FRAC_PI_3: Self = Self::new(0x_4060_a91c);
    const FRAC_PI_4: Self = Self::new(0x_3c90_fdaa);
    const FRAC_PI_6: Self = Self::new(0x_3860_a91c);
    const FRAC_PI_8: Self = Self::new(0x_3490_fdaa);
    const LN_10: Self = Self::new(0x_4935_d8de);
    const LN_2: Self = Self::new(0x_3b17_217f);
    const LOG10_E: Self = Self::new(0x_35e5_bd8b);
    const LOG2_E: Self = Self::new(0x_438a_a3b3);
    const PI: Self = Self::new(0x_4c90_fdaa);
    const SQRT_2: Self = Self::new(0x_4350_4f33);
    const LOG2_10: Self = Self::new(0x_4d49_a785);
    const LOG10_2: Self = Self::new(0x_31a2_09a8);
}

impl P32E2 {
    #[inline]
    pub fn mul_add(self, b: Self, c: Self) -> Self {
        let ui_a = self.to_bits();
        let ui_b = b.to_bits();
        let ui_c = c.to_bits();
        mul_add(ui_a, ui_b, ui_c, crate::MulAddType::Add)
    }
    #[inline]
    pub fn floor(self) -> Self {
        (self - HALF).round()
    }
    #[inline]
    pub fn ceil(self) -> Self {
        (self + HALF).round()
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
        unimplemented!()
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
        if self.is_nan() {
            self
        } else {
            (self + ((self * self) + Self::ONE).sqrt()).ln()
        }
    }
    #[inline]
    pub fn acosh(self) -> Self {
        match self {
            x if x < Self::ONE => Self::NAN,
            x => (x + ((x * x) - Self::ONE).sqrt()).ln(),
        }
    }
    #[inline]
    pub fn atanh(self) -> Self {
        HALF * ((TWO * self) / (Self::ONE - self)).ln_1p()
    }
}

pub(super) fn mul_add(mut ui_a: u32, mut ui_b: u32, mut ui_c: u32, op: MulAddType) -> P32E2 {
    let mut bits_more = false;
    //NaR
    if (ui_a == 0x8000_0000) || (ui_b == 0x8000_0000) || (ui_c == 0x8000_0000) {
        return P32E2::INFINITY;
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
                return P32E2::ZERO;
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
        let frac_z = if reg_z <= 28 {
            //remove hidden bits
            frac64_z &= 0x3FFF_FFFF_FFFF_FFFF;
            bit_n_plus_one = (0x0000_0002_0000_0000 & (frac64_z >> reg_z)) != 0;
            exp_z <<= 28 - reg_z;
            (frac64_z >> (reg_z + 34)) as u32 //frac32Z>>16;
        } else {
            if reg_z == 30 {
                bit_n_plus_one = (exp_z & 0x2) != 0;
                bits_more = (exp_z & 0x1) != 0;
                exp_z = 0;
            } else if reg_z == 29 {
                bit_n_plus_one = (exp_z & 0x1) != 0;
                exp_z >>= 1;
            }
            0
        };
        let mut u_z = P32E2::pack_to_ui(regime, exp_z as u32, frac_z);

        if bit_n_plus_one {
            if (frac64_z << (32 - reg_z)) != 0 {
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
        return P32E2::ZERO;
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
        return P32E2::INFINITY;
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

    let mut frac_z = ((frac_a as u64).wrapping_mul(recip_sqrt)) >> 31;
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

#[test]
fn test_mul_add() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS32 {
        let p_a: P32E2 = rng.gen();
        let p_b: P32E2 = rng.gen();
        let p_c: P32E2 = rng.gen();
        let f_a = f64::from(p_a);
        let f_b = f64::from(p_b);
        let f_c = f64::from(p_c);
        let p = p_a.mul_add(p_b, p_c);
        let f = f_a.mul_add(f_b, f_c);
        assert_eq!(p, P32E2::from(f));
    }
}

#[test]
fn test_sqrt() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS32 {
        let p_a: P32E2 = rng.gen();
        let f_a = f64::from(p_a);
        let p = p_a.sqrt();
        let f = f_a.sqrt();
        assert_eq!(p, P32E2::from(f));
    }
}

#[test]
fn test_round() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS32 {
        let p_a: P32E2 = rng.gen();
        let f_a = f64::from(p_a);
        let p = p_a.round();
        let f = f_a.round();
        if (f - f_a).abs() == 0.5 {
            continue;
        }
        assert_eq!(p, P32E2::from(f));
    }
}
