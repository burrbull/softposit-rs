use crate::{u16_with_sign, u32_with_sign, u8_with_sign};
use crate::{PxE1, PxE2};
use crate::{P16E1, P32E2, P8E0};

macro_rules! convert_float {
    ($posit: ty, $float:ty, $x:ident) => {{
        use crate::RawFloat;
        use crate::RawPosit;
        type UInt = <$float as RawFloat>::UInt;
        type Int = <$float as RawFloat>::Int;

        const fn bitround(mut ui: UInt) -> <$posit as RawPosit>::UInt {
            const D_BITS: usize = <$float>::BITSIZE - <$posit>::BITSIZE; // difference in bits

            // ROUND TO NEAREST, tie to even: create ulp/2 = ..007ff.. or ..0080..
            let (mut ulp_half, _) = (!<$float>::SIGN_MASK).overflowing_shr(<$posit>::BITSIZE as _); // create ..007ff.. (just smaller than ulp/2)
            ulp_half += (ui >> D_BITS) & 0x1; // turn into ..0080.. for odd (=round up if tie)
            ui += ulp_half; // +ulp/2 and
            (ui >> D_BITS) as _ // round down via >> is round nearest
        }

        // reinterpret input
        let ui: UInt = unsafe { transmute($x) };

        // check zero
        if ui & !<$float>::SIGN_MASK == 0 {
            return <$posit>::ZERO;
        }

        // extract exponent bits and shift to tail, then remove bias
        let e = (ui & <$float>::EXPONENT_MASK) >> <$float>::SIGNIFICAND_BITS;
        let e = (e as Int) - <$float>::EXPONENT_BIAS;
        let signbit_e = e < 0; // sign of exponent
        let k = e >> <$posit>::EXPONENT_BITS; // k-value for useed^k in posits

        // ASSEMBLE POSIT REGIME, EXPONENT, MANTISSA
        // get posit exponent_bits and shift to starting from bitposition 3 (they'll be shifted in later)
        let mut exponent_bits = e & <$posit>::EXPONENT_MASK as Int;
        exponent_bits <<= <$float>::BITSIZE - 2 - <$posit>::EXPONENT_BITS;

        // create 01000... (for |x|<1) or 10000... (|x| > 1)
        let regime_bits = (<$float>::SIGN_MASK >> (signbit_e as UInt)) as Int;

        // extract mantissa bits and push to behind exponent rre..emm... (regime still hasn't been shifted)
        let mut mantissa = (ui & <$float>::SIGNIFICAND_MASK) as Int;
        mantissa <<= <$float>::EXPONENT_BITS - <$posit>::EXPONENT_BITS - 1;

        // combine regime, exponent, mantissa and arithmetic bitshift for 11..110em or 00..001em
        let mut regime_exponent_mantissa = regime_bits | exponent_bits | mantissa;
        let shift = (k + 1).abs() + signbit_e as Int;
        if shift < <$float>::BITSIZE as _ {
            regime_exponent_mantissa >>= shift; // arithmetic bitshift
        } else {
            return <$posit>::NAR;
        }
        regime_exponent_mantissa &= !<$float>::SIGN_MASK as Int; // remove possible sign bit from arith shift

        // round to nearest of the result
        let mut p_rounded = bitround(regime_exponent_mantissa as UInt);

        // no under or overflow rounding mode
        let max_k = (<$float>::EXPONENT_BIAS >> 1) + 1;
        let kabs = k.abs();
        p_rounded = p_rounded.wrapping_sub((k.signum() * ((kabs >= <$posit>::BITSIZE as _ && kabs < max_k) as Int))
            as <$posit as RawPosit>::Int as <$posit as RawPosit>::UInt);

        let sign = (ui & <$float>::SIGN_MASK) != 0;
        // two's complement for negative numbers
        <$posit>::from_bits(if sign {
            p_rounded.wrapping_neg()
        } else {
            p_rounded
        })
    }};
}
pub(crate) use convert_float;

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

