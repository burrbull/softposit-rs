use super::PxE1;
use crate::{u32_with_sign, u64_with_sign};
use core::f64;

impl<const N: u32> From<PxE1<{ N }>> for f32 {
    #[inline]
    fn from(a: PxE1<{ N }>) -> Self {
        a.to_f32()
    }
}

impl<const N: u32> From<PxE1<{ N }>> for f64 {
    #[inline]
    fn from(p_a: PxE1<{ N }>) -> Self {
        p_a.to_f64()
    }
}

impl<const N: u32> From<f32> for PxE1<{ N }> {
    #[inline]
    fn from(float: f32) -> Self {
        Self::from_f32(float)
    }
}

impl<const N: u32> From<f64> for PxE1<{ N }> {
    #[inline]
    fn from(float: f64) -> Self {
        Self::from_f64(float)
    }
}

impl<const N: u32> From<PxE1<{ N }>> for i32 {
    #[inline]
    fn from(p_a: PxE1<{ N }>) -> Self {
        p_a.to_i32()
    }
}

impl<const N: u32> From<PxE1<{ N }>> for u32 {
    #[inline]
    fn from(p_a: PxE1<{ N }>) -> Self {
        p_a.to_u32()
    }
}

impl<const N: u32> From<PxE1<{ N }>> for i64 {
    #[inline]
    fn from(p_a: PxE1<{ N }>) -> Self {
        p_a.to_i64()
    }
}

impl<const N: u32> From<PxE1<{ N }>> for u64 {
    #[inline]
    fn from(p_a: PxE1<{ N }>) -> Self {
        p_a.to_u64()
    }
}

impl<const N: u32> From<u64> for PxE1<{ N }> {
    #[inline]
    fn from(a: u64) -> Self {
        Self::from_u64(a)
    }
}

impl<const N: u32> From<i64> for PxE1<{ N }> {
    #[inline]
    fn from(a: i64) -> Self {
        Self::from_i64(a)
    }
}

impl<const N: u32> From<u32> for PxE1<{ N }> {
    #[inline]
    fn from(a: u32) -> Self {
        Self::from_u32(a)
    }
}

impl<const N: u32> From<i32> for PxE1<{ N }> {
    #[inline]
    fn from(a: i32) -> Self {
        Self::from_i32(a)
    }
}

impl<const N: u32> PxE1<{ N }> {
    #[inline]
    pub fn to_f32(self) -> f32 {
        self.to_f64() as f32
    }

