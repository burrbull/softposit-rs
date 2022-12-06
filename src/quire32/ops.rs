use super::Q32E2;
use crate::PxE2;
use crate::P32E2;
use core::ops;

crate::macros::quire_add_sub!(P32E2, Q32E2);
crate::macros::quire_add_sub_array!(P32E2, Q32E2, 1, 2, 3, 4);

crate::macros::quire_add_sub_x!(PxE2<{ N }>, Q32E2);
crate::macros::quire_add_sub_array_x!(PxE2<{ N }>, Q32E2, 1, 2, 3, 4);

pub(super) fn fdp(q: &mut Q32E2, mut ui_a: u32, mut ui_b: u32, plus: bool) {
    let u_z1 = q.to_bits();

    if q.is_nar() || ui_a == 0x_8000_0000 || ui_b == 0x_8000_0000 {
        *q = Q32E2::NAR;
        return;
    } else if ui_a == 0 || ui_b == 0 {
        return;
    }

    //max pos (sign plus and minus)
    let sign_a = P32E2::sign_ui(ui_a);
    let sign_b = P32E2::sign_ui(ui_b);
    let sign_z2 = sign_a ^ sign_b;

    if sign_a {
        ui_a = ui_a.wrapping_neg();
    }
    if sign_b {
        ui_b = ui_b.wrapping_neg();
    }

    let (mut k_a, tmp) = P32E2::separate_bits_tmp(ui_a);
    let mut exp_a = (tmp >> 29) as i32; //to get 2 bits
    let frac_a = (tmp << 2) | 0x_8000_0000;

    let (k_b, tmp) = P32E2::separate_bits_tmp(ui_b);
    k_a += k_b;
    exp_a += (tmp >> 29) as i32;
    let mut frac64_z = (frac_a as u64) * (((tmp << 2) | 0x_8000_0000) as u64);

    if exp_a > 3 {
        k_a += 1;
        exp_a &= 0x3; // -=4
    }
    //Will align frac64_z such that hidden bit is the first bit on the left.
    let rcarry = (frac64_z >> 63) != 0; //1st bit of frac64_z
    if rcarry {
        exp_a += 1;
        if exp_a > 3 {
            k_a += 1;
            exp_a &= 0x3;
        }
    //frac64_z>>=1;
    } else {
        frac64_z <<= 1;
    }

    //default dot is between bit 271 and 272, extreme left bit is bit 0. Last right bit is bit 512.
    //Minpos is 120 position to the right of binary point (dot)
    //Scale = 2^es * k + e  => 2k + e
    let first_pos = 271 - ((k_a as i32) << 2) - exp_a;

    //Moving in chunk of 64. If it is in first chunk, a part might be in the chunk right to it. Simply have to handle that.
    let mut u_z2: [u64; 8] = [0; 8];
    for i in 0usize..8 {
        if first_pos < ((i + 1) * 64) as i32 {
            //Need to check how much of the fraction is in the next 64 bits
            let shift_right = (first_pos - ((i * 64) as i32)) as i16;
            u_z2[i] = frac64_z >> shift_right;
            if (i != 7) && (shift_right != 0) {
                u_z2[i + 1] = frac64_z << (64 - shift_right);
            }
            break;
        }
    }

    if !(sign_z2 ^ plus) {
        let mut j = u_z2.iter_mut().rev();
        while let Some(u) = j.next() {
            if *u > 0 {
                *u = u.wrapping_neg();
                for w in j {
                    *w = !*w;
                }
                break;
            }
        }
    }

    //Addition
    let mut u_z: [u64; 8] = [0; 8];
    let mut rcarry_z = false;
    for (i, (u, (u1, u2))) in (0..8)
        .zip(u_z.iter_mut().zip(u_z1.iter().zip(u_z2.iter())))
        .rev()
    {
        let b1 = (*u1 & 0x1) != 0;
        let b2 = (*u2 & 0x1) != 0;
        if i == 7 {
            let rcarryb = b1 & b2;
            *u = (*u1 >> 1) + (*u2 >> 1) + (rcarryb as u64);
            rcarry_z = *u >> 63 != 0;
            *u = (*u << 1) | ((b1 ^ b2) as u64);
        } else {
            let rcarryb3 = (b1 as i8) + (b2 as i8) + (rcarry_z as i8);
            *u = (*u1 >> 1) + (*u2 >> 1) + ((rcarryb3 >> 1) as u64);
            rcarry_z = *u >> 63 != 0;
            *u = (*u << 1) | ((rcarryb3 & 0x1) as u64);
        }
    }

    //Exception handling
    let q_z = Q32E2::from_bits(u_z);
    *q = if q_z.is_nar() { Q32E2::ZERO } else { q_z }
}

