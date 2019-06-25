use super::PxE1;
use crate::WithSign;

impl<const N: u32> PxE1<{ N }> {
    #[inline]
    pub fn round(p_a: Self) -> Self {
        let mut mask = 0x2000_0000_u32;
        let mut scale = 0_u32;

        let u_a: u32;

        let mut ui_a = p_a.to_bits();
        let sign = (ui_a & 0x8000_0000) != 0;

        // sign is True if pA > NaR.
        if sign {
            ui_a = ui_a.wrapping_neg();
        } // A is now |A|.
        if ui_a <= 0x_3000_0000 {
            // 0 <= |pA| <= 1/2 rounds to zero.
            return Self::ZERO;
        } else if ui_a < 0x_4800_0000 {
            // 1/2 < x < 3/2 rounds to 1.
            u_a = 0x4000_0000;
        } else if ui_a <= 0x_5400_0000 {
            // 3/2 <= x <= 5/2 rounds to 2.
            u_a = 0x_5000_0000;
        } else if ui_a >= 0x_7FE8_0000 {
            // If |A| is 0x7FE800000 (4194304) (posit is pure integer value), leave it unchanged.
            if N > 8 {
                return p_a; // This also takes care of the NaR case, 0x80000000.
            } else {
                let bit_n_plus_one = ((0x80000000_u32 >> N) & ui_a) != 0;
                let tmp = (0x7FFFFFFF_u32 >> N) & ui_a; //bitsMore
                let bit_last = (0x80000000_u32 >> (N - 1)) & ui_a;
                if bit_n_plus_one {
                    if (bit_last | tmp) != 0 {
                        ui_a += bit_last;
                    }
                }
                u_a = ui_a;
            }
        } else {
            // 34% of the cases, we have to decode the posit.

            while (mask & ui_a) != 0 {
                scale += 2;
                mask >>= 1;
            }

            mask >>= 1;
            if (mask & ui_a) != 0 {
                scale += 1;
            }
            mask >>= scale;

            //the rest of the bits
            let bit_last = (ui_a & mask) != 0;
            mask >>= 1;
            let mut tmp = ui_a & mask;
            let bit_n_plus_one = tmp != 0;
            ui_a ^= tmp; // Erase the bit, if it was set.
            tmp = ui_a & (mask - 1); // this is actually bits_more

            ui_a ^= tmp;

            if bit_n_plus_one && (((bit_last as u32) | tmp) != 0) {
                ui_a += mask << 1;
            }
            u_a = ui_a;
        }
        Self::from_bits(u_a.with_sign(sign))
    }
}
