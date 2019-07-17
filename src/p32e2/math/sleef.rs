use crate::{MathConsts, Polynom, P32E2, Q32E2};

use super::{
    HALF, // 0.5
    TWO,  // 2.
};

const THREE: P32E2 = P32E2::new(0x_4c00_0000);

const NAR: P32E2 = P32E2::NAR;
const ZERO: P32E2 = P32E2::ZERO;
const ONE: P32E2 = P32E2::ONE;

const PI_A: P32E2 = P32E2::PI; // 3.141_592_651_605_606
const PI_B: P32E2 = P32E2::new(0x_0071_0b46); // 1.984_187_036_896_401_e-9
const PI_C: P32E2 = P32E2::new(0x_0001_c698); // 1.224_606_353_822_377_3_e-16

const TRIGRANGEMAX: P32E2 = P32E2::new(0x_7d40_0000); // 393_216.

const L10U: P32E2 = P32E2::LOG10_2; // 0.301_029_995_083_808_9
const L10L: P32E2 = P32E2::new(0x_0053_ef3f); // 5.801_719_105_136_272_e-10

const L2U: P32E2 = P32E2::LN_2; // 0.693_147_178_739_309_3
const L2L: P32E2 = P32E2::new(0x_006f_473d); // 1.820_635_198_157_560_8_e-9

const R_LN2: P32E2 = P32E2::new(0x_438a_a3b3); // 1.442_695_040_888_963_407_359_924_681_001_892_137_426_645_954_152_985_934_135_449_406_931

#[cfg(test)]
const NTESTS: usize = 100_000;

pub fn mulsign(x: P32E2, y: P32E2) -> P32E2 {
    if (x.to_bits() ^ y.to_bits()) & P32E2::SIGN_MASK == 0 {
        x
    } else {
        -x
    }
}

mod kernel {
    use super::*;
    // TODO: |n| > 111
    pub fn pow2i(mut n: i32) -> P32E2 {
        let sign = n.is_negative();
        if sign {
            n = -n;
        }
        let k = n >> 2;
        let ex: u32 = ((n & 0x3) as u32) << (27 - k);
        let ui = (0x7FFF_FFFF ^ (0x3FFF_FFFF >> k)) | ex;

        if sign {
            P32E2::from_bits((ui << 1).wrapping_neg() >> 1)
        } else {
            P32E2::from_bits(ui)
        }
    }

    pub fn ilogb(d: P32E2) -> i32 {
        let ui = d.abs().to_bits();
        let (k_a, tmp) = P32E2::separate_bits_tmp(ui);
        ((k_a as i32) << 2) + ((tmp >> 29) as i32)
    }

    pub fn ldexp2(d: P32E2, e: i32) -> P32E2 {
        // faster than ldexpkf, short reach
        d * pow2i(e >> 1) * pow2i(e - (e >> 1))
    }

    #[inline]
    pub fn sin(s: P32E2) -> P32E2 {
        s.poly5(&[
            P32E2::new(-0x_00d3_e191), // -2.4159030332e-8,
            P32E2::new(0x_02b8_d0f3),  // 2.7539761902e-6,
            P32E2::new(-0x_07a0_193a), // -1.9841124231e-4,
            P32E2::new(0x_1444_443e),  // 8.3333326038e-3,
            P32E2::new(-0x_2aaa_aaaa), // -1.6666666605e-1,
            ONE,
        ])
    }

    #[inline]
    pub fn cos(s: P32E2) -> P32E2 {
        s.poly6(&[
            P32E2::new(0x_0071_2590),  // 1.9961422740e-9,
            P32E2::new(-0x_0189_e57f), // -2.7528581792e-7,
            P32E2::new(0x_04a0_181c),  // 2.4801145699e-5,
            P32E2::new(-0x_0cd8_2d77), // -1.3888885487e-3,
            P32E2::new(0x_1d55_5555),  // 4.1666666541e-2,
            -HALF,
            ONE,
        ])
    }

