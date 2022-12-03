use core::cmp::Ordering;

use super::P16E1;
use crate::{MulAddType, WithSign};

impl P16E1 {
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
fn mul_add(mut ui_a: u16, mut ui_b: u16, mut ui_c: u16, op: MulAddType) -> P16E1 {
    let mut bits_more = false;

    //NaR
    if (ui_a == 0x8000) || (ui_b == 0x8000) || (ui_c == 0x8000) {
        return P16E1::NAR;
    } else if (ui_a == 0) || (ui_b == 0) {
        return match op {
            MulAddType::SubC => P16E1::from_bits(ui_c.wrapping_neg()),
            _ => P16E1::from_bits(ui_c),
        };
    }

    let sign_a = P16E1::sign_ui(ui_a);
    let sign_b = P16E1::sign_ui(ui_b);
    let sign_c = P16E1::sign_ui(ui_c); //^ (op == softposit_mulAdd_subC);
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

    let (mut k_a, tmp) = P16E1::separate_bits_tmp(ui_a);
    let mut exp_a = (tmp >> 14) as i8;
    let frac_a = 0x8000 | (tmp << 1); //use first bit here for hidden bit to get more bits

    let (k_b, tmp) = P16E1::separate_bits_tmp(ui_b);
    k_a += k_b;

    exp_a += (tmp >> 14) as i8;
    let mut frac32_z = (frac_a as u32) * ((0x8000 | (tmp << 1)) as u32); // first bit hidden bit

    if exp_a > 1 {
        k_a += 1;
        exp_a ^= 0x2;
    }

    let rcarry = (frac32_z & 0x8000_0000) != 0; //1st bit of frac32_z
    if rcarry {
        if exp_a != 0 {
            k_a += 1;
        }
        exp_a ^= 1;
        frac32_z >>= 1;
    }

    let mut k_z: i8;
    let mut exp_z: i8;
    //Add
    if ui_c != 0 {
        let (k_c, exp_c, frac_c) = P16E1::separate_bits(ui_c);
        let mut frac32_c = (frac_c as u32) << 16;

        let mut shift_right: i16 = (((k_a - k_c) as i16) << 1) + ((exp_a - exp_c) as i16); //actually this is the scale

        exp_z = match shift_right.cmp(&0) {
            Ordering::Less => {
                // |ui_c| > |Prod Z|
                if shift_right <= -31 {
                    bits_more = true;
                    frac32_z = 0;
                    shift_right = 0;
                } else if (frac32_z << (32 + shift_right)/*&0xFFFF_FFFF*/) != 0 {
                    bits_more = true;
                }
                if sign_z == sign_c {
                    frac32_z = frac32_c + (frac32_z >> -shift_right);
                } else {
                    //different signs
                    frac32_z = frac32_c - (frac32_z >> -shift_right);
                    sign_z = sign_c;
                    if bits_more {
                        frac32_z -= 1;
                    }
                }
                k_z = k_c;
                exp_c
            }
            Ordering::Greater => {
                // |ui_c| < |Prod|
                //if (frac32_c&((1<<shift_right)-1)) bits_more = 1;
                if shift_right >= 31 {
                    bits_more = true;
                    frac32_c = 0;
                    shift_right = 0;
                } else if (frac32_c << (32 - shift_right)) != 0 {
                    bits_more = true;
                }
                if sign_z == sign_c {
                    frac32_z += frac32_c >> shift_right;
                } else {
                    frac32_z -= frac32_c >> shift_right;
                    if bits_more {
                        frac32_z -= 1;
                    }
                }
                k_z = k_a;
                exp_a
            }
            Ordering::Equal => {
                if (frac32_c == frac32_z) && (sign_z != sign_c) {
                    //check if same number
                    return P16E1::ZERO;
                } else if sign_z == sign_c {
                    frac32_z += frac32_c;
                } else if frac32_z < frac32_c {
                    frac32_z = frac32_c - frac32_z;
                    sign_z = sign_c;
                } else {
                    frac32_z -= frac32_c;
                }
                k_z = k_a; // actually can be k_c too, no diff
                exp_a //same here
            }
        };

        let rcarry = (frac32_z & 0x8000_0000) != 0; //first left bit
        if rcarry {
            if exp_z != 0 {
                k_z += 1;
            }
            exp_z ^= 1;
            if (frac32_z & 0x1) != 0 {
                bits_more = true;
            }
            frac32_z >>= 1 /*&0x7FFF_FFFF*/;
        } else {
            //for subtract cases
            if frac32_z != 0 {
                while (frac32_z >> 29) == 0 {
                    k_z -= 1;
                    frac32_z <<= 2;
                }
            }
            let ecarry = ((0x4000_0000 & frac32_z) >> 30) != 0;

            if !ecarry {
                if exp_z == 0 {
                    k_z -= 1;
                }
                exp_z ^= 1;
                frac32_z <<= 1;
            }
        }
    } else {
        k_z = k_a;
        exp_z = exp_a;
    }

    let (regime, reg_sz, reg_z) = P16E1::calculate_regime(k_z);

    let u_z = if reg_z > 14 {
        //max or min pos. exp and frac does not matter.
        if reg_sz {
            0x7FFF
        } else {
            0x1
        }
    } else {
        //remove hidden bits
        frac32_z &= 0x3FFF_FFFF;
        let mut frac_z = (frac32_z >> (reg_z + 17)) as u16;

        let mut bit_n_plus_one = false;
        if reg_z != 14 {
            bit_n_plus_one = ((frac32_z >> reg_z) & 0x10000) != 0;
        } else if frac32_z > 0 {
            frac_z = 0;
            bits_more = true;
        }
        if (reg_z == 14) && (exp_z != 0) {
            bit_n_plus_one = true;
        }
        let mut u_z = P16E1::pack_to_ui(regime, reg_z, exp_z as u16, frac_z);
        if bit_n_plus_one {
            if (frac32_z << (16 - reg_z)) != 0 {
                bits_more = true;
            }
            u_z += (u_z & 1) | (bits_more as u16);
        }
        u_z
    };

    P16E1::from_bits(u_z.with_sign(sign_z))
}

#[test]
fn test_mul_add() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS16 {
        let p_a: P16E1 = rng.gen();
        let p_b: P16E1 = rng.gen();
        let p_c: P16E1 = rng.gen();
        let f_a = f64::from(p_a);
        let f_b = f64::from(p_b);
        let f_c = f64::from(p_c);
        let p = p_a.mul_add(p_b, p_c);
        let f = f_a.mul_add(f_b, f_c);
        assert_eq!(p, P16E1::from(f));
    }
}