    pub fn to_f64(self) -> f64 {
        let mut ui_a = self.to_bits();

        if self.is_zero() {
            0.
        } else if self.is_nar() {
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

    #[inline]
    pub fn from_f32(float: f32) -> Self {
        Self::from_f64(float as f64)
    }

    #[allow(clippy::cognitive_complexity)]
    pub fn from_f64(mut float: f64) -> Self {
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
        } else if !(-1. ..=1.).contains(&float) {
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

                u32_with_sign(
                    if reg > (N - 2) {
                        if reg_s {
                            0x_7FFF_FFFF & Self::mask()
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
                    },
                    sign,
                )
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

            u32_with_sign(
                if reg > (N - 2) {
                    if reg_s {
                        0x_7FFF_FFFF & Self::mask()
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
                },
                sign,
            )
        } else {
            //NaR - for NaN, INF and all other combinations
            0x8000_0000
        };
        Self::from_bits(u_z)
    }

    pub const fn to_i32(self) -> i32 {
        //NaR
        if self.is_nar() {
            return i32::min_value();
        }

        let mut ui_a = self.to_bits();

        let sign = ui_a > 0x_8000_0000; // sign is True if pA > NaR.

        if sign {
            ui_a = ui_a.wrapping_neg(); // A is now |A|.
        }

        let i_z = convert_px1bits_to_u32(ui_a);
        u32_with_sign(i_z, sign) as i32
    }

    pub const fn to_u32(self) -> u32 {
        let ui_a = self.to_bits();
        //NaR
        if ui_a >= 0x_8000_0000 {
            0
        } else {
            convert_px1bits_to_u32(ui_a)
        }
    }

    pub const fn to_i64(self) -> i64 {
        //NaR
        if self.is_nar() {
            return i64::min_value();
        }

        let mut ui_a = self.to_bits();

        let sign = ui_a > 0x_8000_0000; // sign is True if pA > NaR.

        if sign {
            ui_a = ui_a.wrapping_neg(); // A is now |A|.
        }

        let i_z = convert_px1bits_to_u64(ui_a);

        u64_with_sign(i_z, sign) as i64
    }

    pub const fn to_u64(self) -> u64 {
        let ui_a = self.to_bits();
        //NaR
        if ui_a >= 0x_8000_0000 {
            0
        } else {
            convert_px1bits_to_u64(ui_a)
        }
    }

    pub const fn from_u64(a: u64) -> Self {
        let ui_a = if a == 0x_8000_0000_0000_0000 {
            0x_8000_0000
        } else if N == 2 {
            if a > 0 {
                0x_4000_0000
            } else {
                0
            }
        } else if a > 0x_8000_0000_0000_0000 {
            //576460752303423488 -> wrong number need to change
            0x_7FFF_FFFF & ((0x_8000_0000_u64 >> (N - 1)) as u32) // 1152921504606847000
        } else {
            convert_u64_to_px1bits::<{ N }>(a)
        };
        Self::from_bits(ui_a)
    }

    pub const fn from_i64(_a: i64) -> Self {
        todo!()
    }

    pub const fn from_i32(a: i32) -> Self {
        let mut log2 = 31_i8; //length of bit (e.g. 2147418111) in int (32 but because we have only 32 bits, so one bit off to accommodate that fact)

        let mut ui_a = 0u32;
        let mask = 0x80000000_u32;

        let sign = a >> 31 != 0;
        let a = if sign { -a as u32 } else { a as u32 };

        //NaR
        if a == 0x80000000 {
            ui_a = 0x80000000;
        } else if N == 2 {
            if a > 0 {
                ui_a = 0x40000000;
            }
        } else if a > 2147418111 {
            ui_a = 0x7FFF9FFF; // 2147483648
                               //if (x<12)  ui_a&=((int32_t)0x80000000>>(x-1));
        } else if a < 0x2 {
            ui_a = (a << 30) as u32;
        } else {
            let mut frac_a = a;
            while (frac_a & mask) == 0 {
                log2 -= 1;
                frac_a <<= 1;
            }
            let k = (log2 >> 1) as u32;
            let exp_a = ((log2 & 0x1) as u32) << (28 - k);
            frac_a ^= mask;

            if k >= (N - 2) {
                //maxpos
                ui_a = 0x7FFFFFFF & Self::mask();
            } else if k == (N - 3) {
                //bitNPlusOne-> first exp bit //bitLast is zero
                ui_a = 0x7FFFFFFF ^ (0x3FFFFFFF >> k);
                if (exp_a & 0x2) != 0 && ((exp_a & 0x1) | frac_a) != 0 {
                    //bitNPlusOne //bitsMore
                    ui_a |= 0x80000000_u32 >> (N - 1);
                }
            } else if k == (N - 4) {
                ui_a = (0x7FFFFFFF ^ (0x3FFFFFFF >> k)) | ((exp_a & 0x2) << (27 - k));
                if exp_a & 0x1 != 0 && (((0x80000000_u32 >> (N - 1)) & ui_a) | frac_a) != 0 {
                    ui_a += 0x80000000_u32 >> (N - 1);
                }
            } else if k == (N - 5) {
                ui_a = (0x7FFFFFFF ^ (0x3FFFFFFF >> k)) | (exp_a << (27 - k));
                let mask = 0x8 << (k - N);
                if (mask & frac_a) != 0 {
                    //bitNPlusOne
                    if (((mask - 1) & frac_a) | (exp_a & 0x1)) != 0 {
                        ui_a += 0x80000000_u32 >> (N - 1);
                    }
                }
            } else {
                ui_a =
                    ((0x7FFFFFFFu32 ^ (0x3FFFFFFF >> k)) | (exp_a << (27 - k)) | frac_a >> (k + 4))
                        & Self::mask();
                let mask = 0x8 << (k - N); //bitNPlusOne
                if (mask & frac_a) != 0 && (((mask - 1) & frac_a) | ((mask << 1) & frac_a)) != 0 {
                    ui_a += 0x80000000_u32 >> (N - 1);
                }
            }
        }
        Self::from_bits(if sign { ui_a.wrapping_neg() } else { ui_a })
    }

    pub const fn from_u32(_a: u32) -> Self {
        todo!()
    }
}

#[inline]
const fn calculate_scale(mut bits: u32) -> (u32, u32) {
    // Decode the posit, left-justifying as we go.
    let mut scale = 0_u32;

    bits -= 0x_4000_0000; // Strip off first regime bit (which is a 1).
    while (0x_2000_0000 & bits) != 0 {
        // Increment scale by 2 for each regime sign bit.
        scale += 2; // Regime sign bit is always 1 in this range.
        bits = (bits - 0x_2000_0000) << 1; // Remove the bit; line up the next regime bit.
    }
    bits <<= 1; // Skip over termination bit, which is 0.
    if (0x_2000_0000 & bits) != 0 {
        scale += 1;
    } // If exponent is 1, increment the scale.

    (scale, bits)
}

const fn convert_px1bits_to_u32(ui_a: u32) -> u32 {
    if ui_a <= 0x_3000_0000 {
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
        let (scale, bits) = calculate_scale(ui_a);

        let mut i_z64 = ((bits | 0x_2000_0000) as u64) << 33; // Left-justify fraction in 64-bit result (one left bit padding)

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

const fn convert_px1bits_to_u64(ui_a: u32) -> u64 {
    if ui_a <= 0x_3000_0000 {
        // 0 <= |pA| <= 1/2 rounds to zero.
        0
    } else if ui_a < 0x_4800_0000 {
        // 1/2 < x < 3/2 rounds to 1.
        1
    } else if ui_a <= 0x_5400_0000 {
        // 3/2 <= x <= 5/2 rounds to 2.
        2
    } else {
        let (scale, bits) = calculate_scale(ui_a);

        let mut i_z = ((bits | 0x_2000_0000) as u64) << 33; // Left-justify fraction in 64-bit result (one left bit padding)
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

const fn convert_u64_to_px1bits<const N: u32>(a: u64) -> u32 {
    let mut log2 = 63_i8; //60;//length of bit (e.g. 576460752303423488 = 2^59) in int (64 but because we have only 64 bits, so one bit off to accommodate that fact)
    let mut mask = 0x_8000_0000_0000_0000_u64;
    if a < 0x2 {
        (a as u32) << 30
    } else {
        let mut frac64_a = a;
        while (frac64_a & mask) == 0 {
            log2 -= 1;
            frac64_a <<= 1;
        }

        let k = (log2 >> 1) as u32;

        let exp_a = (log2 & 0x1) as u32;
        frac64_a ^= mask;
        frac64_a <<= 1;

        let mut ui_a: u32;
        if k >= (N - 2) {
            //maxpos
            ui_a = 0x_7FFF_FFFF & PxE1::<{ N }>::mask();
        } else if k == (N - 3) {
            //bitNPlusOne-> exp bit //bitLast is zero
            ui_a = 0x_7FFF_FFFF ^ (0x_3FFF_FFFF >> k);
            if ((exp_a & 0x1) != 0) && (frac64_a != 0) {
                //bitNPlusOne //bitsMore
                ui_a |= 0x_8000_0000_u32 >> (N - 1);
            }
        } else if k == (N - 4) {
            //bitLast = regime terminating bit
            ui_a = (0x_7FFF_FFFF ^ (0x_3FFF_FFFF >> k)) | (exp_a << (28 - k));
            mask = 0x_0008_0000_0000_u64 << (k + 32 - N);
            if (mask & frac64_a) != 0 {
                //bitNPlusOne
                if (((mask - 1) & frac64_a) | ((exp_a & 0x1) as u64)) != 0 {
                    ui_a += 0x_8000_0000_u32 >> (N - 1);
                }
            }
        } else {
            ui_a = (0x_7FFF_FFFF ^ (0x_3FFF_FFFF >> k))
                | (exp_a << (28 - k))
                | (((frac64_a >> (k + 36)) as u32) & PxE1::<{ N }>::mask());
            mask = 0x_0008_0000_0000_u64 << (k + 32 - N); //bitNPlusOne position
            if ((mask & frac64_a) != 0)
                && ((((mask - 1) & frac64_a) | ((mask << 1) & frac64_a)) != 0)
            {
                ui_a += 0x_8000_0000_u32 >> (N - 1);
            }
        }
        ui_a
    }
}