    #[inline]
    pub fn exp_m1(d: P32E2) -> P32E2 {
        let qf = (d * R_LN2).round();
        let q = i32::from(qf);

        let mut quire = Q32E2::init();
        quire += (d, ONE);
        quire -= (qf, [L2U, L2L]);
        let s = quire.to_posit();

        let mut u = s.poly5(&[
            P32E2::new(0x_079d_b0ca), // 1.9726304345e-4,
            P32E2::new(0x_0cda_fee4), // 1.3942635851e-3,
            P32E2::new(0x_1444_4e5b), // 8.3336340031e-3,
            P32E2::new(0x_1d55_5258), // 4.1666310281e-2,
            P32E2::new(0x_2aaa_aa9a), // 1.6666665114e-1,
            P32E2::new(0x_3800_0002), // 5.0000000745e-1,
        ]);
        u = s * s * u + s;

        if q != 0 {
            ldexp2(u + ONE, q) - ONE
        } else {
            u
        }
    }

    #[inline]
    // TODO: fix coeffs
    pub fn exp(d: P32E2) -> P32E2 {
        let qf = (d * R_LN2).round();
        let q = i32::from(qf);

        let mut quire = Q32E2::init();
        quire += (d, ONE);
        quire -= (qf, [L2U, L2L]);
        let s = quire.clone().to_posit();

        let u = s.poly5(&[
            P32E2::new(0x_079d_b0ca), // 1.9726304345e-4,
            P32E2::new(0x_0cda_fee4), // 1.3942635851e-3,
            P32E2::new(0x_1444_4e5b), // 8.3336340031e-3,
            P32E2::new(0x_1d55_5258), // 4.1666310281e-2,
            P32E2::new(0x_2aaa_aa9a), // 1.6666665114e-1,
            P32E2::new(0x_3800_0002), // 5.0000000745e-1,
        ]);

        quire += (s * s, u);
        quire += (ONE, ONE);

        if d < P32E2::new(-0x_6a80_0000)
        /*-104.*/
        {
            ZERO
        } else {
            ldexp2(quire.to_posit(), q) //ldexpkf
        }
    }

    #[inline]
    pub fn log(d: P32E2) -> P32E2 {
        let e = kernel::ilogb(d * (ONE / P32E2::new(0x_3c00_0000)/*0.75*/)); // ilogb2kf
        let m = kernel::ldexp2(d, -e); //ldexp3kf(d, -e);
        let x = (m - ONE) / (m + ONE);
        let x2 = x * x;

        let t = x2.poly3(&[
            P32E2::new(0x_2cd5_6da8), // 2.0052663237e-1
            P32E2::new(0x_2e2b_8340), // 2.2140580416e-1
            P32E2::new(0x_3124_b46a), // 2.8573055938e-1
            P32E2::new(0x_34cc_cc7e), // 3.9999985322e-1
        ]);

        let mut quire = Q32E2::ZERO;
        quire += (P32E2::new(0x_3aaa_aaa8), P32E2::new(0x_4000_0002)); // 2/3
        quire += (x2, t);
        let z = quire.to_posit();

        let ef = P32E2::from(e);
        let mut quire = Q32E2::init();
        quire += (ef, [L2U, L2L]);
        quire += (x, TWO);
        quire += (x2 * x, z);
        quire.into()
    }

    #[inline]
    pub fn atan2(mut y: P32E2, mut x: P32E2) -> P32E2 {
        let mut q = if x.is_sign_negative() {
            x = -x;
            -2
        } else {
            0
        };

        if y > x {
            let t = x;
            x = y;
            y = -t;
            q += 1;
        }

        let s = y / x;
        let t = s * s;

        let u = t.poly8(&[
            P32E2::new(-0x_0de4_f8b5), // -1.9015722646e-3,
            P32E2::new(0x_15cf_6226),  // 1.1347834719e-2,
            P32E2::new(-0x_1c14_c8ad), // -3.1884273980e-2,
            P32E2::new(0x_1f7e_b681),  // 5.8554471005e-2,
            P32E2::new(-0x_22ca_9b6e), // -8.4308079444e-2,
            P32E2::new(0x_2606_ded6),  // 1.0958466958e-1,
            P32E2::new(-0x_2921_200a), // -1.4264679886e-1,
            P32E2::new(0x_2ccc_8d0c),  // 1.9998480007e-1,
            P32E2::new(-0x_32aa_a9be), // -3.333328932749459636123021434852409e-1,
        ]);

        let t = u * t * s + s;
        P32E2::from(q) * P32E2::FRAC_PI_2 + t
    }
}

