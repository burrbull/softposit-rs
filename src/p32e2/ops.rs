use super::*;
use crate::WithSign;
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

impl Neg for P32E2 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self::from_bits(self.to_bits().wrapping_neg())
    }
}

impl AddAssign for P32E2 {
    #[inline]
    fn add_assign(&mut self, other: Self) {
        *self = *self + other
    }
}

impl SubAssign for P32E2 {
    #[inline]
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other
    }
}

impl MulAssign for P32E2 {
    #[inline]
    fn mul_assign(&mut self, other: Self) {
        *self = *self * other
    }
}

impl DivAssign for P32E2 {
    #[inline]
    fn div_assign(&mut self, other: Self) {
        *self = *self / other
    }
}

impl Add for P32E2 {
    type Output = Self;
    #[inline]
    fn add(self, other: Self) -> Self {
        let ui_a = self.to_bits();
        let ui_b = other.to_bits();

        //Zero or infinity
        if (ui_a == 0) || (ui_b == 0) {
            // Not required but put here for speed
            P32E2::from_bits(ui_a | ui_b)
        } else if (ui_a == 0x8000_0000) || (ui_b == 0x8000_0000) {
            P32E2::from_bits(0x8000_0000)
        } else {
            //different signs
            if ((ui_a ^ ui_b) >> 31) != 0 {
                sub_mags_p32(ui_a, ui_b)
            } else {
                add_mags_p32(ui_a, ui_b)
            }
        }
    }
}

impl Sub for P32E2 {
    type Output = Self;
    #[inline]
    fn sub(self, other: Self) -> Self {
        let ui_a = self.to_bits();
        let ui_b = other.to_bits();

        //infinity
        if (ui_a == 0x8000_0000) || (ui_b == 0x8000_0000) {
            P32E2::from_bits(0x8000_0000)
        } else if (ui_a == 0) || (ui_b == 0) {
            //Zero
            P32E2::from_bits(ui_a | ui_b.wrapping_neg())
        } else {
            //different signs
            if ((ui_a ^ ui_b) >> 31) != 0 {
                add_mags_p32(ui_a, ui_b.wrapping_neg())
            } else {
                sub_mags_p32(ui_a, ui_b.wrapping_neg())
            }
        }
    }
}

impl Div for P32E2 {
    type Output = Self;
    #[inline]
    fn div(self, other: Self) -> Self {
        let mut u_z: u32;

        let mut ui_a = self.to_bits();
        let mut ui_b = other.to_bits();

        //Zero or infinity
        if (ui_a == 0x8000_0000) || (ui_b == 0x8000_0000) || (ui_b == 0) {
            return P32E2::from_bits(0x8000_0000);
        } else if ui_a == 0 {
            return P32E2::from_bits(0);
        }

        let sign_a = P32E2::sign_ui(ui_a);
        let sign_b = P32E2::sign_ui(ui_b);
        let sign_z = sign_a ^ sign_b;

        if sign_a {
            ui_a = ui_a.wrapping_neg()
        };
        if sign_b {
            ui_b = ui_b.wrapping_neg()
        };

        let (mut k_a, mut exp_a, mut frac_a) = P32E2::separate_bits(ui_a);

        let frac64_a = (frac_a as u64) << 30;

        let (k_b, exp_b, frac_b) = P32E2::separate_bits(ui_b);
        k_a -= k_b;
        exp_a -= exp_b;

        let (quot, rem) = crate::lldiv(frac64_a as i64, frac_b as i64);
        let mut frac64_z = quot as u64;

        if exp_a < 0 {
            exp_a += 4;
            k_a -= 1;
        }
        if frac64_z != 0 {
            let rcarry = (frac64_z >> 30) != 0; // this is the hidden bit (14th bit) , extreme right bit is bit 0
            if !rcarry {
                if exp_a == 0 {
                    k_a -= 1;
                    exp_a = 3;
                } else {
                    exp_a -= 1;
                }
                frac64_z <<= 1;
            }
        }

        let (regime, reg_sa, reg_a) = P32E2::calculate_regime(k_a);

        if reg_a > 30 {
            //max or min pos. exp and frac does not matter.
            u_z = if reg_sa { 0x7FFF_FFFF } else { 0x1 };
        } else {
            //remove carry and rcarry bits and shift to correct position
            frac64_z &= 0x3FFF_FFFF;

            frac_a = (frac64_z >> (reg_a + 2)) as u32;

            let mut bit_n_plus_one = false;
            let mut bits_more = false;
            if reg_a <= 28 {
                bit_n_plus_one = ((frac64_z >> (reg_a + 1)) & 0x1) != 0;
                exp_a <<= 28 - reg_a;
                if bit_n_plus_one {
                    bits_more = (((1 << (reg_a + 1)) - 1) & frac64_z) != 0;
                }
            } else {
                if reg_a == 30 {
                    bit_n_plus_one = (exp_a & 0x2) != 0;
                    bits_more = (exp_a & 0x1) != 0;
                    exp_a = 0;
                } else if reg_a == 29 {
                    bit_n_plus_one = (exp_a & 0x1) != 0;
                    exp_a >>= 1; //taken care of by the pack algo
                }
                if frac64_z > 0 {
                    frac_a = 0;
                    bits_more = true;
                }
            }
            if rem != 0 {
                bits_more = true;
            }

            u_z = P32E2::pack_to_ui(regime, exp_a as u32, frac_a);
            if bit_n_plus_one {
                u_z += (u_z & 1) | (bits_more as u32);
            }
        }

        P32E2::from_bits(u_z.with_sign(sign_z))
    }
}

