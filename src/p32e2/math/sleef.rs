use crate::{MathConsts, Polynom, P32E2, Q32E2};

use super::{
    HALF, // 0.5
    TWO,  // 2.
};

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
        let exp_a: u32 = ((n & 0x3) as u32) << (27 - k);
        let ui_a = (0x7FFF_FFFF ^ (0x3FFF_FFFF >> k)) | exp_a;

        if sign {
            P32E2::from_bits((ui_a << 1).wrapping_neg() >> 1)
        } else {
            P32E2::from_bits(ui_a)
        }
    }

    pub fn ldexp2(d: P32E2, e: i32) -> P32E2 {
        // faster than ldexpkf, short reach
        d * pow2i(e >> 1) * pow2i(e - (e >> 1))
    }

    #[inline]
    pub fn exp_m1(d: P32E2) -> P32E2 {
        let qf = (d * R_LN2).round();
        let q = i32::from(qf);

        let mut quire = Q32E2::init();
        quire += (d, ONE);
        quire -= (qf, L2U);
        quire -= (qf, L2L);
        let s = quire.to_posit();

        let mut u = s.poly5(&[
            P32E2::new(0x_07a0_57b4), // 0.000_198_527_617_612_853_646_278_381,
            P32E2::new(0x_0cda_5b24), // 0.001_393_043_552_525_341_510_772_71,
            P32E2::new(0x_1444_4530), // 0.008_333_360_776_305_198_669_433_59,
            P32E2::new(0x_1d55_53d0), // 0.041_666_485_369_205_474_853_515_6,
            P32E2::new(0x_2aaa_aab0), // 0.166_666_671_633_720_397_949_219,
            HALF,
        ]);
        u = s * s * u + s;

        if q != 0 {
            ldexp2(u + ONE, q) - ONE
        } else {
            u
        }
    }
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
        quire -= (qf, PI_A);
        quire -= (qf, PI_B);
        quire -= (qf, PI_C);
        d = quire.into();
    } else {
        unimplemented!()
    }

    let s = d * d;

    if (q & 1) != 0 {
        d = -d;
    }

    let u = s.poly3(&[
        P32E2::new(0x_02af_0a87),  // 2.608_315_980_978_659_354_150_3_e-6
        P32E2::new(-0x_079f_75d6), // -0.000_198_106_907_191_686_332_225_8
        P32E2::new(0x_1444_3bb8),  // 0.008_333_078_585_565_090_179_443_36
        P32E2::new(-0x_2aaa_aa60), // -0.166_666_597_127_914_428_710_938
    ]);

    s.mul_add(u * d, d)
}

#[test]
fn test_sin() {
    test_p_p(sin, f64::sin, -TRIGRANGEMAX.0 + 1, TRIGRANGEMAX.0 - 1, 4);
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
        q = 1 + 2 * i32::from((d * P32E2::FRAC_1_PI).floor());
        let qf = P32E2::from(q);
        let mut quire = Q32E2::init();
        quire += (d, ONE);
        quire -= (qf, PI_A * HALF);
        quire -= (qf, PI_B * HALF);
        quire -= (qf, PI_C * HALF);
        d = quire.into();
    } else {
        unimplemented!()
    }

    let s = d * d;

    if (q & 2) == 0 {
        d = -d;
    }

    let u = s.poly3(&[
        P32E2::new(0x_02af_0a87),  // 2.608_315_980_978_659_354_150_3_e-6
        P32E2::new(-0x_079f_75d6), // -0.000_198_106_907_191_686_332_225_8
        P32E2::new(0x_1444_3bb8),  // 0.008_333_078_585_565_090_179_443_36
        P32E2::new(-0x_2aaa_aa60), // -0.166_666_597_127_914_428_710_938
    ]);

    s.mul_add(u * d, d)
}

#[test]
fn test_cos() {
    test_p_p(cos, f64::cos, -TRIGRANGEMAX.0 + 1, TRIGRANGEMAX.0 - 1, 4);
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
        quire -= (qf, PI_A * HALF);
        quire -= (qf, PI_B * HALF);
        quire -= (qf, PI_C * HALF);
        x = quire.into();
    } else {
        unimplemented!()
    }

    let s = x * x;

    if (q & 1) != 0 {
        x = -x;
    }

    let mut u = s.poly5(&[
        P32E2::new(0x_14bf_5c10), // 0.009_272_458_031_773_567_199_707_03,
        P32E2::new(0x_0f66_475c), // 0.003_319_849_958_643_317_222_595_21,
        P32E2::new(0x_1a38_8320), // 0.024_299_807_846_546_173_095_703_1,
        P32E2::new(0x_1ed7_6f28), // 0.053_449_530_154_466_629_028_320_3,
        P32E2::new(0x_2889_58e0), // 0.133_383_005_857_467_651_367_188,
        P32E2::new(0x_32aa_a790), // 0.333_331_853_151_321_411_132_812,
    ]);

    u = s.mul_add(u * x, x);

    if (q & 1) != 0 {
        u.recip()
    } else {
        u
    }
}