/// Power function
///
/// This function returns the value of ***x*** raised to the power of ***y***.
pub fn pow(x: P32E2, y: P32E2) -> P32E2 {
    if (y == ZERO) || (x == ONE) {
        ONE
    } else if x.is_nar() || y.is_nar() {
        P32E2::NAR
    } else if x == ZERO {
        if -y < ZERO {
            ZERO
        } else {
            P32E2::NAR
        }
    } else {
        let p1_23 = P32E2::from(1u32 << 23);
        let yisint = (y == y.round()) || (y.abs() >= p1_23);
        let yisodd = ((1 & (i32::from(y))) != 0) && yisint && (y.abs() < p1_23);

        kernel::exp(kernel::log(x.abs()) * y)
            * if x >= ZERO {
                ONE
            } else if !yisint {
                P32E2::NAR
            } else if yisodd {
                -ONE
            } else {
                ONE
            }
    }
}

#[test]
fn test_pow() {
    test_pp_p(
        pow,
        f64::powf,
        /*P32E2::MIN.0, P32E2::MAX.0,*/ 0x_3800_0000,
        0x_5200_0000,
        5,
    );
    assert_eq!(pow(ZERO, -ONE), NAR);
    assert_eq!(pow(ZERO, ZERO), ONE);
    assert_eq!(pow(ZERO, ONE), ZERO);
}

/// Arc tangent function of two variables
///
/// These functions evaluates the arc tangent function of (***y*** / ***x***).
pub fn atan2(y: P32E2, x: P32E2) -> P32E2 {
    if x.is_nar() || y.is_nar() {
        return P32E2::NAR;
    }
    let mut r = kernel::atan2(y.abs(), x);

    r = if x == ZERO {
        P32E2::FRAC_PI_2
    } else if y == ZERO {
        (if x.signum() == -ONE { P32E2::PI } else { ZERO })
    } else {
        mulsign(r, x)
    };

    mulsign(r, y)
}

#[test]
fn test_atan2() {
    test_pp_p(atan2, f64::atan2, P32E2::MIN.0, P32E2::MAX.0, 3);
}

/// Natural logarithmic function
///
/// These functions return the natural logarithm of ***a***.
pub fn ln(d: P32E2) -> P32E2 {
    if d <= ZERO {
        return P32E2::NAR;
    }
    kernel::log(d)
}

#[test]
fn test_ln() {
    test_p_p(ln, f64::ln, ZERO.0, P32E2::MAX.0, 2);
}

// TODO: fix coeffs
pub fn log2(d: P32E2) -> P32E2 {
    if d <= ZERO {
        return P32E2::NAR;
    }

    let e = kernel::ilogb(d * (ONE / P32E2::new(0x_3c00_0000)/*0.75*/)); // ilogb2kf
    let m = kernel::ldexp2(d, -e); //ldexp3kf(d, -e);

    let x = (m - ONE) / (m + ONE);
    let x2 = x * x;

    let t = x2.poly3(&[
        // First pass
        P32E2::new(0x_3316_c66e), // 3.4653016599e-1,
        P32E2::new(0x_3529_b329), // 4.1134031280e-1,
        P32E2::new(0x_393b_c226), // 5.7708945172e-1,
        P32E2::new(0x_3f63_84e0), // 9.6179664042e-1,

                                  /*        P32E2::new(0x_35ff_40d0), // 0.437_408_834_7
                                  P32E2::new(0x_3939_47b0), // 0.576_484_382_2
                                  P32E2::new(0x_3f63_8af0), // 0.961_802_423*/
    ]);

    let mut quire = Q32E2::init();
    quire += (x2 * x, t);
    quire += (x, P32E2::new(0x_4b8a_a3b3)); // 2.8853900824
                                            //quire += (x, P32E2::new(0x_4b8a_a3b3)); // 2.8853900879621506
    quire += (P32E2::from(e), ONE);
    quire.into()
}

#[test]
fn test_log2() {
    test_p_p(log2, f64::log2, ZERO.0, P32E2::MAX.0, 3);
}

