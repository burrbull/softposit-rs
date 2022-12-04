use super::P32E2;
use crate::u32_with_sign;

impl P32E2 {
    pub const fn round(self) -> Self {
        let mut mask = 0x2000_0000_u32;
        let mut scale = 0_u32;

        let u_a: u32;

        let mut ui_a = self.to_bits();
        let sign = (ui_a & 0x8000_0000) != 0;

        // sign is True if pA > NaR.
        if sign {
            ui_a = ui_a.wrapping_neg();
        } // A is now |A|.
        if ui_a <= 0x3800_0000 {
            // 0 <= |pA| <= 1/2 rounds to zero.
            return P32E2::ZERO;
        } else if ui_a < 0x4400_0000 {
            // 1/2 < x < 3/2 rounds to 1.
            u_a = 0x4000_0000;
        } else if ui_a <= 0x4A00_0000 {
            // 3/2 <= x <= 5/2 rounds to 2.
            u_a = 0x4800_0000;
        } else if ui_a >= 0x7E80_0000 {
            // If |A| is 0x7E80_0000 (posit is pure integer value), leave it unchanged.
            return self; // This also takes care of the NaR case, 0x8000_0000.
        } else {
            // 34% of the cases, we have to decode the posit.

            while (mask & ui_a) != 0 {
                scale += 4;
                mask >>= 1;
            }
            mask >>= 1;

            //Exponential (2 bits)
            if (mask & ui_a) != 0 {
                scale += 2;
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
        Self::from_bits(u32_with_sign(u_a, sign))
    }
}

#[test]
fn test_round() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS32 {
        let p_a: P32E2 = rng.gen();
        let f_a = f64::from(p_a);
        let p = p_a.round();
        let f = f_a.round();
        if (f - f_a).abs() == 0.5 {
            continue;
        }
        assert_eq!(p, P32E2::from(f));
    }
}