pub(super) fn fdp_one(q: &mut Q32E2, mut ui_a: u32, plus: bool) {
    let u_z1 = q.to_bits();

    if q.is_nar() || ui_a == 0x_8000_0000 {
        *q = Q32E2::NAR;
        return;
    } else if ui_a == 0 {
        return;
    }

    //max pos (sign plus and minus)
    let sign_a = P32E2::sign_ui(ui_a);

    if sign_a {
        ui_a = ui_a.wrapping_neg();
    }

    let (mut k_a, tmp) = P32E2::separate_bits_tmp(ui_a);
    let mut exp_a = (tmp >> 29) as i32; //to get 2 bits
    let frac_a = (tmp << 2) | 0x_8000_0000;

    let mut frac64_z = (frac_a as u64) << 31;

    if exp_a > 3 {
        k_a += 1;
        exp_a &= 0x3; // -=4
    }
    //Will align frac64_z such that hidden bit is the first bit on the left.
    let rcarry = (frac64_z >> 63) != 0; //1st bit of frac64_z
    if rcarry {
        exp_a += 1;
        if exp_a > 3 {
            k_a += 1;
            exp_a &= 0x3;
        }
    //frac64_z>>=1;
    } else {
        frac64_z <<= 1;
    }

    //default dot is between bit 271 and 272, extreme left bit is bit 0. Last right bit is bit 512.
    //Minpos is 120 position to the right of binary point (dot)
    //Scale = 2^es * k + e  => 2k + e
    let first_pos = 271 - ((k_a as i32) << 2) - exp_a;

    //Moving in chunk of 64. If it is in first chunk, a part might be in the chunk right to it. Simply have to handle that.
    let mut u_z2: [u64; 8] = [0; 8];
    for i in 0usize..8 {
        if first_pos < ((i + 1) * 64) as i32 {
            //Need to check how much of the fraction is in the next 64 bits
            let shift_right = (first_pos - ((i * 64) as i32)) as i16;
            u_z2[i] = frac64_z >> shift_right;
            if (i != 7) && (shift_right != 0) {
                u_z2[i + 1] = frac64_z << (64 - shift_right);
            }
            break;
        }
    }

    if !(sign_a ^ plus) {
        let mut j = u_z2.iter_mut().rev();
        while let Some(u) = j.next() {
            if *u > 0 {
                *u = u.wrapping_neg();
                for w in j {
                    *w = !*w;
                }
                break;
            }
        }
    }

    //Addition
    let mut u_z: [u64; 8] = [0; 8];
    let mut rcarry_z = false;
    for (i, (u, (u1, u2))) in (0..8)
        .zip(u_z.iter_mut().zip(u_z1.iter().zip(u_z2.iter())))
        .rev()
    {
        let b1 = (*u1 & 0x1) != 0;
        let b2 = (*u2 & 0x1) != 0;
        if i == 7 {
            let rcarryb = b1 & b2;
            *u = (*u1 >> 1) + (*u2 >> 1) + (rcarryb as u64);
            rcarry_z = *u >> 63 != 0;
            *u = (*u << 1) | ((b1 ^ b2) as u64);
        } else {
            let rcarryb3 = (b1 as i8) + (b2 as i8) + (rcarry_z as i8);
            *u = (*u1 >> 1) + (*u2 >> 1) + ((rcarryb3 >> 1) as u64);
            rcarry_z = *u >> 63 != 0;
            *u = (*u << 1) | ((rcarryb3 & 0x1) as u64);
        }
    }

    //Exception handling
    let q_z = Q32E2::from_bits(u_z);
    *q = if q_z.is_nar() { Q32E2::ZERO } else { q_z }
}

#[test]
fn test_quire_mul_add() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS32 {
        let p_a: P32E2 = rng.gen();
        let p_b: P32E2 = rng.gen();
        let p_c: P32E2 = rng.gen();
        let f_a = f64::from(p_a);
        let f_b = f64::from(p_b);
        let f_c = f64::from(p_c);
        let mut q = Q32E2::init();
        q += (p_a, p_b);
        q += p_c;
        let p = q.to_posit();
        let f = f_a.mul_add(f_b, f_c);
        #[cfg(not(feature = "std"))]
        assert_eq!(p, P32E2::from(f));
        #[cfg(feature = "std")]
        assert_eq!(
            p,
            P32E2::from(f),
            "\n  input: ({p_a:?}, {p_b:?}, {p_c:?})\n   or: {f_a}, {f_b}, {f_c}\n  answer: {}, expected {f}",
            p.to_f64()
        );
    }
}

#[test]
fn test_quire_mul_sub() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..crate::NTESTS32 {
        let p_a: P32E2 = rng.gen();
        let p_b: P32E2 = rng.gen();
        let p_c: P32E2 = rng.gen();
        let f_a = f64::from(p_a);
        let f_b = f64::from(p_b);
        let f_c = f64::from(p_c);
        let mut q = Q32E2::init();
        q -= (p_a, p_b);
        q += p_c;
        let p = q.to_posit();
        let f = (-f_a).mul_add(f_b, f_c);
        assert_eq!(p, P32E2::from(f));
    }
}