/// 2D Euclidian distance function
pub fn hypot(mut x: P32E2, mut y: P32E2) -> P32E2 {
    x = x.abs();
    y = y.abs();
    let min = x.min(y);
    let max = x.max(y);

    let t = min / max;
    if x.is_nar() || y.is_nar() {
        NAR
    } else if min == ZERO {
        max
    } else {
        max * (ONE + t * t).sqrt()
    }
}

#[test]
fn test_hypot() {
    test_pp_p(hypot, f64::hypot, P32E2::MIN.0, P32E2::MAX.0, 4);
}

/// Sine function
///
/// These functions evaluates the sine function of a value in ***a***.
pub fn sin(mut d: P32E2) -> P32E2 {
    if d.is_nar() {
        return NAR;
    }

    let q: i32;

    if d.abs() < TRIGRANGEMAX {
        let qf = (d * P32E2::FRAC_1_PI).round();
        q = qf.into();
        let mut quire = Q32E2::init();
        quire += (d, ONE);
        quire -= (qf, [PI_A, PI_B, PI_C]);
        d = quire.into();
    } else {
        unimplemented!()
    }

    let s = d * d;

    if (q & 1) != 0 {
        d = -d;
    }

    kernel::sin(s) * d
}

#[test]
fn test_sin() {
    test_p_p(sin, f64::sin, -TRIGRANGEMAX.0 + 1, TRIGRANGEMAX.0 - 1, 2);
}

/// Cosine function
///
/// These functions evaluates the cosine function of a value in ***a***.
pub fn cos(mut d: P32E2) -> P32E2 {
    if d.is_nar() {
        return NAR;
    }

    let q: i32;

    if d.abs() < TRIGRANGEMAX {
        q = 1 + 2 * i32::from((d * P32E2::FRAC_1_PI - HALF).round());
        let qf = P32E2::from(q);
        let mut quire = Q32E2::init();
        quire += (d, ONE);
        quire -= (qf, [PI_A * HALF, PI_B * HALF, PI_C * HALF]);
        d = quire.into();
    } else {
        unimplemented!()
    }

    let s = d * d;

    if (q & 2) == 0 {
        d = -d;
    }

    kernel::sin(s) * d
}

#[test]
fn test_cos() {
    test_p_p(cos, f64::cos, -TRIGRANGEMAX.0 + 1, TRIGRANGEMAX.0 - 1, 2);
}

/// Evaluate sine and cosine function simultaneously
///
/// Evaluates the sine and cosine functions of a value in ***a*** at a time,
/// and store the two values in *first* and *second* position in the returned value, respectively.
pub fn sin_cos(d: P32E2) -> (P32E2, P32E2) {
    if d.is_nar() {
        return (NAR, NAR);
    }

    let q: i32;

    let mut s = d;

    if d.abs() < TRIGRANGEMAX {
        let qf = (s * P32E2::FRAC_1_PI).round();
        q = qf.into();
        let mut quire = Q32E2::init();
        quire += (s, ONE);
        quire -= (qf, [PI_A * HALF, PI_B * HALF, PI_C * HALF]);
        s = quire.into();
    } else {
        unimplemented!()
    }

    let t = s;

    s = s * s;

    let mut rsin = kernel::sin(s) * t;

    let mut rcos = kernel::cos(s);

    if (q & 1) != 0 {
        core::mem::swap(&mut rcos, &mut rsin);
    }
    if (q & 2) != 0 {
        rsin = -rsin;
    }
    if ((q + 1) & 2) != 0 {
        rcos = -rcos;
    }

    (rsin, rcos)
}

