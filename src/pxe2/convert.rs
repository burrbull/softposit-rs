use super::PxE2;
use crate::u32_with_sign;
use core::f64;

impl<const N: u32> From<PxE2<{ N }>> for f32 {
    #[inline]
    fn from(a: PxE2<{ N }>) -> Self {
        a.to_f32()
    }
}

impl<const N: u32> From<PxE2<{ N }>> for f64 {
    #[inline]
    fn from(p_a: PxE2<{ N }>) -> Self {
        p_a.to_f64()
    }
}

impl<const N: u32> From<f32> for PxE2<{ N }> {
    #[inline]
    fn from(float: f32) -> Self {
        Self::from_f32(float)
    }
}

impl<const N: u32> From<f64> for PxE2<{ N }> {
    #[inline]
    fn from(float: f64) -> Self {
        Self::from_f64(float)
    }
}

impl<const N: u32> From<i32> for PxE2<{ N }> {
    #[inline]
    fn from(i_a: i32) -> Self {
        Self::from_i32(i_a)
    }
}

impl<const N: u32> From<u32> for PxE2<{ N }> {
    #[inline]
    fn from(a: u32) -> Self {
        Self::from_u32(a)
    }
}

impl<const N: u32> From<i64> for PxE2<{ N }> {
    #[inline]
    fn from(i_a: i64) -> Self {
        Self::from_i64(i_a)
    }
}

impl<const N: u32> From<u64> for PxE2<{ N }> {
    #[inline]
    fn from(a: u64) -> Self {
        Self::from_u64(a)
    }
}

impl<const N: u32> From<PxE2<{ N }>> for i32 {
    #[inline]
    fn from(p_a: PxE2<{ N }>) -> Self {
        crate::P32E2::from_bits(p_a.to_bits()).to_i32()
    }
}

impl<const N: u32> From<PxE2<{ N }>> for u32 {
    #[inline]
    fn from(p_a: PxE2<{ N }>) -> Self {
        crate::P32E2::from_bits(p_a.to_bits()).to_u32()
    }
}

impl<const N: u32> From<PxE2<{ N }>> for u64 {
    #[inline]
    fn from(p_a: PxE2<{ N }>) -> Self {
        crate::P32E2::from_bits(p_a.to_bits()).to_u64()
    }
}

impl<const N: u32> From<PxE2<{ N }>> for i64 {
    #[inline]
    fn from(p_a: PxE2<{ N }>) -> Self {
        crate::P32E2::from_bits(p_a.to_bits()).to_i64()
    }
}

impl<const N: u32> PxE2<{ N }> {
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
            let (k_a, tmp) = PxE2::<{ N }>::separate_bits_tmp(ui_a);