impl Mul for P32E2 {
    type Output = Self;
    #[inline]
    fn mul(self, other: Self) -> Self {
        let mut ui_a = self.to_bits();
        let mut ui_b = other.to_bits();

        //NaR or Zero
        if (ui_a == 0x8000_0000) || (ui_b == 0x8000_0000) {
            return P32E2::from_bits(0x8000_0000);
        } else if (ui_a == 0) || (ui_b == 0) {
            return P32E2::from_bits(0);
        }

        let sign_a = P32E2::sign_ui(ui_a);
        let sign_b = P32E2::sign_ui(ui_b);
        let sign_z = sign_a ^ sign_b;

        if sign_a {
            ui_a = ui_a.wrapping_neg()
        };
        if sign_b {
            ui_b = ui_b.wrapping_neg()
        };

        let (mut k_a, mut exp_a, mut frac_a) = P32E2::separate_bits(ui_a);

        let (k_b, exp_b, frac_b) = P32E2::separate_bits(ui_b);
        k_a += k_b;
        exp_a += exp_b;
        let mut frac64_z = (frac_a as u64) * (frac_b as u64);

        if exp_a > 3 {
            k_a += 1;
            exp_a &= 0x3; // -=4
        }

        let rcarry = (frac64_z >> 61) != 0; //3rd bit of frac64_z
        if rcarry {
            exp_a += 1;
            if exp_a > 3 {
                k_a += 1;
                exp_a &= 0x3;
            }
            frac64_z >>= 1;
        }
        let (regime, reg_sa, reg_a) = P32E2::calculate_regime(k_a);

        let u_z = if reg_a > 30 {
            //max or min pos. exp and frac does not matter.
            if reg_sa {
                0x7FFF_FFFF
            } else {
                0x1
            }
        } else {
            //remove carry and rcarry bits and shift to correct position (2 bits exp, so + 1 than 16 bits)
            frac64_z = (frac64_z & 0xFFF_FFFF_FFFF_FFFF) >> reg_a;
            frac_a = (frac64_z >> 32) as u32;

            let mut bit_n_plus_one = false;
            if reg_a <= 28 {
                bit_n_plus_one = (0x8000_0000 & frac64_z) != 0;
                exp_a <<= 28 - reg_a;
            } else {
                if reg_a == 30 {
                    bit_n_plus_one = (exp_a & 0x2) != 0;
                    exp_a = 0;
                } else if reg_a == 29 {
                    bit_n_plus_one = (exp_a & 0x1) != 0;
                    exp_a >>= 1; //taken care of by the pack algo
                }
                if frac_a > 0 {
                    frac_a = 0;
                }
            }
            //sign is always zero
            let mut u_z = P32E2::pack_to_ui(regime, exp_a as u32, frac_a);
            //n+1 frac bit is 1. Need to check if another bit is 1 too if not round to even
            if bit_n_plus_one {
                let bits_more = (0x7FFF_FFFF & frac64_z) != 0;
                u_z += (u_z & 1) | (bits_more as u32);
            }
            u_z
        };

        P32E2::from_bits(u_z.with_sign(sign_z))
    }
}

