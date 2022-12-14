use super::P8E0;

const P8E0_SQRT: [u8; 128] = [
    0, 8, 11, 14, 16, 18, 20, 21, 23, 24, 25, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 38,
    39, 40, 41, 42, 42, 43, 44, 45, 45, 46, 47, 47, 48, 49, 49, 50, 51, 51, 52, 52, 53, 54, 54, 55,
    55, 56, 57, 57, 58, 58, 59, 59, 60, 60, 61, 61, 62, 62, 63, 63, 64, 64, 65, 65, 66, 66, 67, 67,
    68, 68, 69, 69, 70, 70, 70, 71, 71, 72, 72, 72, 73, 73, 74, 74, 74, 75, 75, 75, 76, 76, 77, 77,
    77, 79, 80, 81, 83, 84, 85, 86, 87, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 100, 101,
    102, 103, 105, 108, 110, 112, 114, 115, 120,
];

impl P8E0 {
    #[inline]
    pub const fn sqrt(self) -> Self {
        let ui_a = self.to_bits();

        if ui_a >= 0x80 {
            Self::NAR
        } else {
            Self::from_bits(P8E0_SQRT[ui_a as usize])
        }
    }
}

#[test]
fn test_sqrt() {
    for i in i8::MIN..i8::MAX {
        let p_a = P8E0::new(i);
        let f_a = f64::from(p_a);
        let p = p_a.sqrt();
        let f = f_a.sqrt();
        assert_eq!(p, P8E0::from(f));
    }
}
