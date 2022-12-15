use super::Q32E2;
use crate::u32_with_sign;
use crate::u64_zero_shr;
use crate::PxE2;
use crate::P32E2;

impl From<P32E2> for Q32E2 {
    #[inline]
    fn from(a: P32E2) -> Self {
        let mut q = Self::ZERO;
        q += (a, P32E2::ONE);
        q
    }
}

impl<const N: u32> From<PxE2<{ N }>> for Q32E2 {
    #[inline]
    fn from(a: PxE2<{ N }>) -> Self {
        let mut q = Self::ZERO;
        q += (a, PxE2::ONE);
        q
    }
}

impl From<Q32E2> for P32E2 {
    #[inline]
    fn from(q_a: Q32E2) -> Self {
        (&q_a).into()
    }
}

impl From<&Q32E2> for P32E2 {
    #[inline]
    fn from(q_a: &Q32E2) -> Self {
        q_a.to_posit()
    }
}

impl Q32E2 {
    pub fn to_posit(&self) -> P32E2 {
        let mut bits_more = false;
        let mut frac64_a = 0_u64;

        if self.is_zero() {
            return P32E2::ZERO;
        } else if self.is_nar() {
            return P32E2::NAR;
        }

        let mut u_z = self.to_bits();

        let sign = (u_z[0] & 0x_8000_0000_0000_0000) != 0;

        if sign {
            let mut j = u_z.iter_mut().rev();
            while let Some(u) = j.next() {
                if *u > 0 {
                    *u = u.wrapping_neg();
                    for w in j {
                        *w = !*w;
                    }
                    break;
                }
            }
        }
        //minpos and maxpos

        let mut no_lz = 0_isize;

        let mut j = u_z.iter_mut().enumerate();
        while let Some((i, u)) = j.next() {
            if *u == 0 {
                no_lz += 64;
            } else {
                let mut tmp = *u;
                let mut no_lztmp = 0_isize;

                while (tmp >> 63) == 0 {
                    no_lztmp += 1;
                    tmp <<= 1;
                }

                no_lz += no_lztmp;
                frac64_a = tmp;
                if (i != 7) && (no_lztmp != 0) {
                    let (_, w) = j.next().unwrap();
                    frac64_a += *w >> (64 - no_lztmp);
                    if (*w & ((0x1_u64 << (64 - no_lztmp)) - 1)) != 0 {
                        bits_more = true;
                    }
                }
                for (_, w) in j {
                    if *w > 0 {
                        bits_more = true;
                        break;
                    }
                }
                break;
            }
        }

        //default dot is between bit 271 and 272, extreme left bit is bit 0. Last right bit is bit 511.
        //Equations derived from quire32_mult  last_pos = 271 - (k_a<<2) - exp_a and first_pos = last_pos - frac_len
        let k_a = ((271 - no_lz) >> 2) as i8;
        let mut exp_a = 271 - (no_lz as i32) - ((k_a << 2) as i32);

        let (regime, reg_sa, reg_a) = P32E2::calculate_regime(k_a);

        let u_a = if reg_a > 30 {
            //max or min pos. exp and frac does not matter.
            if reg_sa {
                0x7FFF_FFFF
            } else {
                0x1
            }
        } else {
            //remove hidden bit
            frac64_a &= 0x7FFF_FFFF_FFFF_FFFF;

            let shift = reg_a + 35; //2 es bit, 1 sign bit and 1 r terminating bit , 31+4

            let mut frac_a = u64_zero_shr(frac64_a, shift as u32) as u32;
            let mut bit_n_plus_one = false;
            if reg_a <= 28 {
                bit_n_plus_one = ((frac64_a >> (shift - 1)) & 0x1) != 0;
                exp_a <<= 28 - reg_a;
                if (frac64_a << (65 - shift)) != 0 {
                    bits_more = true;
                }
            } else {
                if reg_a == 30 {
                    bit_n_plus_one = (exp_a & 0x2) != 0;
                    bits_more = (exp_a & 0x1) != 0;
                    exp_a = 0;
                } else if reg_a == 29 {
                    bit_n_plus_one = (exp_a & 0x1) != 0;
                    exp_a >>= 1; //taken care of by the pack algo
                }
                if frac64_a > 0 {
                    frac_a = 0;
                    bits_more = true;
                }
            }

            let mut u_a = P32E2::pack_to_ui(regime, exp_a as u32, frac_a);
            if bit_n_plus_one {
                u_a += (u_a & 1) | (bits_more as u32);
            }
            u_a
        };
        P32E2::from_bits(u32_with_sign(u_a, sign))
    }
}

