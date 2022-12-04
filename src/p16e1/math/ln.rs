use super::P16E1;

impl P16E1 {
    pub const fn ln(self) -> Self {
        let ui_a = self.to_bits();

        let mut f = ui_a as u64;

        if (f > 0x7FFF) || (f == 0) {
            // if input is 0, or greater than maxpos, return NaR
            return Self::NAR;
        }

        let mut s: i32;
        if (f & 0x4000) != 0 {
            // decode regime
            s = 0;
            while (f & 0x2000) != 0 {
                f <<= 1;
                s += 2;
            }
        } else {
            s = -2;
            while (f & 0x2000) == 0 {
                f <<= 1;
                s -= 2;
            }
        }

        if (f & 0x1000) != 0 {
            s += 1; // decode exponent
        }
        f &= 0x0FFF; // get 12-bit fraction, without hidden bit
        if f != 0 {
            f = poly(f); // turn fraction into mantissa of logarithm
        }
        f |= ((if s < 0 { 64 + s } else { s }) as u64) << 30;

        f = if s < 0 {
            0x_0010_0000_0000 - (((0x_0010_0000_0000 - f) * 186_065_280) >> 28)
        } else {
            (f * 186_065_279) >> 28
        };

        let sign = (f & 0x_0008_0000_0000) != 0;
        if sign {
            f = 0x_0010_0000_0000 - f; // take absolute value of fixed-point result
        }
        if f < 0x_4000_0000 {
            // turn fixed-point into posit format
            if f != 0 {
                s = 34;
                while (f & 0x_2000_0000) == 0 {
                    f <<= 1;
                    s += 1;
                }
                f = (f ^ 0x_6000_0000) | (((1 ^ (s & 1)) as u64) << 29);
                s >>= 1;
                let bit = 1_u64 << (s - 1);
                if ((f & bit) != 0) && (((f & (bit - 1)) != 0) || ((f & (bit << 1)) != 0)) {
                    f += bit;
                }
                f >>= s;
            }
        } else {
            s = 0;
            while f > 0x_7FFF_FFFF {
                f = (f & 1) | (f >> 1);
                s += 1;
            }
            f &= 0x_3FFF_FFFF;
            if (s & 1) != 0 {
                f |= 0x_4000_0000;
            }
            s >>= 1;
            f |= (0x_0002_0000_0000_u64 << s) - 0x_0001_0000_0000;
            let bit = 0x_0002_0000_u64 << s;
            if ((f & bit) != 0) && (((f & (bit - 1)) != 0) || ((f & (bit << 1)) != 0)) {
                f += bit;
            }
            f >>= s + 18;
        }
        if sign {
            f = 0x_0001_0000 - f; // restore sign
        }
        Self::from_bits(f as u16)
    }
}

#[inline]
const fn poly(f: u64) -> u64 {
    let z = ((f << 31) + 2) / (f + 8192); // fixed-point divide; discard remainder
    let zsq = (z * z) >> 30; // fixed-point squaring
    let mut s = (zsq * 1_584) >> 28;
    s = (zsq * (26_661 + s)) >> 29;
    s = (zsq * (302_676 + s)) >> 27;
    s = (zsq * (16_136_153 + s)) >> 30;
    (z * (193_635_259 + s)) >> 27
}

#[test]
fn test_ln() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS16 {
        let p_a: P16E1 = rng.gen();
        let f_a = f64::from(p_a);
        let p = p_a.ln();
        let f = f_a.ln();
        let expected = P16E1::from(f);
        assert_eq!(p, expected);
    }
}
