use super::Q8E0;
use crate::u8_with_sign;
use crate::P8E0;

impl From<P8E0> for Q8E0 {
    #[inline]
    fn from(a: P8E0) -> Self {
        let mut q = Self::ZERO;
        q += (a, P8E0::ONE);
        q
    }
}

impl From<Q8E0> for P8E0 {
    #[inline]
    fn from(q_a: Q8E0) -> Self {
        (&q_a).into()
    }
}

impl From<&Q8E0> for P8E0 {
    fn from(q_a: &Q8E0) -> Self {
        q_a.to_posit()
    }
}

impl Q8E0 {
    pub const fn to_posit(&self) -> P8E0 {
        if self.is_zero() {
            return P8E0::ZERO;
        } else if self.is_nar() {
            return P8E0::NAR;
        }

        let mut u_z = self.to_bits();

        let sign = (u_z & 0x8000_0000) != 0;

        if sign {
            u_z = u_z.wrapping_neg();
        }

        let mut no_lz = 0_i8;
        let mut tmp = u_z;
        while (tmp >> 31) == 0 {
            no_lz += 1;
            tmp <<= 1;
        }
        let mut frac32_a = tmp;

        //default dot is between bit 19 and 20, extreme left bit is bit 0. Last right bit is bit 31.
        //Scale =  k
        let k_a = 19 - no_lz;

        let (regime, reg_sa, reg_a) = P8E0::calculate_regime(k_a);

        let u_a = if reg_a > 6 {
            //max or min pos. exp and frac does not matter.
            if reg_sa {
                0x7F
            } else {
                0x1
            }
        } else {
            //remove hidden bit
            frac32_a &= 0x7FFF_FFFF;
            let shift = reg_a + 25; // 1 sign bit and 1 r terminating bit , 16+7+2
            let frac_a = (frac32_a >> shift) as u8;

            let bit_n_plus_one = ((frac32_a >> (shift - 1)) & 0x1) != 0;

            let mut u_a = P8E0::pack_to_ui(regime, frac_a);

            if bit_n_plus_one {
                let bits_more = (frac32_a << (33 - shift)) != 0;
                u_a += (u_a & 1) | (bits_more as u8);
            }
            u_a
        };

        P8E0::from_bits(u8_with_sign(u_a, sign))
    }
}
