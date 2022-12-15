use super::P16E1;

impl P16E1 {
    pub const fn exp2(self) -> Self {
        let ui_a = self.to_bits();

        let mut f = ui_a as u64;

        // Calculate the exponential for given posit pA
        if ui_a < 29377 {
            // result does not round up to maxpos

            if ui_a < 221 {
                // cases that round down to 1.
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
            f = if s < 0 { f >> -s } else { f << s };
            let mut s = f >> 20; // s now stores floor(x)
            f = poly(f & 0x_000F_FFFF); // fraction bits of exp2(x)
            let mut bit = (s & 1) << 26; // exponent bit of exp2(x)
            s >>= 1; // regime length of exp2(x)
            f |= ((0x_2000_0000_u64 << s) - 0x_1000_0000) | bit;

            bit = 1_u64 << (13 + s); // location of bit n-plus-1
            if ((f & bit) != 0) && (((f & (bit - 1)) != 0) || ((f & (bit << 1)) != 0)) {
                f += bit;
            }
            return Self::from_bits((f >> (14 + s)) as u16); // return rounded exp2(x) as posit
        } else if ui_a > 36159 {
            if ui_a > 65379 {
                // cases that round up to 1.
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
            let mut s = (f >> 20).wrapping_sub(32); // s now stores floor(x)
            f = poly(f & 0x_000F_FFFF); // fraction bits of exp2(x)
            let mut bit = (s & 1) << 26; // exponent bit of exp2(x)
            s = ((-1 - (s as i64)) >> 1) as u64;
            f |= 0x_0800_0000 | bit; // Install regime end bit

            bit = 1_u64 << (13 + s); // location of bit n-plus-1
            if ((f & bit) != 0) && (((f & (bit - 1)) != 0) || ((f & (bit << 1)) != 0)) {
                f += bit;
            }
            return Self::from_bits((f >> (14 + s)) as u16); // return rounded exp2(x) as posit
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
    let mut s = (f * (0x_9BA0_0000 + (f * 491))) >> 34;
    s = (f * (0x_0013_F840 + s)) >> 20;
    s = (f * (0x_0071_8A80 + s)) >> 16;
    s = (f * (0x_1EC0_4000 + s)) >> 21;
    (f * (0x_2C5C_8000 + s)) >> 24
}

#[test]
fn test_exp2() {
    for i in i16::MIN..i16::MAX {
        let p_a = P16E1::new(i);
        let f_a = f64::from(p_a);
        let p = p_a.exp2();
        let f = f_a.exp2();
        let expected = P16E1::from(f);
        if expected.is_zero() || expected.is_nar() {
            continue;
        }
        assert_eq!(p, expected);
    }
}
