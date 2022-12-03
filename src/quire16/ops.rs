use super::Q16E1;
use crate::P16E1;
use core::ops;

crate::macros::quire_add_sub!(P16E1, Q16E1);
crate::macros::quire_add_sub_array!(P16E1, Q16E1, 1, 2, 3, 4);

pub(super) fn fdp(q: &mut Q16E1, mut ui_a: u16, mut ui_b: u16, plus: bool) {
    let u_z1 = q.to_bits();

    if q.is_nar() || ui_a == 0x_8000 || ui_b == 0x_8000 {
        *q = Q16E1::NAR;
        return;
    } else if ui_a == 0 || ui_b == 0 {
        return;
    }

    //max pos (sign plus and minus)
    let sign_a = P16E1::sign_ui(ui_a);
    let sign_b = P16E1::sign_ui(ui_b);
    let sign_z2 = sign_a ^ sign_b;

    if sign_a {
        ui_a = ui_a.wrapping_neg();
    }
    if sign_b {
        ui_b = ui_b.wrapping_neg();
    }

    let (mut k_a, mut exp_a, frac_a) = P16E1::separate_bits(ui_a);

    let (k_b, exp_b, frac_b) = P16E1::separate_bits(ui_b);
    k_a += k_b;
    exp_a += exp_b;
    let mut frac32_z = (frac_a as u32) * (frac_b as u32);

    if exp_a > 1 {
        k_a += 1;
        exp_a ^= 0x2;
    }

    let rcarry = (frac32_z >> 29) != 0; //3rd bit (position 2) of frac32_z, hidden bit is 4th bit (position 3)
    if rcarry {
        if exp_a != 0 {
            k_a += 1;
        }
        exp_a ^= 1;
        frac32_z >>= 1;
    }

    //default dot is between bit 71 and 72, extreme left bit is bit 0. Last right bit is bit 127.
    //Scale = 2^es * k + e  => 2k + e
    let shift_right = -(28 + ((k_a as i16) << 1) + (exp_a as i16));
    let mut u_z2 = if shift_right < 0 {
        (frac32_z as u128) << -shift_right
    } else {
        (frac32_z as u128) >> shift_right
    };

    if !(sign_z2 ^ plus) {
        u_z2 = u_z2.wrapping_neg();
    }
    //Addition
    let u_z = u_z2.wrapping_add(u_z1);

    //Exception handling for NaR
    let q_z = Q16E1::from_bits(u_z);
    *q = if q_z.is_nar() { Q16E1::ZERO } else { q_z }
}

pub(super) fn fdp_one(q: &mut Q16E1, mut ui_a: u16, plus: bool) {
    let u_z1 = q.to_bits();

    if q.is_nar() || ui_a == 0x_8000 {
        *q = Q16E1::NAR;
        return;
    } else if ui_a == 0 {
        return;
    }

    //max pos (sign plus and minus)
    let sign_a = P16E1::sign_ui(ui_a);

    if sign_a {
        ui_a = ui_a.wrapping_neg();
    }

    let (mut k_a, mut exp_a, frac_a) = P16E1::separate_bits(ui_a);
    let mut frac32_z = (frac_a as u32) << 14;

    if exp_a > 1 {
        k_a += 1;
        exp_a ^= 0x2;
    }

    let rcarry = (frac32_z >> 29) != 0; //3rd bit (position 2) of frac32_z, hidden bit is 4th bit (position 3)
    if rcarry {
        if exp_a != 0 {
            k_a += 1;
        }
        exp_a ^= 1;
        frac32_z >>= 1;
    }

    //default dot is between bit 71 and 72, extreme left bit is bit 0. Last right bit is bit 127.
    //Scale = 2^es * k + e  => 2k + e
    let shift_right = -(28 + ((k_a as i16) << 1) + (exp_a as i16));
    let mut u_z2 = if shift_right < 0 {
        (frac32_z as u128) << -shift_right
    } else {
        (frac32_z as u128) >> shift_right
    };

    if !(sign_a ^ plus) {
        u_z2 = u_z2.wrapping_neg();
    }
    //Addition
    let u_z = u_z2.wrapping_add(u_z1);

    //Exception handling for NaR
    let q_z = Q16E1::from_bits(u_z);
    *q = if q_z.is_nar() { Q16E1::ZERO } else { q_z }
}

#[cfg(test)]
fn ulp(x: P16E1, y: P16E1) -> i16 {
    let xi = x.to_bits() as i16;
    let yi = y.to_bits() as i16;
    (xi - yi).abs()
}

#[test]
fn test_quire_mul_add() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS16 {
        let p_a: P16E1 = rng.gen();
        let p_b: P16E1 = rng.gen();
        let p_c: P16E1 = rng.gen();
        let f_a = f64::from(p_a);
        let f_b = f64::from(p_b);
        let f_c = f64::from(p_c);
        let mut q = Q16E1::init();
        q += (p_a, p_b);
        q += p_c;
        let p = q.to_posit();
        let f = f_a.mul_add(f_b, f_c);
        assert!(ulp(p, P16E1::from(f)) <= 1);
    }
}

#[test]
fn test_quire_mul_sub() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS16 {
        let p_a: P16E1 = rng.gen();
        let p_b: P16E1 = rng.gen();
        let p_c: P16E1 = rng.gen();
        let f_a = f64::from(p_a);
        let f_b = f64::from(p_b);
        let f_c = f64::from(p_c);
        let mut q = Q16E1::init();
        q -= (p_a, p_b);
        q += p_c;
        let p = q.to_posit();
        let f = (-f_a).mul_add(f_b, f_c);
        assert!(
            ulp(p, P16E1::from(f)) <= 1 /*, "p_a = {}\tp_b = {}\tp_c = {}\tp = {}\tf = {}", p_a, p_b, p_c, p, f*/
        );
    }
}
