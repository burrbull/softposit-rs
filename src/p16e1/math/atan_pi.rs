use super::P16E1;

impl P16E1 {
    pub const fn atan_pi(self) -> Self {
        let ui_a = self.to_bits();

        let mut f = ui_a as u64;

        if (f & 0x7FFF) == 0 {
            return self; // dispense with NaR and 0 cases
        }

        if (ui_a >> 15) != 0 {
            f = 0x10000 - f; // f = |f|
        }

        let mut s: i32;
        if f < 0x4000 {
            // SE quadrant; regime bit is 0
            if f > 4925 {
                s = 14;
                while (f & 0x2000) == 0 {
                    // decode regime
                    f <<= 1;
                    s -= 2;
                }
                if (f & 0x1000) != 0 {
                    s += 1; // decode exponent
                }
                f = (f & 0x0FFF) | 0x1000; // get 12-bit fraction and restore hidden bit
                f <<= s;
                f = poly(f);
            } else {
                // use small x approximation
                s = 13;
                while (f & 0x1000) == 0 {
                    // decode regime
                    f <<= 1;
                    s -= 2;
                }
                if (f & 0x800) != 0 {
                    s += 1; // decode exponent
                }
                f = (f & 0x7FF) | 0x800; // get 12-bit fraction and restore hidden bit
                f = if s < 0 { f >> -s } else { f << s };
                f = (f << 30) / ((((f * f) >> 34) * 67) + 843_314_118);
            }
        } else {
            // NE quadrant; regime bit is 1
            if f < 27_109 {
                s = 0;
                while (f & 0x2000) != 0 {
                    // decode regime
                    f <<= 1;
                    s += 2;
                }
                if (f & 0x1000) != 0 {
                    s += 1; // decode exponent
                }
                f = (f & 0x0FFF) | 0x1000; // get 12-bit fraction and restore hidden bit
                f <<= s;
                f = 0x_0100_0000_0000 / f; // fixed-point reciprocal
                f = 0x_2000_0000 - poly(f);
            } else {
                s = -1;
                while (f & 0x1000) != 0 {
                    // decode regime
                    f <<= 1;
                    s += 2;
                }
                if (f & 0x800) != 0 {
                    s += 1; // decode exponent
                }
                f = (f & 0x7FF) | 0x800; // get 12-bit fraction and restore hidden bit
                f <<= s; // use large value approx. on fixed point:
                f = 0x_2000_0000 - (0x_0002_8BE5_FF80_0000 / ((f << 13) + (0x_0AA5_5000 / f)));
            }
        }

        // convert fixed-point to a posit
        if f > 1 {
            // leave f = 0 and f = minpos alone
            s = 34;
            while (f & 0x_2000_0000) == 0 {
                f <<= 1;
                s += 1;
            }
            f = (f ^ 0x_6000_0000) | (((1 ^ (s & 1)) as u64) << 29);
            s >>= 1;
            let bit = 1_u64 << (s - 1);
            if (f & bit) != 0 {
                // round to nearest, tie to even
                if ((f & (bit - 1)) != 0) || ((f & (bit << 1)) != 0) {
                    f += bit;
                }
            }
            f >>= s;
        }

        Self::from_bits((if (ui_a >> 15) != 0 { 0x10000 - f } else { f }) as u16)
    }
}

#[inline]
const fn poly(f: u64) -> u64 {
    let fsq = (f * f) >> 28;
    let mut s = (fsq * 6_969) >> 24;
    s = (fsq * (530_432 - s)) >> 28;
    s = (fsq * (1_273_944 - s)) >> 28;
    s = (fsq * (2_358_656 - s)) >> 27;
    s = (fsq * (9_340_208 - s)) >> 29;
    s = (fsq * (17_568_064 - s)) >> 24;
    ((f + 1) << 30) / (843_315_168 + s)
}
