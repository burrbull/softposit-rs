use super::{P32E2, Q32E2};
use crate::{MulAddType, WithSign};

pub mod sleef;

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

impl crate::MathConsts for Q32E2 {
    const E: Self = Self(0, 0, 0, 0, 0x_0002_b7e1_5162_8aed, 0x_2a6a_bf71_5880_9cf4, 0x_f3c7_62e7_160f_38b4, 0x_da56_a784_d904_5190);
    const FRAC_1_PI: Self = Self(0, 0, 0, 0, 0x_0000_517c_c1b7_2722, 0x_0a94_fe13_abe8_fa9a, 0x_6ee0_6db1_4acc_9e21, 0x_c820_ff28_b1d5_ef5d);
    const FRAC_1_SQRT_2: Self = Self(0, 0, 0, 0, 0x_0000_b504_f333_f9de, 0x_6484_597d_89b3_754a, 0x_be9f_1d6f_60ba_893b, 0x_a84c_ed17_ac85_8333);
    const FRAC_2_PI: Self = Self(0, 0, 0, 0, 0x_0000_a2f9_836e_4e44, 0x_1529_fc27_57d1_f534, 0x_ddc0_db62_9599_3c43, 0x_9041_fe51_63ab_debb);
    const FRAC_2_SQRT_PI: Self = Self(0, 0, 0, 0, 0x_0001_20dd_7504_29b6, 0x_d11a_e3a9_14fe_d7fd, 0x_8688_2813_41d7_587c, 0x_ea2e_7342_b061_99cc);
    const FRAC_PI_2: Self = Self(0, 0, 0, 0, 0x_0001_921f_b544_42d1, 0x_8469_898c_c517_01b8, 0x_39a2_5204_9c11_14cf, 0x_98e8_0417_7d4c_7627);
    const FRAC_PI_3: Self = Self(0, 0, 0, 0, 0x_0001_0c15_2382_d736, 0x_5846_5bb3_2e0f_567a, 0x_d116_e158_680b_6335, 0x_109a_ad64_fe32_f96f);
    const FRAC_PI_4: Self = Self(0, 0, 0, 0, 0x_0000_c90f_daa2_2168, 0x_c234_c4c6_628b_80dc, 0x_1cd1_2902_4e08_8a67, 0x_cc74_020b_bea6_3b13);
    const FRAC_PI_6: Self = Self(0, 0, 0, 0, 0x_0000_860a_91c1_6b9b, 0x_2c23_2dd9_9707_ab3d, 0x_688b_70ac_3405_b19a, 0x_884d_56b2_7f19_7cb7);
    const FRAC_PI_8: Self = Self(0, 0, 0, 0, 0x_0000_6487_ed51_10b4, 0x_611a_6263_3145_c06e, 0x_0e68_9481_2704_4533, 0x_e63a_0105_df53_1d89);
    const LN_10: Self = Self(0, 0, 0, 0, 0x_0002_4d76_3776_aaa2, 0x_b05b_a95b_58ae_0b4c, 0x_28a3_8a3f_b3e7_6977, 0x_e43a_0f18_7a08_07c0);
    const LN_2: Self = Self(0, 0, 0, 0, 0x_0000_b172_17f7_d1cf, 0x_79ab_c9e3_b398_03f2, 0x_f6af_40f3_4326_7298, 0x_b62d_8a0d_175b_8baa);
    const LOG10_E: Self = Self(0, 0, 0, 0, 0x_0000_6f2d_ec54_9b94, 0x_38ca_9aad_d557_d699, 0x_ee19_1f71_a301_22e4, 0x_d101_1d1f_96a2_7bc7);
    const LOG2_E: Self = Self(0, 0, 0, 0, 0x_0001_7154_7652_b82f, 0x_e177_7d0f_fda0_d23a, 0x_7d11_d6ae_f551_bad2, 0x_b4b1_164a_2cd9_a342);
    const PI: Self = Self(0, 0, 0, 0, 0x_0003_243f_6a88_85a3, 0x08d3_1319_8a2e_0370, 0x_7344_a409_3822_299f, 0x_31d0_082e_fa98_ec4e);
    const SQRT_2: Self = Self(0, 0, 0, 0, 0x_0001_6a09_e667_f3bc, 0x_c908_b2fb_1366_ea95, 0x_7d3e_3ade_c175_1277, 0x_5099_da2f_590b_0667);
    const LOG2_10: Self = Self(0, 0, 0, 0, 0x_0003_5269_e12f_346e, 0x_2bf9_24af_dbfd_36bf, 0x_6d33_65b1_57f8_dece, 0x_b53a_46da_b202_0b9e);
    const LOG10_2: Self = Self(0, 0, 0, 0, 0x_0000_4d10_4d42_7de7, 0x_fbcc_47c4_acd6_05be, 0x_48bc_1356_9862_a1e8, 0x_f9a4_c52f_3793_5be6);
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
        sleef::sin(self)
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

pub(super) fn mul_add(mut ui_a: u32, mut ui_b: u32, mut ui_c: u32, op: MulAddType) -> P32E2 {
    let mut bits_more = false;
    //NaR
    if (ui_a == 0x8000_0000) || (ui_b == 0x8000_0000) || (ui_c == 0x8000_0000) {
        return P32E2::NAR;
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
        return P32E2::NAR;
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
