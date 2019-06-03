use crate::{P32E2, Q32E2, MathConsts, Polynom};

const PI_A:  P32E2 = P32E2::new(0x_4c90_fda0); // 3.141592502593994e0
const PI_B:  P32E2 = P32E2::new(0x_0151_10b0); // 1.509956746303942e-7
const PI_CL: P32E2 = P32E2::new(0x_000a_308d); // 1.244672767630739e-13

const PI_C:  P32E2 = P32E2::new(0x_000a_3080); // 1.2445600106048005e-13
const PI_D:  P32E2 = P32E2::new(0x_0000_f4c0); // 1.1438332919722072e-17
const PI_E:  P32E2 = P32E2::new(0x_0000_2180); // 4.0234064994579266e-21
const PI_F:  P32E2 = P32E2::new(0x_0000_0930); // 2.1093105620102206e-23
const PI_HL: P32E2 = P32E2::new(0x_0000_01d0); // 3.2311742677852644e-26

const TRIGRANGEMAX2: P32E2 = P32E2::new(1944584192);// 1000.;.
const TRIGRANGEMAX: P32E2 = P32E2::new(2101243392); // 390_000.

#[cfg(test)]
const NTESTS: usize = 100_000;

pub fn sin(mut d: P32E2) -> P32E2 {
    let q: i32;

    if d.abs() < TRIGRANGEMAX {
        let qf = (d * P32E2::FRAC_1_PI).round();
        q = qf.into();
        let mut quire = Q32E2::init();
        quire += (d, P32E2::ONE);
        quire -= (qf, PI_A);
        quire -= (qf, PI_B);
        if d.abs() < TRIGRANGEMAX2 {
            quire -= (qf, PI_CL);
        } else {
            quire -= (qf, PI_C);
            quire -= (qf, PI_D);
            quire -= (qf, PI_E);
            quire -= (qf, PI_F);
            quire -= (qf, PI_HL);
        }
        d = quire.into();
    } else {
        unimplemented!()
    }

    let s = d * d;

    if (q & 1) != 0 {
        d = -d;
    }

    let u = s.poly3(&[
        P32E2::new(45025927), // 2.608_315_980_978_659_354_150_3_e-6
        P32E2::new(-127890902), // -0.000_198_106_907_191_686_332_225_8
        P32E2::new(340016056), // 0.008_333_078_585_565_090_179_443_36
        P32E2::new(-715827808), // -0.166_666_597_127_914_428_710_938
    ]);

    s.mul_add(u * d, d)
}

#[cfg(test)]
fn ulp(x: P32E2, y: P32E2) -> i32 {
    let xi = x.to_bits() as i32;
    let yi = y.to_bits() as i32;
    (xi.wrapping_sub(yi)).abs()
}

#[test]
fn test_sin() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
//    let mut ncorrect = 0;
//    let mut max_ulp = 0;
    for _i in 0..NTESTS {
        let n_a = rng.gen_range(-2101243392+1, 2101243392-1); // (-3.14, 3.14)
        let p_a = P32E2::new(n_a);
        let f_a = f64::from(p_a);
        let answer = sin(p_a);
        let correct = P32E2::from(f_a.sin());
        let u = ulp(answer, correct);
/*
        if u > max_ulp {
            max_ulp = u;
        }
*/
        assert!(
            u <= 4,
            "x = {}, answer = {}, correct = {}, ulp = {}",
            f_a,
            answer,
            correct,
            u,
        );
        /*if (u <= 3) {
            ncorrect += 1;
        }
        if i == NTESTS - 1 {
            assert!(false, "Correct = {} %, max_ulp = {}", (ncorrect*100) as f32 / (NTESTS as f32), max_ulp);
        }*/
    }
}