/// Tangent function
///
/// These functions evaluates the tangent function of a value in ***a***.
pub fn tan(d: P32E2) -> P32E2 {
    if d.is_nar() {
        return NAR;
    }

    let q: i32;

    let mut x: P32E2;

    if d.abs() < TRIGRANGEMAX {
        let qf = (d * P32E2::FRAC_2_PI).round();
        q = qf.into();
        let mut quire = Q32E2::init();
        quire += (d, ONE);
        quire -= (qf, [PI_A * HALF, PI_B * HALF, PI_C * HALF]);
        x = quire.into();
    } else {
        unimplemented!()
    }

    let s = x * x;

    if (q & 1) != 0 {
        x = -x;
    }

    let u = s.poly7(&[
        P32E2::new(0x_1043_9ddd), // 4.1641870630e-3,
        P32E2::new(0x_0a23_312c), // 5.2184302331e-4,
        P32E2::new(0x_155f_d348), // 1.0496714152e-2,
        P32E2::new(0x_197b_2043), // 2.1410004003e-2,
        P32E2::new(0x_1eea_ab3b), // 5.4036525544e-2,
        P32E2::new(0x_2888_73e8), // 1.3332841545e-1,
        P32E2::new(0x_32aa_aaf4), // 3.333334698957821126613114826434348e-1,
        ONE,
    ]) * x;

    if (q & 1) != 0 {
        u.recip()
    } else {
        u
    }
}

#[test]
fn test_tan() {
    test_p_p(tan, f64::tan, -TRIGRANGEMAX.0 + 1, TRIGRANGEMAX.0 - 1, 3);
}

/// Arc tangent function
///
/// These functions evaluates the arc tangent function of a value in ***a***.
pub fn atan(mut s: P32E2) -> P32E2 {
    let mut q = if s.is_sign_negative() {
        s = -s;
        2
    } else {
        0
    };

    if s > ONE {
        s = s.recip();
        q |= 1;
    }

    let mut t = s * s;

    let u = t.poly8(&[
        P32E2::new(-0x_0de4_f8b5), // -1.9015722646e-3,
        P32E2::new(0x_15cf_6226),  // 1.1347834719e-2,
        P32E2::new(-0x_1c14_c8ad), // -3.1884273980e-2,
        P32E2::new(0x_1f7e_b681),  // 5.8554471005e-2,
        P32E2::new(-0x_22ca_9b6e), // -8.4308079444e-2,
        P32E2::new(0x_2606_ded6),  // 1.0958466958e-1,
        P32E2::new(-0x_2921_200a), // -1.4264679886e-1,
        P32E2::new(0x_2ccc_8d0c),  // 1.9998480007e-1,
        P32E2::new(-0x_32aa_a9be), // -3.333328932749459636123021434852409e-1,
    ]);

    t = s + s * (t * u);

    if (q & 1) != 0 {
        t = P32E2::new(0x_4490_fdaa) - t;
    }
    if (q & 2) != 0 {
        -t
    } else {
        t
    }
}

#[test]
fn test_atan() {
    test_p_p(atan, f64::atan, P32E2::MIN.0, P32E2::MAX.0, 3);
}

/// Arc sine function
///
/// These functions evaluates the arc sine function of a value in ***a***.
/// The error bound of the returned value is 3.5 ULP.
pub fn asin(d: P32E2) -> P32E2 {
    let o = d.abs() < HALF;
    let x2 = if o { d * d } else { (ONE - d.abs()) * HALF };
    let x = if o { d.abs() } else { x2.sqrt() };

    let u = x2.poly6(&[
        P32E2::new(0x_1cc5_4185), // 3.7269773427e-2,
        P32E2::new(0x_1775_4679), // 1.4566614409e-2,
        P32E2::new(0x_1c11_bbd3), // 3.1791189220e-2,
        P32E2::new(0x_1db2_b15f), // 4.4515773188e-2,
        P32E2::new(0x_2199_c6fe), // 7.5005411170e-2,
        P32E2::new(0x_2aaa_aa4e), // 1.6666658036e-1,
        ONE,
    ]) * x;

    let r = if o { u } else { (P32E2::FRAC_PI_2 - TWO * u) };
    mulsign(r, d)
}

#[test]
fn test_asin() {
    test_p_p(asin, f64::asin, -ONE.0, ONE.0, 3);
}

