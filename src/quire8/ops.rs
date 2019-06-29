use super::Q8E0;
use crate::P8E0;
use core::ops;

crate::quire_add_sub!(P8E0, Q8E0);
crate::quire_add_sub_array!(P8E0, Q8E0, 1, 2, 3, 4);

pub(super) fn fdp_add(q: &mut Q8E0, mut ui_a: u8, mut ui_b: u8) {
    let uq_z1 = q.to_bits();

    if q.is_nar() || ui_a == 0x80 || ui_b == 0x80 {
        *q = Q8E0::NAR;
        return;
    } else if ui_a == 0 || ui_b == 0 {
        return;
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

    let mut frac32_z = ((frac_a as u32) * (frac_b as u32)) << 16;

    let rcarry = (frac32_z & 0x8000_0000) != 0; //1st bit (position 2) of frac32_z, hidden bit is 4th bit (position 3)
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
    let uq_z = uq_z2.wrapping_add(uq_z1);

    //Exception handling
    let q_z = Q8E0::from_bits(uq_z);
    *q = if q_z.is_nar() { Q8E0::ZERO } else { q_z }
}

//q - (p_a*p_b)

pub(super) fn fdp_sub(q: &mut Q8E0, mut ui_a: u8, mut ui_b: u8) {
    let uq_z1 = q.to_bits();

    if q.is_nar() || ui_a == 0x80 || ui_b == 0x80 {
        *q = Q8E0::NAR;
        return;
    } else if ui_a == 0 || ui_b == 0 {
        return;
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

    let mut frac32_z = ((frac_a as u32) * (frac_b as u32)) << 16;

    let rcarry = (frac32_z & 0x8000_0000) != 0; //1st bit (position 2) of frac32_z, hidden bit is 4th bit (position 3)
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
    let uq_z = uq_z2.wrapping_add(uq_z1);

    //Exception handling
    let q_z = Q8E0::from_bits(uq_z);
    *q = if q_z.is_nar() { Q8E0::ZERO } else { q_z }
}

#[test]
fn test_quire_mul_add() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS8 {
        let p_a: P8E0 = rng.gen();
        let p_b: P8E0 = rng.gen();
        let p_c: P8E0 = rng.gen();
        let f_a = f64::from(p_a);
        let f_b = f64::from(p_b);
        let f_c = f64::from(p_c);
        let mut q = Q8E0::init();
        q += (p_a, p_b);
        q += (p_c, P8E0::ONE);
        let p = q.to_posit();
        let f = f_a.mul_add(f_b, f_c);
        assert_eq!(p, P8E0::from(f));
    }
}

#[test]
fn test_quire_mul_sub() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS8 {
        let p_a: P8E0 = rng.gen();
        let p_b: P8E0 = rng.gen();
        let p_c: P8E0 = rng.gen();
        let f_a = f64::from(p_a);
        let f_b = f64::from(p_b);
        let f_c = f64::from(p_c);
        let mut q = Q8E0::init();
        q -= (p_a, p_b);
        q += (p_c, P8E0::ONE);
        let p = q.to_posit();
        let f = (-f_a).mul_add(f_b, f_c);
        assert_eq!(p, P8E0::from(f));
    }
}
