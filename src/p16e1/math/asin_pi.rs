use super::P16E1;

impl P16E1 {
    pub fn asin_pi(self) -> Self {
        let ui_a = self.to_bits();

        let mut f = ui_a as u64;

        if (ui_a & 0x7FFF) == 0 {
            return self; // Handle 0 and NaR exceptions
        }
        if (ui_a & 0x8000) != 0 {
            f = 0x10000 - f;
        }
        if f > 0x4000 {
            // return NaR unless -1 <= input <= 1
            return Self::NAR;
        }

        let mut s: i32;
        if f < 0x3000 {
            // input is less than 1/2
            s = 14; // convert to 28-bit fixed point
            while (f & 0x2000) == 0 {
                f <<= 1;
                s -= 2;
            }
            if (f & 0x1000) != 0 {
                s += 1;
            }
            f = (f & 0xFFF) | 0x1000;
            f = if s < 0 { f >> -s } else { f << s };
            f = poly(f);
        } else {
            f = 0x_2000_0000 - (poly(super::kernel::isqrt((0x4000 - f) << 42)) << 1);
        }
        s = 34; // convert to posit form
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

        Self::from_bits((if (ui_a >> 15) != 0 { 0x10000 - f } else { f }) as u16)
    }
}

#[inline]
fn poly(f: u64) -> u64 {
    let fsq = (f * f) >> 28;
    let mut s = 13_944 + ((fsq * 3_855) >> 26);
    s = 100_344 + ((fsq * s) >> 26);
    s = 1_780_112 + ((fsq * s) >> 25);
    s = 42_722_832 + ((fsq * s) >> 26);
    (f * s) >> 25
}
