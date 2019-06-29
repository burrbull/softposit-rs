use super::PxE1;
use crate::WithSign;
use core::convert::From;
use core::f64;

impl<const N: u32> From<PxE1<{ N }>> for f32 {
    #[inline]
    fn from(a: PxE1<{ N }>) -> Self {
        f64::from(a) as f32
    }
}

impl<const N: u32> From<PxE1<{ N }>> for f64 {
    #[inline]
    fn from(p_a: PxE1<{ N }>) -> Self {
        let mut ui_a = p_a.to_bits();

        if p_a.is_zero() {
            0.
        } else if p_a.is_nar() {
            f64::NAN
        } else {
            let sign_a = ui_a & 0x_8000_0000;
            if sign_a != 0 {
                ui_a = ui_a.wrapping_neg();
            }
            let (k_a, tmp) = PxE1::<{ N }>::separate_bits_tmp(ui_a);

            let frac_a = ((tmp << 2) as u64) << 20;
            let exp_a = (((k_a as u64) << 1) + ((tmp >> 30) as u64)).wrapping_add(1023) << 52;

            f64::from_bits(exp_a + frac_a + ((sign_a as u64) << 32))
        }
    }
}

impl<const N: u32> From<f64> for PxE1<{ N }> {
    #[allow(clippy::cognitive_complexity)]
    fn from(mut float: f64) -> Self {
        let mut reg: u32;
        let mut frac = 0_u32;
        let mut exp = 0_i32;
        let mut bit_n_plus_one = false;
        let mut bits_more = false;

        if float == 0. {
            return Self::ZERO;
        } else if !float.is_finite() {
            return Self::NAR;
        }

        let sign = float < 0.;

        let u_z: u32 = if float == 1. {
            0x4000_0000
        } else if float == -1. {
            0xC000_0000
        } else if (float > 1.) || (float < -1.) {
            if sign {
                //Make negative numbers positive for easier computation
                float = -float;
            }

            let reg_s = true;
            reg = 1; //because k = m-1; so need to add back 1
                     // minpos
            if (N == 2) && (float <= 8.673_617_379_884_035_e-19) {
                1
            } else {
                //regime
                while float >= 4. {
                    float *= 0.25; // float/=4;
                    reg += 1;
                }
                if float >= 2. {
                    float *= 0.5;
                    exp += 1;
                }
                let frac_length = (N - 3) as isize - (reg as isize);
                if frac_length < 0 {
                    if reg == N - 2 {
                        bit_n_plus_one = exp != 0;
                        exp = 0;
                    }
                    if float > 1. {
                        bits_more = true;
                    }
                } else {
                    frac = crate::convert_fraction_p32(
                        float,
                        frac_length as u16,
                        &mut bit_n_plus_one,
                        &mut bits_more,
                    );
                }

                if (reg == 30) && (frac > 0) {
                    bits_more = true;
                    frac = 0;
                }

                if reg > (N - 2) {
                    if reg_s {
                        0x_7FFF_FFFF & Self::MASK
                    } else {
                        0x1 << (32 - N)
                    }
                } else {
                    //rounding off fraction bits

                    let regime = if reg_s { ((1 << reg) - 1) << 1 } else { 1_u32 };

                    let mut u_z = (regime << (30 - reg))
                        + ((exp as u32) << (29 - reg))
                        + ((frac << (32 - N)) as u32);
                    //minpos
                    if (u_z == 0) && (frac > 0) {
                        u_z = 0x1 << (32 - N);
                    }
                    if bit_n_plus_one {
                        u_z += (((u_z >> (32 - N)) & 0x1) | (bits_more as u32)) << (32 - N);
                    }
                    u_z
                }
                .with_sign(sign)
            }
        } else if (float < 1.) || (float > -1.) {
            if sign {
                //Make negative numbers positive for easier computation
                float = -float;
            }

            let reg_s = false;
            reg = 0;

            //regime
            while float < 1. {
                float *= 4.;
                reg += 1;
            }

            if float >= 2. {
                float *= 0.5;
                exp += 1;
            }

            let frac_length = (N - 3) as isize - (reg as isize);
            if frac_length < 0 {
                if reg == N - 2 {
                    bit_n_plus_one = exp != 0;
                    exp = 0;
                }

                if float > 1. {
                    bits_more = true;
                }
            } else {
                frac = crate::convert_fraction_p32(
                    float,
                    frac_length as u16,
                    &mut bit_n_plus_one,
                    &mut bits_more,
                );
            }

            if (reg == 30) && (frac > 0) {
                bits_more = true;
                frac = 0;
            }

            if reg > (N - 2) {
                if reg_s {
                    0x_7FFF_FFFF & Self::MASK
                } else {
                    0x1 << (32 - N)
                }
            } else {
                //rounding off fraction bits

                let regime = if reg_s { ((1 << reg) - 1) << 1 } else { 1_u32 };

                let mut u_z = (regime << (30 - reg))
                    + ((exp as u32) << (29 - reg))
                    + ((frac << (32 - N)) as u32);
                //minpos
                if (u_z == 0) && (frac > 0) {
                    u_z = 0x1 << (32 - N);
                }

                if bit_n_plus_one {
                    u_z += (((u_z >> (32 - N)) & 0x1) | (bits_more as u32)) << (32 - N);
                }
                u_z
            }
            .with_sign(sign)
        } else {
            //NaR - for NaN, INF and all other combinations
            0x8000_0000
        };
        Self::from_bits(u_z)
    }
}