            let frac_a = ((tmp << 3) as u64) << 20;
            let exp_a = (((k_a as u64) << 2) + ((tmp >> 29) as u64)).wrapping_add(1023) << 52;

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
            if (N == 2) && (float <= 7.523_163_845_262_64_e-37) {
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

                let frac_length = (N - 4) as isize - (reg as isize);

                if frac_length < 0 {
                    //in both cases, reg=29 and 30, e is n+1 bit and frac are sticky bits
                    if reg == N - 3 {
                        bit_n_plus_one = (exp & 0x1) != 0;
                        //exp>>=1; //taken care of by the pack algo
                        exp &= 0x2;
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

                        if (N == 32) && (reg == 29) {
                            exp >>= 1;
                        } else if reg <= 28 {
                            exp <<= 28 - reg;
                        }

                        let mut u_z = (regime << (30 - reg)) + (exp as u32) + (frac << (32 - N));
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
                float *= 16.;
                reg += 1;
            }

            while float >= 2. {
                float *= 0.5;
                exp += 1;
            }

            let frac_length = (N - 4) as isize - (reg as isize);
            if frac_length < 0 {
                //in both cases, reg=29 and 30, e is n+1 bit and frac are sticky bits
                if reg == N - 3 {
                    bit_n_plus_one = (exp & 0x1) != 0;
                    //exp>>=1; //taken care of by the pack algo
                    exp &= 0x2;
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

                    if (N == 32) && (reg == 29) {
                        exp >>= 1;
                    } else if reg <= 28 {
                        exp <<= 28 - reg;
                    }

                    let mut u_z = (regime << (30 - reg)) + (exp as u32) + (frac << (32 - N));
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

    pub const fn from_i32(mut i_a: i32) -> Self {
        if i_a < -2_147_483_135 {
            Self::from_bits(0x_8050_0000);
        }

        let sign = i_a.is_negative();
        if sign {
            i_a = -i_a;
        }

        let ui_a = if (N == 2) && (i_a > 0) {
            0x_4000_0000
        } else if i_a > 2_147_483_135 {
            //2147483136 to 2147483647 rounds to P32 value (2147483648)=> 0x7FB00000
            let mut ui_a = 0x_7FB0_0000; // 2147483648
            if N < 10 {
                ui_a &= Self::mask();
            } else if N < 12 {
                ui_a = 0x_7FF0_0000 & Self::mask();
            }
            ui_a
        } else {
            convert_u32_to_px2bits::<{ N }>(i_a as u32)
        };
        Self::from_bits(u32_with_sign(ui_a, sign))
    }

    pub const fn from_u32(a: u32) -> Self {
        let ui_a = if (N == 2) && (a > 0) {
            0x_4000_0000
        } else if a > 0x_FFFF_FBFF {
            //4294966271
            let mut ui_a = 0x_7FC0_0000; // 4294967296
            if N < 12 {
                ui_a &= Self::mask();
            }
            ui_a
        } else {
            convert_u32_to_px2bits::<{ N }>(a)
        };
        Self::from_bits(ui_a)
    }

    pub const fn from_i64(mut i_a: i64) -> Self {
        let sign = i_a.is_negative();
        if sign {
            i_a = -i_a;
        }

        let ui_a = if (N == 2) && (i_a > 0) {
            0x_4000_0000
        } else if i_a > 0x_7FFD_FFFF_FFFF_FFFF {
            //9222809086901354495
            let mut ui_a = 0x_7FFF_B000; // P32: 9223372036854775808
            if N < 18 {
                ui_a &= Self::mask();
            }
            ui_a
        } else {
            convert_u32_to_px2bits::<{ N }>(i_a as u32)
        };
        Self::from_bits(u32_with_sign(ui_a, sign))
    }

    pub const fn from_u64(a: u64) -> Self {
        let ui_a = if (N == 2) && (a > 0) {
            0x_4000_0000
        } else if a > 0x_FFFB_FFFF_FFFF_FFFF {
            //18445618173802708991
            let mut ui_a = 0x_7FFF_C000; // 18446744073709552000
            if N < 18 {
                ui_a &= Self::mask();
            }
            ui_a
        } else {
            convert_u64_to_px2bits::<{ N }>(a)
        };
        Self::from_bits(ui_a)
    }

    #[inline]
    pub const fn to_i32(self) -> i32 {
        crate::P32E2::from_bits(self.to_bits()).to_i32()
    }

    #[inline]
    pub const fn to_u32(self) -> u32 {
        crate::P32E2::from_bits(self.to_bits()).to_u32()
    }

    #[inline]
    pub const fn to_u64(self) -> u64 {
        crate::P32E2::from_bits(self.to_bits()).to_u64()
    }

    #[inline]
    pub const fn to_i64(self) -> i64 {
        crate::P32E2::from_bits(self.to_bits()).to_i64()
    }
}

const fn convert_u32_to_px2bits<const N: u32>(a: u32) -> u32 {
    let mut log2 = 31_i8; //length of bit (e.g. 4294966271) in int (32 but because we have only 32 bits, so one bit off to accomdate that fact)
    let mut mask = 0x_8000_0000_u32;
    if a < 0x2 {
        a << 30
    } else {
        let mut frac_a = a;

        while (frac_a & mask) == 0 {
            log2 -= 1;
            frac_a <<= 1;
        }
        let k = (log2 >> 2) as u32;
        let exp_a = (log2 & 0x3) as u32;
        frac_a ^= mask;

        let mut ui_a: u32;
        if k >= (N - 2) {
            //maxpos
            ui_a = 0x_7FFF_FFFF & PxE2::<{ N }>::mask();
        } else if k == (N - 3) {
            //bitNPlusOne-> first exp bit //bitLast is zero
            ui_a = 0x_7FFF_FFFF ^ (0x_3FFF_FFFF >> k);
            if ((exp_a & 0x2) != 0) && (((exp_a & 0x1) | frac_a) != 0) {
                //bitNPlusOne //bitsMore
                ui_a |= 0x_8000_0000_u32 >> (N - 1);
            }
        } else if k == (N - 4) {
            ui_a = (0x_7FFF_FFFF ^ (0x_3FFF_FFFF >> k)) | ((exp_a & 0x2) << (27 - k));
            if ((exp_a & 0x1) != 0) && ((((0x_8000_0000_u32 >> (N - 1)) & ui_a) | frac_a) != 0) {
                ui_a += 0x_8000_0000_u32 >> (N - 1);
            }
        } else if k == (N - 5) {
            ui_a = (0x_7FFF_FFFF ^ (0x_3FFF_FFFF >> k)) | (exp_a << (27 - k));
            mask = 0x8 << (k - N);
            if ((mask & frac_a) != 0) && ((((mask - 1) & frac_a) | (exp_a & 0x1)) != 0) {
                //bitNPlusOne
                ui_a += 0x_8000_0000_u32 >> (N - 1);
            }
        } else {
            ui_a = ((0x_7FFF_FFFF ^ (0x_3FFF_FFFF >> k)) | (exp_a << (27 - k)) | frac_a >> (k + 4))
                & PxE2::<{ N }>::mask();
            mask = 0x8 << (k - N); //bitNPlusOne
            if ((mask & frac_a) != 0) && ((((mask - 1) & frac_a) | ((mask << 1) & frac_a)) != 0) {
                ui_a += 0x_8000_0000_u32 >> (N - 1);
            }
        }
        ui_a
    }
}

const fn convert_u64_to_px2bits<const N: u32>(a: u64) -> u32 {
    let mut log2 = 63_i8; //length of bit (e.g. 18445618173802708991) in int (64 but because we have only 64 bits, so one bit off to accommodate that fact)
    let mut mask = 0x_8000_0000_0000_0000_u64;
    if a < 0x2 {
        (a as u32) << 30
    } else {
        let mut frac64_a = a;
        while (frac64_a & mask) == 0 {
            log2 -= 1;
            frac64_a <<= 1;
        }

        let k = (log2 >> 2) as u32;

        let exp_a = (log2 & 0x3) as u32;
        frac64_a ^= mask;

        let mut ui_a: u32;
        if k >= (N - 2) {
            //maxpos
            ui_a = 0x_7FFF_FFFF & PxE2::<{ N }>::mask();
        } else if k == (N - 3) {
            //bitNPlusOne-> first exp bit //bitLast is zero
            ui_a = 0x_7FFF_FFFF ^ (0x_3FFF_FFFF >> k);
            if ((exp_a & 0x2) != 0) && (((exp_a & 0x1) as u64 | frac64_a) != 0) {
                //bitNPlusOne //bitsMore
                ui_a |= 0x_8000_0000_u32 >> (N - 1);
            }
        } else if k == (N - 4) {
            ui_a = (0x_7FFF_FFFF ^ (0x_3FFF_FFFF >> k)) | ((exp_a & 0x2) << (27 - k));
            if ((exp_a & 0x1) != 0)
                && ((((0x_8000_0000_u32 >> (N - 1)) & ui_a) != 0) || (frac64_a != 0))
            {
                ui_a += 0x_8000_0000_u32 >> (N - 1);
            }
        } else if k == (N - 5) {
            ui_a = (0x_7FFF_FFFF ^ (0x_3FFF_FFFF >> k)) | (exp_a << (27 - k));
            mask = 0x_0008_0000_0000_u64 << (k + 32 - N);
            if (mask & frac64_a) != 0 {
                //bitNPlusOne
                if (((mask - 1) & frac64_a) | ((exp_a & 0x1) as u64)) != 0 {
                    ui_a += 0x_8000_0000_u32 >> (N - 1);
                }
            }
        } else {
            ui_a = (0x_7FFF_FFFF ^ (0x_3FFF_FFFF >> k))
                | (exp_a << (27 - k))
                | (((frac64_a >> (k + 36)) as u32) & PxE2::<{ N }>::mask());
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
