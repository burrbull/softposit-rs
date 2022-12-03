use super::P16E1;
use crate::WithSign;
use core::{f32, f64};

crate::impl_convert!(P16E1);

impl From<i32> for P16E1 {
    #[inline]
    fn from(mut i_a: i32) -> Self {
        if i_a < -134_217_728 {
            //-2147483648 to -134217729 rounds to P32 value -268435456
            return Self::MIN;
        }
        let sign = i_a.is_negative();
        if sign {
            i_a = -i_a;
        }
        Self::from_bits(convert_u32_to_p16bits(i_a as u32).with_sign(sign))
    }
}

impl From<u32> for P16E1 {
    #[inline]
    fn from(a: u32) -> Self {
        Self::from_bits(convert_u32_to_p16bits(a))
    }
}

fn convert_u32_to_p16bits(a: u32) -> u16 {
    if a > 0x0800_0000 {
        0x7FFF
    } else if a > 0x02FF_FFFF {
        0x7FFE
    } else if a < 2 {
        (a as u16) << 14
    } else {
        let mut frac_a = a;
        let mask = 0x0200_0000_u32;
        let mut log2 = 25_i8;
        while (frac_a & mask) == 0 {
            log2 -= 1;
            frac_a <<= 1;
        }
        let k: i8 = log2 >> 1;
        let exp_a: u32 = ((log2 & 0x1) as u32) << (12 - k);
        frac_a ^= mask;

        let mut ui_a = ((0x7FFF ^ (0x3FFF >> k)) | exp_a | (frac_a >> (k + 13))) as u16;
        let mask = 0x1000 << k; //bit_n_plus_one
        if ((mask & frac_a) != 0) && ((((mask - 1) & frac_a) | ((mask << 1) & frac_a)) != 0) {
            ui_a += 1;
        }
        ui_a
    }
}

impl From<i64> for P16E1 {
    #[inline]
    fn from(mut i_a: i64) -> Self {
        if i_a < -134_217_728 {
            //-2147483648 to -134217729 rounds to P32 value -268435456
            return Self::MIN;
        }
        let sign = i_a.is_negative();
        if sign {
            i_a = -i_a;
        }
        Self::from_bits(convert_u64_to_p16bits(i_a as u64).with_sign(sign))
    }
}

impl From<u64> for P16E1 {
    #[inline]
    fn from(a: u64) -> Self {
        Self::from_bits(convert_u64_to_p16bits(a))
    }
}

fn convert_u64_to_p16bits(a: u64) -> u16 {
    if a > 0x0000_0000_0800_0000 {
        0x7FFF
    } else if a > 0x0000_0000_02FF_FFFF {
        0x7FFE
    } else if a < 2 {
        (a as u16) << 14
    } else {
        let mut mask = 0x0000_0000_0200_0000_u64;
        let mut frac_a = a;
        let mut log2 = 25_i8;
        while (frac_a & mask) == 0 {
            log2 -= 1;
            frac_a <<= 1;
        }
        let k = log2 >> 1;
        let exp_a = ((log2 & 0x1) << (12 - k)) as u64;
        frac_a ^= mask;
        let mut ui_a = ((0x7FFF ^ (0x3FFF >> k)) | exp_a | (frac_a >> (k + 13))) as u16;
        mask = 0x1000 << k;
        if ((mask & frac_a) != 0) && ((((mask - 1) & frac_a) | ((mask << 1) & frac_a)) != 0) {
            ui_a += 1;
        }
        ui_a
    }
}

impl From<P16E1> for i32 {
    #[inline]
    fn from(p_a: P16E1) -> Self {
        let mut ui_a = p_a.to_bits(); // Copy of the input.

        if ui_a == 0x8000 {
            return 0;
        }

        let sign = ui_a > 0x8000; // sign is True if pA > NaR.
        if sign {
            ui_a = ui_a.wrapping_neg(); // A is now |A|.
        }
        let i_z = convert_p16bits_to_u32(ui_a);

        i_z.with_sign(sign) as i32
    }
}

impl From<P16E1> for u32 {
    #[inline]
    fn from(p_a: P16E1) -> Self {
        let ui_a = p_a.to_bits(); // Copy of the input.

        if ui_a >= 0x8000 {
            return 0; //negative
        }
        convert_p16bits_to_u32(ui_a)
    }
}

fn convert_p16bits_to_u32(ui_a: u16) -> u32 {
    if ui_a <= 0x3000 {
        // 0 <= |pA| <= 1/2 rounds to zero.
        0
    } else if ui_a < 0x4800 {
        // 1/2 < x < 3/2 rounds to 1.
        1
    } else if ui_a <= 0x5400 {
        // 3/2 <= x <= 5/2 rounds to 2.
        2
    } else {
        let (scale, bits) = P16E1::calculate_scale(ui_a);

        let mut i_z = ((bits as u32) | 0x2000) << 17; // Left-justify fraction in 32-bit result (one left bit padding)
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
            if ((bit_last as u32) | tmp) != 0 {
                i_z += mask << 1;
            }
        }
        i_z >> (30 - scale) // Right-justify the integer.
    }
}

