impl super::P16E1 {
    pub const fn floor(self) -> Self {
        let mut mask = 0x2000_u16;
        let mut scale = 0_u16;

        let mut ui_a = self.to_bits();
        let sign = ui_a > 0x8000;

        // sign is True if p_a > NaR.
        if sign {
            ui_a = ui_a.wrapping_neg() // A is now |A|.
        };

        let u_a = if ui_a < 0x4000 {
            // 0 <= |pA| < 1 floor to zero.(if not negative and whole number)
            if sign && (ui_a != 0x0) {
                0x4000
            } else {
                0x0
            }
        } else if ui_a < 0x5000 {
            // 1 <= x < 2 floor to 1 (if not negative and whole number)
            if sign && (ui_a != 0x4000) {
                0x5000
            } else {
                0x4000
            }
        } else if ui_a < 0x5800 {
            // 2 <= x < 3 floor to 2 (if not negative and whole number)
            if sign & (ui_a != 0x5000) {
                0x5800
            } else {
                0x5000
            }
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

            mask >>= 1;
            let mut tmp = ui_a & mask;
            let bit_n_plus_one = tmp; // "True" if nonzero.
            ui_a ^= tmp; // Erase the bit, if it was set.
            tmp = ui_a & (mask - 1); // tmp has any remaining bits = bitsMore
            ui_a ^= tmp; // Erase those bits, if any were set.

            if sign && ((bit_n_plus_one | tmp) != 0) {
                ui_a += mask << 1;
            }
            ui_a
        };
        Self::from_bits(u_a).with_sign(sign)
    }
}
