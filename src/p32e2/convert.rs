use super::*;
use crate::WithSign;
use core::convert::From;
use core::f64;

fn check_extra_p32_two_bits(
    mut float: f64,
    mut temp: f64,
    bits_n_plus_one: &mut bool,
    bits_more: &mut bool,
) {
    temp /= 2.;
    if temp <= float {
        *bits_n_plus_one = true;
        float -= temp;
    }
    if float > 0. {
        *bits_more = true;
    }
}

fn convert_fraction_p32(
    mut float: f64,
    mut frac_length: u16,
    bits_n_plus_one: &mut bool,
    bits_more: &mut bool,
) -> u32 {
    let mut frac = 0_u32;

    if float == 0. {
        return 0;
    } else if float == f64::INFINITY {
        return 0x8000_0000;
    }

    float -= 1.; //remove hidden bit
    if frac_length == 0 {
        check_extra_p32_two_bits(float, 1.0, bits_n_plus_one, bits_more);
    } else {
        let mut temp = 1_f64;
        loop {
            temp /= 2.;
            if temp <= float {
                float -= temp;
                frac_length -= 1;
                frac = (frac << 1) + 1; //shift in one
                if float == 0. {
                    frac <<= frac_length as u16;
                    break;
                }

                if frac_length == 0 {
                    check_extra_p32_two_bits(float, temp, bits_n_plus_one, bits_more);
                    break;
                }
            } else {
                frac <<= 1; //shift in a zero
                frac_length -= 1;
                if frac_length == 0 {
                    check_extra_p32_two_bits(float, temp, bits_n_plus_one, bits_more);
                    break;
                }
            }
        }
    }
    frac
}

impl From<f32> for P32E2 {
    fn from(float: f32) -> Self {
        P32E2::from(float as f64)
    }
}

