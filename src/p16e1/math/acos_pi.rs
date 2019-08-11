use super::P16E1;

impl P16E1 {
    pub fn acos_pi(self) -> Self {
        let ui_a = self.to_bits();

        let mut f = ui_a as u64;

        if (f > 0x4000) && (f < 0xC000) {
            // return NaR unless -1 <= input <= 1
            return Self::NAR;
        }
        if (f < 165) || (f > 65_307) {
            // return 1/2 for inputs near 0
            return Self::from_bits(0x3000);
        }
        if f < 0x3000 {
            // input is less than 1/2
            f = poly(to_fixed28_acos_pi(f));
        } else if f < 0x4001 {
            f = 0x_4000_0000 - (poly(super::kernel::isqrt((0x4000 - f) << 42)) << 1);
        } else if f > 53248 {
            f = 0x_4000_0000 - poly(to_fixed28_acos_pi(0x10000 - f));
        } else {
            f = poly(super::kernel::isqrt((f - 0xC000) << 42)) << 1;
        }
        let mut s = 35; // convert to posit form
        if f > 1 {
            while (f & 0x_4000_0000) == 0 {
                f <<= 1;
                s += 1;
            }
            f = (f ^ 0x_C000_0000) | ((1 ^ (s & 1)) << 30);
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
fn poly(f: u64) -> u64 {
    let fsq = (f * f) >> 28;
    let mut s = 13_696 + ((fsq * 7_955) >> 27);
    s = 100_510 + ((fsq * s) >> 26);
    s = 1_780_047 + ((fsq * s) >> 25);
    s = 42_722_829 + ((fsq * s) >> 26);
    0x_2000_0000 - ((f * s) >> 25)
}

fn to_fixed28_acos_pi(i: u64) -> u64 {
    let mut s = 14_i32;

    let mut f = i;
    while (f & 0x2000) == 0 {
        f <<= 1;
        s -= 2;
    }
    if (f & 0x1000) != 0 {
        s += 1;
    }
    f = (f & 0xFFF) | 0x1000;
    f << s
}
