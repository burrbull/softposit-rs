use super::P16E1;
use crate::u16_with_sign;

impl P16E1 {
    pub const fn round(self) -> Self {
        let mut mask = 0x2000_u16;
        let mut scale = 0_u16;

        let mut ui_a = self.to_bits();
        let sign = ui_a > 0x8000;

        // sign is True if p_a > NaR.
        if sign {
            ui_a = ui_a.wrapping_neg() // A is now |A|.
        };
        let u_a = if ui_a <= 0x3000 {
            // 0 <= |p_a| <= 1/2 rounds to zero.
            return Self::ZERO;
        } else if ui_a < 0x4800 {
            // 1/2 < x < 3/2 rounds to 1.
            0x4000
        } else if ui_a <= 0x5400 {
            // 3/2 <= x <= 5/2 rounds to 2.
            0x5000
        } else if ui_a >= 0x7C00 {
            // If |A| is 256 or greater, leave it unchanged.
            return self; // This also takes care of the NaR case, 0x8000.
        } else {
            // 34% of the cases, we have to decode the posit.
            while (mask & ui_a) != 0 {
                // Increment scale by 2 for each regime sign bit.
                scale += 2; // Regime sign bit is always 1 in this range.
                mask >>= 1; // Move the mask right, to the next bit.
            }
            mask >>= 1; // Skip over termination bit.
            if (mask & ui_a) != 0 {
                scale += 1; // If exponent is 1, increment the scale.
            }
            mask >>= scale; // Point to the last bit of the integer part.
            let bit_last = (ui_a & mask) != 0; // Extract the bit, without shifting it.

            mask >>= 1;
            let mut tmp = ui_a & mask;
            let bit_n_plus_one = tmp != 0; // "True" if nonzero.
            ui_a ^= tmp; // Erase the bit, if it was set.
            tmp = ui_a & (mask - 1); // tmp has any remaining bits.
            ui_a ^= tmp; // Erase those bits, if any were set.

            if bit_n_plus_one {
                // logic for round to nearest, tie to even
                if (bit_last as u16 | tmp) != 0 {
                    ui_a += mask << 1;
                }
            }
            ui_a
        };
        Self::from_bits(u16_with_sign(u_a, sign))
    }
}

#[test]
fn test_round() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS16 {
        let p_a: P16E1 = rng.gen();
        let f_a = f64::from(p_a);
        let p = p_a.round();
        let f = f_a.round();
        if (f - f_a).abs() == 0.5 {
            continue;
        }
        assert_eq!(p, P16E1::from(f));
    }
}
