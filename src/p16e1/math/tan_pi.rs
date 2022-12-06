use super::P16E1;

impl P16E1 {
    pub const fn tan_pi(self) -> Self {
        let ui_a = self.to_bits();

        let mut f = ui_a as u64;

        let mut sign = f & 0x8000;
        if sign != 0 {
            f = 0x_0001_0000 - f; // 2's complement if negative
        }
        if f > 31743 {
            // input value is an integer?
            return if f == 0x8000 { Self::NAR } else { Self::ZERO }; // handles NaR and integer cases
        }
        if f != 0 {
            // decode posit to fixed-point
            let mut s: i32;
            if (f & 0x4000) != 0 {
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
            f = (f & 0x0FFF) | 0x1000; // get 12-bit fraction; restore hidden bit
            f = if s < 0 { f >> -s } else { f << s };
        }
        f &= 0x_0FFF_FFFF; // 28-bit fraction

        if f == 0 {
            // tanpi is zero for integer inputs
            return Self::ZERO;
        }
        let s = f >> 27; // record quadrant = multiple of 1/2
        f &= 0x_07FF_FFFF; // input modulo 1/2
        if f == 0 {
            // tanpi is NaR at 1/2 * odd integers
            return Self::NAR;
        }
        if s != 0 {
            sign ^= 0x8000; // flip sign for odd quadrants
        }
        if (f & 0x_03FF_FFFF) == 0 {
            // tanpi is +1 or -1 at 1/4 * odd integers
            return Self::from_bits(sign as u16 | 0x4000);
        }
        if sign != 0 {
            f = 0x_0800_0000 - f; // reverse input direction for odd quadrants
        }
        if (ui_a & 0x8000) != 0 {
            f = 0x_0800_0000 - f; // reverse if original input value is negative
        }

        f = poly(f); // apply the polynomial approximation

        let mut s: i32;
        if f > 0x_0FFF_FFFF {
            // convert 28-bit fixed-point to a posit
            s = 12;
            while (f & 0x_0100_0000_0000) == 0 {
                f <<= 1;
                s -= 1;
            }
            if (s & 1) == 0 {
                f &= 0x_00FF_FFFF_FFFF;
            }
            s = (s >> 1) + 28;
            f |= (2_u64 << (14 + s)) - (1_u64 << 42);
        } else {
            s = 1;
            while (f & 0x_0800_0000) == 0 {
                f <<= 1;
                s += 1;
            }
            let bit = s & 1;
            s = (s >> 1) + 14 + bit;
            if bit == 0 {
                f &= 0x_07FF_FFFF;
            }
            f |= 0x_1000_0000;
        }
        let bit = 1_u64 << (s - 1);
        if (f & bit) != 0 {
            // round to nearest, tie to even
            if ((f & (bit - 1)) != 0) || ((f & (bit << 1)) != 0) {
                f += bit;
            }
        }
        f >>= s;
        Self::from_bits((if sign != 0 { 0x_0001_0000 - f } else { f }) as u16)
    }
}

#[inline]
const fn poly(f: u64) -> u64 {
    if f < 0xE001 {
        return (f * 102_943) >> 15; // linear approximation suffices
    }
    let fs = f >> 9;
    let fsq = (fs * fs) >> 10;
    // alternating num, den may help superscalar speed
    let mut num = (fsq * 182_527) >> 27;
    let den = (fsq * 13_335_493) >> 25;
    num = (fsq * (3_648_552 - num)) >> 23;
    let den = 0x_0800_0000 - ((fsq * (295_106_440 - den)) >> 27);
    num = (fs * (105_414_368 - num)) << 11;
    num / den
}

#[test]
fn test_tan_pi() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS16 {
        let p_a: P16E1 = rng.gen();
        let f_a = f64::from(p_a);
        let p = p_a.tan_pi();
        let f = (f_a * core::f64::consts::PI).tan();
        let expected = P16E1::from(f);
        if p.is_zero() || p.is_nar() {
            continue;
        }
        assert_eq!(p, expected);
    }
}
