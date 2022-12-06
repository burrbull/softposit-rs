use super::P16E1;

impl P16E1 {
    pub const fn exp(self) -> Self {
        let ui_a = self.to_bits();

        let mut f = ui_a as u64;

        // Calculate the exponential for given posit pA
        if ui_a < 28846 {
            // result does not round up to maxpos
            if ui_a < 192 {
                // small positive values that round to 1
                return Self::ONE;
            }

            let mut s: i32;
            if (f & 0x4000) != 0 {
                // decode regime
                s = 8;
                while (f & 0x2000) != 0 {
                    f <<= 1;
                    s += 2;
                }
            } else {
                s = 6;
                while (f & 0x2000) == 0 {
                    f <<= 1;
                    s -= 2;
                }
            }

            if (f & 0x1000) != 0 {
                s += 1; // decode exponent
            }
            f = (f & 0x0FFF) | 0x1000; // decode fraction
            f = ((if s < 0 { f >> -s } else { f << s }) * 48_408_813) >> 20;
            let mut s = f >> 25; // s now stores floor(x)
            f = poly(f & 0x_01FF_FFFF); // 37 fraction bits of exp(x)
            let mut bit = (s & 1) << 37; // exponent bit of exp(x)
            s >>= 1; // regime length of exp(x)
            f |= ((0x_0100_0000_0000 << s) - 0x_0080_0000_0000) | bit;

            bit = 1_u64 << (24 + s); // location of bit n-plus-1
            if ((f & bit) != 0) && (((f & (bit - 1)) != 0) || ((f & (bit << 1)) != 0)) {
                f += bit;
            }
            return Self::from_bits((f >> (25 + s)) as u16); // return rounded exp(x) as posit
        } else if ui_a > 36690 {
            // result does not round up to minpos
            if ui_a > 65407 {
                // small negative values that round to 1
                return Self::ONE;
            }

            let mut s: i32;
            if (f & 0x4000) != 0 {
                // decode regime
                s = 7;
                while (f & 0x2000) != 0 {
                    f <<= 1;
                    s -= 2;
                }
            } else {
                s = 9;
                while (f & 0x2000) == 0 {
                    f <<= 1;
                    s += 2;
                }
            }

            if (f & 0x1000) != 0 {
                s -= 1; // decode exponent
            }
            f = (f & 0x0FFF) | 0x_01FF_E000; // decode fraction
            f = if s < 0 {
                (f >> -s) | (0x_0200_0000 - (1 << (13 + s)))
            } else {
                (f << s) & 0x_01ff_ffff
            };
            f = (0x_0004_0000_0000_0000 - ((0x_0200_0000 - f) * 48_408_813)) >> 20;

            let mut s = (f >> 25).wrapping_sub(32); // s now stores floor(x)
            f = poly(f & 0x_01FF_FFFF); // 37 fraction bits of exp(x)
            let mut bit = (s & 1) << 37; // exponent bit of exp(x)
            s = ((-1 - (s as i64)) >> 1) as u64;
            f |= 0x_0040_0000_0000 | bit; // Install regime end bit

            bit = 1_u64 << (24 + s); // location of bit n-plus-1
            if ((f & bit) != 0) && (((f & (bit - 1)) != 0) || ((f & (bit << 1)) != 0)) {
                f += bit;
            }
            return Self::from_bits((f >> (25 + s)) as u16); // return rounded exp(x) as posit
        }

        // Section for exception cases
        if ui_a < 0x8000 {
            Self::MAX // return maxpos
        } else if ui_a > 0x8000 {
            Self::MIN_POSITIVE // return minpos
        } else {
            Self::NAR // return NaR
        }
    }
}

#[inline]
const fn poly(f: u64) -> u64 {
    let mut s = (f * 7_529) >> 26;
    s = (f * (20_487 + s)) >> 20;
    s = (f * (0x_004F_8300 + s)) >> 24;
    s = (f * (0x_038C_C980 + s)) >> 20;
    s = (f * (0x_0001_EBFF_C800 + s)) >> 26;
    ((f * (0x_0002_C5C8_3600 + s)) >> 22) + 2048
}

#[test]
fn test_exp() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS16 {
        let p_a: P16E1 = rng.gen();
        let f_a = f64::from(p_a);
        let p = p_a.exp();
        let f = f_a.exp();
        let expected = P16E1::from(f);
        if expected.is_zero() || expected.is_nar() {
            continue;
        }
        assert_eq!(p, expected);
    }
}