#[inline]
fn add_mags_p32(mut ui_a: u32, mut ui_b: u32) -> P32E2 {
    let sign = P32E2::sign_ui(ui_a);
    if sign {
        ui_a = ui_a.wrapping_neg();
        ui_b = ui_b.wrapping_neg();
    }

    if (ui_a as i32) < (ui_b as i32) {
        ui_a ^= ui_b;
        ui_b ^= ui_a;
        ui_a ^= ui_b;
    }

    let (mut k_a, mut exp_a, frac_a) = P32E2::separate_bits(ui_a);

    let mut frac64_a = (frac_a as u64) << 32;

    let (k_b, exp_b, frac_b) = P32E2::separate_bits(ui_b);

    let mut shift_right = (k_a as i16) - (k_b as i16);
    let mut frac64_b = (frac_b as u64) << 32;

    //This is 4kZ + expZ; (where kZ=k_a-kB and expZ=exp_a-expB)
    shift_right = (shift_right << 2) + (exp_a as i16) - (exp_b as i16);

    //Manage CLANG (LLVM) compiler when shifting right more than number of bits
    if shift_right > 63 {
        frac64_b = 0;
    } else {
        frac64_b >>= shift_right;
    }

    frac64_a += frac64_b;

    let rcarry = (0x8000_0000_0000_0000 & frac64_a) != 0; //first left bit
    if rcarry {
        exp_a += 1;
        if exp_a > 3 {
            k_a += 1;
            exp_a &= 0x3;
        }
        frac64_a >>= 1;
    }
    let (regime, reg_sa, reg_a) = P32E2::calculate_regime(k_a);

    let u_z = if reg_a > 30 {
        //max or min pos. exp and frac does not matter.
        if reg_sa {
            0x7FFF_FFFF
        } else {
            0x1
        }
    } else {
        //remove hidden bits
        frac64_a = (frac64_a & 0x3FFF_FFFF_FFFF_FFFF) >> (reg_a + 2); // 2 bits exp

        let mut frac_a = (frac64_a >> 32) as u32;

        let mut bit_n_plus_one = false;
        if reg_a <= 28 {
            bit_n_plus_one = (0x8000_0000 & frac64_a) != 0;
            exp_a <<= 28 - reg_a;
        } else {
            if reg_a == 30 {
                bit_n_plus_one = (exp_a & 0x2) != 0;
                exp_a = 0;
            } else if reg_a == 29 {
                bit_n_plus_one = (exp_a & 0x1) != 0;
                exp_a >>= 1;
            }
            if frac_a > 0 {
                frac_a = 0;
            }
        }

        let mut u_z = P32E2::pack_to_ui(regime, exp_a as u32, frac_a);
        //n+1 frac bit is 1. Need to check if another bit is 1 too if not round to even
        if bit_n_plus_one {
            let bits_more = (0x7FFF_FFFF & frac64_a) != 0;
            u_z += (u_z & 1) | (bits_more as u32);
        }
        u_z
    };

    P32E2::from_bits(u_z.with_sign(sign))
}

#[inline]
fn sub_mags_p32(mut ui_a: u32, mut ui_b: u32) -> P32E2 {
    let mut sign = P32E2::sign_ui(ui_a);
    if sign {
        ui_a = ui_a.wrapping_neg();
    } else {
        ui_b = ui_b.wrapping_neg();
    }

    if ui_a == ui_b {
        //essential, if not need special handling
        return P32E2::from_bits(0);
    }
    if (ui_a as i32) < (ui_b as i32) {
        ui_a ^= ui_b;
        ui_b ^= ui_a;
        ui_a ^= ui_b;
        sign = !sign; //A becomes B
    }

    let (mut k_a, mut exp_a, frac_a) = P32E2::separate_bits(ui_a);

    let mut frac64_a = (frac_a as u64) << 32;

    let (k_b, exp_b, frac_b) = P32E2::separate_bits(ui_b);

    let mut shift_right = (k_a as i16) - (k_b as i16);
    let mut frac64_b = (frac_b as u64) << 32;
    //This is 4kZ + expZ; (where kZ=k_a-kB and expZ=exp_a-expB)
    shift_right = (shift_right << 2) + (exp_a as i16) - (exp_b as i16);

    if shift_right > 63 {
        return P32E2::from_bits(if sign { ui_a.wrapping_neg() } else { ui_a });
    } else {
        frac64_b >>= shift_right;
    }

    frac64_a -= frac64_b;

    while (frac64_a >> 59) == 0 {
        k_a -= 1;
        frac64_a <<= 4;
    }
    let mut ecarry = (0x4000_0000_0000_0000 & frac64_a) != 0; //(0x4000_0000_0000_0000 & frac64_a)>>62;
    while !ecarry {
        if exp_a == 0 {
            k_a -= 1;
            exp_a = 3;
        } else {
            exp_a -= 1;
        }
        frac64_a <<= 1;
        ecarry = (0x4000_0000_0000_0000 & frac64_a) != 0;
    }

    let (regime, reg_sa, reg_a) = P32E2::calculate_regime(k_a);

    let u_z = if reg_a > 30 {
        //max or min pos. exp and frac does not matter.
        if reg_sa {
            0x7FFF_FFFF
        } else {
            0x1
        }
    } else {
        //remove hidden bits
        frac64_a = (frac64_a & 0x3FFF_FFFF_FFFF_FFFF) >> (reg_a + 2); // 2 bits exp

        let mut frac_a = (frac64_a >> 32) as u32;

        let mut bit_n_plus_one = false;
        if reg_a <= 28 {
            bit_n_plus_one = (0x8000_0000 & frac64_a) != 0;
            exp_a <<= 28 - reg_a;
        } else {
            if reg_a == 30 {
                bit_n_plus_one = (exp_a & 0x2) != 0;
                exp_a = 0;
            } else if reg_a == 29 {
                bit_n_plus_one = (exp_a & 0x1) != 0;
                exp_a >>= 1;
            }
            if frac_a > 0 {
                frac_a = 0;
            }
        }

        let mut u_z = P32E2::pack_to_ui(regime, exp_a as u32, frac_a);
        //n+1 frac bit is 1. Need to check if another bit is 1 too if not round to even
        if bit_n_plus_one {
            let bits_more = (0x7FFF_FFFF & frac64_a) != 0;
            u_z += (u_z & 1) | (bits_more as u32);
        }
        u_z
    };

    P32E2::from_bits(u_z.with_sign(sign))
}