impl<const N: u32> From<PxE1<{ N }>> for u32 {
    #[inline]
    fn from(p_a: PxE1<{ N }>) -> Self {
        let mut ui_a = p_a.to_bits();
        //NaR
        if (ui_a >= 0x_8000_0000) && (ui_a <= 0x_3000_0000) {
            // 0 <= |pA| <= 1/2 rounds to zero.
            0
        } else if ui_a < 0x_4800_0000 {
            // 1/2 < x < 3/2 rounds to 1.
            1
        } else if ui_a <= 0x_5400_0000 {
            // 3/2 <= x <= 5/2 rounds to 2.
            2
        } else if ui_a > 0x_7FFF_BFFF {
            //4294836223
            4_294_967_295
        } else {
            // Decode the posit, left-justifying as we go.
            let mut scale = 0_u32;

            ui_a -= 0x_4000_0000; // Strip off first regime bit (which is a 1).
            while (0x_2000_0000 & ui_a) != 0 {
                // Increment scale by 2 for each regime sign bit.
                scale += 2; // Regime sign bit is always 1 in this range.
                ui_a = (ui_a - 0x_2000_0000) << 1; // Remove the bit; line up the next regime bit.
            }
            ui_a <<= 1; // Skip over termination bit, which is 0.
            if (0x_2000_0000 & ui_a) != 0 {
                scale += 1;
            } // If exponent is 1, increment the scale.
            let mut i_z64 = ((ui_a | 0x_2000_0000) as u64) << 33; // Left-justify fraction in 64-bit result (one left bit padding)

            let mut mask = 0x4000_0000_0000_0000_u64 >> scale; // Point to the last bit of the integer part.

            let bit_last = i_z64 & mask; // Extract the bit, without shifting it.
            mask >>= 1;
            let mut tmp = i_z64 & mask;
            let bit_n_plus_one = tmp != 0; // "True" if nonzero.
            i_z64 ^= tmp; // Erase the bit, if it was set.
            tmp = i_z64 & (mask - 1); // tmp has any remaining bits. // This is bits_more
            i_z64 ^= tmp; // Erase those bits, if any were set.

            if bit_n_plus_one {
                // logic for round to nearest, tie to even
                if (bit_last | tmp) != 0 {
                    i_z64 += mask << 1;
                }
            }

            (i_z64 >> (62 - scale)) as u32 // Right-justify the integer.
        }
    }
}

impl<const N: u32> From<PxE1<{ N }>> for i32 {
    #[inline]
    fn from(p_a: PxE1<{ N }>) -> Self {
        //NaR
        if p_a.is_nar() {
            return i32::min_value();
        }

        let mut ui_a = p_a.to_bits();

        let sign = ui_a > 0x_8000_0000; // sign is True if pA > NaR.

        if sign {
            ui_a = ui_a.wrapping_neg(); // A is now |A|.
        }

        if ui_a <= 0x_3000_0000 {
            // 0 <= |pA| <= 1/2 rounds to zero.
            return 0;
        }
        let i_z = if ui_a < 0x_4800_0000 {
            // 1/2 < x < 3/2 rounds to 1.
            1
        } else if ui_a <= 0x_5400_0000 {
            // 3/2 <= x <= 5/2 rounds to 2.
            2
        } else if ui_a > 0x_7FFF_9FFF {
            //2147418112
            return if sign { -2_147_483_648 } else { 2_147_483_647 };
        } else {
            // Decode the posit, left-justifying as we go.
            let mut scale = 0_u32;

            ui_a -= 0x_4000_0000; // Strip off first regime bit (which is a 1).
            while (0x_2000_0000 & ui_a) != 0 {
                // Increment scale by 2 for each regime sign bit.
                scale += 2; // Regime sign bit is always 1 in this range.
                ui_a = (ui_a - 0x_2000_0000) << 1; // Remove the bit; line up the next regime bit.
            }
            ui_a <<= 1; // Skip over termination bit, which is 0.
            if (0x_2000_0000 & ui_a) != 0 {
                scale += 1;
            } // If exponent is 1, increment the scale.
            let mut i_z64 = ((ui_a | 0x_2000_0000) as u64) << 33; // Left-justify fraction in 64-bit result (one left bit padding)

            let mut mask = 0x4000_0000_0000_0000_u64 >> scale; // Point to the last bit of the integer part.

            let bit_last = i_z64 & mask; // Extract the bit, without shifting it.
            mask >>= 1;
            let mut tmp = i_z64 & mask;
            let bit_n_plus_one = tmp != 0; // "True" if nonzero.
            i_z64 ^= tmp; // Erase the bit, if it was set.
            tmp = i_z64 & (mask - 1); // tmp has any remaining bits. // This is bits_more
            i_z64 ^= tmp; // Erase those bits, if any were set.

            if bit_n_plus_one {
                // logic for round to nearest, tie to even
                if (bit_last | tmp) != 0 {
                    i_z64 += mask << 1;
                }
            }

            (i_z64 >> (62 - scale)) as u32 // Right-justify the integer.
        };
        i_z.with_sign(sign) as i32
    }
}

