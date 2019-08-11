use super::P16E1;

impl P16E1 {
    pub fn sin_pi(self) -> Self {
        let ui_a = self.to_bits();

        let mut f = ui_a as u64;

        let mut sign = f & 0x8000;
        if sign != 0 {
            f = 0x10000 - f; // 2's complement if negative
        }
        if f > 31743 {
            // input value is an integer?
            if f == 0x8000 {
                return Self::NAR; // sinpi(NaR) is NaR
            } else {
                return Self::ZERO; // sinpi of an integer is zero
            }
        }
        if f == 0 {
            // sinpi(0) = 0
            return Self::ZERO;
        }
        let mut s: i32;
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
        f = if s < 0 { f >> -s } else { f << s };
        f &= 0x_1FFF_FFFF; // fixed-point with 28-bit fraction
        let mut s = f >> 27; // the quadrant is the multiple of 1/2
        f &= 0x_07FF_FFFF; // input value modulo 1/2
        if (s & 2) != 0 {
            sign ^= 0x8000; // quadrants 2 and 3 flip the sign
        }
        if f == 0 {
            return Self::from_bits(if (s & 1) != 0 {
                (sign as u16) | 0x4000
            } else {
                0
            });
        }
        if (s & 1) != 0 {
            f = 0x_0800_0000 - f;
        }
        f = poly(f);
        s = 1; // convert 28-bit fixed-point to a posit
        while (f & 0x_0800_0000) == 0 {
            f <<= 1;
            s += 1;
        }
        let bit = s & 1;
        s = (s >> 1) + 14 + bit;
        if bit == 0 {
            f &= 0x_07FF_FFFF; // encode exponent bit
        }
        f |= 0x_1000_0000; // encode regime termination bit
        let bit = 1_u64 << (s - 1);
        if ((f & bit) != 0) && (((f & (bit - 1)) != 0) || ((f & (bit << 1)) != 0)) {
            // round to nearest, tie to even
            f += bit;
        }
        f >>= s;
        Self::from_bits((if sign != 0 { 0x10000 - f } else { f }) as u16)
    }
}

#[inline]
fn poly(f: u64) -> u64 {
    if f < 0x_000A_5801 {
        return (f * 102_943) >> 15; // linear approximation suffices
    }
    let fs = f >> 11;
    let fsq = (fs * fs) >> 8;
    let mut s = (fsq * 650) >> 25;
    s = (fsq * (9_813 - s)) >> 23;
    s = (fsq * (334_253 - s)) >> 23;
    s = (fsq * (5_418_741 - s)) >> 22;
    (fs * (52_707_180 - s)) >> 13
}

#[test]
fn test_sin_pi() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS16 {
        let p_a: P16E1 = rng.gen();
        let f_a = f64::from(p_a);
        let p = p_a.sin_pi();
        let f = (f_a * core::f64::consts::PI).sin();
        let expected = P16E1::from(f);
        if p.is_zero() {
            continue;
        }
        assert_eq!(p, expected);
    }
}