fn convert_p16bits_to_u64(ui_a: u16) -> u64 {
    if ui_a <= 0x3000 {
        0
    } else if ui_a < 0x4800 {
        1
    } else if ui_a <= 0x5400 {
        2
    } else {
        let (scale, bits) = P16E1::calculate_scale(ui_a);

        let mut i_z = ((bits as u64) | 0x2000) << 49;

        let mut mask = 0x4000_0000_0000_0000_u64 >> scale;

        let bit_last = (i_z & mask) != 0;
        mask >>= 1;
        let mut tmp = i_z & mask;
        let bit_n_plus_one = tmp != 0;
        i_z ^= tmp;
        tmp = i_z & (mask - 1); // bits_more
        i_z ^= tmp;

        if bit_n_plus_one && (((bit_last as u64) | tmp) != 0) {
            i_z += mask << 1;
        }
        i_z >> (62 - scale)
    }
}

impl From<P16E1> for i64 {
    #[inline]
    fn from(p_a: P16E1) -> Self {
        let mut ui_a = p_a.to_bits();

        // NaR
        if ui_a == 0x8000 {
            return 0;
        }

        let sign = (ui_a & 0x_8000) != 0;
        if sign {
            ui_a = ui_a.wrapping_neg();
        }

        let i_z = convert_p16bits_to_u64(ui_a);

        i_z.with_sign(sign) as i64
    }
}

impl From<P16E1> for u64 {
    #[inline]
    fn from(p_a: P16E1) -> Self {
        let ui_a = p_a.to_bits();

        if ui_a >= 0x8000 {
            return 0;
        }
        convert_p16bits_to_u64(ui_a)
    }
}

fn check_extra_two_bits_p16(
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
fn convert_fraction_p16(
    mut float: f64,
    mut frac_length: u8,
    bits_n_plus_one: &mut bool,
    bits_more: &mut bool,
) -> u16 {
    let mut frac = 0_u16;

    if float == 0. {
        return 0;
    } else if float == f64::INFINITY {
        return 0x8000;
    }

    float -= 1.; //remove hidden bit
    if frac_length == 0 {
        check_extra_two_bits_p16(float, 1., bits_n_plus_one, bits_more);
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
                    check_extra_two_bits_p16(float, temp, bits_n_plus_one, bits_more);
                    break;
                }
            } else {
                frac <<= 1; //shift in a zero
                frac_length -= 1;
                if frac_length == 0 {
                    check_extra_two_bits_p16(float, temp, bits_n_plus_one, bits_more);
                    break;
                }
            }
        }
    }

    frac
}

impl From<f32> for P16E1 {
    fn from(float: f32) -> Self {
        Self::from(float as f64)
    }
}

