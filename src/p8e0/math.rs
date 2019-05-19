use super::P8E0;
use crate::{MulAddType, WithSign};

const HALF: P8E0 = P8E0::new(0x_20);
const TWO: P8E0 = P8E0::new(0x_60);

impl crate::MathConsts for P8E0 {
    const E: Self = Self::new(0x_66);
    const FRAC_1_PI: Self = Self::new(0x_14);
    const FRAC_1_SQRT_2: Self = Self::new(0x_2d);
    const FRAC_2_PI: Self = Self::new(0x_29);
    const FRAC_2_SQRT_PI: Self = Self::new(0x_44);
    const FRAC_PI_2: Self = Self::new(0x_52);
    const FRAC_PI_3: Self = Self::new(0x_42);
    const FRAC_PI_4: Self = Self::new(0x_32);
    const FRAC_PI_6: Self = Self::new(0x_22);
    const FRAC_PI_8: Self = Self::new(0x_19);
    const LN_10: Self = Self::new(0x_62);
    const LN_2: Self = Self::new(0x_2c);
    const LOG10_E: Self = Self::new(0x_1c);
    const LOG2_E: Self = Self::new(0x_4e);
    const PI: Self = Self::new(0x_69);
    const SQRT_2: Self = Self::new(0x_4d);
    const LOG2_10: Self = Self::new(0x_6b);
    const LOG10_2: Self = Self::new(0x_13);
}