pub(crate) fn convert_fraction_p32(
    mut float: f64,
    mut frac_length: u16,
    bits_n_plus_one: &mut bool,
    bits_more: &mut bool,
) -> u32 {
    let mut frac = 0_u32;

    if float == 0. {
        return 0;
    } else if float == core::f64::INFINITY {
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

impl From<P8E0> for P16E1 {
    #[inline]
    fn from(p_a: P8E0) -> Self {
        let mut ui_a = p_a.to_bits();

        //NaR or zero
        if (ui_a == 0x80) || (ui_a == 0) {
            return P16E1::from_bits((ui_a as u16) << 8);
        }

        let sign = P8E0::sign_ui(ui_a);

        if sign {
            ui_a = ui_a.wrapping_neg();
        }

        let (k_a, tmp) = P8E0::separate_bits_tmp(ui_a);

        let mut exp_frac16_a = (tmp as u16) << 8;

        let mut reg_a: i8;
        let regime = if k_a < 0 {
            reg_a = -k_a;
            if (reg_a & 0x1) != 0 {
                exp_frac16_a |= 0x8000;
            }
            reg_a = (reg_a + 1) >> 1;
            if reg_a == 0 {
                reg_a = 1;
            }
            0x4000 >> reg_a
        } else {
            if (k_a & 0x1) != 0 {
                exp_frac16_a |= 0x8000;
            }
            reg_a = (k_a + 2) >> 1;
            if reg_a == 0 {
                reg_a = 1;
            }
            0x7FFF - (0x7FFF >> reg_a)
        };

        exp_frac16_a >>= reg_a + 2; //2 because of sign and regime terminating bit

        let u_z = regime + exp_frac16_a;

        P16E1::from_bits(u16_with_sign(u_z, sign))
    }
}

impl From<P16E1> for P8E0 {
    #[inline]
    fn from(p_a: P16E1) -> Self {
        let mut ui_a = p_a.to_bits();

        if (ui_a == 0x8000) || (ui_a == 0) {
            return P8E0::from_bits((ui_a >> 8) as u8);
        }

        let sign = P16E1::sign_ui(ui_a);

        if sign {
            ui_a = ui_a.wrapping_neg();
        }

        let (k_a, tmp) = P16E1::separate_bits_tmp(ui_a);

        let mut exp_frac16_a = 0_u16;
        let mut reg_a = 0_i8;
        let mut u_z: u8 = if !(-3..3).contains(&k_a) {
            if k_a < 0 {
                0x1
            } else {
                0x7F
            }
        } else {
            //2nd bit exp
            exp_frac16_a = tmp;
            let regime = if k_a < 0 {
                reg_a = ((-k_a) << 1) - ((exp_frac16_a >> 14) as i8);
                if reg_a == 0 {
                    reg_a = 1;
                }
                0x40 >> reg_a
            } else {
                reg_a = if k_a == 0 {
                    1 + ((exp_frac16_a >> 14) as i8)
                } else {
                    ((k_a + 1) << 1) + ((exp_frac16_a >> 14) as i8) - 1
                };
                0x7F - (0x7F >> reg_a)
            };
            if reg_a > 5 {
                regime
            } else {
                //int shift = reg_a+8;
                //exp_frac16_a= ((exp_frac16_a)&0x3FFF) >> shift; //first 2 bits already empty (for sign and regime terminating bit)
                regime + ((((exp_frac16_a) & 0x3FFF) >> (reg_a + 8)) as u8)
            }
        };

        if (exp_frac16_a & (0x80 << reg_a)) != 0 {
            let bits_more = (exp_frac16_a & (0xFFFF >> (9 - reg_a))) != 0;
            u_z += (u_z & 1) | (bits_more as u8);
        }

        P8E0::from_bits(u8_with_sign(u_z, sign))
    }
}

impl From<P16E1> for P32E2 {
    #[inline]
    fn from(p_a: P16E1) -> Self {
        let mut ui_a = p_a.to_bits();

        if (ui_a == 0x8000) || (ui_a == 0) {
            return P32E2::from_bits((ui_a as u32) << 16);
        }

        let sign = P16E1::sign_ui(ui_a);

        if sign {
            ui_a = ui_a.wrapping_neg();
        }

        let (k_a, tmp) = P16E1::separate_bits_tmp(ui_a);

        let mut exp_frac32_a = (tmp as u32) << 16;

        let mut reg_a: i8;
        let regime = if k_a < 0 {
            reg_a = -k_a;
            //if (reg_a&0x1) exp_frac32_a |= 0x8000_0000;
            exp_frac32_a |= ((reg_a & 0x1) as u32) << 31;
            reg_a = (reg_a + 1) >> 1;
            if reg_a == 0 {
                reg_a = 1;
            }
            0x4000_0000 >> reg_a
        } else {
            exp_frac32_a |= ((k_a & 0x1) as u32) << 31;
            reg_a = if k_a == 0 { 1 } else { (k_a + 2) >> 1 };

            0x7FFF_FFFF - (0x7FFF_FFFF >> reg_a)
        };

        exp_frac32_a >>= reg_a + 2; //2 because of sign and regime terminating bit

        let u_z = regime + exp_frac32_a;

        P32E2::from_bits(u32_with_sign(u_z, sign))
    }
}

impl From<P32E2> for P16E1 {
    #[inline]
    fn from(p_a: P32E2) -> Self {
        let mut bits_more = false;
        let mut bit_n_plus_one = false;

        let mut ui_a = p_a.to_bits();

        if (ui_a == 0x8000_0000) || (ui_a == 0) {
            return P16E1::from_bits((ui_a >> 16) as u16);
        }

        let sign = P32E2::sign_ui(ui_a);
        if sign {
            ui_a = ui_a.wrapping_neg();
        }

        let u_z = if ui_a > 0x7F60_0000 {
            0x7FFF
        } else if ui_a < 0x00A0_0000 {
            0x1
        } else {
            let (k_a, tmp) = P32E2::separate_bits_tmp(ui_a);

            //exp and frac
            let mut reg_a: i8;
            let mut exp_frac32_a = tmp << 1;
            let regime = if k_a < 0 {
                reg_a = (-k_a) << 1;
                if (exp_frac32_a & 0x8000_0000) != 0 {
                    reg_a -= 1;
                }
                exp_frac32_a <<= 1;
                0x4000 >> reg_a
            } else {
                reg_a = (k_a << 1) + 1;
                if (exp_frac32_a & 0x8000_0000) != 0 {
                    reg_a += 1;
                }
                exp_frac32_a <<= 1;
                0x7FFF - (0x7FFF >> reg_a)
            };
            if ((exp_frac32_a >> (17 + reg_a)) & 0x1) != 0 {
                bit_n_plus_one = true;
            }
            let exp_frac = if reg_a < 14 {
                (exp_frac32_a >> (18 + reg_a)) as u16
            } else {
                0_u16
            };

            let mut u_z = regime + exp_frac;
            if bit_n_plus_one {
                if (exp_frac32_a << (15 - reg_a)) != 0 {
                    bits_more = true;
                }
                u_z += (bit_n_plus_one as u16 & (u_z & 1)) | ((bit_n_plus_one & bits_more) as u16);
            }
            u_z
        };

        P16E1::from_bits(u16_with_sign(u_z, sign))
    }
}

impl From<P8E0> for P32E2 {
    #[inline]
    fn from(p_a: P8E0) -> Self {
        let mut ui_a = p_a.to_bits();

        if (ui_a == 0x80) || (ui_a == 0) {
            return P32E2::from_bits((ui_a as u32) << 24);
        }

        let sign = P8E0::sign_ui(ui_a);

        if sign {
            ui_a = ui_a.wrapping_neg();
        }

        let (k_a, tmp) = P8E0::separate_bits_tmp(ui_a);

        let mut exp_frac32_a = (tmp as u32) << 22;

        let mut reg_a: i8;
        let regime = if k_a < 0 {
            reg_a = -k_a;
            // Place exponent bits
            exp_frac32_a |= (((reg_a & 0x1) | ((reg_a + 1) & 0x2)) as u32) << 29;

            reg_a = (reg_a + 3) >> 2;
            if reg_a == 0 {
                reg_a = 1;
            }
            0x4000_0000 >> reg_a
        } else {
            exp_frac32_a |= ((k_a & 0x3) as u32) << 29;

            reg_a = (k_a + 4) >> 2;
            if reg_a == 0 {
                reg_a = 1;
            }
            0x7FFF_FFFF - (0x7FFF_FFFF >> reg_a)
        };

        exp_frac32_a >>= reg_a + 1; //2 because of sign and regime terminating bit

        let u_z = regime + exp_frac32_a;

        P32E2::from_bits(u32_with_sign(u_z, sign))
    }
}

impl From<P32E2> for P8E0 {
    #[inline]
    fn from(p_a: P32E2) -> Self {
        let mut ui_a = p_a.to_bits();

        if (ui_a == 0x8000_0000) || (ui_a == 0) {
            return P8E0::from_bits((ui_a >> 24) as u8);
        }

        let sign = P32E2::sign_ui(ui_a);
        if sign {
            ui_a = ui_a.wrapping_neg();
        }

        let u_z = if ui_a > 0x6600_0000 {
            0x7F
        } else if ui_a < 0x1A00_0000 {
            0x1
        } else {
            let (k_a, tmp) = P32E2::separate_bits_tmp(ui_a);
            let k_a = k_a as i8;

            //2nd and 3rd bit exp
            let mut exp_frac32_a = tmp as u32;

            let mut reg_a: i8;
            let regime = if k_a < 0 {
                reg_a = ((-k_a) << 2) - ((exp_frac32_a >> 29) as i8);

                if reg_a == 0 {
                    reg_a = 1;
                }
                if reg_a > 6 {
                    0x1
                } else {
                    0x40 >> reg_a
                }
            } else {
                reg_a = if k_a == 0 {
                    (1 + (exp_frac32_a >> 29)) as i8
                } else {
                    (k_a << 2) + ((exp_frac32_a >> 29) as i8) + 1
                };
                0x7F - (0x7F >> reg_a)
            };
            exp_frac32_a <<= 3;
            let mut u_z = if reg_a > 5 {
                regime as u8
            } else {
                //exp_frac32_a= ((exp_frac32_a)&0x3F) >> shift; //first 2 bits already empty (for sign and regime terminating bit)
                (regime | (exp_frac32_a >> (reg_a + 26))) as u8
            };
            if (exp_frac32_a & (0x_0200_0000 << reg_a)) != 0 {
                let bits_more = exp_frac32_a & (0x_FFFF_FFFF >> (7 - reg_a)) != 0;
                u_z += (u_z & 1) | (bits_more as u8);
            }
            u_z
        };

        P8E0::from_bits(u8_with_sign(u_z, sign))
    }
}

impl<const N: u32> From<P32E2> for PxE2<{ N }> {
    #[inline]
    fn from(p_a: P32E2) -> Self {
        let mut ui_a = p_a.to_bits();

        if p_a.is_nar() || p_a.is_zero() {
            Self::from_bits(ui_a);
        }

        let sign = P32E2::sign_ui(ui_a);
        if sign {
            ui_a = ui_a.wrapping_neg();
        }

        let u_z = if N == 2 {
            if ui_a > 0 {
                0x_4000_0000
            } else {
                0
            }
        } else if (N == 32) || (((0x_FFFF_FFFF_u32 >> N) & ui_a) == 0) {
            ui_a
        } else {
            let shift = 32 - N;
            if ((ui_a >> shift) != (0x_7FFF_FFFF >> shift))
                && (((0x_8000_0000_u32 >> N) & ui_a) != 0)
                && ((((0x_8000_0000_u32 >> (N - 1)) & ui_a) != 0)
                    || (((0x_7FFF_FFFF_u32 >> N) & ui_a) != 0))
            {
                ui_a += 0x1 << shift;
            }
            let mut u_z = ui_a & Self::mask();
            if u_z == 0 {
                u_z = 0x1 << shift;
            }
            u_z
        };
        Self::from_bits(u32_with_sign(u_z, sign))
    }
}

impl<const N: u32> From<P16E1> for PxE2<{ N }> {
    #[inline]
    fn from(p_a: P16E1) -> Self {
        let mut ui_a = p_a.to_bits();

        if p_a.is_nar() || p_a.is_zero() {
            Self::from_bits((ui_a as u32) << 16);
        }

        let sign = P16E1::sign_ui(ui_a);
        if sign {
            ui_a = ui_a.wrapping_neg();
        }

        let u_z = if N == 2 {
            if ui_a > 0 {
                0x_4000_0000
            } else {
                0
            }
        } else {
            let (k_a, tmp) = P16E1::separate_bits_tmp(ui_a);

            let mut exp_frac32_a = (tmp as u32) << 16;

            exp_frac32_a |= ((k_a.abs() & 0x1) as u32) << 31;
            let mut reg_a: u32;
            let reg_sa: bool;
            let regime = if k_a < 0 {
                reg_a = ((-k_a + 1) >> 1) as u32;
                if reg_a == 0 {
                    reg_a = 1;
                }
                reg_sa = false;
                0x_4000_0000_u32 >> reg_a
            } else {
                reg_a = (if k_a == 0 { 1 } else { (k_a + 2) >> 1 }) as u32;
                reg_sa = true;
                0x_7FFF_FFFF - (0x_7FFF_FFFF >> reg_a)
            };

            if reg_a > (N - 2) {
                //max or min pos. exp and frac does not matter.
                if reg_sa {
                    0x_7FFF_FFFF & Self::mask()
                } else {
                    0x1 << (32 - N)
                }
            } else {
                exp_frac32_a >>= reg_a + 2; //2 because of sign and regime terminating bit

                let mut u_z = regime + exp_frac32_a;

                let shift = 32 - N;
                if ((u_z >> shift) != (0x_7FFF_FFFF >> shift))
                    && (((0x_8000_0000_u32 >> N) & u_z) != 0)
                    && ((((0x_8000_0000_u32 >> (N - 1)) & u_z) != 0)
                        || (((0x_7FFF_FFFF_u32 >> N) & u_z) != 0))
                {
                    u_z += 0x1 << shift;
                }

                u_z &= Self::mask();
                if u_z == 0 {
                    u_z = 0x1 << shift;
                }
                u_z
            }
        };
        Self::from_bits(u32_with_sign(u_z, sign))
    }
}

impl<const N: u32> From<P32E2> for PxE1<{ N }> {
    #[inline]
    fn from(p_a: P32E2) -> Self {
        let mut ui_a = p_a.to_bits();

        if (ui_a == 0x_8000_0000) || (ui_a == 0) {
            return Self::from_bits(ui_a);
        }

        let sign = P32E2::sign_ui(ui_a);
        if sign {
            ui_a = ui_a.wrapping_neg();
        }

        let u_z = if N == 2 {
            if ui_a > 0 {
                0x_4000_0000
            } else {
                0
            }
        } else {
            let (k_a, tmp) = P32E2::separate_bits_tmp(ui_a);

            //exp and frac
            let mut exp_frac32_a = tmp << 1;
            let mut reg_a: u32;
            let (reg_sa, regime) = if k_a < 0 {
                reg_a = ((-k_a) << 1) as u32;
                if (exp_frac32_a & 0x_8000_0000) != 0 {
                    reg_a -= 1;
                }
                exp_frac32_a <<= 1;
                (false, 0x_4000_0000_u32.checked_shr(reg_a).unwrap_or(0))
            } else {
                reg_a = ((k_a << 1) + 1) as u32;
                if (exp_frac32_a & 0x_8000_0000) != 0 {
                    reg_a += 1;
                }
                exp_frac32_a <<= 1;
                (
                    true,
                    0x_7fff_ffff - 0x_7fff_ffff_u32.checked_shr(reg_a).unwrap_or(0),
                )
            };

            if reg_a > (N - 2) {
                //max or min pos. exp and frac does not matter.
                if reg_sa {
                    0x_7FFF_FFFF & Self::mask()
                } else {
                    0x1 << (32 - N)
                }
            } else {
                let bit_n_plus_one = ((exp_frac32_a >> (reg_a + 33 - N)) & 0x1) != 0;
                let bits_more = (exp_frac32_a & (0x_7FFF_FFFF >> (N - reg_a - 2))) != 0;

                if reg_a < 30 {
                    exp_frac32_a >>= 2 + reg_a;
                } else {
                    exp_frac32_a = 0;
                }
                let mut u_z = regime + (exp_frac32_a & Self::mask());

                if u_z == 0 {
                    u_z = 0x1 << (32 - N);
                } else if bit_n_plus_one {
                    u_z += (((u_z >> (32 - N)) & 1) | (bits_more as u32)) << (32 - N);
                }
                u_z
            }
        };
        Self::from_bits(u32_with_sign(u_z, sign))
    }
}

impl<const N: u32> From<P8E0> for PxE1<{ N }> {
    #[inline]
    fn from(p_a: P8E0) -> Self {
        let mut ui_a = p_a.to_bits();

        if (ui_a == 0x80) || (ui_a == 0) {
            return Self::from_bits((ui_a as u32) << 24);
        }

        let sign = P8E0::sign_ui(ui_a);
        if sign {
            ui_a = ui_a.wrapping_neg();
        }

        let u_z = if N == 2 {
            if ui_a > 0 {
                0x_4000_0000
            } else {
                0
            }
        } else {
            let (k_a, tmp) = P8E0::separate_bits_tmp(ui_a);

            let mut exp_frac32_a = (tmp as u32) << 24;

            let mut reg_a: u32;
            let regime = if k_a < 0 {
                reg_a = (-k_a) as u32;
                // Place exponent bits
                if (reg_a & 0x1) != 0 {
                    exp_frac32_a |= 0x_8000_0000;
                }

                reg_a = (reg_a + 1) >> 1;
                if reg_a == 0 {
                    reg_a = 1;
                }
                0x_4000_0000_u32.checked_shr(reg_a).unwrap_or(0)
            } else {
                if (k_a & 0x1) != 0 {
                    exp_frac32_a |= 0x_8000_0000;
                }

                reg_a = ((k_a + 2) >> 1) as u32;
                if reg_a == 0 {
                    reg_a = 1;
                }
                0x_7fff_ffff - 0x_7fff_ffff_u32.checked_shr(reg_a).unwrap_or(0)
            };
            exp_frac32_a >>= reg_a + 2; //2 because of sign and regime terminating bit

            let mut u_z = regime + exp_frac32_a;

            let shift = 32 - N;

            if ((u_z >> shift) != (0x_7FFF_FFFF >> shift))
                && (((0x_8000_0000_u32 >> N) & u_z) != 0)
                && ((((0x_8000_0000_u32 >> (N - 1)) & u_z) != 0)
                    || (((0x_7FFF_FFFF_u32 >> N) & u_z) != 0))
            {
                u_z += 0x1 << shift;
            }

            u_z &= Self::mask();
            if u_z == 0 {
                u_z = 0x1 << shift;
            }
            u_z
        };

        Self::from_bits(u32_with_sign(u_z, sign))
    }
}

impl<const N: u32> From<P8E0> for PxE2<{ N }> {
    #[inline]
    fn from(p_a: P8E0) -> Self {
        let mut ui_a = p_a.to_bits();

        if p_a.is_nar() || p_a.is_zero() {
            Self::from_bits((ui_a as u32) << 16);
        }

        let sign = P8E0::sign_ui(ui_a);
        if sign {
            ui_a = ui_a.wrapping_neg();
        }

        let u_z = if N == 2 {
            if ui_a > 0 {
                0x_4000_0000
            } else {
                0
            }
        } else {
            let (k_a, tmp) = P8E0::separate_bits_tmp(ui_a);

            let mut exp_frac32_a = (tmp as u32) << 22;

            let mut reg_a: u32;
            let regime = if k_a < 0 {
                reg_a = (-k_a) as u32;
                // Place exponent bits
                exp_frac32_a |= ((reg_a & 0x1) | ((reg_a + 1) & 0x2)) << 29;

                reg_a = (reg_a + 3) >> 2;
                if reg_a == 0 {
                    reg_a = 1;
                }
                0x_4000_0000_u32 >> reg_a
            } else {
                exp_frac32_a |= ((k_a & 0x3) as u32) << 29;

                reg_a = ((k_a + 4) >> 2) as u32;
                if reg_a == 0 {
                    reg_a = 1;
                }
                0x_7FFF_FFFF - (0x_7FFF_FFFF >> reg_a)
            };

            exp_frac32_a >>= reg_a + 1; //2 because of sign and regime terminating bit

            let mut u_z = regime + exp_frac32_a;

            let shift = 32 - N;
            if ((u_z >> shift) != (0x_7FFF_FFFF >> shift))
                && ((((0x_8000_0000_u32 >> N) & u_z) != 0)
                    && ((((0x_8000_0000_u32 >> (N - 1)) & u_z) != 0)
                        || (((0x_7FFF_FFFF_u32 >> N) & u_z) != 0)))
            {
                u_z += 0x1 << shift;
            }
            u_z &= Self::mask();
            if u_z == 0 {
                u_z = 0x1 << shift;
            }
            u_z
        };
        Self::from_bits(u32_with_sign(u_z, sign))
    }
}

impl<const N: u32> From<P16E1> for PxE1<{ N }> {
    fn from(p_a: P16E1) -> Self {
        let mut ui_a = (p_a.to_bits() as u32) << 16;

        if (ui_a == 0x_8000_0000) || (ui_a == 0) {
            return Self::from_bits(ui_a);
        }

        let sign = Self::sign_ui(ui_a);
        if sign {
            ui_a = ui_a.wrapping_neg();
        }

        let u_z = if N == 2 {
            if ui_a > 0 {
                0x_4000_0000
            } else {
                0
            }
        } else if (N == 32) || (((0x_FFFF_FFFF_u32 >> N) & ui_a) == 0) {
            ui_a
        } else {
            let shift = 32 - N;
            if ((ui_a >> shift) != (0x_7FFF_FFFF >> shift))
                && (((0x_8000_0000_u32 >> N) & ui_a) != 0)
                && (((0x_8000_0000_u32 >> (N - 1)) & ui_a) != 0
                    || ((0x_7FFF_FFFF_u32 >> N) & ui_a) != 0)
            {
                ui_a += 0x1 << shift;
            }
            let mut u_z = ui_a & Self::mask();
            if u_z == 0 {
                u_z = 0x1 << shift;
            }
            u_z
        };

        Self::from_bits(u32_with_sign(u_z, sign))
    }
}

/*
pub trait Gate<const GATE: bool>{}
impl<const M: u32, const N: u32> From<PxE2<{ M }>> for PxE2<{ N }>
where
    Self: Gate<{ M != N }>
{
    #[inline]
    fn from(p_a: PxE2<{ M }>) -> Self {
        Self::from(P32E2::from_bits(p_a.to_bits()))

    }
}*/

impl<const M: u32, const N: u32> From<PxE2<{ M }>> for PxE1<{ N }> {
    #[inline]
    fn from(p_a: PxE2<{ M }>) -> Self {
        let mut ui_a = p_a.to_bits();

        if (ui_a == 0x_8000_0000) || (ui_a == 0) {
            return Self::from_bits(ui_a);
        }

        let sign = PxE2::<{ M }>::sign_ui(ui_a);
        if sign {
            ui_a = ui_a.wrapping_neg();
        }

        let u_z = if N == 2 {
            if ui_a > 0 {
                0x_4000_0000
            } else {
                0
            }
        } else {
            let (k_a, tmp) = PxE2::<{ M }>::separate_bits_tmp(ui_a);

            //exp and frac
            let mut exp_frac32_a = tmp << 1;
            let mut reg_a: u32;
            let (reg_sa, regime) = if k_a < 0 {
                reg_a = ((-k_a) << 1) as u32;
                if (exp_frac32_a & 0x_8000_0000) != 0 {
                    reg_a -= 1;
                }
                exp_frac32_a <<= 1;
                (false, 0x_4000_0000_u32.checked_shr(reg_a).unwrap_or(0))
            } else {
                reg_a = ((k_a << 1) + 1) as u32;
                if (exp_frac32_a & 0x_8000_0000) != 0 {
                    reg_a += 1;
                }
                exp_frac32_a <<= 1;
                (
                    true,
                    0x_7fff_ffff - 0x_7fff_ffff_u32.checked_shr(reg_a).unwrap_or(0),
                )
            };
            if reg_a > (N - 2) {
                //max or min pos. exp and frac does not matter.
                if reg_sa {
                    0x_7FFF_FFFF & Self::mask()
                } else {
                    0x1 << (32 - N)
                }
            } else {
                let bit_n_plus_one = ((exp_frac32_a >> (reg_a + 33 - N)) & 0x1) != 0;
                let bits_more = exp_frac32_a & (0x_7FFF_FFFF >> (N - reg_a - 2));

                if reg_a < 30 {
                    exp_frac32_a >>= 2 + reg_a;
                } else {
                    exp_frac32_a = 0;
                }
                let mut u_z = regime + (exp_frac32_a & Self::mask());

                if u_z == 0 {
                    u_z = 0x1 << (32 - N);
                } else if bit_n_plus_one {
                    u_z += (((u_z >> (32 - N)) & 1) | (bits_more as u32)) << (32 - N);
                }
                u_z
            }
        };

        Self::from_bits(u32_with_sign(u_z, sign))
    }
}

impl<const M: u32, const N: u32> From<PxE1<{ M }>> for PxE2<{ N }> {
    #[inline]
    fn from(p_a: PxE1<{ M }>) -> Self {
        let mut ui_a = p_a.to_bits();

        if (ui_a == 0x_8000_0000) || (ui_a == 0) {
            return Self::from_bits(ui_a);
        }

        let sign = PxE1::<{ M }>::sign_ui(ui_a);
        if sign {
            ui_a = ui_a.wrapping_neg();
        }

        let u_z = if N == 2 {
            if ui_a > 0 {
                0x_4000_0000
            } else {
                0
            }
        } else {
            let (k_a, tmp) = PxE1::<{ M }>::separate_bits_tmp(ui_a);

            //2nd bit exp
            let mut exp_frac32_a = tmp;

            let mut reg_a: u32;
            let (reg_sa, regime) = if k_a < 0 {
                reg_a = -k_a as u32;
                exp_frac32_a |= ((reg_a & 0x1) as u32) << 31;
                reg_a = (reg_a + 1) >> 1;
                if reg_a == 0 {
                    reg_a = 1;
                }
                (false, 0x_4000_0000_u32.checked_shr(reg_a).unwrap_or(0))
            } else {
                exp_frac32_a |= ((k_a & 0x1) as u32) << 31;
                reg_a = if k_a == 0 { 1 } else { ((k_a + 2) >> 1) as u32 };
                (
                    true,
                    0x_7fff_ffff - 0x_7fff_ffff_u32.checked_shr(reg_a).unwrap_or(0),
                )
            };
            if reg_a > (N - 2) {
                //max or min pos. exp and frac does not matter.
                if reg_sa {
                    0x_7FFF_FFFF & Self::mask()
                } else {
                    0x1 << (32 - N)
                }
            } else {
                let bit_n_plus_one = ((exp_frac32_a >> (reg_a + 33 - N)) & 0x1) != 0;
                let bits_more = (exp_frac32_a & (0x_7FFF_FFFF >> (N - reg_a - 2))) != 0;
                exp_frac32_a >>= reg_a + 2; //2 because of sign and regime terminating bit
                let mut u_z = regime + (exp_frac32_a & Self::mask());

                if u_z == 0 {
                    u_z = 0x1_u32 << (32 - N);
                } else if bit_n_plus_one {
                    u_z += (((u_z >> (32 - N)) & 1) | (bits_more as u32)) << (32 - N);
                }
                u_z
            }
        };

        Self::from_bits(u32_with_sign(u_z, sign))
    }
}

impl<const N: u32> From<PxE2<{ N }>> for P32E2 {
    #[inline]
    fn from(p_a: PxE2<{ N }>) -> Self {
        Self::from_bits(p_a.to_bits())
    }
}

impl<const N: u32> From<PxE2<{ N }>> for P16E1 {
    #[inline]
    fn from(p_a: PxE2<{ N }>) -> Self {
        Self::from(P32E2::from_bits(p_a.to_bits()))
    }
}

impl<const N: u32> From<PxE2<{ N }>> for P8E0 {
    #[inline]
    fn from(p_a: PxE2<{ N }>) -> Self {
        Self::from(P32E2::from_bits(p_a.to_bits()))
    }
}

impl<const N: u32> From<PxE1<{ N }>> for P32E2 {
    #[inline]
    fn from(p_a: PxE1<{ N }>) -> Self {
        let mut ui_a = p_a.to_bits();

        if (ui_a == 0x_8000_0000) || (ui_a == 0) {
            return Self::from_bits(ui_a);
        }
        let sign = PxE2::<{ N }>::sign_ui(ui_a);
        if sign {
            ui_a = ui_a.wrapping_neg();
        }

        let (k_a, tmp) = PxE1::<{ N }>::separate_bits_tmp(ui_a);

        //2nd bit exp
        let mut exp_frac32_a = tmp;

        let mut reg_a: u32;
        let regime = if k_a < 0 {
            reg_a = (-k_a) as u32;
            exp_frac32_a |= (reg_a & 0x1) << 31;
            reg_a = (reg_a + 1) >> 1;
            if reg_a == 0 {
                reg_a = 1;
            }
            0x_4000_0000_u32.checked_shr(reg_a).unwrap_or(0)
        } else {
            exp_frac32_a |= ((k_a & 0x1) as u32) << 31;
            reg_a = if k_a == 0 { 1 } else { ((k_a + 2) >> 1) as u32 };
            0x_7fff_ffff - 0x_7fff_ffff_u32.checked_shr(reg_a).unwrap_or(0)
        };

        let bit_n_plus_one = ((exp_frac32_a >> (reg_a + 1)) & 0x1) != 0;
        let bits_more = (exp_frac32_a & (0x_7FFF_FFFF >> (31 - reg_a))) != 0;

        exp_frac32_a >>= reg_a + 2; //2 because of sign and regime terminating bit
        let mut u_z = regime + exp_frac32_a;

        if bit_n_plus_one {
            u_z += (u_z & 1) | (bits_more as u32);
        }

        Self::from_bits(u32_with_sign(u_z, sign))
    }
}

impl<const N: u32> From<PxE1<{ N }>> for P16E1 {
    #[inline]
    fn from(p_a: PxE1<{ N }>) -> Self {
        let mut ui_a = p_a.to_bits();

        if (ui_a == 0x_8000_0000) || (ui_a == 0) {
            return Self::from_bits((ui_a >> 16) as u16);
        }

        let sign = PxE1::<{ N }>::sign_ui(ui_a);
        if sign {
            ui_a = ui_a.wrapping_neg();
        }

        let u_z = if (ui_a & 0xFFFF) == 0 {
            (ui_a >> 16) as u16
        } else {
            if ((ui_a >> 16) != 0x7FFF)
                && (((0x_8000_u32 & ui_a) != 0)
                    && (((0x_0001_0000_u32 & ui_a) != 0) || ((0x_7FFF_u32 & ui_a) != 0)))
            {
                ui_a += 0x_0001_0000_u32;
            }
            let mut u_z = (ui_a >> 16) as u16;
            if u_z == 0 {
                u_z = 0x1;
            }
            u_z
        };
        Self::from_bits(u16_with_sign(u_z, sign))
    }
}

impl<const N: u32> From<PxE1<{ N }>> for P8E0 {
    #[inline]
    fn from(p_a: PxE1<{ N }>) -> Self {
        let mut ui_a = p_a.to_bits();

        if (ui_a == 0x_8000_0000) || (ui_a == 0) {
            return P8E0::from_bits((ui_a >> 24) as u8);
        }

        let sign = PxE1::<{ N }>::sign_ui(ui_a);

        if sign {
            ui_a = ui_a.wrapping_neg();
        }

        let (k_a, tmp) = PxE1::<{ N }>::separate_bits_tmp(ui_a);

        let mut exp_frac32_a = 0_u32;
        let mut reg_a = 0_i8;
        let mut u_z: u8 = if !(-3..3).contains(&k_a) {
            if k_a < 0 {
                0x1
            } else {
                0x7F
            }
        } else {
            //2nd bit exp
            exp_frac32_a = tmp;
            let regime = if k_a < 0 {
                reg_a = ((-k_a) << 1) - ((exp_frac32_a >> 30) as i8);
                if reg_a == 0 {
                    reg_a = 1;
                }
                0x40 >> reg_a
            } else {
                reg_a = if k_a == 0 {
                    1 + ((exp_frac32_a >> 30) as i8)
                } else {
                    ((k_a + 1) << 1) + ((exp_frac32_a >> 30) as i8) - 1
                };
                0x7F - (0x7F >> reg_a)
            };
            if reg_a > 5 {
                regime
            } else {
                regime + ((((exp_frac32_a) & 0x_3FFF_FFFF) >> (reg_a + 24)) as u8)
            }
        };

        if (exp_frac32_a & (0x_0080_0000 << reg_a)) != 0 {
            let bits_more = (exp_frac32_a & ((0x_0080_0000 << reg_a) - 1)) != 0;
            u_z += (u_z & 1) | (bits_more as u8);
        }

        P8E0::from_bits(u8_with_sign(u_z, sign))
    }
}

#[cfg(feature = "simba")]
crate::macros::simba::impl_subset_into!(
    u8 as P8E0, P16E1, P32E2;
    u16 as P8E0, P16E1, P32E2;
    u32 as P8E0, P16E1, P32E2;
    u64 as P8E0, P16E1, P32E2;
    usize as P8E0, P16E1, P32E2;

    i8 as P8E0, P16E1, P32E2;
    i16 as P8E0, P16E1, P32E2;
    i32 as P8E0, P16E1, P32E2;
    i64 as P8E0, P16E1, P32E2;
    isize as P8E0, P16E1, P32E2;

    f32 as P8E0, P16E1, P32E2;
    f64 as P8E0, P16E1, P32E2;

    P8E0  as P8E0, P16E1, P32E2;
    P16E1 as P8E0, P16E1, P32E2;
    P32E2 as P8E0, P16E1, P32E2;
);
