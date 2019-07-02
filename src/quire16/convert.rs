use super::Q16E1;
use crate::WithSign;
use crate::P16E1;
use core::convert::From;

impl From<P16E1> for Q16E1 {
    #[inline]
    fn from(a: P16E1) -> Self {
        let mut q = Self::ZERO;
        q += (a, P16E1::ONE);
        q
    }
}

impl From<Q16E1> for P16E1 {
    #[inline]
    fn from(q_a: Q16E1) -> Self {
        (&q_a).into()
    }
}

impl From<&Q16E1> for P16E1 {
    fn from(q_a: &Q16E1) -> Self {
        if q_a.is_zero() {
            return Self::ZERO;
        } else if q_a.is_nar() {
            return Self::NAR;
        }

        let mut u_z = q_a.to_bits();

        let sign = (u_z & 0x_8000_0000_0000_0000__0000_0000_0000_0000) != 0;

        if sign {
            u_z = u_z.wrapping_neg();
        }

        let mut no_lz = 0_i8;
        let mut tmp = u_z;
        while (tmp >> 127) == 0 {
            no_lz += 1;
            tmp <<= 1;
        }
        let mut bits_more = (u_z << no_lz) != 0;
        let mut frac64_a = (tmp >> 64) as u64;

        //default dot is between bit 71 and 72, extreme left bit is bit 0. Last right bit is bit 127.
        //Equations derived from quire16_mult  last_pos = 71 - (k_a<<1) - exp_a and first_pos = last_pos - frac_len
        let k_a = (71 - no_lz) >> 1;
        let exp_a = 71 - no_lz - (k_a << 1);

        let (regime, reg_sa, reg_a) = Self::calculate_regime(k_a);

        let u_a = if reg_a > 14 {
            //max or min pos. exp and frac does not matter.
            if reg_sa {
                0x7FFF
            } else {
                0x1
            }
        } else {
            //remove hidden bit
            frac64_a &= 0x7FFF_FFFF_FFFF_FFFF;
            let shift = reg_a + 50; //1 es bit, 1 sign bit and 1 r terminating bit , 16+31+3
            let mut frac_a = frac64_a.checked_shr(shift as u32).unwrap_or(0) as u16;

            let mut bit_n_plus_one = false;
            if reg_a != 14 {
                bit_n_plus_one = ((frac64_a >> (shift - 1)) & 0x1) != 0;
                if (frac64_a << (65 - shift)) != 0 {
                    bits_more = true;
                }
            } else if frac_a > 0 {
                frac_a = 0;
                bits_more = true;
            }
            if (reg_a == 14) && (exp_a != 0) {
                bit_n_plus_one = true;
            }
            let mut u_a = Self::pack_to_ui(regime, reg_a, exp_a as u16, frac_a);
            if bit_n_plus_one {
                u_a += (u_a & 1) | (bits_more as u16);
            }
            u_a
        };

        Self::from_bits(u_a.with_sign(sign))
    }
}