impl P8E0 {
    #[inline]
    pub fn mul_add(self, b: Self, c: Self) -> Self {
        let ui_a = self.to_bits();
        let ui_b = b.to_bits();
        let ui_c = c.to_bits();
        mul_add(ui_a, ui_b, ui_c, crate::MulAddType::Add)
    }
    #[inline]
    pub fn floor(self) -> Self {
        (self - HALF).round()
    }
    #[inline]
    pub fn ceil(self) -> Self {
        (self + HALF).round()
    }
    #[inline]
    pub fn round(self) -> Self {
        round(self)
    }
    // TODO: optimize
    #[inline]
    pub fn trunc(self) -> Self {
        if self > Self::ZERO {
            self.floor()
        } else {
            self.ceil()
        }
    }
    #[inline]
    pub fn fract(self) -> Self {
        self - self.trunc()
    }
    #[inline]
    pub fn div_euclid(self, rhs: Self) -> Self {
        let q = (self / rhs).trunc();
        if self % rhs < Self::ZERO {
            return if rhs > Self::ZERO {
                q - Self::ONE
            } else {
                q + Self::ONE
            };
        }
        q
    }
    #[inline]
    pub fn rem_euclid(self, rhs: Self) -> Self {
        let r = self % rhs;
        if r < Self::ZERO {
            r + rhs.abs()
        } else {
            r
        }
    }
    #[inline]
    pub fn powi(self, _n: i32) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn powf(self, _n: Self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn sqrt(self) -> Self {
        sqrt(self)
    }
    #[inline]
    pub fn exp(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn exp2(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn ln(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn log(self, _base: Self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn log2(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn log10(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn cbrt(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn hypot(self, _other: Self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn sin(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn cos(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn tan(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn asin(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn acos(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn atan(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn atan2(self, _other: Self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn sin_cos(self) -> (Self, Self) {
        (self.sin(), self.cos())
    }
    #[inline]
    pub fn exp_m1(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn ln_1p(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn sinh(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn cosh(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn tanh(self) -> Self {
        unimplemented!()
    }
    #[inline]
    pub fn asinh(self) -> Self {
        if self.is_nan() {
            self
        } else {
            (self + ((self * self) + Self::ONE).sqrt()).ln()
        }
    }
    #[inline]
    pub fn acosh(self) -> Self {
        match self {
            x if x < Self::ONE => Self::NAN,
            x => (x + ((x * x) - Self::ONE).sqrt()).ln(),
        }
    }
    #[inline]
    pub fn atanh(self) -> Self {
        HALF * ((TWO * self) / (Self::ONE - self)).ln_1p()
    }
}

fn round(p_a: P8E0) -> P8E0 {
    let mut mask = 0x20_u8;
    let mut scale = 0_u8;

    let mut u_a = p_a.to_bits();
    let mut ui_a = u_a;
    let sign = ui_a > 0x80;

    // sign is True if p_a > NaR.
    if sign {
        ui_a = ui_a.wrapping_neg();
    }
    if ui_a <= 0x20 {
        // 0 <= |p_a| <= 1/2 rounds to zero.
        return P8E0::ZERO;
    } else if ui_a < 0x50 {
        // 1/2 < x < 3/2 rounds to 1.
        u_a = 0x40;
    } else if ui_a <= 0x64 {
        // 3/2 <= x <= 5/2 rounds to 2.
        u_a = 0x60;
    } else if ui_a >= 0x78 {
        // If |A| is 8 or greater, leave it unchanged.
        return P8E0::from_bits(u_a); // This also takes care of the NaR case, 0x80.
    } else {
        while (mask & ui_a) != 0 {
            scale += 1;
            mask >>= 1;
        }

        mask >>= scale;
        let bit_last = (ui_a & mask) != 0;

        mask >>= 1;
        let mut tmp = ui_a & mask;
        let bit_n_plus_one = tmp != 0;
        ui_a ^= tmp;
        tmp = ui_a & (mask - 1); //bits_more
        ui_a ^= tmp;

        if bit_n_plus_one && (((bit_last as u8) | tmp) != 0) {
            ui_a += mask << 1;
        }
        u_a = ui_a;
    }
    P8E0::from_bits(u_a.with_sign(sign))
}

const P8E0_SQRT: [u8; 128] = [
    0, 8, 11, 14, 16, 18, 20, 21, 23, 24, 25, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 38,
    39, 40, 41, 42, 42, 43, 44, 45, 45, 46, 47, 47, 48, 49, 49, 50, 51, 51, 52, 52, 53, 54, 54, 55,
    55, 56, 57, 57, 58, 58, 59, 59, 60, 60, 61, 61, 62, 62, 63, 63, 64, 64, 65, 65, 66, 66, 67, 67,
    68, 68, 69, 69, 70, 70, 70, 71, 71, 72, 72, 72, 73, 73, 74, 74, 74, 75, 75, 75, 76, 76, 77, 77,
    77, 79, 80, 81, 83, 84, 85, 86, 87, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 100, 101,
    102, 103, 105, 108, 110, 112, 114, 115, 120,
];

#[inline]
fn sqrt(p_a: P8E0) -> P8E0 {
    let ui_a = p_a.to_bits();

    if ui_a >= 0x80 {
        P8E0::INFINITY
    } else {
        P8E0::from_bits(P8E0_SQRT[ui_a as usize])
    }
}

//softposit_mulAdd_subC => (ui_a*ui_b)-ui_c
//softposit_mulAdd_subProd => ui_c - (ui_a*ui_b)
//Default is always op==0
fn mul_add(mut ui_a: u8, mut ui_b: u8, mut ui_c: u8, op: MulAddType) -> P8E0 {
    let mut bits_more = false;

    //NaR
    if (ui_a == 0x80) || (ui_b == 0x80) || (ui_c == 0x80) {
        return P8E0::INFINITY;
    } else if (ui_a == 0) || (ui_b == 0) {
        return match op {
            MulAddType::SubC => P8E0::from_bits(ui_c.wrapping_neg()),
            _ => P8E0::from_bits(ui_c),
        };
    }

    let sign_a = P8E0::sign_ui(ui_a);
    let sign_b = P8E0::sign_ui(ui_b);
    let sign_c = P8E0::sign_ui(ui_c); //^ (op == softposit_mulAdd_subC);
    let mut sign_z = sign_a ^ sign_b; // ^ (op == softposit_mulAdd_subProd);

    if sign_a {
        ui_a = ui_a.wrapping_neg();
    }
    if sign_b {
        ui_b = ui_b.wrapping_neg();
    }
    if sign_c {
        ui_c = ui_c.wrapping_neg();
    }

    let (mut k_a, frac_a) = P8E0::separate_bits(ui_a);

    let (k_b, frac_b) = P8E0::separate_bits(ui_b);
    k_a += k_b;
    let mut frac16_z = (frac_a as u16) * (frac_b as u16);

    let rcarry = (frac16_z & 0x_8000) != 0; //1st bit of frac16_z
    if rcarry {
        k_a += 1;
        frac16_z >>= 1;
    }

    let mut k_z: i8;
    if ui_c != 0 {
        let (k_c, frac_c) = P8E0::separate_bits(ui_c);
        let mut frac16_c = (frac_c as u16) << 7;
        let mut shift_right = k_a - k_c;

        if shift_right < 0 {
            // |ui_c| > |Prod|
            if shift_right <= -15 {
                bits_more = true;
                frac16_z = 0;
                shift_right = 0;
            } else if (frac16_z << (16 + shift_right)/*&0xFFFF*/) != 0 {
                bits_more = true;
            }
            if sign_z == sign_c {
                frac16_z = frac16_c + (frac16_z >> -shift_right);
            } else {
                //different signs
                frac16_z = frac16_c - (frac16_z >> -shift_right);
                sign_z = sign_c;
                if bits_more {
                    frac16_z -= 1;
                }
            }
            k_z = k_c;
        } else if shift_right > 0 {
            // |ui_c| < |Prod|

            if shift_right >= 15 {
                bits_more = true;
                frac16_c = 0;
                shift_right = 0;
            } else if (frac16_c << (16 - shift_right)/*&0xFFFF*/) != 0 {
                bits_more = true;
            }
            if sign_z == sign_c {
                frac16_z += frac16_c >> shift_right;
            } else {
                frac16_z -= frac16_c >> shift_right;
                if bits_more {
                    frac16_z -= 1;
                }
            }
            k_z = k_a;
        } else {
            if (frac16_c == frac16_z) && (sign_z != sign_c) {
                //check if same number
                return P8E0::ZERO;
            } else if sign_z == sign_c {
                frac16_z += frac16_c;
            } else if frac16_z < frac16_c {
                frac16_z = frac16_c - frac16_z;
                sign_z = sign_c;
            } else {
                frac16_z -= frac16_c;
            }
            k_z = k_a; // actually can be k_c too, no diff
        }

        let rcarry = (0x8000 & frac16_z) != 0; //first left bit
        if rcarry {
            k_z += 1;
            frac16_z = (frac16_z >> 1) & 0x7FFF;
        } else {
            //for subtract cases
            if frac16_z != 0 {
                while (frac16_z >> 14) == 0 {
                    k_z -= 1;
                    frac16_z <<= 1;
                }
            }
        }
    } else {
        k_z = k_a;
    }

    let (regime, reg_sz, reg_z) = P8E0::calculate_regime(k_z);

    let u_z = if reg_z > 6 {
        //max or min pos. exp and frac does not matter.
        if reg_sz {
            0x7F
        } else {
            0x1
        }
    } else {
        //remove hidden bits
        frac16_z &= 0x3FFF;

        let frac_z = ((frac16_z >> reg_z) >> 8) as u8;

        let bit_n_plus_one = ((frac16_z >> reg_z) & 0x80) != 0;
        let mut u_z = P8E0::pack_to_ui(regime, frac_z);

        if bit_n_plus_one {
            if (frac16_z << (9 - reg_z)) != 0 {
                bits_more = true;
            }
            u_z += (u_z & 1) | (bits_more as u8);
        }
        u_z
    };
    P8E0::from_bits(u_z.with_sign(sign_z))
}

#[test]
fn test_mul_add() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS8 {
        let p_a: P8E0 = rng.gen();
        let p_b: P8E0 = rng.gen();
        let p_c: P8E0 = rng.gen();
        let f_a = f64::from(p_a);
        let f_b = f64::from(p_b);
        let f_c = f64::from(p_c);
        let p = p_a.mul_add(p_b, p_c);
        let f = f_a.mul_add(f_b, f_c);
        assert_eq!(p, P8E0::from(f));
    }
}

#[test]
fn test_sqrt() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS8 {
        let p_a: P8E0 = rng.gen();
        let f_a = f64::from(p_a);
        let p = p_a.sqrt();
        let f = f_a.sqrt();
        assert_eq!(p, P8E0::from(f));
    }
}

#[test]
fn test_round() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS8 {
        let p_a: P8E0 = rng.gen();
        let f_a = f64::from(p_a);
        let p = p_a.round();
        let f = f_a.round();
        if (f - f_a).abs() == 0.5 {
            continue;
        }
        assert_eq!(p, P8E0::from(f));
    }
}
