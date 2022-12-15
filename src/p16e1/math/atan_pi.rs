use super::P16E1;

impl P16E1 {
    pub const fn atan_pi(self) -> Self {
        let ui_a = self.to_bits();

        if (ui_a & 0x7FFF) == 0 {
            return self; // dispense with NaR and 0 cases
        }

        let sign = (ui_a >> 15) != 0;
        let mut f = if sign { ui_a.wrapping_neg() } else { ui_a };

        let mut f = if f < 0x4000 {
            // SE quadrant; regime bit is 0
            if f > 4925 {
                let mut s = 14_i8;
                while (f & 0x2000) == 0 {
                    // decode regime
                    f <<= 1;
                    s -= 2;
                }
                if (f & 0x1000) != 0 {
                    s += 1; // decode exponent
                }
                f = (f & 0x0FFF) | 0x1000; // get 12-bit fraction and restore hidden bit
                let mut f = f as u64;
                f <<= s;
                poly(f)
            } else {
                // use small x approximation
                let mut s = 13_i8;
                while (f & 0x1000) == 0 {
                    // decode regime
                    f <<= 1;
                    s -= 2;
                }
                if (f & 0x800) != 0 {
                    s += 1; // decode exponent
                }
                f = (f & 0x7FF) | 0x800; // get 12-bit fraction and restore hidden bit
                let mut f = f as u64;
                f = if s < 0 { f >> -s } else { f << s };
                (f << 30) / ((((f * f) >> 34) * 67) + 843_314_118)
            }
        } else {
            // NE quadrant; regime bit is 1
            if f < 27_109 {
                let mut s = 0_u8;
                while (f & 0x2000) != 0 {
                    // decode regime
                    f <<= 1;
                    s += 2;
                }
                if (f & 0x1000) != 0 {
                    s += 1; // decode exponent
                }
                f = (f & 0x0FFF) | 0x1000; // get 12-bit fraction and restore hidden bit
                let mut f = f as u64;
                f <<= s;
                f = 0x_0100_0000_0000 / f; // fixed-point reciprocal
                0x_2000_0000 - poly(f)
            } else {
                let mut s = -1_i8;
                while (f & 0x1000) != 0 {
                    // decode regime
                    f <<= 1;
                    s += 2;
                }
                if (f & 0x800) != 0 {
                    s += 1; // decode exponent
                }
                f = (f & 0x7FF) | 0x800; // get 12-bit fraction and restore hidden bit
                let mut f = f as u64;
                f <<= s; // use large value approx. on fixed point:
                0x_2000_0000 - (0x_0002_8BE5_FF80_0000 / ((f << 13) + (0x_0AA5_5000 / f)))
            }
        };

        // convert fixed-point to a posit
        if f > 1 {
            // leave f = 0 and f = minpos alone
            let mut s = 34_u8;
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

        Self::from_bits(f as u16).with_sign(sign)
    }
}

#[inline]
const fn poly(f: u64) -> u64 {
    let fsq = (f * f) >> 28;
    let s = (fsq * 6_969) >> 24;
    let s = (fsq * (530_432 - s)) >> 28;
    let s = (fsq * (1_273_944 - s)) >> 28;
    let s = (fsq * (2_358_656 - s)) >> 27;
    let s = (fsq * (9_340_208 - s)) >> 29;
    let s = (fsq * (17_568_064 - s)) >> 24;
    ((f + 1) << 30) / (843_315_168 + s)
}

#[test]
fn test_atan_pi() {
    for i in i16::MIN..i16::MAX {
        let p_a = P16E1::new(i);
        let f_a = f64::from(p_a);
        let answer = p_a.atan_pi();
        let f = f_a.atan() / core::f64::consts::PI;
        let expected = P16E1::from(f);
        assert_eq!(answer, expected);
    }
}
