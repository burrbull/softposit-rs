use super::Q32E2;

impl crate::MathConsts for Q32E2 {
    const E: Self = Self(
        0,
        0,
        0,
        0,
        0x_0002_b7e1_5162_8aed,
        0x_2a6a_bf71_5880_9cf4,
        0x_f3c7_62e7_160f_38b4,
        0x_da56_a784_d904_5190,
    );
    const FRAC_1_PI: Self = Self(
        0,
        0,
        0,
        0,
        0x_0000_517c_c1b7_2722,
        0x_0a94_fe13_abe8_fa9a,
        0x_6ee0_6db1_4acc_9e21,
        0x_c820_ff28_b1d5_ef5d,
    );
    const FRAC_1_SQRT_2: Self = Self(
        0,
        0,
        0,
        0,
        0x_0000_b504_f333_f9de,
        0x_6484_597d_89b3_754a,
        0x_be9f_1d6f_60ba_893b,
        0x_a84c_ed17_ac85_8333,
    );
    const FRAC_2_PI: Self = Self(
        0,
        0,
        0,
        0,
        0x_0000_a2f9_836e_4e44,
        0x_1529_fc27_57d1_f534,
        0x_ddc0_db62_9599_3c43,
        0x_9041_fe51_63ab_debb,
    );
    const FRAC_2_SQRT_PI: Self = Self(
        0,
        0,
        0,
        0,
        0x_0001_20dd_7504_29b6,
        0x_d11a_e3a9_14fe_d7fd,
        0x_8688_2813_41d7_587c,
        0x_ea2e_7342_b061_99cc,
    );
    const FRAC_PI_2: Self = Self(
        0,
        0,
        0,
        0,
        0x_0001_921f_b544_42d1,
        0x_8469_898c_c517_01b8,
        0x_39a2_5204_9c11_14cf,
        0x_98e8_0417_7d4c_7627,
    );
    const FRAC_PI_3: Self = Self(
        0,
        0,
        0,
        0,
        0x_0001_0c15_2382_d736,
        0x_5846_5bb3_2e0f_567a,
        0x_d116_e158_680b_6335,
        0x_109a_ad64_fe32_f96f,
    );
    const FRAC_PI_4: Self = Self(
        0,
        0,
        0,
        0,
        0x_0000_c90f_daa2_2168,
        0x_c234_c4c6_628b_80dc,
        0x_1cd1_2902_4e08_8a67,
        0x_cc74_020b_bea6_3b13,
    );
    const FRAC_PI_6: Self = Self(
        0,
        0,
        0,
        0,
        0x_0000_860a_91c1_6b9b,
        0x_2c23_2dd9_9707_ab3d,
        0x_688b_70ac_3405_b19a,
        0x_884d_56b2_7f19_7cb7,
    );
    const FRAC_PI_8: Self = Self(
        0,
        0,
        0,
        0,
        0x_0000_6487_ed51_10b4,
        0x_611a_6263_3145_c06e,
        0x_0e68_9481_2704_4533,
        0x_e63a_0105_df53_1d89,
    );
    const LN_10: Self = Self(
        0,
        0,
        0,
        0,
        0x_0002_4d76_3776_aaa2,
        0x_b05b_a95b_58ae_0b4c,
        0x_28a3_8a3f_b3e7_6977,
        0x_e43a_0f18_7a08_07c0,
    );
    const LN_2: Self = Self(
        0,
        0,
        0,
        0,
        0x_0000_b172_17f7_d1cf,
        0x_79ab_c9e3_b398_03f2,
        0x_f6af_40f3_4326_7298,
        0x_b62d_8a0d_175b_8baa,
    );
    const LOG10_E: Self = Self(
        0,
        0,
        0,
        0,
        0x_0000_6f2d_ec54_9b94,
        0x_38ca_9aad_d557_d699,
        0x_ee19_1f71_a301_22e4,
        0x_d101_1d1f_96a2_7bc7,
    );
    const LOG2_E: Self = Self(
        0,
        0,
        0,
        0,
        0x_0001_7154_7652_b82f,
        0x_e177_7d0f_fda0_d23a,
        0x_7d11_d6ae_f551_bad2,
        0x_b4b1_164a_2cd9_a342,
    );
    const PI: Self = Self(
        0,
        0,
        0,
        0,
        0x_0003_243f_6a88_85a3,
        0x08d3_1319_8a2e_0370,
        0x_7344_a409_3822_299f,
        0x_31d0_082e_fa98_ec4e,
    );
    const SQRT_2: Self = Self(
        0,
        0,
        0,
        0,
        0x_0001_6a09_e667_f3bc,
        0x_c908_b2fb_1366_ea95,
        0x_7d3e_3ade_c175_1277,
        0x_5099_da2f_590b_0667,
    );
    const LOG2_10: Self = Self(
        0,
        0,
        0,
        0,
        0x_0003_5269_e12f_346e,
        0x_2bf9_24af_dbfd_36bf,
        0x_6d33_65b1_57f8_dece,
        0x_b53a_46da_b202_0b9e,
    );
    const LOG10_2: Self = Self(
        0,
        0,
        0,
        0,
        0x_0000_4d10_4d42_7de7,
        0x_fbcc_47c4_acd6_05be,
        0x_48bc_1356_9862_a1e8,
        0x_f9a4_c52f_3793_5be6,
    );
}