impl From<f64> for P32E2 {
    fn from(mut float: f64) -> Self {
        let mut reg: u32;
        let mut frac = 0_u32;
        let mut exp = 0_i32;
        let mut bit_n_plus_one = false;
        let mut bits_more = false;

        if float == 0. {
            return ZERO;
        } else if !float.is_finite() {
            return INFINITY;
        } else if float >= 1.329_227_995_784_916_e36 {
            //maxpos
            return MAX;
        } else if float <= -1.329_227_995_784_916_e36 {
            // -maxpos
            return MIN;
        }

        let sign = float < 0.;

        let u_z: u32 = if float == 1. {
            0x4000_0000
        } else if float == -1. {
            0xC000_0000
        } else if (float <= 7.523_163_845_262_64_e-37) && !sign {
            //minpos
            0x1
        } else if (float >= -7.523_163_845_262_64_e-37) && sign {
            //-minpos
            0xFFFF_FFFF
        } else if (float > 1.) || (float < -1.) {
            if sign {
                //Make negative numbers positive for easier computation
                float = -float;
            }

            reg = 1; //because k = m-1; so need to add back 1
                     // minpos
            if float <= 7.523_163_845_262_64_e-37 {
                1
            } else {
                //regime
                while float >= 16. {
                    float *= 0.0625; // float/=16;
                    reg += 1;
                }
                while float >= 2. {
                    float *= 0.5;
                    exp += 1;
                }

                let frac_length = 28 - (reg as i8);

                if frac_length < 0 {
                    //in both cases, reg=29 and 30, e is n+1 bit and frac are sticky bits
                    if reg == 29 {
                        bit_n_plus_one = (exp & 0x1) != 0;
                        exp >>= 1; //taken care of by the pack algo
                    } else {
                        //reg=30
                        bit_n_plus_one = (exp >> 1) != 0;
                        bits_more = (exp & 0x1) != 0;
                        exp = 0;
                    }
                    if float != 1. {
                        //because of hidden bit
                        bits_more = true;
                        frac = 0;
                    }
                } else {
                    frac = convert_fraction_p32(
                        float,
                        frac_length as u16,
                        &mut bit_n_plus_one,
                        &mut bits_more,
                    );
                }

                if reg > 30 {
                    0x7FFF_FFFF
                } else {
                    //rounding off fraction bits

                    let regime = ((1 << reg) - 1) << 1;
                    if reg <= 28 {
                        exp <<= 28 - reg;
                    }
                    let u_z = ((regime as u32) << (30 - reg)) + (exp as u32) + (frac as u32);
                    u_z + (((bit_n_plus_one as u32) & (u_z & 1))
                        | ((bit_n_plus_one & bits_more) as u32))
                }
                .with_sign(sign)
            }
        } else if (float < 1.) || (float > -1.) {
            if sign {
                //Make negative numbers positive for easier computation
                float = -float;
            }
            reg = 0;

            //regime
            while float < 1. {
                float *= 16.;
                reg += 1;
            }

            while float >= 2. {
                float *= 0.5;
                exp += 1;
            }

            //only possible combination for reg=15 to reach here is 7FFF (maxpos) and FFFF (-minpos)
            //but since it should be caught on top, so no need to handle
            let frac_length = 28 - (reg as i8);
            if frac_length < 0 {
                //in both cases, reg=29 and 30, e is n+1 bit and frac are sticky bits
                if reg == 29 {
                    bit_n_plus_one = (exp & 0x1) != 0;
                    exp >>= 1; //taken care of by the pack algo
                } else {
                    //reg=30
                    bit_n_plus_one = (exp >> 1) != 0;
                    bits_more = (exp & 0x1) != 0;
                    exp = 0;
                }
                if float != 1. {
                    //because of hidden bit
                    bits_more = true;
                    frac = 0;
                }
            } else {
                frac = convert_fraction_p32(
                    float,
                    frac_length as u16,
                    &mut bit_n_plus_one,
                    &mut bits_more,
                );
            }

            if reg > 30 {
                0x1
            } else {
                //rounding off fraction bits

                let regime = 1_u32;
                if reg <= 28 {
                    exp <<= 28 - reg;
                }
                let u_z = ((regime as u32) << (30 - reg)) + (exp as u32) + (frac as u32);
                u_z + (((bit_n_plus_one as u32) & (u_z & 1))
                    | ((bit_n_plus_one & bits_more) as u32))
            }
            .with_sign(sign)
        } else {
            //NaR - for NaN, INF and all other combinations
            0x8000_0000
        };
        P32E2::from_bits(u_z)
    }
}

impl From<P32E2> for f32 {
    #[inline]
    fn from(a: P32E2) -> Self {
        f64::from(a) as f32
    }
}

impl From<P32E2> for f64 {
    #[inline]
    fn from(p_a: P32E2) -> Self {
        let mut ui_a = p_a.to_bits();

        if p_a.is_zero() {
            0.
        } else if p_a.is_infinite() {
            f64::NAN
        } else {
            let sign_a = P32E2::sign_ui( ui_a );
            if sign_a {
                ui_a = ui_a.wrapping_neg();
            }
            let (k_a, tmp) = P32E2::separate_bits_tmp(ui_a);

            let mut exp_a = (tmp>>29) as u64; //to get 2 bits

            let frac_a = (((tmp as u64)<<3) & 0xFFFF_FFFF)<<20;
            exp_a = ((((k_a as u64)<<2)+exp_a) + 1023) << 52;

            f64::from_bits(exp_a + frac_a + (((sign_a as u64)&0x1)<<63))
        }
    }
}

