use super::P8E0;
use crate::{u32_with_sign, u64_with_sign, u8_with_sign};
use core::{f32, f64, mem::transmute};

crate::macros::impl_convert!(P8E0);

impl P8E0 {
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
        } else if uip >= 0x_4280_0000 {
            // +- 64.
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
        } else if uip <= 0x_3c80_0000 {
            // +- 0.015_625
            if !sign {
                Self::MIN_POSITIVE
            } else {
                Self::MIN_POSITIVE.neg()
            }
        } else {
            Self::from_bits(crate::convert::convert_float!(P8E0, f32, ui))
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
        } else if uip >= 0x_4050_0000_0000_0000 {
            // +- 64.
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
        } else if uip <= 0x_3f90_0000_0000_0000 {
            // +- 0.015_625
            if !sign {
                Self::MIN_POSITIVE
            } else {
                Self::MIN_POSITIVE.neg()
            }
        } else {
            Self::from_bits(crate::convert::convert_float!(P8E0, f64, ui))
        }
    }

    pub const fn to_f32(self) -> f32 {
        let mut ui_a = self.to_bits();

        if self.is_zero() {
            0.
        } else if self.is_nar() {
            f32::NAN
        } else {
            let sign_a = ui_a & P8E0::SIGN_MASK;
            if sign_a != 0 {
                ui_a = ui_a.wrapping_neg();
            }
            let (k_a, tmp) = P8E0::separate_bits_tmp(ui_a);

            let frac_a = ((tmp << 1) as u32) << 15;
            let exp_a = (k_a as u32).wrapping_add(127) << 23;

            unsafe { transmute(exp_a + frac_a + ((sign_a as u32) << 24)) }
        }
    }

    pub const fn to_f64(self) -> f64 {
        let mut ui_a = self.to_bits();

        if self.is_zero() {
            0.
        } else if self.is_nar() {
            f64::NAN
        } else {
            let sign_a = ui_a & P8E0::SIGN_MASK;
            if sign_a != 0 {
                ui_a = ui_a.wrapping_neg();
            }
            let (k_a, tmp) = P8E0::separate_bits_tmp(ui_a);

            let frac_a = ((tmp << 1) as u64) << 44;
            let exp_a = (k_a as u64).wrapping_add(1023) << 52;

            unsafe { transmute(exp_a + frac_a + ((sign_a as u64) << 56)) }
        }
    }

    #[inline]
    pub const fn to_i32(self) -> i32 {
        let mut ui_a = self.to_bits();
        //NaR
        if ui_a == 0x80 {
            return i32::min_value();
        }

        let sign = ui_a > 0x80; // sign is True if `self` > `NaR`.
        if sign {
            ui_a = ui_a.wrapping_neg(); // A is now |A|.
        }
        let i_z = convert_p8bits_to_u32(ui_a);

        u32_with_sign(i_z, sign) as i32
    }

    #[inline]
    pub const fn to_u32(self) -> u32 {
        let ui_a = self.to_bits();

        if ui_a >= 0x80 {
            return 0; //negative
        }
        convert_p8bits_to_u32(ui_a)
    }

    #[inline]
    pub const fn from_u32(a: u32) -> Self {
        Self::from_bits(convert_u32_to_p8bits(a))
    }

    #[inline]
    pub const fn from_i32(mut i_a: i32) -> Self {
        if i_a < -48 {
            //-48 to -MAX_INT rounds to P32 value -268435456
            return Self::MIN;
        }

        let sign = i_a.is_negative();
        if sign {
            i_a = -i_a;
        }
        Self::from_bits(u8_with_sign(convert_u32_to_p8bits(i_a as u32), sign))
    }

    #[inline]
    pub const fn to_i64(self) -> i64 {
        let mut ui_a = self.to_bits();

        //NaR
        if ui_a == 0x80 {
            return i64::min_value();
        }

        let sign = (ui_a & 0x_80) != 0;
        if sign {
            ui_a = ui_a.wrapping_neg();
        }

        let i_z = convert_p8bits_to_u64(ui_a);

        u64_with_sign(i_z, sign) as i64
    }

    #[inline]
    pub const fn to_u64(self) -> u64 {
        let ui_a = self.to_bits();

        if ui_a >= 0x80 {
            return 0; //negative
        }
        convert_p8bits_to_u64(ui_a)
    }

    #[inline]
    pub const fn from_u64(a: u64) -> Self {
        Self::from_bits(convert_u64_to_p8bits(a))
    }

    pub const fn from_i64(mut i_a: i64) -> Self {
        if i_a < -48 {
            //-48 to -MAX_INT rounds to P32 value -268435456
            return Self::MIN;
        }

        let sign = i_a.is_negative();
        if sign {
            i_a = -i_a;
        }
        Self::from_bits(u8_with_sign(convert_u64_to_p8bits(i_a as u64), sign))
    }
}

const fn convert_p8bits_to_u32(ui_a: u8) -> u32 {
    if ui_a <= 0x20 {
        // 0 <= |p_a| <= 1/2 rounds to zero.
        0
    } else if ui_a < 0x50 {
        // 1/2 < x < 3/2 rounds to 1.
        1
    } else {
        let (scale, bits) = P8E0::calculate_scale(ui_a);

        let mut i_z = ((bits as u32) | 0x40) << 24; // Left-justify fraction in 32-bit result (one left bit padding)

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
        i_z >> (30 - scale) // Right-justify the integer.
    }
}

const fn convert_p8bits_to_u64(ui_a: u8) -> u64 {
    if ui_a <= 0x20 {
        // 0 <= |p_a| <= 1/2 rounds to zero.
        0
    } else if ui_a < 0x50 {
        // 1/2 < x < 3/2 rounds to 1.
        1
    } else {
        let (scale, bits) = P8E0::calculate_scale(ui_a);

        let mut i_z = ((bits as u64) | 0x40) << 55; // Left-justify fraction in 32-bit result (one left bit padding)

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
        i_z >> (61 - scale) // Right-justify the integer.
    }
}

const fn convert_u32_to_p8bits(a: u32) -> u8 {
    if a > 48 {
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

const fn convert_u64_to_p8bits(a: u64) -> u8 {
    if a > 48 {
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

#[test]
fn convert_p8_f64() {
    for n in -0x_80_i8..0x_7f {
        let p = P8E0::new(n);
        let f = f64::from(p);
        assert_eq!(p, P8E0::from(f));
    }
}

#[test]
fn convert_p8_f32() {
    for n in -0x_80_i8..0x_7f {
        let p = P8E0::new(n);
        let f = f32::from(p);
        assert_eq!(p, P8E0::from(f));
    }
}

#[test]
fn convert_f64_p8_rand() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS8 {
        let f: f64 = rng.gen();
        let _p = P8E0::from(f);
    }
}

#[test]
fn convert_f32_p8_rand() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS8 {
        let f: f32 = rng.gen();
        let _p = P8E0::from(f);
    }
}
