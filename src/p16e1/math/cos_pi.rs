use super::P16E1;

impl P16E1 {
    pub const fn cos_pi(self) -> Self {
        let ui_a = self.to_bits();
        if ui_a == 0x8000 {
            return Self::NAR; // dispense with the NaR case
        }

        let mut f = if (ui_a & 0x8000) != 0 {
            ui_a.wrapping_neg()
        } else {
            ui_a
        };

        let mut f = if f != 0 {
            let mut s: i8;
            if (f & 0x4000) != 0 {
                // decode regime
                s = 16;
                while (f & 0x2000) != 0 {
                    f <<= 1;
                    s += 2;
                }
            } else {
                s = 14;
                while (f & 0x2000) == 0 {
                    f <<= 1;
                    s -= 2;
                }
            }
            if (f & 0x1000) != 0 {
                s += 1; // decode exponent
            }
            f = (f & 0x0FFF) | 0x1000; // get 12-bit fraction and restore hidden bit
            let f = f as u64;
            if s < 0 {
                f >> -s
            } else {
                f << s
            }
        } else {
            0
        };
        let s = f >> 27; // the quadrant is the multiple of 1/2
        f &= 0x_07FF_FFFF; // input value modulo 1/2
        let sign = if ((s + 1) & 2) != 0 {
            0x8000 // cos is negative for quadrants 2 and 3
        } else {
            0
        };
        if f == 0 {
            return Self::from_bits(if (s & 1) != 0 { 0 } else { sign | 0x4000 });
        }
        if (s & 1) != 0 {
            f = 0x_0800_0000 - f;
        }
        let mut f = poly(f);
        let mut s = 1u8; // convert fixed-point to a posit
        while (f & 0x_0100_0000) == 0 {
            f <<= 1;
            s += 1;
        }
        let bit = s & 1;
        if bit == 0 {
            f &= 0x_00FF_FFFF; // encode exponent bit
        }
        let mut s = (s >> 1) + 12;
        if bit == 0 {
            s -= 1;
        }

        f |= 0x_0200_0000; // encode regime termination bit
        let bit = 1_u64 << (s - 1);
        if (f & bit) != 0 {
            // round to nearest, tie to even
            if ((f & (bit - 1)) != 0) || ((f & (bit << 1)) != 0) {
                f += bit;
            }
        }
        Self::from_bits((f >> s) as u16).with_sign(sign != 0)
    }
}

#[inline]
pub const fn poly(f: u64) -> u64 {
    if f < 0x_000E_6001 {
        return 0x_01FF_FFFF; // this rounds up to 1.0
    }
    let fsq = f >> 11; // convert to 17-bit fixed point
    let fsq = (fsq * fsq) >> 8;
    let s = 349_194 - ((fsq * 28_875) >> 25);
    let s = 4_255_560 - ((fsq * s) >> 24);
    let s = 20_698_014 - ((fsq * s) >> 24);
    33_554_428 - ((fsq * s) >> 23)
}

#[test]
fn test_cos_pi() {
    for i in i16::MIN..i16::MAX {
        let p_a = P16E1::new(i);
        let f_a = f64::from(p_a);
        let p = p_a.cos_pi();
        let f = (f_a * core::f64::consts::PI).cos();
        let expected = P16E1::from(f);
        if p.is_zero() {
            continue;
        }
        assert_eq!(p, expected);
    }
}
