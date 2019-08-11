use super::P32E2;
use crate::WithSign;

impl P32E2 {
    pub fn floor(self) -> Self {
        let mut mask = 0x2000_0000_u32;
        let mut scale = 0_u32;

        let mut ui_a = self.to_bits();
        let sign = (ui_a & 0x8000_0000) != 0;

        // sign is True if pA > NaR.
        if sign {
            ui_a = ui_a.wrapping_neg();
        } // A is now |A|.
        let u_a = if ui_a < 0x_4000_0000 {
            // 0 <= |pA| < 1 floor to zero.(if not negative and whole number)
            if sign && (ui_a != 0x0) {
                0x0
            } else {
                0x_4000_0000
            }
        } else if ui_a < 0x_4800_0000 {
            // 0 <= |pA| < 1 floor to 1.(if not negative and whole number)
            if sign && (ui_a != 0x_4000_0000) {
                0x_4800_0000
            } else {
                0x_4000_0000
            }
        } else if ui_a <= 0x_4C00_0000 {
            // 0 <= |pA| < 2 floor to zero.(if not negative and whole number)
            if sign && (ui_a != 0x_4800_0000) {
                0x_4C00_0000
            } else {
                0x_4800_0000
            }
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
            mask >>= 1;
            let mut tmp = ui_a & mask;
            let bit_n_plus_one = tmp;
            ui_a ^= tmp; // Erase the bit, if it was set.
            tmp = ui_a & (mask - 1); // this is actually bits_more

            ui_a ^= tmp;

            if sign && (bit_n_plus_one | tmp) != 0 {
                ui_a += mask << 1;
            }
            ui_a
        };
        Self::from_bits(u_a.with_sign(sign))
    }
}
