use super::P32E2;
use crate::{u32_with_sign, u64_with_sign};
use core::f64;
use core::mem::transmute;

crate::macros::impl_convert!(P32E2);

impl P32E2 {
    pub const fn from_f32(float: f32) -> Self {
        use crate::RawFloat;
        let ui: u32 = unsafe { transmute(float) };

        let sign = (ui & f32::SIGN_MASK) != 0;

        let uip = ui & !f32::SIGN_MASK;
        // check zero
        if uip == 0 {
            Self::ZERO
        } else if uip >= 0x_7f80_0000 {
            Self::NAR
        } else if uip >= 0x_7b80_0000 {
            // +- 1.329_227_995_784_916_e36
            if !sign {
                Self::MAX
            } else {
                Self::MIN
            }
        } else if uip == 0x_3f80_0000 {
            // +- 1.
            if !sign {
                Self::ONE
            } else {
                Self::ONE.neg()
            }
        } else if uip <= 0x_0380_0000 {
            // +- 7.523_163_845_262_64_e-37
            if !sign {
                Self::MIN_POSITIVE
            } else {
                Self::MIN_POSITIVE.neg()
            }
        } else {
            Self::from_bits(crate::convert::convert_float!(P32E2, f32, ui, u64, i64))
        }
    }

    pub const fn from_f64(float: f64) -> Self {
        use crate::RawFloat;
        let ui: u64 = unsafe { transmute(float) };

        let sign = (ui & f64::SIGN_MASK) != 0;

        let uip = ui & !f64::SIGN_MASK;
        // check zero
        if uip == 0 {
            Self::ZERO
        } else if uip >= 0x_7ff0_0000_0000_0000 {
            Self::NAR
        } else if uip >= 0x_4770_0000_0000_0000 {
            // +- 1.329_227_995_784_916_e36
            if !sign {
                Self::MAX
            } else {
                Self::MIN
            }
        } else if uip == 0x_3ff0_0000_0000_0000 {
            // +- 1.
            if !sign {
                Self::ONE
            } else {
                Self::ONE.neg()
            }
        } else if uip <= 0x_3870_0000_0000_0000 {
            // +- 7.523_163_845_262_64_e-37
            if !sign {
                Self::MIN_POSITIVE
            } else {
                Self::MIN_POSITIVE.neg()
            }
        } else {
            Self::from_bits(crate::convert::convert_float!(P32E2, f64, ui))
        }
    }

    #[inline]
    pub const fn to_f32(self) -> f32 {
        self.to_f64() as f32
    }

    pub const fn to_f64(self) -> f64 {
        let mut ui_a = self.to_bits();

        if self.is_zero() {
            0.
        } else if self.is_nar() {
            f64::NAN
        } else {
            let sign_a = ui_a & P32E2::SIGN_MASK;
            if sign_a != 0 {
                ui_a = ui_a.wrapping_neg();
            }
            let (k_a, tmp) = P32E2::separate_bits_tmp(ui_a);

            let frac_a = ((tmp << 3) as u64) << 20;
            let exp_a = (((k_a as u64) << 2) + ((tmp >> 29) as u64)).wrapping_add(1023) << 52;

            unsafe { transmute(exp_a + frac_a + ((sign_a as u64) << 32)) }
        }
    }

    #[inline]
    pub const fn to_i32(self) -> i32 {
        if self.is_nar() {
            return i32::min_value();
        }

        let mut ui_a = self.to_bits();

        let sign = (ui_a & 0x8000_0000) != 0;
        if sign {
            ui_a = ui_a.wrapping_neg();
        }

        if ui_a > 0x_7faf_ffff {
            return if sign {
                i32::min_value()
            } else {
                i32::max_value()
            };
        };

        let i_z = convert_p32bits_to_u32(ui_a);

        u32_with_sign(i_z, sign) as i32
    }

    #[inline]
    pub const fn to_u32(self) -> u32 {
        if self.is_nar() {
            return 0x8000_0000; // Error: Should be u32::max_value()
        }

        let ui_a = self.to_bits();

        //negative
        if ui_a > 0x8000_0000 {
            return 0;
        }
        convert_p32bits_to_u32(ui_a)
    }

    #[inline]
    pub const fn to_i64(self) -> i64 {
        let mut ui_a = self.to_bits();

        if ui_a == 0x8000_0000 {
            return i64::min_value();
        }

        let sign = (ui_a & 0x8000_0000) != 0;
        if sign {
            ui_a = ui_a.wrapping_neg();
        }

        if ui_a > 0x_7fff_afff {
            return if sign {
                i64::min_value()
            } else {
                i64::max_value()
            };
        };

        let i_z = convert_p32bits_to_u64(ui_a);

        u64_with_sign(i_z, sign) as i64
    }

    #[inline]
    pub const fn to_u64(self) -> u64 {
        let ui_a = self.to_bits();

        //NaR
        if ui_a == 0x8000_0000 {
            0x8000_0000_0000_0000
        } else if ui_a > 0x8000_0000 {
            0
        } else {
            convert_p32bits_to_u64(ui_a)
        }
    }

    #[inline]
    pub const fn from_i32(mut i_a: i32) -> Self {
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
        Self::from_bits(u32_with_sign(convert_u32_to_p32bits(i_a as u32), sign))
    }

    #[inline]
    pub const fn from_u32(i_a: u32) -> Self {
        Self::from_bits(convert_u32_to_p32bits(i_a))
    }

    #[inline]
    pub const fn from_i64(mut i_a: i64) -> Self {
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
        Self::from_bits(u32_with_sign(convert_u64_to_p32bits(i_a as u64), sign))
    }

    #[inline]
    pub const fn from_u64(a: u64) -> Self {
        Self::from_bits(convert_u64_to_p32bits(a))
    }
}

const fn convert_p32bits_to_u32(ui_a: u32) -> u32 {
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

const fn convert_p32bits_to_u64(ui_a: u32) -> u64 {
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
        } else if scale > 64 {
            i_z << (scale - 62)
        } else {
            i_z
        }
    }
}

const fn convert_u32_to_p32bits(a: u32) -> u32 {
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

const fn convert_u64_to_p32bits(a: u64) -> u32 {
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
fn convert_f64_p32_rand() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..100_000 {
        let f: f64 = rng.gen();
        let _p = P32E2::from(f);
    }
}

#[test]
fn convert_f32_p32_rand() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..100_000 {
        let f: f32 = rng.gen();
        let _p = P32E2::from(f);
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
        if f as i32 == i32::min_value() {
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
        if f as i64 == i64::min_value() {
            continue;
        }
        assert_eq!(i64::from(p), f as i64);
    }
}