/// Arc cosine function
///
/// These functions evaluates the arc cosine function of a value in ***a***.
pub fn acos(d: P32E2) -> P32E2 {
    let o = d.abs() < HALF;
    let x2 = if o { d * d } else { (ONE - d.abs()) * HALF };
    let mut x = if o { d.abs() } else { x2.sqrt() };
    x = if d.abs() == ONE { ZERO } else { x };

    let u = x2.poly5(&[
        P32E2::new(0x_1cc5_4185), // 3.7269773427e-2,
        P32E2::new(0x_1775_4679), // 1.4566614409e-2,
        P32E2::new(0x_1c11_bbd3), // 3.1791189220e-2,
        P32E2::new(0x_1db2_b15f), // 4.4515773188e-2,
        P32E2::new(0x_2199_c6fe), // 7.5005411170e-2,
        P32E2::new(0x_2aaa_aa4e), // 1.6666658036e-1,
    ]) * (x * x2);

    let y = P32E2::FRAC_PI_2 - (mulsign(x, d) + mulsign(u, d));
    x += u;
    let r = if o { y } else { x * TWO };
    if !o && (d < ZERO) {
        let mut quire = Q32E2::PI;
        quire -= (r, ONE);
        quire.into()
    } else {
        r
    }
}

#[test]
fn test_acos() {
    test_p_p(acos, f64::acos, -ONE.0, ONE.0, 2);
}

/// Cube root function
///
/// These functions return the real cube root of ***a***.
// TODO: fix coeffs
pub fn cbrt(mut d: P32E2) -> P32E2 {
    let e = kernel::ilogb(d /*.abs()*/) + 1;
    d = kernel::ldexp2(d, -e);
    let r = (e + 6144) % 3;
    let mut q = if r == 1 {
        P32E2::new(0x_4214_517d) // 1.259_921_049_894_873_164_767_210_6
    } else {
        ONE
    };
    q = if r == 2 {
        P32E2::new(0x_44b2_ff53) // 1.587_401_051_968_199_474_751_705_6
    } else {
        q
    };
    q = kernel::ldexp2(q, (e + 6144) / 3 - 2048);

    q = mulsign(q, d);
    d = d.abs();

    let x = d.poly5(&[
        P32E2::new(-0x_39a0_0210), //-0.601564466953277587890625
        P32E2::new(0x_4b48_9730),  // 2.8208892345428466796875
        P32E2::new(-0x_5310_7a30), // -5.532182216644287109375
        P32E2::new(0x_53cb_e910),  // 5.898262500762939453125
        P32E2::new(-0x_4f3c_f880), // -3.8095417022705078125
        P32E2::new(0x_48e5_8130),  // 2.2241256237030029296875
    ]);

    let y = d * x * x;
    (y - (TWO / THREE) * y * (y * x - ONE)) * q
}

#[test]
fn test_cbrt() {
    test_p_p(cbrt, f64::cbrt, P32E2::MIN.0, P32E2::MAX.0, 4);
}

// TODO: fix coeffs
pub fn exp2(d: P32E2) -> P32E2 {
    let q = d.round();

    let s = d - q;

    let mut u = s.poly7(&[
        // First phase
        P32E2::new(0x_03ff_5322), // 1.5218540611e-5,
        P32E2::new(0x_0743_a155), // 1.5431890756e-4,
        P32E2::new(0x_0cbb_1128), // 1.3333645047e-3,
        P32E2::new(0x_14ec_aa3e), // 9.6181107656e-3,
        P32E2::new(0x_1f1a_c235), // 5.5504108226e-2,
        P32E2::new(0x_2f5f_df00), // 2.4022650730e-1,
        P32E2::new(0x_3b17_217f), // 6.9314718060e-1,
        ONE,
    ]);

    u = kernel::ldexp2(u, q.into());

    if d < P32E2::new(-0x_6cb0_0000)
    /* -150.*/
    {
        ZERO
    } else if d >= P32E2::new(0x_6c00_0000)
    /*128.*/
    {
        NAR
    } else {
        u
    }
}

#[test]
fn test_exp2() {
    test_p_p(exp2, f64::exp2, -0x_6cb0_0000, 0x_6c00_0000, 1);
}