#[test]
fn test_tan() {
    test_p_p(tan, f64::tan, -TRIGRANGEMAX.0 + 1, TRIGRANGEMAX.0 - 1, 13);
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

    let u = t.poly7(&[
        P32E2::new(0x_0ee4_3334),  // 0.002_823_638_962_581_753_730_773_93,
        P32E2::new(-0x_1815_c068), // -0.015_956_902_876_496_315_002_441_4,
        P32E2::new(0x_1d70_cdb0),  // 0.042_504_988_610_744_476_318_359_4,
        P32E2::new(-0x_2195_ffa0), // -0.074_890_092_015_266_418_457_031_2,
        P32E2::new(0x_259c_cf20),  // 0.106_347_933_411_598_205_566_406,
        P32E2::new(-0x_2916_f9f0), // -0.142_027_363_181_114_196_777_344,
        P32E2::new(0x_2ccb_9a70),  // 0.199_926_957_488_059_997_558_594,
        P32E2::new(-0x_32aa_a5d0), //-0.333_331_018_686_294_555_664_062,
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
    test_p_p(atan, f64::atan, P32E2::MIN.0, P32E2::MAX.0, 13);
}

/// Arc sine function
///
/// These functions evaluates the arc sine function of a value in ***a***.
/// The error bound of the returned value is 3.5 ULP.
pub fn asin(d: P32E2) -> P32E2 {
    let o = d.abs() < HALF;
    let x2 = if o { d * d } else { (ONE - d.abs()) * HALF };
    let x = if o { d.abs() } else { x2.sqrt() };

    let u = x2
        .poly4(&[
            P32E2::new(0x_1d5f_6c08), // 0.419_745_482_5_e-1
            P32E2::new(0x_1a34_9f70), // 0.242_404_602_5_e-1
            P32E2::new(0x_1dd2_1990), // 0.454_742_386_9_e-1
            P32E2::new(0x_2197_f8a0), // 0.749_502_927_1_e-1
            P32E2::new(0x_2aaa_af20), // 0.166_667_729_6
        ])
        .mul_add(x * x2, x);

    let r = if o { u } else { (P32E2::FRAC_PI_2 - TWO * u) };
    mulsign(r, d)
}

#[test]
fn test_asin() {
    test_p_p(asin, f64::asin, -ONE.0, ONE.0, 4);
}

/// Arc cosine function
///
/// These functions evaluates the arc cosine function of a value in ***a***.
pub fn acos(d: P32E2) -> P32E2 {
    let o = d.abs() < HALF;
    let x2 = if o { d * d } else { (ONE - d.abs()) * HALF };
    let mut x = if o { d.abs() } else { x2.sqrt() };
    x = if d.abs() == ONE { ZERO } else { x };

    let mut u = x2.poly4(&[
        P32E2::new(0x_1d5f_6c08), // 0.419_745_482_5_e-1
        P32E2::new(0x_1a34_9f70), // 0.242_404_602_5_e-1
        P32E2::new(0x_1dd2_1990), // 0.454_742_386_9_e-1
        P32E2::new(0x_2197_f8a0), // 0.749_502_927_1_e-1
        P32E2::new(0x_2aaa_af20), // 0.166_667_729_6
    ]);
    u *= x * x2;

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
    test_p_p(acos, f64::acos, -ONE.0, ONE.0, 4);
}

pub fn exp2(d: P32E2) -> P32E2 {
    let q = d.round();

    let s = d - q;

    let mut u = s.poly6(&[
        P32E2::new(0x_0742_1b20), // 0.153_592_089_2_e-3
        P32E2::new(0x_0cbe_28cc), // 0.133_926_270_1_e-2
        P32E2::new(0x_14ec_b370), // 0.961_838_476_4_e-2
        P32E2::new(0x_1f1a_bce0), // 0.555_034_726_9_e-1
        P32E2::new(0x_2f5f_dec0), // 0.240_226_447_6
        P32E2::new(0x_3b17_2180), // 0.693_147_182_5
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
    test_p_p(exp2, f64::exp2, -0x_6cb0_0000, 0x_6c00_0000, 4);
}

pub fn exp10(d: P32E2) -> P32E2 {
    let q = (d * P32E2::LOG10_2).round();

    let mut quire = Q32E2::init();
    quire += (d, ONE);
    quire -= (q, L10U);
    quire -= (q, L10L);
    let s = quire.to_posit();

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
    quire -= (qf, L2U);
    quire -= (qf, L2L);
    let s = quire.to_posit();

    let mut u = s.poly5(&[
        P32E2::new(0x_07a0_57b4), // 0.000_198_527_617_612_853_646_278_381_f32
        P32E2::new(0x_0cda_5b24), // 0.001_393_043_552_525_341_510_772_71)
        P32E2::new(0x_1444_4530), // 0.008_333_360_776_305_198_669_433_59)
        P32E2::new(0x_1d55_53d0), // 0.041_666_485_369_205_474_853_515_6)
        P32E2::new(0x_2aaa_aab0), // 0.166_666_671_633_720_397_949_219)
        HALF,
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
    test_p_p(exp, f64::exp, -0x_6a80_0000, 0x_6a80_0000, 4);
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
    test_p_p(cosh, f64::cosh, -0x_6980_0000, 0x_6980_0000, 4);
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
    //    let mut ncorrect = 0;
    //    let mut max_ulp = 0;
    for _i in 0..NTESTS {
        let n_a = rng.gen_range(mn, mx);
        let p_a = P32E2::new(n_a);
        let f_a = f64::from(p_a);
        let answer = fun_p(p_a);
        let correct = P32E2::from(fun_f(f_a));
        let u = ulp(answer, correct);
        /*
        if u > max_ulp {
            max_ulp = u;
        }
        */
        assert!(
            u <= expected_ulp,
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
    for _i in 0..NTESTS {
        let n_a = rng.gen_range(mn, mx);
        let n_b = rng.gen_range(mn, mx);
        let p_a = P32E2::new(n_a);
        let p_b = P32E2::new(n_b);
        let f_a = f64::from(p_a);
        let f_b = f64::from(p_b);
        let answer = fun_p(p_a, p_b);
        let correct = P32E2::from(fun_f(f_a, f_b));
        let u = ulp(answer, correct);
        assert!(
            u <= expected_ulp,
            "x = {}, y = {}, answer = {}, correct = {}, ulp = {}",
            f_a,
            f_b,
            answer,
            correct,
            u,
        );
    }
}
