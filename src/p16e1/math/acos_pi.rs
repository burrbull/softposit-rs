use super::P16E1;

impl P16E1 {
    pub const fn acos_pi(self) -> Self {
        let ui_a = self.to_bits();

        if (ui_a > 0x4000) && (ui_a < 0xC000) {
            // return NaR unless -1 <= input <= 1
            return Self::NAR;
        }
        if ui_a < 165 || ui_a > 65_307 {
            // return 1/2 for inputs near 0
            return Self::from_bits(0x3000);
        }
        let mut f = if ui_a < 0x3000 {
            // input is less than 1/2
            poly(to_fixed28_acos_pi(ui_a))
        } else if ui_a < 0x4001 {
            0x_4000_0000 - (poly(super::kernel::isqrt(((0x4000 - ui_a) as u64) << 42)) << 1)
        } else if ui_a > 53248 {
            0x_4000_0000 - poly(to_fixed28_acos_pi(ui_a.wrapping_neg()))
        } else {
            poly(super::kernel::isqrt(((ui_a - 0xC000) as u64) << 42)) << 1
        };
        let mut s = 35_u8; // convert to posit form
        if f > 1 {
            while (f & 0x_4000_0000) == 0 {
                f <<= 1;
                s += 1;
            }
            f = (f ^ 0x_C000_0000) | (((1 ^ (s & 1)) as u64) << 30);
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
        Self::from_bits(f as u16)
    }
}

#[inline]
const fn poly(f: u64) -> u64 {
    let fsq = (f * f) >> 28;
    let s = 13_696 + ((fsq * 7_955) >> 27);
    let s = 100_510 + ((fsq * s) >> 26);
    let s = 1_780_047 + ((fsq * s) >> 25);
    let s = 42_722_829 + ((fsq * s) >> 26);
    0x_2000_0000 - ((f * s) >> 25)
}

const fn to_fixed28_acos_pi(mut f: u16) -> u64 {
    let mut s = 14_i8;

    while (f & 0x2000) == 0 {
        f <<= 1;
        s -= 2;
    }
    if (f & 0x1000) != 0 {
        s += 1;
    }
    f = (f & 0xFFF) | 0x1000;
    (f as u64) << s
}

#[test]
fn test_acos_pi() {
    for i in i16::MIN..i16::MAX {
        let p_a = P16E1::new(i);
        let f_a = f64::from(p_a);
        let answer = p_a.acos_pi();
        let f = f_a.acos() / core::f64::consts::PI;
        let expected = P16E1::from(f);
        assert_eq!(answer, expected);
    }
}
