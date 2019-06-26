use super::PxE2;
use crate::{MulAddType, WithSign};

impl<const N: u32> PxE2<{ N }> {
    #[inline]
    pub fn mul_add(self, b: Self, c: Self) -> Self {
        let ui_a = self.to_bits();
        let ui_b = b.to_bits();
        let ui_c = c.to_bits();
        Self::mul_add_ui(ui_a, ui_b, ui_c, crate::MulAddType::Add)
    }

    #[allow(clippy::cognitive_complexity)]
    fn mul_add_ui(mut ui_a: u32, mut ui_b: u32, mut ui_c: u32, op: MulAddType) -> Self {
        let mut bits_more = false;
        //NaR
        if (ui_a == 0x8000_0000) || (ui_b == 0x8000_0000) || (ui_c == 0x8000_0000) {
            return Self::NAR;
        } else if (ui_a == 0) || (ui_b == 0) {
            return match op {
                MulAddType::SubC => Self::from_bits(ui_c.wrapping_neg()),
                _ => Self::from_bits(ui_c),
            };
        }

        let sign_a = Self::sign_ui(ui_a);
        let sign_b = Self::sign_ui(ui_b);
        let sign_c = Self::sign_ui(ui_c); //^ (op == softposit_mulAdd_subC);
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

        if N == 2 {
            let reg_sa = Self::sign_reg_ui(ui_a);
            let reg_sb = Self::sign_reg_ui(ui_b);
            let mut u_z = if reg_sa & reg_sb { 0x40000000_u32 } else { 0x0 };
            if sign_z {
                // i.e. negative prod
                if sign_c {
                    u_z |= ui_c;
                    u_z = u_z.wrapping_neg();
                } else {
                    //prod is negative
                    u_z = if ui_c == u_z {
                        0
                    } else if u_z > 0 {
                        0x40000000
                    } else {
                        0xC0000000
                    };
                }
            } else {
                //prod : same sign signZ=0
                if sign_c {
                    u_z = if ui_c == u_z {
                        0
                    } else if u_z > 0 {
                        0x40000000
                    } else {
                        0xC0000000
                    };
                } else {
                    //C is positive
                    u_z |= ui_c;
                }
            }
            return Self::from_bits(u_z);
        } else {
            let (mut k_a, tmp) = Self::separate_bits_tmp(ui_a);
            let mut exp_a = (tmp >> 29) as i32; //to get 2 bits
            let frac_a = (tmp << 2) | 0x8000_0000;

            let (k_b, tmp) = Self::separate_bits_tmp(ui_b);
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
                let (k_c, exp_c, frac_c) = Self::separate_bits(ui_c);
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
                        return Self::ZERO;
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

            let (mut regime, reg_sz, reg_z) = Self::calculate_regime(k_z);
            let reg_z = reg_z as u32;

            let u_z = if reg_z > (N - 2) {
                //max or min pos. exp and frac does not matter.
                if reg_sz {
                    0x7FFFFFFF & (((-0x80000000_i32) >> (N - 1)) as u32)
                } else {
                    0x1 << (32 - N)
                }
            } else {
                let mut bit_n_plus_one = false;
                let mut frac_z: u32;
                if reg_z < N {
                    //remove hidden bits
                    frac64_z &= 0x3FFFFFFFFFFFFFFF;
                    frac_z = (frac64_z >> (reg_z + 34)) as u32; //frac32Z>>16;

                    if reg_z <= (N - 4) {
                        bit_n_plus_one =
                            ((0x8000000000000000_u64 >> (N - reg_z - 2)) & frac64_z) != 0;
                        bits_more = ((0x7FFFFFFFFFFFFFFF_u64 >> (N - reg_z - 2)) & frac64_z) != 0;
                        frac_z &= ((-0x80000000_i32) >> (N - 1)) as u32;
                    } else {
                        if reg_z == (N - 2) {
                            bit_n_plus_one = (exp_z & 0x2) != 0;
                            bits_more = (exp_z & 0x1) != 0;
                            exp_z = 0;
                        } else if reg_z == (N - 3) {
                            bit_n_plus_one = (exp_z & 0x1) != 0;
                            exp_z &= 0x2;
                        }
                        if frac64_z > 0 {
                            frac_z = 0;
                            bits_more = true;
                        }
                    }
                } else {
                    regime = if reg_sz {
                        regime & (((-0x80000000_i32) >> (N - 1)) as u32)
                    } else {
                        regime << (32 - N)
                    };
                    exp_z = 0;
                    frac_z = 0;
                }

                exp_z <<= 28 - reg_z;

                let mut u_z = Self::pack_to_ui(regime, exp_z as u32, frac_z);

                if bit_n_plus_one {
                    u_z += (((u_z >> (32 - N)) & 1) | (bits_more as u32)) << (32 - N);
                }
                u_z
            };
            Self::from_bits(u_z.with_sign(sign_z))
        }
    }

