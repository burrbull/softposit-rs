use super::Q16E1;
use crate::P16E1;
use core::ops;

crate::quire_add_sub!(P16E1, Q16E1);
crate::quire_add_sub_array!(P16E1, Q16E1, 1, 2, 3, 4);

pub(super) fn fdp_add(q: &mut Q16E1, mut ui_a: u16, mut ui_b: u16) {
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
    let first_pos = 71 - ((k_a as i16) << 1) - (exp_a as i16);

    //No worries about hidden bit moving before position 4 because fraction is right aligned so
    //there are 16 spare bits
    let mut u_z2: [u64; 2] = [0, 0];
    if first_pos > 63 {
        //This means entire fraction is in right 64 bits
        u_z2[0] = 0;
        let shift_right = first_pos - 99; //99 = 63+ 4+ 32
        if shift_right < 0 {
            //shiftLeft
            u_z2[1] = (frac32_z as u64) << -shift_right;
        } else {
            u_z2[1] = (frac32_z as u64) >> shift_right;
        }
    } else {
        //frac32_z can be in both left64 and right64
        let shift_right = first_pos - 35; // -35= -3-32
        if shift_right < 0 {
            u_z2[0] = (frac32_z as u64) << -shift_right;
        } else {
            u_z2[0] = (frac32_z as u64) >> shift_right;
            u_z2[1] = (frac32_z as u64)
                .checked_shl((64 - shift_right) as u32)
                .unwrap_or(0);
        }
    }

    if sign_z2 {
        if u_z2[1] > 0 {
            u_z2[1] = u_z2[1].wrapping_neg();
            u_z2[0] = !u_z2[0];
        } else {
            u_z2[0] = u_z2[0].wrapping_neg();
        }
    }

    //Addition
    let b1 = u_z1[0] & 0x1 != 0;
    let b2 = u_z2[0] & 0x1 != 0;
    let rcarryb = b1 & b2;
    let mut u_z: [u64; 2] = [0, (u_z1[1] >> 1) + (u_z2[1] >> 1) + (rcarryb as u64)];

    let rcarry_z = (u_z[1] & 0x_8000_0000_0000_0000) != 0;

    u_z[1] = u_z[1] << 1 | ((b1 ^ b2) as u64);

    let b1 = u_z1[0] & 0x1 != 0;
    let b2 = u_z2[0] & 0x1 != 0;
    //rcarryb = b1 & b2 ;
    let rcarryb3 = (b1 as i8) + (b2 as i8) + (rcarry_z as i8);

    u_z[0] = (u_z1[0] >> 1) + (u_z2[0] >> 1) + (((rcarryb3 >> 1) & 0x1) as u64);
    //rcarrySignZ = u_z[0]>>63;

    u_z[0] = u_z[0] << 1 | ((rcarryb3 & 0x1) as u64);

    //Exception handling for NaR
    let q_z = Q16E1::from_bits(u_z);
    *q = if q_z.is_nar() { Q16E1::ZERO } else { q_z }
}

pub(super) fn fdp_sub(q: &mut Q16E1, mut ui_a: u16, mut ui_b: u16) {
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
    let first_pos = 71 - ((k_a as i16) << 1) - (exp_a as i16);

    //No worries about hidden bit moving before position 4 because fraction is right aligned so
    //there are 16 spare bits
    let mut u_z2: [u64; 2] = [0, 0];
    if first_pos > 63 {
        //This means entire fraction is in right 64 bits
        u_z2[0] = 0;
        let shift_right = first_pos - 99; //99 = 63+ 4+ 32
        if shift_right < 0 {
            //shiftLeft
            u_z2[1] = (frac32_z as u64) << -shift_right;
        } else {
            u_z2[1] = (frac32_z as u64) >> shift_right;
        }
    } else {
        //frac32_z can be in both left64 and right64
        let shift_right = first_pos - 35; // -35= -3-32
        if shift_right < 0 {
            u_z2[0] = (frac32_z as u64) << -shift_right;
        } else {
            u_z2[0] = (frac32_z as u64) >> shift_right;
            u_z2[1] = (frac32_z as u64)
                .checked_shl((64 - shift_right) as u32)
                .unwrap_or(0);
        }
    }

    //This is the only difference from ADD (sign_z2) and (!sign_z2)
    if !sign_z2 {
        if u_z2[1] > 0 {
            u_z2[1] = u_z2[1].wrapping_neg();
            u_z2[0] = !u_z2[0];
        } else {
            u_z2[0] = u_z2[0].wrapping_neg();
        }
    }

    //Subtraction
    let b1 = u_z1[1] & 0x1 != 0;
    let b2 = u_z2[1] & 0x1 != 0;
    let rcarryb = b1 & b2;
    let mut u_z: [u64; 2] = [0, (u_z1[1] >> 1) + (u_z2[1] >> 1) + (rcarryb as u64)];

    let rcarry_z = (u_z[1] & 0x_8000_0000_0000_0000) != 0;

    u_z[1] = u_z[1] << 1 | ((b1 ^ b2) as u64);

    let b1 = u_z1[0] & 0x1 != 0;
    let b2 = u_z2[0] & 0x1 != 0;
    //let rcarryb = b1 & b2;
    let rcarryb3 = (b1 as i8) + (b2 as i8) + (rcarry_z as i8);

    u_z[0] = (u_z1[0] >> 1) + (u_z2[0] >> 1) + (((rcarryb3 >> 1) & 0x1) as u64);
    //rcarrySignZ = u_z[0]>>63;

    u_z[0] = u_z[0] << 1 | ((rcarryb3 & 0x1) as u64);

    //Exception handling
    let q_z = Q16E1::from_bits(u_z);
    *q = if q_z.is_nar() { Q16E1::ZERO } else { q_z }
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
        q += (p_c, P16E1::ONE);
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
        q += (p_c, P16E1::ONE);
        let p = q.to_posit();
        let f = (-f_a).mul_add(f_b, f_c);
        assert!(
            ulp(p, P16E1::from(f)) <= 1 /*, "p_a = {}\tp_b = {}\tp_c = {}\tp = {}\tf = {}", p_a, p_b, p_c, p, f*/
        );
    }
}
