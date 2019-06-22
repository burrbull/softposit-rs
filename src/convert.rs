use crate::WithSign;
use crate::{P16E1, P32E2, P8E0};
#[cfg(feature="nightly")]
use crate::PxE2;
use core::convert::From;

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

        P16E1::from_bits(u_z.with_sign(sign))
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
        let mut u_z: u8 = if (k_a < -3) || (k_a >= 3) {
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

        P8E0::from_bits(u_z.with_sign(sign))
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

        P32E2::from_bits(u_z.with_sign(sign))
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

        P16E1::from_bits(u_z.with_sign(sign))
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

        P32E2::from_bits(u_z.with_sign(sign))
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
            if (exp_frac32_a & (0x200_0000 << reg_a)) != 0 {
                let bits_more = exp_frac32_a & (0xFFFF_FFFF >> (7 - reg_a)) != 0;
                u_z += (u_z & 1) | (bits_more as u8);
            }
            u_z
        };

        P8E0::from_bits(u_z.with_sign(sign))
    }
}

#[cfg(feature="nightly")]
impl<const N: u32> From<P32E2> for PxE2<{ N }> {
    #[inline]
    fn from(p_a: P32E2) -> Self {
        let mut ui_a = p_a.to_bits();

        if p_a.is_nar() || p_a.is_zero() {
            Self::from_bits(ui_a);
        }

        let sign = P32E2::sign_ui( ui_a );
        if sign {
            ui_a = ui_a.wrapping_neg();
        }

        let u_z = if N==2 {
            if ui_a>0 { 0x40000000 } else {0}
        }
        else if (N==32) || (((0xFFFFFFFF_u32>>N) & ui_a)==0 ){
            ui_a
        }
        else {

            let shift = 32-N;
            if  (ui_a>>shift) != (0x7FFFFFFF>>shift) {
                if( (0x80000000_u32>>N) & ui_a) != 0 {
                    if ( ( (0x80000000_u32>>(N-1)) & ui_a) != 0) || (((0x7FFFFFFF_u32>>N) & ui_a) != 0 ) {
                        ui_a += 0x1<<shift;
                    }
                }
            }
            let mut u_z = ui_a & (((-0x80000000_i32)>>(N-1)) as u32);
            if u_z==0 {
                u_z = 0x1<<shift;
            }
            u_z
        };
        Self::from_bits(u_z.with_sign(sign))
    }
}
/*
#[cfg(feature="nightly")]
pub trait Gate<const GATE: bool>{}
#[cfg(feature="nightly")]
impl<const M: u32, const N: u32> From<PxE2<{ M }>> for PxE2<{ N }>
where
    Self: Gate<{ M != N }>
{
    #[inline]
    fn from(p_a: PxE2<{ M }>) -> Self {
        Self::from(P32E2::from_bits(p_a.to_bits()))
        
    }
}*/

#[cfg(feature = "alga")]
crate::impl_subset_into!(
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
