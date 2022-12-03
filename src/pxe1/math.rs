use core::cmp::Ordering;

use super::PxE1;
use crate::{MulAddType, WithSign};

impl<const N: u32> PxE1<{ N }> {
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
        if (ui_a == 0x_8000_0000) || (ui_b == 0x_8000_0000) || (ui_c == 0x_8000_0000) {
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
            let mut u_z = if reg_sa & reg_sb {
                0x_4000_0000_u32
            } else {
                0x0
            };
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
                        0x_4000_0000
                    } else {
                        0x_C000_0000
                    };
                }
            } else {
                //prod : same sign signZ=0
                if sign_c {
                    u_z = if ui_c == u_z {
                        0
                    } else if u_z > 0 {
                        0x_4000_0000
                    } else {
                        0x_C000_0000
                    };
                } else {
                    //C is positive
                    u_z |= ui_c;
                }
            }
            Self::from_bits(u_z)
        } else {
            let (mut k_a, tmp) = Self::separate_bits_tmp(ui_a);
            let mut exp_a = (tmp >> 29) as i32; //to get 2 bits
            let frac_a = (tmp << 2) | 0x_8000_0000;

            let (k_b, tmp) = Self::separate_bits_tmp(ui_b);
            k_a += k_b;
            exp_a += (tmp >> 29) as i32;
            let mut frac64_z = (frac_a as u64) * (((tmp << 2) | 0x_8000_0000) as u64);

            if exp_a > 1 {
                k_a += 1;
                exp_a ^= 0x2;
            }

            let rcarry = (frac64_z & 0x_8000_0000_0000_0000) != 0; //1st bit of frac64_z
            if rcarry {
                if exp_a != 0 {
                    k_a += 1;
                }
                exp_a ^= 1;
                frac64_z >>= 1;
            }

            let mut k_z;
            let mut exp_z: i32;
            if ui_c != 0 {
                let (k_c, exp_c, frac_c) = Self::separate_bits(ui_c);
                let mut frac64_c = (frac_c as u64) << 32;
                let mut shift_right = (((k_a - k_c) as i16) << 1) + (exp_a - exp_c) as i16;

                exp_z = match shift_right.cmp(&0) {
                    Ordering::Less => {
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
                    }
                    Ordering::Greater => {
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
                    }
                    Ordering::Equal => {
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
                    }
                };
                let rcarry = (frac64_z & 0x_8000_0000_0000_0000) != 0; //first left bit

                if rcarry {
                    if exp_z != 0 {
                        k_z += 1;
                    }
                    exp_z ^= 1;
                    if (frac64_z & 0x1) != 0 {
                        bits_more = false;
                    }
                    frac64_z = (frac64_z >> 1) & 0x_7FFF_FFFF_FFFF_FFFF;
                } else {
                    //for subtract cases
                    if frac64_z != 0 {
                        while (frac64_z >> 61) == 0 {
                            k_z -= 1;
                            frac64_z <<= 2;
                        }
                    }
                    let ecarry = ((0x_4000_0000_0000_0000 & frac64_z) >> 62) != 0;

                    if !ecarry {
                        if exp_z == 0 {
                            k_z -= 1;
                        }
                        exp_z ^= 1;
                        frac64_z <<= 1;
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
                    0x_7FFF_FFFF & Self::mask()
                } else {
                    0x1 << (32 - N)
                }
            } else {
                let mut bit_n_plus_one = false;
                let mut frac_z: u32;
                if reg_z < N {
                    //remove hidden bits
                    frac64_z &= 0x_3FFF_FFFF_FFFF_FFFF;
                    frac_z = (frac64_z >> (reg_z + 33)) as u32; //frac32Z>>16;

                    if reg_z != (N - 2) {
                        bit_n_plus_one =
                            ((0x_8000_0000_0000_0000_u64 >> (N - reg_z - 1)) & frac64_z) != 0;
                        bits_more =
                            ((0x_7FFF_FFFF_FFFF_FFFF_u64 >> (N - reg_z - 1)) & frac64_z) != 0;
                        frac_z &= Self::mask();
                    } else if frac64_z > 0 {
                        frac_z = 0;
                        bits_more = true;
                    }
                    if (reg_z == (N - 2)) && (exp_z != 0) {
                        bit_n_plus_one = true;
                        exp_z = 0;
                    }
                } else {
                    regime = if reg_sz {
                        regime & Self::mask()
                    } else {
                        regime << (32 - N)
                    };
                    exp_z = 0;
                    frac_z = 0;
                }

                exp_z <<= 29 - reg_z;

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
    pub fn round(p_a: Self) -> Self {
        let mut mask = 0x_2000_0000_u32;
        let mut scale = 0_u32;

        let u_a: u32;

        let mut ui_a = p_a.to_bits();
        let sign = (ui_a & 0x_8000_0000) != 0;

        // sign is True if pA > NaR.
        if sign {
            ui_a = ui_a.wrapping_neg();
        } // A is now |A|.
        if ui_a <= 0x_3000_0000 {
            // 0 <= |pA| <= 1/2 rounds to zero.
            return Self::ZERO;
        } else if ui_a < 0x_4800_0000 {
            // 1/2 < x < 3/2 rounds to 1.
            u_a = 0x_4000_0000;
        } else if ui_a <= 0x_5400_0000 {
            // 3/2 <= x <= 5/2 rounds to 2.
            u_a = 0x_5000_0000;
        } else if ui_a >= 0x_7FE8_0000 {
            // If |A| is 0x7FE800000 (4194304) (posit is pure integer value), leave it unchanged.
            if N > 8 {
                return p_a; // This also takes care of the NaR case, 0x80000000.
            } else {
                let bit_n_plus_one = ((0x_8000_0000_u32 >> N) & ui_a) != 0;
                let tmp = (0x_7FFF_FFFF_u32 >> N) & ui_a; //bitsMore
                let bit_last = (0x_8000_0000_u32 >> (N - 1)) & ui_a;
                if bit_n_plus_one && ((bit_last | tmp) != 0) {
                    ui_a += bit_last;
                }
                u_a = ui_a;
            }
        } else {
            // 34% of the cases, we have to decode the posit.

            while (mask & ui_a) != 0 {
                scale += 2;
                mask >>= 1;
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
