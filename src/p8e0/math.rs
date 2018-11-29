use super::*;
use crate::{MulAddType, WithSign};

impl P8E0 {
    #[inline]
    pub fn mul_add(self, b: Self, c: Self) -> Self {
        let ui_a = self.to_bits();
        let ui_b = b.to_bits();
        let ui_c = c.to_bits();
        mul_add(ui_a, ui_b, ui_c, crate::MulAddType::Add)
    }
    #[inline]
    pub fn round(self) -> Self {
        round(self)
    }
    #[inline]
    pub fn sqrt(self) -> Self {
        sqrt(self)
    }
}

impl Q8E0 {
    #[inline]
    pub fn fdp_add(self, p_a: P8E0, p_b: P8E0) -> Self {
        q8_fdp_add(self, p_a, p_b)
    }
    #[inline]
    pub fn fdp_sub(self, p_a: P8E0, p_b: P8E0) -> Self {
        q8_fdp_sub(self, p_a, p_b)
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
        return ZERO;
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
        INFINITY
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
        return INFINITY;
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

    let rcarry = (frac16_z >> 15) != 0; //1st bit of frac16_z
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
                return ZERO;
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

fn q8_fdp_add(q: Q8E0, p_a: P8E0, p_b: P8E0) -> Q8E0 {
    let uq_z1 = q.to_bits();

    let mut ui_a = p_a.to_bits();
    let mut ui_b = p_b.to_bits();

    if q.is_nan() || p_a.is_nan() || p_b.is_nan() {
        return Q8E0::new(-0x8000_0000);
    } else if (ui_a == 0) || (ui_b == 0) {
        return q;
    }

    //max pos (sign plus and minus)
    let sign_a = P8E0::sign_ui(ui_a);
    let sign_b = P8E0::sign_ui(ui_b);
    let sign_z2 = sign_a ^ sign_b;

    if sign_a {
        ui_a = ui_a.wrapping_neg();
    }
    if sign_b {
        ui_b = ui_b.wrapping_neg();
    }

    let (mut k_a, frac_a) = P8E0::separate_bits(ui_a);

    let (k_b, frac_b) = P8E0::separate_bits(ui_b);
    k_a += k_b;

    let mut frac32_z = ((frac_a * frac_b) as u32) << 16;

    let rcarry = (frac32_z >> 31) != 0; //1st bit (position 2) of frac32_z, hidden bit is 4th bit (position 3)
    if rcarry {
        k_a += 1;
        frac32_z >>= 1;
    }

    //default dot is between bit 19 and 20, extreme left bit is bit 0. Last right bit is bit 31.
    //Scale = 2^es * k + e  => 2k + e // firstPost = 19-k_a, shift = firstPos -1 (because frac32_z start from 2nd bit)
    //int firstPos = 19 - k_a;
    let shift_right = 18 - k_a;

    let mut uq_z2 = frac32_z >> shift_right;

    if sign_z2 {
        uq_z2 = uq_z2.wrapping_neg();
    }

    //Addition
    let uq_z = uq_z2 + uq_z1;

    //Exception handling
    let q_z = Q8E0::from_bits(uq_z);
    if q_z.is_nan() {
        Q8E0::new(0)
    } else {
        q_z
    }
}

//q - (p_a*p_b)

fn q8_fdp_sub(q: Q8E0, p_a: P8E0, p_b: P8E0) -> Q8E0 {
    let uq_z1 = q.to_bits();

    let mut ui_a = p_a.to_bits();
    let mut ui_b = p_b.to_bits();

    if q.is_nan() || p_a.is_nan() || p_b.is_nan() {
        return Q8E0::new(-0x8000_0000);
    } else if (ui_a == 0) || (ui_b == 0) {
        return q;
    }

    //max pos (sign plus and minus)
    let sign_a = P8E0::sign_ui(ui_a);
    let sign_b = P8E0::sign_ui(ui_b);
    let sign_z2 = sign_a ^ sign_b;

    if sign_a {
        ui_a = ui_a.wrapping_neg();
    }
    if sign_b {
        ui_b = ui_b.wrapping_neg();
    }

    let (mut k_a, frac_a) = P8E0::separate_bits(ui_a);

    let (k_b, frac_b) = P8E0::separate_bits(ui_b);
    k_a += k_b;

    let mut frac32_z = ((frac_a * frac_b) as u32) << 16;

    let rcarry = (frac32_z >> 31) != 0; //1st bit (position 2) of frac32_z, hidden bit is 4th bit (position 3)
    if rcarry {
        k_a += 1;
        frac32_z >>= 1;
    }

    //default dot is between bit 19 and 20, extreme left bit is bit 0. Last right bit is bit 31.
    //Scale = 2^es * k + e  => 2k + e // firstPost = 19-k_a, shift = firstPos -1 (because frac32_z start from 2nd bit)
    //int firstPos = 19 - k_a;
    let shift_right = 18 - k_a;

    let mut uq_z2 = frac32_z >> shift_right;

    //This is the only difference from ADD (sign_z2) and (!sign_z2)
    if !sign_z2 {
        uq_z2 = uq_z2.wrapping_neg();
    }

    //Addition
    let uq_z = uq_z2 + uq_z1;

    //Exception handling
    let q_z = Q8E0::from_bits(uq_z);
    if q_z.is_nan() {
        Q8E0::new(0)
    } else {
        q_z
    }
}