    #[inline]
    pub fn sqrt(self) -> Self {
        let mut ui_a = self.to_bits();

        // If NaR or a negative number, return NaR.
        if (ui_a & 0x8000_0000) != 0 {
            return Self::NAR;
        }
        // If the argument is zero, return zero.
        else if ui_a == 0 {
            return self;
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

        let mut frac64_z = ((frac_a as u64).wrapping_mul(recip_sqrt)) >> 31;
        if exp_a != 0 {
            frac64_z >>= 1;
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
        frac64_z += 1;

        if (frac64_z & 0xF) == 0 {
            let shifted_frac64_z = frac64_z >> 1;
            let neg_rem = (shifted_frac64_z * shifted_frac64_z) & 0x1_FFFF_FFFF;
            if (neg_rem & 0x1_0000_0000) != 0 {
                frac64_z |= 1;
            } else if neg_rem != 0 {
                frac64_z -= 1;
            }
        }
        // Strip off the hidden bit and round-to-nearest using last shift+5 bits.
        frac64_z &= 0xFFFFFFFF;
        let mask = 1 << (36 + shift - N);
        let u_a = if (mask & frac64_z) != 0 {
            if (((mask - 1) & frac64_z) | ((mask << 1) & frac64_z)) != 0 {
                frac64_z += mask << 1;
            }
            // Assemble the result and return it.
            ui_z | (exp_z << (27 - shift)) | ((frac64_z >> (5 + shift)) as u32)
        } else {
            // Assemble the result and return it.
            let mut u_a = ui_z | (exp_z << (27 - shift)) | ((frac64_z >> (5 + shift)) as u32);
            //Check if rounding bits in regime or exp and clean off unwanted bits
            if ((0x80000000_u32 >> N) & u_a) != 0 {
                if (((0x80000000_u32 >> (N - 1)) & u_a) != 0)
                    || (((0x7FFFFFFF_u32 >> N) & u_a) != 0)
                {
                    u_a = (u_a & (((-0x80000000_i32) >> (N - 1)) as u32))
                        + (0x80000000_u32 >> (N - 1));
                }
            }
            u_a
        };

        Self::from_bits(u_a & (((-0x80000000_i32) >> (N - 1)) as u32))
    }

    pub fn round(p_a: Self) -> Self {
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
            return Self::ZERO;
        } else if ui_a < 0x4400_0000 {
            // 1/2 < x < 3/2 rounds to 1.
            u_a = 0x4000_0000;
        } else if ui_a <= 0x4A00_0000 {
            u_a = if N > 4 { 0x_4800_0000 } else { 0x_4000_0000 };
        } else if ui_a >= 0x7E80_0000 {
            // If |A| is 0x7E800000 (4194304) (posit is pure integer value), leave it unchanged.
            if N > 8 {
                return p_a; // This also takes care of the NaR case, 0x80000000.
            } else {
                let bit_n_plus_one = ((0x8000_0000_u32 >> N) & ui_a) != 0;
                let tmp = (0x7FFF_FFFF_u32 >> N) & ui_a; //bitsMore
                let bit_last = (0x8000_0000_u32 >> (N - 1)) & ui_a;
                if bit_n_plus_one {
                    if (bit_last | tmp) != 0 {
                        ui_a += bit_last;
                    }
                }
                u_a = ui_a;
            }
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
        Self::from_bits(u_a.with_sign(sign))
    }
}