impl<const N: u32> From<Q32E2> for PxE2<{ N }> {
    #[inline]
    fn from(q_a: Q32E2) -> Self {
        (&q_a).into()
    }
}

impl<const N: u32> From<&Q32E2> for PxE2<{ N }> {
    fn from(q_a: &Q32E2) -> Self {
        let mut bits_more = false;
        let mut frac64_a = 0_u64;

        if q_a.is_zero() {
            return Self::ZERO;
        } else if q_a.is_nar() {
            return Self::NAR;
        }

        let mut u_z = q_a.to_bits();

        let sign = (u_z[0] & 0x_8000_0000_0000_0000) != 0;

        if sign {
            let mut j = u_z.iter_mut().rev();
            while let Some(u) = j.next() {
                if *u > 0 {
                    *u = u.wrapping_neg();
                    for w in j {
                        *w = !*w;
                    }
                    break;
                }
            }
        }
        //minpos and maxpos

        let mut no_lz = 0_isize;

        let mut j = u_z.iter_mut().enumerate();
        while let Some((i, u)) = j.next() {
            if *u == 0 {
                no_lz += 64;
            } else {
                let mut tmp = *u;
                let mut no_lztmp = 0_isize;

                while (tmp >> 63) == 0 {
                    no_lztmp += 1;
                    tmp <<= 1;
                }

                no_lz += no_lztmp;
                frac64_a = tmp;
                if (i != 7) && (no_lztmp != 0) {
                    let (_, w) = j.next().unwrap();
                    frac64_a += *w >> (64 - no_lztmp);
                    if (*w & ((0x1_u64 << (64 - no_lztmp)) - 1)) != 0 {
                        bits_more = true;
                    }
                }
                for (_, w) in j {
                    if *w > 0 {
                        bits_more = true;
                        break;
                    }
                }
                break;
            }
        }

        //default dot is between bit 271 and 272, extreme left bit is bit 0. Last right bit is bit 511.
        //Equations derived from quire32_mult  last_pos = 271 - (k_a<<2) - exp_a and first_pos = last_pos - frac_len
        let k_a = ((271 - no_lz) >> 2) as i8;
        let mut exp_a = 271 - (no_lz as i32) - ((k_a << 2) as i32);

        let (mut regime, reg_sa, reg_a) = Self::calculate_regime(k_a);

        let u_a = if reg_a > (N - 2) {
            //max or min pos. exp and frac does not matter.
            if reg_sa {
                0x_7FFF_FFFF & Self::mask()
            } else {
                0x1 << (32 - N)
            }
        } else {
            //remove hidden bit
            frac64_a &= 0x_7FFF_FFFF_FFFF_FFFF;

            let shift = reg_a + 35; //2 es bit, 1 sign bit and 1 r terminating bit , 31+4
            let mut frac_a = (frac64_a >> shift) as u32;

            //regime length is smaller than length of posit
            let mut bit_n_plus_one = false;
            if reg_a < N {
                if reg_a <= (N - 4) {
                    bit_n_plus_one = ((frac64_a >> (shift + 31 - N)) & 0x1) != 0;
                    if (frac64_a << (33 - shift + N)) != 0 {
                        bits_more = true;
                    }
                } else {
                    if reg_a == (N - 2) {
                        bit_n_plus_one = (exp_a & 0x2) != 0;
                        exp_a = 0;
                    } else if reg_a == (N - 3) {
                        bit_n_plus_one = (exp_a & 0x1) != 0;
                        //exp_a>>=1;
                        exp_a &= 0x2;
                    }
                    if frac64_a > 0 {
                        frac_a = 0;
                    }
                }
            } else {
                regime = if reg_sa {
                    regime & Self::mask()
                } else {
                    regime << (32 - N)
                };
                exp_a = 0;
                frac_a = 0;
            }

            exp_a <<= 28 - reg_a;
            let mut u_a = Self::pack_to_ui(regime, exp_a as u32, frac_a) & Self::mask();

            if bit_n_plus_one {
                u_a += (((u_a >> (32 - N)) & 0x1) | (bits_more as u32)) << (32 - N);
            }
            u_a
        };
        Self::from_bits(u32_with_sign(u_a, sign))
    }
}