impl From<f64> for P16E1 {
    #[allow(clippy::cognitive_complexity)]
    fn from(mut float: f64) -> Self {
        let mut reg: u16;
        let mut bit_n_plus_one = false;
        let mut bits_more = false;
        let mut frac = 0_u16;
        let mut exp = 0_i8;

        if float == 0. {
            return Self::ZERO;
        } else if !float.is_finite() {
            return Self::NAR;
        } else if float >= 268_435_456. {
            //maxpos
            return Self::MAX;
        } else if float <= -268_435_456. {
            // -maxpos
            return Self::MIN;
        }

        let sign = float < 0.;

        let u_z: u16 = if float == 1. {
            0x4000
        } else if float == -1. {
            0xC000
        } else if (float <= 3.725_290_298_461_914_e-9) && !sign {
            //minpos
            1
        } else if (float >= -3.725_290_298_461_914_e-9) && sign {
            //-minpos
            0xFFFF
        } else if !(-1. ..=1.).contains(&float) {
            if sign {
                //Make negative numbers positive for easier computation
                float = -float;
            }

            reg = 1; //because k = m-1; so need to add back 1
                     // minpos
            if float <= 3.725_290_298_461_914_e-9 {
                1
            } else {
                //regime
                while float >= 4. {
                    float *= 0.25;
                    reg += 1;
                }
                if float >= 2. {
                    float *= 0.5;
                    exp += 1;
                }

                let frac_length = 13 - (reg as i8);

                if frac_length < 0 {
                    //reg == 14, means rounding bits is exp and just the rest.
                    if float > 1. {
                        bits_more = true;
                    }
                } else {
                    frac = convert_fraction_p16(
                        float,
                        frac_length as u8,
                        &mut bit_n_plus_one,
                        &mut bits_more,
                    );
                }
                if (reg == 14) && (frac > 0) {
                    bits_more = true;
                    frac = 0;
                }
                if reg > 14 {
                    0x7FFF
                } else {
                    let regime = ((1_u16 << reg) - 1) << 1;
                    let ex = if reg == 14 {
                        0
                    } else {
                        (exp as u16) << (13 - reg)
                    };
                    let mut u_z = ((regime as u16) << (14 - reg)) + ex + frac;
                    //n+1 frac bit is 1. Need to check if another bit is 1 too if not round to even
                    if (reg == 14) && (exp != 0) {
                        bit_n_plus_one = true;
                    }
                    u_z += ((bit_n_plus_one as u16) & (u_z & 1))
                        | ((bit_n_plus_one & bits_more) as u16);
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
            while float < 1. {
                float *= 4.;
                reg += 1;
            }
            if float >= 2. {
                float /= 2.;
                exp += 1;
            }
            if reg == 14 {
                bit_n_plus_one = exp != 0;
                if frac > 1 {
                    bits_more = true;
                }
            } else {
                //only possible combination for reg=15 to reach here is 7FFF (maxpos) and FFFF (-minpos)
                //but since it should be caught on top, so no need to handle
                let frac_length = 13 - reg;
                frac = convert_fraction_p16(
                    float,
                    frac_length as u8,
                    &mut bit_n_plus_one,
                    &mut bits_more,
                );
            }

            if (reg == 14) && (frac > 0) {
                bits_more = true;
                frac = 0;
            }
            if reg > 14 {
                0x1
            } else {
                let regime = 1_u16;
                let ex = if reg == 14 {
                    0
                } else {
                    (exp as u16) << (13 - reg)
                };
                let mut u_z = ((regime as u16) << (14 - reg)) + ex + frac;
                //n+1 frac bit is 1. Need to check if another bit is 1 too if not round to even
                if (reg == 14) && (exp != 0) {
                    bit_n_plus_one = true;
                }
                u_z +=
                    ((bit_n_plus_one as u16) & (u_z & 1)) | ((bit_n_plus_one & bits_more) as u16);
                u_z
            }
            .with_sign(sign)
        } else {
            //NaR - for NaN, INF and all other combinations
            0x8000
        };
        Self::from_bits(u_z)
    }
}

impl From<P16E1> for f32 {
    #[inline]
    fn from(p_a: P16E1) -> Self {
        let mut ui_a = p_a.to_bits();

        if p_a.is_zero() {
            0.
        } else if p_a.is_nar() {
            f32::NAN
        } else {
            let sign_a = ui_a & P16E1::SIGN_MASK;
            if sign_a != 0 {
                ui_a = ui_a.wrapping_neg();
            }
            let (k_a, tmp) = P16E1::separate_bits_tmp(ui_a);

            let frac_a = ((tmp << 2) as u32) << 7;
            let exp_a = (((k_a as u32) << 1) + ((tmp >> 14) as u32)).wrapping_add(127) << 23;

            f32::from_bits(exp_a + frac_a + ((sign_a as u32) << 16))
        }
    }
}

impl From<P16E1> for f64 {
    #[inline]
    fn from(p_a: P16E1) -> Self {
        let mut ui_a = p_a.to_bits();

        if p_a.is_zero() {
            0.
        } else if p_a.is_nar() {
            f64::NAN
        } else {
            let sign_a = ui_a & P16E1::SIGN_MASK;
            if sign_a != 0 {
                ui_a = ui_a.wrapping_neg();
            }
            let (k_a, tmp) = P16E1::separate_bits_tmp(ui_a);

            let frac_a = ((tmp << 2) as u64) << 36;
            let exp_a = (((k_a as u64) << 1) + ((tmp >> 14) as u64)).wrapping_add(1023) << 52;

            f64::from_bits(exp_a + frac_a + ((sign_a as u64) << 48))
        }
    }
}

#[test]
fn convert_p16_f64() {
    for n in -0x_8000_i16..0x_7fff {
        let p = P16E1::new(n);
        let f = f64::from(p);
        assert_eq!(p, P16E1::from(f));
    }
}

#[test]
fn convert_p16_f32() {
    for n in -0x_8000_i16..0x_7fff {
        let p = P16E1::new(n);
        let f = f32::from(p);
        assert_eq!(p, P16E1::from(f));
    }
}

#[test]
fn convert_p16_i32() {
    for n in -0x_8000_i16..0x_7fff {
        let p = P16E1::new(n);
        let f = f64::from(p).round();
        if p % P16E1::new(0x_3000) == P16E1::ZERO {
            continue;
        }
        assert_eq!(i32::from(p), f as i32);
    }
}

#[test]
fn convert_p16_i64() {
    for n in -0x_8000_i16..0x_7fff {
        let p = P16E1::new(n);
        let f = f64::from(p).round();
        if p % P16E1::new(0x_3000) == P16E1::ZERO {
            continue;
        }
        assert_eq!(i64::from(p), f as i64);
    }
}