// TODO: fix coeffs
pub fn exp10(d: P32E2) -> P32E2 {
    let q = (d * P32E2::LOG10_2).round();

    let mut quire = Q32E2::init();
    quire += (d, ONE);
    quire -= (q, [L10U, L10L]);
    let s = quire.to_posit();

    /* First phase
    6.6708447179e-2,
    2.1117800497e-1,
    5.3978904942e-1,
    1.1709893206,
    2.0346547477,
    2.6509541366,
    2.3025853876,
    9.9999998463e-1,
    */

    let mut u = s.poly6(&[
        P32E2::new(0x_2d35_aa70), // 0.206_400_498_7
        P32E2::new(0x_38ab_29a0), // 0.541_787_743_6)
        P32E2::new(0x_415e_cba0), // 0.117_128_682_1_e+1)
        P32E2::new(0x_4823_7ce0), // 0.203_465_604_8_e+1)
        P32E2::new(0x_4a9a_9250), // 0.265_094_876_3_e+1)
        P32E2::new(0x_4935_d8e0), // 0.230_258_512_5_e+1)
        ONE,
    ]);

    u = kernel::ldexp2(u, q.into());

    if d < P32E2::new(-0x_6640_0000)
    /* -50. */
    {
        ZERO
    } else if d > P32E2::new(0x_64d1_04d4)
    /* 38.531_839_419_103_623_894_138_7*/
    {
        NAR
    } else {
        u
    }
}
/*
#[test]
fn test_exp10() {
    test_p_p(
        exp10,
        libm::exp10,
        -0x_6640_0000,
        0x_64d1_04d4,
        4,
    );
}*/

/// Base-*e* exponential function
///
/// This function returns the value of *e* raised to ***a***.
pub fn exp(d: P32E2) -> P32E2 {
    let qf = (d * R_LN2).round();
    let q = i32::from(qf);

    let mut quire = Q32E2::init();
    quire += (d, ONE);
    quire -= (qf, [L2U, L2L]);
    let s = quire.to_posit();

    let mut u = s.poly5(&[
        P32E2::new(0x_079d_b0ca), // 1.9726304345e-4,
        P32E2::new(0x_0cda_fee4), // 1.3942635851e-3,
        P32E2::new(0x_1444_4e5b), // 8.3336340031e-3,
        P32E2::new(0x_1d55_5258), // 4.1666310281e-2,
        P32E2::new(0x_2aaa_aa9a), // 1.6666665114e-1,
        P32E2::new(0x_3800_0002), // 5.0000000745e-1,
    ]);

    u = s * s * u + s + ONE;

    if d < P32E2::new(-0x_6a80_0000)
    /* -104.*/
    {
        ZERO
    } else if d > P32E2::new(0x_6a80_0000)
    /* 104.*/
    {
        NAR
    } else {
        kernel::ldexp2(u, q)
    }
}

#[test]
fn test_exp() {
    test_p_p(exp, f64::exp, -0x_6a80_0000, 0x_6a80_0000, 1);
}

/// Hyperbolic sine function
///
/// These functions evaluates the hyperbolic sine function of a value in ***a***.
pub fn sinh(x: P32E2) -> P32E2 {
    let e = kernel::exp_m1(x.abs());
    let mut y = (e + TWO) / (e + ONE) * (HALF * e);

    y = if x.abs() > P32E2::new(0x_6980_0000)
    /* 88. */
    {
        P32E2::NAR
    } else {
        y
    };
    y = if y.is_nar() { P32E2::NAR } else { y };
    y = mulsign(y, x);
    if x.is_nar() {
        P32E2::NAR
    } else {
        y
    }
}

#[test]
fn test_sinh() {
    test_p_p(sinh, f64::sinh, -0x_6980_0000, 0x_6980_0000, 4);
}

/// Hyperbolic cosine function
///
/// These functions evaluates the hyperbolic cosine function of a value in ***a***.
pub fn cosh(x: P32E2) -> P32E2 {
    let e = x.abs().exp();
    let mut y = HALF * e + HALF / e;

    y = if x.abs() > P32E2::new(0x_6980_0000)
    /* 88. */
    {
        P32E2::NAR
    } else {
        y
    };
    y = if y.is_nar() { P32E2::NAR } else { y };
    if x.is_nar() {
        P32E2::NAR
    } else {
        y
    }
}

#[test]
fn test_cosh() {
    test_p_p(cosh, f64::cosh, -0x_6980_0000, 0x_6980_0000, 2);
}

