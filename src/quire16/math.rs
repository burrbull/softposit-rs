use super::Q16E1;

impl crate::MathConsts for Q16E1 {
    const E: Self = Self(0, 0x_02b7_e151_628a_ed2a);
    const FRAC_1_PI: Self = Self(0, 0x_0051_7cc1_b727_220b);
    const FRAC_1_SQRT_2: Self = Self(0, 0x_00b5_04f3_33f9_de65);
    const FRAC_2_PI: Self = Self(0, 0x_00a2_f983_6e4e_4415);
    const FRAC_2_SQRT_PI: Self = Self(0, 0x_0120_dd75_0429_b6d1);
    const FRAC_PI_2: Self = Self(0, 0x_0192_1fb5_4442_d184);
    const FRAC_PI_3: Self = Self(0, 0x_010c_1523_82d7_3658);
    const FRAC_PI_4: Self = Self(0, 0x_00c9_0fda_a221_68c2);
    const FRAC_PI_6: Self = Self(0, 0x_0086_0a91_c16b_9b2c);
    const FRAC_PI_8: Self = Self(0, 0x_0064_87ed_5110_b461);
    const LN_10: Self = Self(0, 0x_024d_7637_76aa_a2b0);
    const LN_2: Self = Self(0, 0x_00b1_7217_f7d1_cf7a);
    const LOG10_E: Self = Self(0, 0x_006f_2dec_549b_9439);
    const LOG2_E: Self = Self(0, 0x_0171_5476_52b8_2fe1);
    const PI: Self = Self(0, 0x_0324_3f6a_8885_a309);
    const SQRT_2: Self = Self(0, 0x_016a_09e6_67f3_bcc9);
    const LOG2_10: Self = Self(0, 0x_0352_69e1_2f34_6e2c);
    const LOG10_2: Self = Self(0, 0x_004d_104d_427d_e7fc);
}
