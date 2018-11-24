use super::*;
use core::convert::From;
use core::f64;
use crate::WithSign;

fn check_extra_two_bits_p8(
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

fn convert_fraction_p8(
    mut float: f64,
    mut frac_length: u8,
    bits_n_plus_one: &mut bool,
    bits_more: &mut bool,
) -> u8 {
    let mut frac = 0_u8;

    if float == 0. {
        return 0;
    } else if float == f64::INFINITY {
        return 0x80;
    }

    float -= 1.; //remove hidden bit
    if frac_length == 0 {
        check_extra_two_bits_p8(float, 1., bits_n_plus_one, bits_more);
    } else {
        let mut temp = 1_f64;
        loop {
            temp /= 2.;
            if temp <= float {
                float -= temp;
                frac_length -= 1;
                frac = (frac << 1) + 1; //shift in one
                if float == 0. {
                    //put in the rest of the bits
                    frac <<= frac_length as u8;
                    break;
                }

                if frac_length == 0 {
                    check_extra_two_bits_p8(float, temp, bits_n_plus_one, bits_more);

                    break;
                }
            } else {
                frac <<= 1; //shift in a zero
                frac_length -= 1;
                if frac_length == 0 {
                    check_extra_two_bits_p8(float, temp, bits_n_plus_one, bits_more);
                    break;
                }
            }
        }
    }
    frac
}

impl From<f32> for P8E0 {
    fn from(float: f32) -> Self {
        P8E0::from(float as f64)
    }
}

impl From<f64> for P8E0 {
    fn from(mut float: f64) -> Self {
        let mut reg: u8;
        let mut bit_n_plus_one = false;
        let mut bits_more = false;

        if float == 0. {
            return P8E0::new(0);
        } else if !float.is_finite() {
            return INFINITY;
        } else if float >= 64. {
            //maxpos
            return MAX;
        } else if float <= -64. {
            // -maxpos
            return MIN;
        }

        let sign = float < 0.;
        // sign: 1 bit, frac: 8 bits, mantisa: 23 bits
        //sign = a.parts.sign;
        //frac = a.parts.fraction;
        //exp = a.parts.exponent;

        let u_z: u8 = if float == 0. {
            0
        } else if float == 1. {
            0x40
        } else if float == -1. {
            0xC0
        } else if (float <= 0.015_625) && !sign {
            //minpos
            0x1
        } else if (float >= -0.015_625) && sign {
            //-minpos
            0xFF
        } else if (float > 1.) || (float < -1.) {
            if sign {
                //Make negative numbers positive for easier computation
                float = -float;
            }
            reg = 1; //because k = m-1; so need to add back 1
                     // minpos
            if float <= 0.015_625 {
                1
            } else {
                //regime
                while float >= 2. {
                    float *= 0.5;
                    reg += 1;
                }

                //rounding off regime bits
                if reg > 6 {
                    0x7F
                } else {
                    let frac_length = 6 - reg;
                    let frac = convert_fraction_p8(
                        float,
                        frac_length,
                        &mut bit_n_plus_one,
                        &mut bits_more,
                    );
                    let regime = 0x7F - (0x7F >> reg);
                    let mut u_z = P8E0::pack_to_ui(regime, frac);
                    if bit_n_plus_one {
                        u_z += (u_z & 1) | (bits_more as u8);
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
            reg = 0;

            //regime
            //printf("here we go\n");
            while float < 1. {
                float *= 2.;
                reg += 1;
            }
            //rounding off regime bits
            if reg > 6 {
                0x1
            } else {
                let frac_length = 6 - reg;
                let frac =
                    convert_fraction_p8(float, frac_length, &mut bit_n_plus_one, &mut bits_more);
                let regime = 0x40 >> reg;
                let mut u_z = P8E0::pack_to_ui(regime, frac);
                if bit_n_plus_one {
                    u_z += (u_z & 1) | (bits_more as u8);
                }
                u_z
            }
            .with_sign(sign)
        } else {
            //NaR - for NaN, INF and all other combinations
            0x80
        };
        P8E0::from_bits(u_z)
    }
}

#[cfg(feature="float_convert")]
impl From<P8E0> for f32 {
    #[inline]
    fn from(a: P8E0) -> Self {
        f64::from(a) as f32
    }
}

#[cfg(feature="float_convert")]
impl From<P8E0> for f64 {
    #[inline]
    fn from(a: P8E0) -> Self {
        let mut u_z = a.to_bits();

        if u_z == 0 {
            return 0.;
        } else if u_z == 0x7F {
            //maxpos
            return 64.;
        } else if u_z == 0x81 {
            //-maxpos
            return -64.;
        } else if u_z == 0x80 {
            //NaR
            return f64::INFINITY;
        }

        let sign = P8E0::sign_ui(u_z);
        if sign {
            u_z = u_z.wrapping_neg()
        };
        let reg_s = P8E0::sign_reg_ui(u_z);

        let mut shift = 2_u8;
        let mut k = 0_i8;
        let mut tmp = u_z<<2 /* & 0xFF*/;
        let reg = if reg_s {
            while (tmp >> 7) != 0 {
                k += 1;
                shift += 1;
                tmp <<= 1 /* & 0xFF*/;
            }
            k + 1
        } else {
            k = -1;
            while (tmp >> 7) == 0 {
                k -= 1;
                shift += 1;
                tmp <<= 1 /* & 0xFF*/;
            }
            tmp &= 0x7F;
            -k
        } as u8;
        let frac = (tmp & 0x7F) >> shift;

        let fraction_max = libm::pow(2., (6 - reg) as f64);
        let d8 = (libm::pow(2., k as f64) * (1. + ((frac as f64) / fraction_max))) as f64;

        if sign {
            -d8
        } else {
            d8
        }
    }
}

impl From<P8E0> for i32 {
    #[inline]
    fn from(p_a: P8E0) -> Self {
        let mut i_z: i32;

        let mut ui_a = p_a.to_bits();

        //NaR
        if ui_a == 0x80 {
            return -0x8000_0000;
        }

        let sign = ui_a > 0x80; // sign is True if p_a > NaR.
        if sign {
            ui_a = ui_a.wrapping_neg(); // A is now |A|.
        }
        if ui_a <= 0x20 {
            // 0 <= |p_a| <= 1/2 rounds to zero.
            return 0;
        } else if ui_a < 0x50 {
            // 1/2 < x < 3/2 rounds to 1.
            i_z = 1;
        } else {
            let (scale, bits) = P8E0::calculate_scale(ui_a);

            i_z = ((bits | 0x40) as i32) << 24; // Left-justify fraction in 32-bit result (one left bit padding)
            let mut mask = 0x4000_0000_i32 >> scale; // Point to the last bit of the integer part.

            let bit_last = (i_z & mask) != 0; // Extract the bit, without shifting it.
            mask >>= 1;
            let mut tmp = i_z & mask;
            let bit_n_plus_one = tmp != 0; // "True" if nonzero.
            i_z ^= tmp; // Erase the bit, if it was set.
            tmp = i_z & (mask - 1); // tmp has any remaining bits. // This is bits_more
            i_z ^= tmp; // Erase those bits, if any were set.

            if bit_n_plus_one {
                // logic for round to nearest, tie to even
                if (bit_last as i32 | tmp) != 0 {
                    i_z += mask << 1;
                }
            }

            i_z >>= 30 - scale; // Right-justify the integer.
        }

        if sign {
            i_z = -i_z; // Apply the sign of the input.
        }
        i_z
    }
}

impl From<P8E0> for i64 {
    #[inline]
    fn from(p_a: P8E0) -> Self {
        let mut i_z: i64;

        let mut ui_a = p_a.to_bits();

        //NaR
        if ui_a == 0x80 {
            return -0x8000_0000_0000_0000;
        }

        let sign = (ui_a >> 7) != 0;
        if sign {
            ui_a = ui_a.wrapping_neg();
        }

        if ui_a <= 0x20 {
            // 0 <= |p_a| <= 1/2 rounds to zero.
            return 0;
        } else if ui_a < 0x50 {
            // 1/2 < x < 3/2 rounds to 1.
            i_z = 1;
        } else {
            let (scale, bits) = P8E0::calculate_scale(ui_a);

            i_z = (((bits as u64) | 0x40) << 55) as i64; // Left-justify fraction in 32-bit result (one left bit padding)

            let mut mask = 0x2000_0000_0000_0000_i64 >> scale; // Point to the last bit of the integer part.

            let bit_last = (i_z & mask) != 0; // Extract the bit, without shifting it.
            mask >>= 1;
            let mut tmp = i_z & mask;
            let bit_n_plus_one = tmp != 0; // "True" if nonzero.
            i_z ^= tmp; // Erase the bit, if it was set.
            tmp = i_z & (mask - 1); // tmp has any remaining bits. // This is bits_more
            i_z ^= tmp; // Erase those bits, if any were set.

            if bit_n_plus_one {
                // logic for round to nearest, tie to even
                if (bit_last as i64 | tmp) != 0 {
                    i_z += mask << 1;
                }
            }
            i_z = ((i_z as u64) >> (61 - scale)) as i64; // Right-justify the integer.
        }

        if sign {
            i_z = -i_z; // Apply the sign of the input.
        }
        i_z
    }
}

impl From<P8E0> for u32 {
    #[inline]
    fn from(p_a: P8E0) -> Self {
        let mut i_z: u32;

        let ui_a = p_a.to_bits();

        //NaR
        if ui_a == 0x80 {
            return 0x8000_0000;
        } else if ui_a > 0x80 {
            return 0; //negative
        }
        if ui_a <= 0x20 {
            // 0 <= |p_a| <= 1/2 rounds to zero.
            return 0;
        } else if ui_a < 0x50 {
            // 1/2 < x < 3/2 rounds to 1.
            i_z = 1;
        } else {
            let (scale, bits) = P8E0::calculate_scale(ui_a);

            i_z = ((bits | 0x40) as u32) << 24; // Left-justify fraction in 32-bit result (one left bit padding)

            let mut mask = 0x4000_0000_u32 >> scale; // Point to the last bit of the integer part.

            let bit_last = (i_z & mask) != 0; // Extract the bit, without shifting it.
            mask >>= 1;
            let mut tmp = i_z & mask;
            let bit_n_plus_one = tmp != 0; // "True" if nonzero.
            i_z ^= tmp; // Erase the bit, if it was set.
            tmp = i_z & (mask - 1); // tmp has any remaining bits. // This is bits_more
            i_z ^= tmp; // Erase those bits, if any were set.

            if bit_n_plus_one {
                // logic for round to nearest, tie to even
                if (bit_last as u32 | tmp) != 0 {
                    i_z += mask << 1;
                }
            }
            i_z >>= 30 - scale; // Right-justify the integer.
        }

        i_z
    }
}

impl From<P8E0> for u64 {
    #[inline]
    fn from(p_a: P8E0) -> Self {
        let mut i_z: u64;

        let ui_a = p_a.to_bits();
        //NaR
        if ui_a == 0x80 {
            return 0x8000_0000_0000_0000;
        } else if ui_a > 0x80 {
            return 0; //negative
        }
        if ui_a <= 0x20 {
            // 0 <= |p_a| <= 1/2 rounds to zero.
            return 0;
        } else if ui_a < 0x50 {
            // 1/2 < x < 3/2 rounds to 1.
            i_z = 1;
        } else {
            let (scale, bits) = P8E0::calculate_scale(ui_a);

            i_z = ((bits as u64) | 0x40) << 55; // Left-justify fraction in 32-bit result (one left bit padding)

            let mut mask = 0x2000_0000_0000_0000_u64 >> scale; // Point to the last bit of the integer part.

            let bit_last = (i_z & mask) != 0; // Extract the bit, without shifting it.
            mask >>= 1;
            let mut tmp = i_z & mask;
            let bit_n_plus_one = tmp != 0; // "True" if nonzero.
            i_z ^= tmp; // Erase the bit, if it was set.
            tmp = i_z & (mask - 1); // tmp has any remaining bits. // This is bits_more
            i_z ^= tmp; // Erase those bits, if any were set.

            if bit_n_plus_one {
                // logic for round to nearest, tie to even
                if (bit_last as u64 | tmp) != 0 {
                    i_z += mask << 1;
                }
            }
            i_z >>= 61 - scale; // Right-justify the integer.
        }

        i_z
    }
}

impl From<u32> for P8E0 {
    #[inline]
    fn from(a: u32) -> Self {
        P8E0::from_bits(convert_u32_to_p8bits(a))
    }
}

impl From<i32> for P8E0 {
    #[inline]
    fn from(mut a: i32) -> Self {
        let sign = (a >> 31) != 0;
        if sign {
            a = -a;
        }
        P8E0::from_bits(convert_u32_to_p8bits(a as u32).with_sign(sign))
    }
}

impl From<u64> for P8E0 {
    #[inline]
    fn from(a: u64) -> Self {
        P8E0::from_bits(convert_u64_to_p8bits(a))
    }
}

impl From<i64> for P8E0 {
    #[inline]
    fn from(mut a: i64) -> Self {
        let sign = (a >> 63) != 0;
        if sign {
            a = -a;
        }
        P8E0::from_bits(convert_u64_to_p8bits(a as u64).with_sign(sign))
    }
}

fn convert_u32_to_p8bits(a: u32) -> u8 {
    if a == 0x8000_0000 {
        0x80
    } else if a > 48 {
        0x7F
    } else if a < 2 {
        (a << 6) as u8
    } else {
        let mut log2 = 6_i8; //length of bit
        let mut mask = 0x40_u32;
        let mut frac_a = a;
        while (frac_a & mask) == 0 {
            log2 -= 1;
            frac_a <<= 1;
        }

        let k = log2;

        frac_a ^= mask;

        let mut ui_a: u8 = (0x7F ^ (0x3F >> k)) | (frac_a >> (k + 1)) as u8;

        mask = 0x1 << k; //bit_n_plus_one
        if ((mask & frac_a) != 0) && ((((mask - 1) & frac_a) | ((mask << 1) & frac_a)) != 0) {
            ui_a += 1;
        }
        ui_a
    }
}

fn convert_u64_to_p8bits(a: u64) -> u8 {
    if a == 0x8000_0000_0000_0000 {
        0x80
    } else if a > 48 {
        0x7F
    } else if a < 2 {
        (a << 6) as u8
    } else {
        let mut log2 = 6_i8; //length of bit
        let mut mask = 0x40_u64;
        let mut frac_a = a;
        while (frac_a & mask) == 0 {
            log2 -= 1;
            frac_a <<= 1;
        }

        let k = log2;

        frac_a ^= mask;

        let mut ui_a: u8 = (0x7F ^ (0x3F >> k)) | (frac_a >> (k + 1)) as u8;

        mask = 0x1 << k; //bit_n_plus_one
        if ((mask & frac_a) != 0) && ((((mask - 1) & frac_a) | ((mask << 1) & frac_a)) != 0) {
            ui_a += 1;
        }
        ui_a
    }
}