/// Hyperbolic tangent function
///
/// These functions evaluates the hyperbolic tangent function of a value in ***a***.
pub fn tanh(x: P32E2) -> P32E2 {
    let mut y = x.abs();
    let d = kernel::exp_m1(TWO * y);
    y = d / (d + TWO);

    y = if x.abs() > P32E2::new(0x_60ad_c222)
    /* 18.714_973_875 */
    {
        ONE
    } else {
        y
    };
    y = if y.is_nar() { ONE } else { y };
    y = mulsign(y, x);
    if x.is_nar() {
        P32E2::NAR
    } else {
        y
    }
}
/*
#[test]
fn test_tanh() {
    test_p_p(tanh, f64::tanh, -0x_60ad_c222, 0x_60ad_c222, 4);
}
*/
#[cfg(test)]
fn ulp(x: P32E2, y: P32E2) -> i32 {
    let xi = x.to_bits() as i32;
    let yi = y.to_bits() as i32;
    (xi.wrapping_sub(yi)).abs()
}

#[cfg(test)]
fn test_p_p(fun_p: fn(P32E2) -> P32E2, fun_f: fn(f64) -> f64, mn: i32, mx: i32, expected_ulp: i32) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut av_ulp = 0_f64;
    let mut ncorrect = 0;
    let mut max_ulp = 0;
    let mut inc_x = 0_f64;
    let mut inc_answer = P32E2::ZERO;
    let mut inc_correct = P32E2::ZERO;
    for i in 0..NTESTS {
        let n_a = rng.gen_range(mn, mx);
        let p_a = P32E2::new(n_a);
        let f_a = f64::from(p_a);
        let answer = fun_p(p_a);
        let correct = P32E2::from(fun_f(f_a));
        let u = ulp(answer, correct);

        if u > max_ulp {
            max_ulp = u;
        }
        if u <= expected_ulp {
            ncorrect += 1;
        } else {
            inc_x = f_a;
            inc_answer = answer;
            inc_correct = correct;
        }
        av_ulp += u as f64;
        if (i == NTESTS - 1) && (max_ulp > expected_ulp) {
            av_ulp /= NTESTS as f64;
            assert!(false, "Correct = {} %, max_ulp = {}, av_ulp = {}\nLast: x = {}, answer = {}, correct = {}",
                (ncorrect*100) as f32 / (NTESTS as f32),
                max_ulp,
                av_ulp,
                inc_x,
                inc_answer,
                inc_correct
            );
        }
    }
}

#[cfg(test)]
fn test_pp_p(
    fun_p: fn(P32E2, P32E2) -> P32E2,
    fun_f: fn(f64, f64) -> f64,
    mn: i32,
    mx: i32,
    expected_ulp: i32,
) {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut av_ulp = 0_f64;
    let mut ncorrect = 0;
    let mut max_ulp = 0;
    let mut inc_x = 0_f64;
    let mut inc_y = 0_f64;
    let mut inc_answer = P32E2::ZERO;
    let mut inc_correct = P32E2::ZERO;
    for i in 0..NTESTS {
        let n_a = rng.gen_range(mn, mx);
        let n_b = rng.gen_range(mn, mx);
        let p_a = P32E2::new(n_a);
        let p_b = P32E2::new(n_b);
        let f_a = f64::from(p_a);
        let f_b = f64::from(p_b);
        let answer = fun_p(p_a, p_b);
        let correct = P32E2::from(fun_f(f_a, f_b));
        let u = ulp(answer, correct);

        if u > max_ulp {
            max_ulp = u;
        }
        if u <= expected_ulp {
            ncorrect += 1;
        } else {
            inc_x = f_a;
            inc_y = f_b;
            inc_answer = answer;
            inc_correct = correct;
        }
        av_ulp += u as f64;
        if (i == NTESTS - 1) && (max_ulp > expected_ulp) {
            av_ulp /= NTESTS as f64;
            assert!(false, "Correct = {} %, max_ulp = {}, av_ulp = {}\nLast: x = {}, y = {}, answer = {}, correct = {}",
                (ncorrect*100) as f32 / (NTESTS as f32),
                max_ulp,
                av_ulp,
                inc_x,
                inc_y,
                inc_answer,
                inc_correct
            );
        }
    }
}
