use super::PxE2;
use crate::WithSign;
use core::convert::From;
use core::f64;

impl<const N: u32> From<PxE2<{ N }>> for f32 {
    #[inline]
    fn from(a: PxE2<{ N }>) -> Self {
        f64::from(a) as f32
    }
}

impl<const N: u32> From<PxE2<{ N }>> for f64 {
    #[inline]
    fn from(p_a: PxE2<{ N }>) -> Self {
        let mut ui_a = p_a.to_bits();

        if p_a.is_zero() {
            0.
        } else if p_a.is_nar() {
            f64::NAN
        } else {
            let sign_a = PxE2::<{ N }>::sign_ui(ui_a);
            if sign_a {
                ui_a = ui_a.wrapping_neg();
            }
            let (k_a, tmp) = PxE2::<{ N }>::separate_bits_tmp(ui_a);

            let frac_a = ((tmp << 3) as u64) << 20;
            let exp_a = (((k_a as u64) << 2) + ((tmp >> 29) as u64)).wrapping_add(1023) << 52;

            f64::from_bits(exp_a + frac_a + (((sign_a as u64) & 0x1) << 63))
        }
    }
}

impl<const N: u32> From<f64> for PxE2<{ N }> {
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
        } /* else if float >= 1.329_227_995_784_916_e36 {
              //maxpos
              return Self::MAX;
          } else if float <= -1.329_227_995_784_916_e36 {
              // -maxpos
              return Self::MIN;
          }*/

        let sign = float < 0.;

        let u_z: u32 = if float == 1. {
            0x4000_0000
        } else if float == -1. {
            0xC000_0000
        /*} else if (float <= 7.523_163_845_262_64_e-37) && !sign {
            //minpos
            0x1
        } else if (float >= -7.523_163_845_262_64_e-37) && sign {
            //-minpos
            0xFFFF_FFFF*/
        } else if (float > 1.) || (float < -1.) {
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

                if reg > (N - 2) {
                    if reg_s {
                        0x7FFFFFFF & (((-0x80000000_i32) >> (N - 1)) as u32)
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

                    let mut u_z = ((regime as u32) << (30 - reg))
                        + (exp as u32)
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

            if reg > (N - 2) {
                if reg_s {
                    0x7FFFFFFF & (((-0x80000000_i32) >> (N - 1)) as u32)
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

                let mut u_z =
                    ((regime as u32) << (30 - reg)) + (exp as u32) + ((frac << (32 - N)) as u32);
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