impl From<P32E2> for i32 {
    #[inline]
    fn from(p_a: P32E2) -> Self {
        if p_a.is_infinite() {
            return i32::min_value();
        }

        let mut ui_a = p_a.to_bits();

        let sign = (ui_a >> 31) != 0;
        if sign {
            ui_a = ui_a.wrapping_neg();
        }

        let i_z: i32 = if ui_a <= 0x3800_0000 {
            0 // 0 <= |pA| <= 1/2 rounds to zero.
        } else if ui_a < 0x4400_0000 {
            1 // 1/2 < x < 3/2 rounds to 1.
        } else if ui_a <= 0x4A00_0000 {
            2 // 3/2 <= x <= 5/2 rounds to 2. // For speed. Can be commented out
        } else if ui_a > 0x7FAF_FFFF {
            //overflow so return max integer value
            0x7FFF_FFFF
        } else {
            let (scale, bits) = P32E2::calculate_scale(ui_a);

            let mut i_z64 = (((bits | 0x1000_0000) & 0x1FFF_FFFF) as u64) << 34; // Left-justify fraction in 32-bit result (one left bit padding)
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

            (i_z64 >> (62 - scale)) as i32 // Right-justify the integer.
        };

        i_z.with_sign(sign)
    }
}

impl From<P32E2> for u32 {
    #[inline]
    fn from(p_a: P32E2) -> Self {
        if p_a.is_infinite() {
            return 0x8000_0000; // Error: Should be u32::max_value()
        }

        let ui_a = p_a.to_bits();

        //negative
        if ui_a > 0x8000_0000 {
            return 0;
        }
        if ui_a <= 0x3800_0000 {
            0 // 0 <= |pA| <= 1/2 rounds to zero.
        } else if ui_a < 0x4400_0000 {
            1 // 1/2 < x < 3/2 rounds to 1.
        } else if ui_a <= 0x4A00_0000 {
            2 // 3/2 <= x <= 5/2 rounds to 2. // For speed. Can be commented out
        } else if ui_a > 0x7FAF_FFFF {
            //overflow so return max integer value
            0x7FFF_FFFF
        } else {
            let (scale, bits) = P32E2::calculate_scale(ui_a);

            let mut i_z64 = (((bits | 0x1000_0000) & 0x1FFF_FFFF) as u64) << 34; // Left-justify fraction in 32-bit result (one left bit padding)
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

impl From<P32E2> for i64 {
    #[inline]
    fn from(p_a: P32E2) -> Self {
        let mut ui_a = p_a.to_bits();

        if ui_a == 0x8000_0000 {
            return (ui_a as i64) << 32;
        }

        let sign = (ui_a >> 31) != 0;
        if sign {
            ui_a = ui_a.wrapping_neg();
        }

        if ui_a <= 0x3800_0000 {
            return 0; // 0 <= |pA| <= 1/2 rounds to zero.
        }
        let i_z: i64 = if ui_a < 0x4400_0000 {
            1 // 1/2 < x < 3/2 rounds to 1.
        } else if ui_a <= 0x4A00_0000 {
            2 // 3/2 <= x <= 5/2 rounds to 2. // For speed. Can be commented out
              //}else if ui_a < 0x4E00_0000  {3        // 5/2 < x < 7/2 rounds to 3
              //} else if ui_a <= 0x5100_0000  { i_z = 4
              //overflow so return max integer value
        } else if ui_a > 0x7FFF_AFFF {
            0x7FFF_FFFF_FFFF_FFFF
        } else {
            let (scale, bits) = P32E2::calculate_scale(ui_a);

            let mut i_z = (((bits | 0x1000_0000) & 0x1FFF_FFFF) as u64) << 34; // Left-justify fraction in 32-bit result (one left bit padding)

            if scale < 62 {
                let mut mask = 0x4000_0000_0000_0000_u64 >> scale; // Point to the last bit of the integer part.

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
                i_z = (i_z as u64) >> (62 - scale); // Right-justify the integer.
            } else if scale > 62 {
                i_z <<= scale - 62;
            }
            i_z as i64
        };

        i_z.with_sign(sign)
    }
}

impl From<P32E2> for u64 {
    #[inline]
    fn from(p_a: P32E2) -> Self {
        let ui_a = p_a.to_bits();

        //NaR
        if ui_a == 0x8000_0000 {
            0x8000_0000_0000_0000
        //negative
        } else if (ui_a > 0x8000_0000) || (ui_a <= 0x3800_0000) {
            0 // 0 <= |pA| <= 1/2 rounds to zero.
        } else if ui_a < 0x4400_0000 {
            1 // 1/2 < x < 3/2 rounds to 1.
        } else if ui_a <= 0x4A00_0000 {
            2 // 3/2 <= x <= 5/2 rounds to 2. // For speed. Can be commented out
        } else if ui_a > 0x7FFF_BFFF {
            0xFFFF_FFFF_FFFF_FFFF
        } else {
            let (scale, bits) = P32E2::calculate_scale(ui_a);

            let mut i_z: u64 = (((bits | 0x1000_0000) & 0x1FFF_FFFF) as u64) << 34; // Left-justify fraction in 32-bit result (one left bit padding)

            if scale < 62 {
                let mut mask = 0x4000_0000_0000_0000_u64 >> scale; // Point to the last bit of the integer part.

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
                i_z >>= 62 - scale; // Right-justify the integer.
            } else if scale > 62 {
                i_z <<= scale - 62;
            }
            i_z
        }
    }
}

impl From<i32> for P32E2 {
    #[inline]
    fn from(mut a: i32) -> Self {
        let sign = (a >> 31) != 0;
        if sign {
            a = -a; // &0xFFFF_FFFF;
        }
        P32E2::from_bits(convert_u32_to_p32bits(a as u32).with_sign(sign))
    }
}

impl From<u32> for P32E2 {
    #[inline]
    fn from(a: u32) -> Self {
        P32E2::from_bits(convert_u32_to_p32bits(a))
    }
}

impl From<i64> for P32E2 {
    #[inline]
    fn from(mut a: i64) -> Self {
        let sign = (a >> 63) != 0;
        if sign {
            a = -a;
        }
        P32E2::from_bits(convert_u64_to_p32bits(a as u64).with_sign(sign))
    }
}

impl From<u64> for P32E2 {
    #[inline]
    fn from(a: u64) -> Self {
        P32E2::from_bits(convert_u64_to_p32bits(a))
    }
}

fn convert_u32_to_p32bits(a: u32) -> u32 {
    let mut mask = 0x8000_0000_u32;
    // NaR
    if a == 0x8000_0000 {
        a
    } else if a > 0xFFFF_FBFF {
        // 4294966271
        0x7FC0_0000 // 4294967296
    } else if a < 0x2 {
        a << 30
    } else {
        let mut frac_a = a;
        // length of bit (e.g. 4294966271) in int
        // (32 but because we have only 32 bits, so one bit off to accomdate that fact)
        let mut log2 = 31_i8;
        while (frac_a & mask) == 0 {
            log2 -= 1;
            frac_a <<= 1;
        }

        let k = log2 >> 2;

        let exp_a: u32 = ((log2 & 0x3) as u32) << (27 - k);
        frac_a ^= mask;

        let mut ui_a = (0x7FFF_FFFF ^ (0x3FFF_FFFF >> k)) | exp_a | frac_a >> (k + 4);

        mask = 0x8 << k; //bit_n_plus_one

        if ((mask & frac_a) != 0) && ((((mask - 1) & frac_a) | ((mask << 1) & frac_a)) != 0) {
            ui_a += 1;
        }
        ui_a
    }
}

fn convert_u64_to_p32bits(a: u64) -> u32 {
    let mut mask = 0x8000_0000_0000_0000_u64;
    // NaR
    if a == 0x8000_0000_0000_0000 {
        0x8000_0000
    } else if a > 0xFFFB_FFFF_FFFF_FBFF {
        // 18445618173802707967
        0x7FFF_C000 // 18446744073709552000
    } else if a < 0x2 {
        (a << 30) as u32
    } else {
        let mut frac_a = a;
        // length of bit (e.g. 18445618173802707967) in int
        // (64 but because we have only 64 bits, so one bit off to accommodate that fact)
        let mut log2 = 63_i8;
        while (frac_a & mask) == 0 {
            log2 -= 1;
            frac_a <<= 1;
        }

        let k = log2 >> 2;

        let exp_a: u32 = ((log2 & 0x3) as u32) << (27 - k);
        frac_a ^= mask;

        let mut ui_a: u64 =
            (0x7FFF_FFFF ^ (0x3FFF_FFFF >> k)) as u64 | exp_a as u64 | (frac_a >> (k + 36));

        mask = 0x8_0000_0000 << k; //bit_n_plus_one

        if ((mask & frac_a) != 0) && ((((mask - 1) & frac_a) | ((mask << 1) & frac_a)) != 0) {
            ui_a += 1;
        }
        ui_a as u32
    }
}

impl From<Q32E2> for P32E2 {
    fn from(q_a: Q32E2) -> Self {
        let mut bits_more = false;
        let mut frac64_a = 0_u64;

        if q_a.is_zero() {
            return ZERO;
        } else if q_a.is_nan() {
            return NAN;
        }

        let mut u_z = q_a.to_bits();

        let sign = (u_z[0] >> 63) != 0;

        if sign {
            let mut j = u_z.iter_mut().rev();
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
        //minpos and maxpos

        let mut no_lz = 0_isize;

        let mut j = u_z.iter_mut().enumerate();
        while let Some((i, u)) = j.next() {
            if *u == 0 {
                no_lz += 64;
            } else {
                let mut tmp = *u;
                let mut no_lztmp = 0_isize;

                while (tmp >> 63) == 0 {
                    no_lztmp += 1;
                    tmp <<= 1;
                }

                no_lz += no_lztmp;
                frac64_a = tmp;
                if (i != 7) && (no_lztmp != 0) {
                    let (_, w) = j.next().unwrap();
                    frac64_a += *w >> (64 - no_lztmp);
                    if (*w & ((0x1_u64 << (64 - no_lztmp)) - 1)) != 0 {
                        bits_more = true;
                    }
                }
                while let Some((_, w)) = j.next() {
                    if *w > 0 {
                        bits_more = true;
                        break;
                    }
                }
                break;
            }
        }

        //default dot is between bit 271 and 272, extreme left bit is bit 0. Last right bit is bit 511.
        //Equations derived from quire32_mult  last_pos = 271 - (k_a<<2) - exp_a and first_pos = last_pos - frac_len
        let k_a = ((271 - no_lz) >> 2) as i8;
        let mut exp_a = 271 - (no_lz as i32) - ((k_a << 2) as i32);

        let (regime, reg_sa, reg_a) = P32E2::calculate_regime(k_a);

        let u_a = if reg_a > 30 {
            //max or min pos. exp and frac does not matter.
            if reg_sa {
                0x7FFF_FFFF
            } else {
                0x1
            }
        } else {
            //remove hidden bit
            frac64_a &= 0x7FFF_FFFF_FFFF_FFFF;

            let shift = reg_a + 35; //2 es bit, 1 sign bit and 1 r terminating bit , 31+4

            let mut frac_a = (frac64_a >> shift) as u32;
            let mut bit_n_plus_one = false;
            if reg_a <= 28 {
                bit_n_plus_one = ((frac64_a >> (shift - 1)) & 0x1) != 0;
                exp_a <<= 28 - reg_a;
                if (frac64_a << (65 - shift)) != 0 {
                    bits_more = true;
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
                if frac64_a > 0 {
                    frac_a = 0;
                    bits_more = true;
                }
            }

            let mut u_a = P32E2::pack_to_ui(regime, exp_a as u32, frac_a);
            if bit_n_plus_one {
                u_a += (u_a & 1) | (bits_more as u32);
            }
            u_a
        };
        P32E2::from_bits(u_a.with_sign(sign))
    }
}
