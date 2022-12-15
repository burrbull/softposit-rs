use super::P16E1;

impl P16E1 {
    pub const fn asin_pi(self) -> Self {
        let ui_a = self.to_bits();

        if (ui_a & 0x7FFF) == 0 {
            return self; // Handle 0 and NaR exceptions
        }
        let sign = (ui_a & 0x8000) != 0;
        let mut f = if sign { ui_a.wrapping_neg() } else { ui_a };
        if f > 0x4000 {
            // return NaR unless -1 <= input <= 1
            return Self::NAR;
        }

        let mut f = if f < 0x3000 {
            // input is less than 1/2
            let mut s = 14_i8; // convert to 28-bit fixed point
            while (f & 0x2000) == 0 {
                f <<= 1;
                s -= 2;
            }
            if (f & 0x1000) != 0 {
                s += 1;
            }
            f = (f & 0xFFF) | 0x1000;
            let mut f = f as u64;
            f = if s < 0 { f >> -s } else { f << s };
            poly(f)
        } else {
            0x_2000_0000 - (poly(super::kernel::isqrt(((0x4000 - f) as u64) << 42)) << 1)
        };
        let mut s = 34_u8; // convert to posit form
        if f > 4 {
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
    let s = 13_944 + ((fsq * 3_855) >> 26);
    let s = 100_344 + ((fsq * s) >> 26);
    let s = 1_780_112 + ((fsq * s) >> 25);
    let s = 42_722_832 + ((fsq * s) >> 26);
    (f * s) >> 25
}

#[test]
fn test_asin_pi() {
    for i in i16::MIN..i16::MAX {
        let p_a = P16E1::new(i);
        let f_a = f64::from(p_a);
        let answer = p_a.asin_pi();
        let f = f_a.asin() / core::f64::consts::PI;
        let expected = P16E1::from(f);
        assert_eq!(answer, expected);
    }
}
