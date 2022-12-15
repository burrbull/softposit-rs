use super::P16E1;

impl P16E1 {
    pub const fn sin_pi(self) -> Self {
        let ui_a = self.to_bits();

        let mut sign = ui_a & 0x8000;
        let mut f = if sign != 0 {
            ui_a.wrapping_neg() // 2's complement if negative
        } else {
            ui_a
        };
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
        let f = ((f & 0x0FFF) | 0x1000) as u64; // get 12-bit fraction and restore hidden bit
        let mut f = if s < 0 { f >> -s } else { f << s };
        f &= 0x_1FFF_FFFF; // fixed-point with 28-bit fraction
        let s = f >> 27; // the quadrant is the multiple of 1/2
        f &= 0x_07FF_FFFF; // input value modulo 1/2
        if (s & 2) != 0 {
            sign ^= 0x8000; // quadrants 2 and 3 flip the sign
        }
        if f == 0 {
            return Self::from_bits(if (s & 1) != 0 { sign | 0x4000 } else { 0 });
        }
        if (s & 1) != 0 {
            f = 0x_0800_0000 - f;
        }
        let mut f = poly(f);
        let mut s = 1u8; // convert 28-bit fixed-point to a posit
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
        Self::from_bits((f >> s) as u16).with_sign(sign != 0)
    }
}

#[inline]
const fn poly(f: u64) -> u64 {
    if f < 0x_000A_5801 {
        return (f * 102_943) >> 15; // linear approximation suffices
    }
    let fs = f >> 11;
    let fsq = (fs * fs) >> 8;
    let s = (fsq * 650) >> 25;
    let s = (fsq * (9_813 - s)) >> 23;
    let s = (fsq * (334_253 - s)) >> 23;
    let s = (fsq * (5_418_741 - s)) >> 22;
    (fs * (52_707_180 - s)) >> 13
}

#[test]
fn test_sin_pi() {
    for i in i16::MIN..i16::MAX {
        let p_a = P16E1::new(i);
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
