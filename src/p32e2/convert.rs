use super::P32E2;
use crate::WithSign;
use core::convert::From;
use core::f64;

crate::impl_convert!(P32E2);

impl From<f32> for P32E2 {
    fn from(float: f32) -> Self {
        Self::from(float as f64)
    }
}

impl From<f64> for P32E2 {
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
        } else if float >= 1.329_227_995_784_916_e36 {
            //maxpos
            return Self::MAX;
        } else if float <= -1.329_227_995_784_916_e36 {
            // -maxpos
            return Self::MIN;
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
                    frac = crate::convert_fraction_p32(
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
                frac = crate::convert_fraction_p32(
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
        Self::from_bits(u_z)
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
        } else if p_a.is_nar() {
            f64::NAN
        } else {
            let sign_a = ui_a & P32E2::SIGN_MASK;
            if sign_a != 0 {
                ui_a = ui_a.wrapping_neg();
            }
            let (k_a, tmp) = P32E2::separate_bits_tmp(ui_a);

            let frac_a = ((tmp << 3) as u64) << 20;
            let exp_a = (((k_a as u64) << 2) + ((tmp >> 29) as u64)).wrapping_add(1023) << 52;

            f64::from_bits(exp_a + frac_a + ((sign_a as u64) << 32))
        }
    }
}

impl From<P32E2> for i32 {
    #[inline]
    fn from(p_a: P32E2) -> Self {
        if p_a.is_nar() {
            return i32::min_value();
        }

        let mut ui_a = p_a.to_bits();

        let sign = (ui_a & 0x8000_0000) != 0;
        if sign {
            ui_a = ui_a.wrapping_neg();
        }

        let i_z = convert_p32bits_to_u32(ui_a);

        i_z.with_sign(sign) as i32
    }
}

impl From<P32E2> for u32 {
    #[inline]
    fn from(p_a: P32E2) -> Self {
        if p_a.is_nar() {
            return 0x8000_0000; // Error: Should be u32::max_value()
        }

        let ui_a = p_a.to_bits();

        //negative
        if ui_a > 0x8000_0000 {
            return 0;
        }
        convert_p32bits_to_u32(ui_a)
    }
}

fn convert_p32bits_to_u32(ui_a: u32) -> u32 {
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

        let mut i_z64 = (((bits as u64) | 0x1000_0000) & 0x1FFF_FFFF) << 34; // Left-justify fraction in 32-bit result (one left bit padding)
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

impl From<P32E2> for i64 {
    #[inline]
    fn from(p_a: P32E2) -> Self {
        let mut ui_a = p_a.to_bits();

        if ui_a == 0x8000_0000 {
            return (ui_a as i64) << 32;
        }

        let sign = (ui_a & 0x8000_0000) != 0;
        if sign {
            ui_a = ui_a.wrapping_neg();
        }

        let i_z = convert_p32bits_to_u64(ui_a);

        i_z.with_sign(sign) as i64
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
        } else if ui_a > 0x8000_0000 {
            0
        } else {
            convert_p32bits_to_u64(ui_a)
        }
    }
}

fn convert_p32bits_to_u64(ui_a: u32) -> u64 {
    if ui_a <= 0x3800_0000 {
        0 // 0 <= |pA| <= 1/2 rounds to zero.
    } else if ui_a < 0x4400_0000 {
        1 // 1/2 < x < 3/2 rounds to 1.
    } else if ui_a <= 0x4A00_0000 {
        2 // 3/2 <= x <= 5/2 rounds to 2. // For speed. Can be commented out
    } else if ui_a > 0x7FFF_BFFF {
        0xFFFF_FFFF_FFFF_FFFF
    } else {
        let (scale, bits) = P32E2::calculate_scale(ui_a);

        let mut i_z: u64 = (((bits as u64) | 0x1000_0000) & 0x1FFF_FFFF) << 34; // Left-justify fraction in 32-bit result (one left bit padding)

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
            i_z >> (62 - scale) // Right-justify the integer.
        } else if scale > 62 {
            i_z << (scale - 62)
        } else {
            i_z
        }
    }
}

impl From<i32> for P32E2 {
    #[inline]
    fn from(mut i_a: i32) -> Self {
        if i_a < -2_147_483_135 {
            //-2147483648 to -2147483136 rounds to P32 value -2147483648
            return Self::from_bits(0x_8050_0000);
        }
        if i_a > 2_147_483_135 {
            //2147483136 to 2147483647 rounds to P32 value (2147483648)=> 0x7FB00000
            return Self::from_bits(0x_7FB0_0000);
        }

        let sign = i_a.is_negative();
        if sign {
            i_a = -i_a;
        }
        Self::from_bits(convert_u32_to_p32bits(i_a as u32).with_sign(sign))
    }
}

impl From<u32> for P32E2 {
    #[inline]
    fn from(a: u32) -> Self {
        Self::from_bits(convert_u32_to_p32bits(a))
    }
}

impl From<i64> for P32E2 {
    #[inline]
    fn from(mut i_a: i64) -> Self {
        if i_a < -9_222_809_086_901_354_495 {
            //-9222809086901354496 to -9223372036854775808 will be P32 value -9223372036854775808
            return Self::from_bits(0x_8000_5000);
        }
        if i_a > 9_222_809_086_901_354_495 {
            //9222809086901354496 to 9223372036854775807 will be P32 value 9223372036854775808
            return Self::from_bits(0x_7FFF_B000); // 9223372036854775808
        }
        let sign = i_a.is_negative();
        if sign {
            i_a = -i_a;
        }
        Self::from_bits(convert_u64_to_p32bits(i_a as u64).with_sign(sign))
    }
}

impl From<u64> for P32E2 {
    #[inline]
    fn from(a: u64) -> Self {
        Self::from_bits(convert_u64_to_p32bits(a))
    }
}

fn convert_u32_to_p32bits(a: u32) -> u32 {
    let mut mask = 0x8000_0000_u32;
    // NaR
    if a > 0xFFFF_FBFF {
        // 4294966271
        0x7FC0_0000 // 4294967296
    } else if a < 0x2 {
        a << 30
    } else {
        let mut frac_a = a;
        // length of bit (e.g. 4294966271) in int
        // (32 but because we have only 32 bits, so one bit off to accommodate that fact)
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
    if a > 0xFFFB_FFFF_FFFF_FBFF {
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

#[test]
fn convert_p32_f64() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..100_000 {
        let p: P32E2 = rng.gen();
        let f = f64::from(p);
        assert_eq!(p, P32E2::from(f));
    }
}

#[test]
fn convert_p32_i32() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..100_000 {
        let p: P32E2 = rng.gen();
        let f = f64::from(p).round();
        if p % P32E2::new(0x_3800_0000) == P32E2::ZERO {
            continue;
        }
        assert_eq!(i32::from(p), f as i32);
    }
}

#[test]
fn convert_p32_i64() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..100_000 {
        let p: P32E2 = rng.gen();
        let f = f64::from(p).round();
        if p % P32E2::new(0x_3800_0000) == P32E2::ZERO {
            continue;
        }
        assert_eq!(i64::from(p), f as i64);
    }
}
