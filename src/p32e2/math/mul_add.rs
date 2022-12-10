use super::P32E2;
use crate::MulAddType;

impl P32E2 {
    #[inline]
    pub const fn mul_add(self, b: Self, c: Self) -> Self {
        let ui_a = self.to_bits();
        let ui_b = b.to_bits();
        let ui_c = c.to_bits();
        mul_add(ui_a, ui_b, ui_c, crate::MulAddType::Add)
    }
    #[inline]
    pub const fn mul_sub(self, b: Self, c: Self) -> Self {
        let ui_a = self.to_bits();
        let ui_b = b.to_bits();
        let ui_c = c.to_bits();
        mul_add(ui_a, ui_b, ui_c, crate::MulAddType::SubC)
    }
    #[inline]
    pub const fn sub_product(self, a: Self, b: Self) -> Self {
        let ui_a = a.to_bits();
        let ui_b = b.to_bits();
        let ui_c = self.to_bits();
        mul_add(ui_a, ui_b, ui_c, crate::MulAddType::SubProd)
    }
}

#[allow(clippy::cognitive_complexity)]
const fn mul_add(mut ui_a: u32, mut ui_b: u32, mut ui_c: u32, op: MulAddType) -> P32E2 {
    let mut bits_more = false;
    //NaR
    if (ui_a == 0x8000_0000) || (ui_b == 0x8000_0000) || (ui_c == 0x8000_0000) {
        return P32E2::NAR;
    } else if (ui_a == 0) || (ui_b == 0) {
        return match op {
            MulAddType::SubC => P32E2::from_bits(ui_c.wrapping_neg()),
            _ => P32E2::from_bits(ui_c),
        };
    }

    let sign_a = P32E2::sign_ui(ui_a);
    let sign_b = P32E2::sign_ui(ui_b);
    let sign_c = P32E2::sign_ui(ui_c); //^ (op == softposit_mulAdd_subC);
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

    let (mut k_a, tmp) = P32E2::separate_bits_tmp(ui_a);
    let mut exp_a = (tmp >> 29) as i32; //to get 2 bits
    let frac_a = (tmp << 2) | 0x8000_0000;

    let (k_b, tmp) = P32E2::separate_bits_tmp(ui_b);
    k_a += k_b;
    exp_a += (tmp >> 29) as i32;
    let mut frac64_z = (frac_a as u64) * (((tmp << 2) | 0x8000_0000) as u64);

    if exp_a > 3 {
        k_a += 1;
        exp_a &= 0x3; // -=4
    }

    let rcarry = (frac64_z & 0x_8000_0000_0000_0000) != 0; //1st bit of frac64_z
    if rcarry {
        exp_a += 1;
        if exp_a > 3 {
            k_a += 1;
            exp_a &= 0x3;
        }
        frac64_z >>= 1;
    }

    let mut k_z;
    let mut exp_z: i32;
    if ui_c != 0 {
        let (k_c, exp_c, frac_c) = P32E2::separate_bits(ui_c);
        let mut frac64_c = (frac_c as u64) << 32;
        let mut shift_right = (((k_a - k_c) as i16) << 2) + (exp_a - exp_c) as i16;

        exp_z = if shift_right < 0 {
            // |ui_c| > |Prod|
            if shift_right <= -63 {
                bits_more = true;
                frac64_z = 0;
                shift_right = 0;
            //set bits_more to one?
            } else if (frac64_z << (64 + shift_right)) != 0 {
                bits_more = true;
            }
            if sign_z == sign_c {
                frac64_z = frac64_c + (frac64_z >> -shift_right);
            } else {
                //different signs
                frac64_z = frac64_c - (frac64_z >> -shift_right);
                sign_z = sign_c;
                if bits_more {
                    frac64_z -= 1;
                }
            }
            k_z = k_c;
            exp_c
        } else if shift_right > 0 {
            // |ui_c| < |Prod|
            //if frac32C&((1<<shift_right)-1) {bits_more = true;}
            if shift_right >= 63 {
                bits_more = true;
                frac64_c = 0;
                shift_right = 0;
            } else if (frac64_c << (64 - shift_right)) != 0 {
                bits_more = true;
            }
            if sign_z == sign_c {
                frac64_z += frac64_c >> shift_right;
            } else {
                frac64_z -= frac64_c >> shift_right;
                if bits_more {
                    frac64_z -= 1;
                }
            }
            k_z = k_a;
            exp_a
        } else {
            if (frac64_c == frac64_z) && (sign_z != sign_c) {
                //check if same number
                return P32E2::ZERO;
            } else if sign_z == sign_c {
                frac64_z += frac64_c;
            } else if frac64_z < frac64_c {
                frac64_z = frac64_c - frac64_z;
                sign_z = sign_c;
            } else {
                frac64_z -= frac64_c;
            }
            k_z = k_a; // actually can be k_c too, no diff
            exp_a //same here
        };

        let rcarry = (frac64_z & 0x_8000_0000_0000_0000) != 0; //first left bit

        if rcarry {
            exp_z += 1;
            if exp_z > 3 {
                k_z += 1;
                exp_z &= 0x3;
            }
            frac64_z = (frac64_z >> 1) & 0x7FFF_FFFF_FFFF_FFFF;
        } else {
            //for subtract cases
            if frac64_z != 0 {
                while (frac64_z >> 59) == 0 {
                    k_z -= 1;
                    frac64_z <<= 4;
                }
                while (frac64_z >> 62) == 0 {
                    exp_z -= 1;
                    frac64_z <<= 1;
                    if exp_z < 0 {
                        k_z -= 1;
                        exp_z = 3;
                    }
                }
            }
        }
    } else {
        k_z = k_a;
        exp_z = exp_a;
    }

    let (regime, reg_sz, reg_z) = P32E2::calculate_regime(k_z);

    let u_z = if reg_z > 30 {
        //max or min pos. exp and frac does not matter.
        if reg_sz {
            0x7FFF_FFFF
        } else {
            0x1
        }
    } else {
        let mut bit_n_plus_one = false;
        let frac_z = if reg_z <= 28 {
            //remove hidden bits
            frac64_z &= 0x3FFF_FFFF_FFFF_FFFF;
            bit_n_plus_one = (0x0000_0002_0000_0000 & (frac64_z >> reg_z)) != 0;
            exp_z <<= 28 - reg_z;
            (frac64_z >> (reg_z + 34)) as u32 //frac32Z>>16;
        } else {
            if reg_z == 30 {
                bit_n_plus_one = (exp_z & 0x2) != 0;
                bits_more = (exp_z & 0x1) != 0;
                exp_z = 0;
            } else if reg_z == 29 {
                bit_n_plus_one = (exp_z & 0x1) != 0;
                exp_z >>= 1;
            }
            0
        };
        let mut u_z = P32E2::pack_to_ui(regime, exp_z as u32, frac_z);

        if bit_n_plus_one {
            if (frac64_z << (31 - reg_z)) != 0 {
                bits_more = true;
            }
            u_z += (u_z & 1) | (bits_more as u32);
        }
        u_z
    };
    P32E2::from_bits(u_z).with_sign(sign_z)
}

#[test]
fn test_mul_add() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..100_000_000 {
        let p_a = P32E2::new(rng.gen());
        let p_b = P32E2::new(rng.gen());
        let p_c = P32E2::new(rng.gen());
        let f_a = f64::from(p_a);
        let f_b = f64::from(p_b);
        let f_c = f64::from(p_c);
        let p = p_a.mul_add(p_b, p_c);
        let f = f_a.mul_add(f_b, f_c);
        assert_eq!(
            p,
            P32E2::from(f),
            "\n  input: ({p_a:?}, {p_b:?}, {p_c:?})\n   or: {f_a}, {f_b}, {f_c}\n  answer: {}, expected {f}, nearest {}",
            p.to_f64(),
            P32E2::from_f64(f).to_f64()
        );
    }
}
