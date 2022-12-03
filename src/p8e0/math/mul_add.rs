use core::cmp::Ordering;

use super::P8E0;
use crate::{MulAddType, WithSign};

impl P8E0 {
    #[inline]
    pub fn mul_add(self, b: Self, c: Self) -> Self {
        let ui_a = self.to_bits();
        let ui_b = b.to_bits();
        let ui_c = c.to_bits();
        mul_add(ui_a, ui_b, ui_c, crate::MulAddType::Add)
    }
}

//softposit_mulAdd_subC => (ui_a*ui_b)-ui_c
//softposit_mulAdd_subProd => ui_c - (ui_a*ui_b)
//Default is always op==0
#[allow(clippy::cognitive_complexity)]
fn mul_add(mut ui_a: u8, mut ui_b: u8, mut ui_c: u8, op: MulAddType) -> P8E0 {
    let mut bits_more = false;

    //NaR
    if (ui_a == 0x80) || (ui_b == 0x80) || (ui_c == 0x80) {
        return P8E0::NAR;
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

        match shift_right.cmp(&0) {
            Ordering::Less => {
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
            }
            Ordering::Greater => {
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
            }
            Ordering::Equal => {
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
