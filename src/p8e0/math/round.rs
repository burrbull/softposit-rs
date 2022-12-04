use super::P8E0;
use crate::u8_with_sign;

impl P8E0 {
    pub fn round(self) -> Self {
        let mut mask = 0x20_u8;
        let mut scale = 0_u8;

        let mut ui_a = self.to_bits();
        let sign = ui_a > 0x80;

        // sign is True if self > NaR.
        if sign {
            ui_a = ui_a.wrapping_neg();
        }
        let u_a = if ui_a <= 0x20 {
            // 0 <= |self| <= 1/2 rounds to zero.
            return Self::ZERO;
        } else if ui_a < 0x50 {
            // 1/2 < x < 3/2 rounds to 1.
            0x40
        } else if ui_a <= 0x64 {
            // 3/2 <= x <= 5/2 rounds to 2.
            0x60
        } else if ui_a >= 0x78 {
            // If |A| is 8 or greater, leave it unchanged.
            return self; // This also takes care of the NaR case, 0x80.
        } else {
            while (mask & ui_a) != 0 {
                scale += 1;
                mask >>= 1;
            }

            mask >>= scale;
            let bit_last = (ui_a & mask) != 0;

            mask >>= 1;
            let mut tmp = ui_a & mask;
            let bit_n_plus_one = tmp != 0;
            ui_a ^= tmp;
            tmp = ui_a & (mask - 1); //bits_more
            ui_a ^= tmp;

            if bit_n_plus_one && (((bit_last as u8) | tmp) != 0) {
                ui_a += mask << 1;
            }
            ui_a
        };
        Self::from_bits(u8_with_sign(u_a, sign))
    }
}

#[test]
fn test_round() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS8 {
        let p_a: P8E0 = rng.gen();
        let f_a = f64::from(p_a);
        let p = p_a.round();
        let f = f_a.round();
        if (f - f_a).abs() == 0.5 {
            continue;
        }
        assert_eq!(p, P8E0::from(f));
    }
}