impl<const N: u32> From<PxE1<{ N }>> for u64 {
    #[inline]
    fn from(p_a: PxE1<{ N }>) -> Self {
        let mut ui_a = p_a.to_bits();
        //NaR
        if (ui_a >= 0x_8000_0000) && (ui_a <= 0x_3000_0000) {
            // 0 <= |pA| <= 1/2 rounds to zero.
            0
        } else if ui_a < 0x_4800_0000 {
            // 1/2 < x < 3/2 rounds to 1.
            1
        } else if ui_a <= 0x_5400_0000 {
            // 3/2 <= x <= 5/2 rounds to 2.
            2
        } else {
            // Decode the posit, left-justifying as we go.
            let mut scale = 0_u32;

            ui_a -= 0x_4000_0000; // Strip off first regime bit (which is a 1).
            while (0x_2000_0000 & ui_a) != 0 {
                // Increment scale by 2 for each regime sign bit.
                scale += 2; // Regime sign bit is always 1 in this range.
                ui_a = (ui_a - 0x_2000_0000) << 1; // Remove the bit; line up the next regime bit.
            }
            ui_a <<= 1; // Skip over termination bit, which is 0.
            if (0x_2000_0000 & ui_a) != 0 {
                scale += 1;
            } // If exponent is 1, increment the scale.
            let mut i_z = ((ui_a | 0x_2000_0000) as u64) << 33; // Left-justify fraction in 64-bit result (one left bit padding)
            let mut mask = 0x_4000_0000_0000_0000 >> scale; // Point to the last bit of the integer part.

            let bit_last = i_z & mask; // Extract the bit, without shifting it.
            mask >>= 1;
            let mut tmp = i_z & mask;
            let bit_n_plus_one = tmp != 0; // "True" if nonzero.
            i_z ^= tmp; // Erase the bit, if it was set.
            tmp = i_z & (mask - 1); // tmp has any remaining bits. // This is bits_more
            i_z ^= tmp; // Erase those bits, if any were set.

            if bit_n_plus_one {
                // logic for round to nearest, tie to even
                if (bit_last | tmp) != 0 {
                    i_z += mask << 1;
                }
            }

            i_z >> (62 - scale) // Right-justify the integer.
        }
    }
}

impl<const N: u32> From<PxE1<{ N }>> for i64 {
    #[inline]
    fn from(p_a: PxE1<{ N }>) -> Self {
        //NaR
        if p_a.is_nar() {
            return i64::min_value();
        }

        let mut ui_a = p_a.to_bits();

        let sign = ui_a > 0x_8000_0000; // sign is True if pA > NaR.

        if sign {
            ui_a = ui_a.wrapping_neg(); // A is now |A|.
        }

        if ui_a <= 0x_3000_0000 {
            // 0 <= |pA| <= 1/2 rounds to zero.
            return 0;
        }
        let i_z = if ui_a < 0x_4800_0000 {
            // 1/2 < x < 3/2 rounds to 1.
            1
        } else if ui_a <= 0x_5400_0000 {
            // 3/2 <= x <= 5/2 rounds to 2.
            2
        } else {
            // Decode the posit, left-justifying as we go.
            let mut scale = 0_u32;

            ui_a -= 0x_4000_0000; // Strip off first regime bit (which is a 1).
            while (0x_2000_0000 & ui_a) != 0 {
                // Increment scale by 2 for each regime sign bit.
                scale += 2; // Regime sign bit is always 1 in this range.
                ui_a = (ui_a - 0x_2000_0000) << 1; // Remove the bit; line up the next regime bit.
            }
            ui_a <<= 1; // Skip over termination bit, which is 0.
            if (0x_2000_0000 & ui_a) != 0 {
                scale += 1;
            } // If exponent is 1, increment the scale.
            let mut i_z = ((ui_a | 0x_2000_0000) as u64) << 33; // Left-justify fraction in 64-bit result (one left bit padding)
            let mut mask = 0x_4000_0000_0000_0000 >> scale; // Point to the last bit of the integer part.

            let bit_last = i_z & mask; // Extract the bit, without shifting it.
            mask >>= 1;
            let mut tmp = i_z & mask;
            let bit_n_plus_one = tmp != 0; // "True" if nonzero.
            i_z ^= tmp; // Erase the bit, if it was set.
            tmp = i_z & (mask - 1); // tmp has any remaining bits. // This is bits_more
            i_z ^= tmp; // Erase those bits, if any were set.

            if bit_n_plus_one {
                // logic for round to nearest, tie to even
                if (bit_last | tmp) != 0 {
                    i_z += mask << 1;
                }
            }

            i_z >> (62 - scale) // Right-justify the integer.
        };
        i_z.with_sign(sign) as i64
    }
}
