use super::P8E0;
use crate::u8_with_sign;

impl P8E0 {
    pub const fn floor(self) -> Self {
        let mut mask = 0x20_u8;
        let mut scale = 0_u8;

        let mut ui_a = self.to_bits();
        let sign = ui_a > 0x80;

        // sign is True if self > NaR.
        if sign {
            ui_a = ui_a.wrapping_neg();
        }
        let u_a = if ui_a == 0 {
            return self;
        } else if ui_a < 0x40 {
            // 0 <= |pA| < 1 floor to zero.(if not negative and whole number)
            if sign && (ui_a != 0x0) {
                0x40
            } else {
                0x0
            }
        } else if ui_a < 0x60 {
            // 1 <= x < 2 floor to 1 (if not negative and whole number)
            if sign && (ui_a != 0x40) {
                0x60
            } else {
                0x40
            }
        } else if ui_a < 0x68 {
            // 2 <= x < 3 floor to 2 (if not negative and whole number)
            if sign && (ui_a != 0x60) {
                0x68
            } else {
                0x60
            }
        } else if ui_a >= 0x78 {
            // If |A| is 8 or greater, leave it unchanged.
            return self; // This also takes care of the NaR case, 0x80.
        } else {
            while (mask & ui_a) != 0 {
                scale += 1;
                mask >>= 1;
            }

            mask >>= scale;

            mask >>= 1;
            let mut tmp = ui_a & mask;
            let bit_n_plus_one = tmp;
            ui_a ^= tmp;
            tmp = ui_a & (mask - 1); //bits_more
            ui_a ^= tmp;

            if sign && ((bit_n_plus_one | tmp) != 0) {
                ui_a += mask << 1;
            }
            ui_a
        };
        Self::from_bits(u8_with_sign(u_a, sign))
    }
}
