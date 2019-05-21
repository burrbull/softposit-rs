pub(crate) mod poly {
    use crate::{AssociatedQuire, Quire};

    pub trait Poly
    where
        Self:
            Copy + AssociatedQuire<Self> + core::convert::From<<Self as AssociatedQuire<Self>>::Q>,
        Self::Q: core::ops::AddAssign<(Self, Self)>,
    {
        #[inline]
        fn poly1k(x: Self, c0: Self, c1: Self) -> Self {
            let mut q = Self::Q::from_posit(c1); // QCLR.S + QMADD.S
            q += (c0, x); // QMADD.S
            q.into() // QROUND.S
        }
        #[inline]
        fn poly2k(x: Self, x2: Self, c0: Self, c: &[Self]) -> Self {
            let mut q = Self::Q::from_posit(c[1]);
            q += (c[0], x);
            q += (c0, x2);
            q.into()
        }
        #[inline]
        fn poly3k(x: Self, x2: Self, x3: Self, c0: Self, c: &[Self]) -> Self {
            let mut q = Self::Q::from_posit(c[2]);
            q += (c[1], x);
            q += (c[0], x2);
            q += (c0, x3);
            q.into()
        }
        #[inline]
        fn poly4k(x: Self, x2: Self, x3: Self, x4: Self, c0: Self, c: &[Self]) -> Self {
            let mut q = Self::Q::from_posit(c[3]);
            q += (c[2], x);
            q += (c[1], x2);
            q += (c[0], x3);
            q += (c0, x4);
            q.into()
        }
        #[inline]
        fn poly5k(x: Self, x2: Self, x3: Self, p: Self, c: &[Self]) -> Self {
            let p = Self::poly2k(x, x2, p, &c[..2]);
            Self::poly3k(x, x2, x3, p, &c[2..])
        }
        #[inline]
        fn poly6k(x: Self, x2: Self, x3: Self, p: Self, c: &[Self]) -> Self {
            let p = Self::poly3k(x, x2, x3, p, &c[..3]);
            Self::poly3k(x, x2, x3, p, &c[3..])
        }
        #[inline]
        fn poly7k(x: Self, x2: Self, x3: Self, x4: Self, p: Self, c: &[Self]) -> Self {
            let p = Self::poly3k(x, x2, x3, p, &c[..3]);
            Self::poly4k(x, x2, x3, x4, p, &c[3..])
        }
        #[inline]
        fn poly8k(x: Self, x2: Self, x3: Self, x4: Self, p: Self, c: &[Self]) -> Self {
            let p = Self::poly4k(x, x2, x3, x4, p, &c[..4]);
            Self::poly4k(x, x2, x3, x4, p, &c[4..])
        }
        #[inline]
        fn poly9k(x: Self, x2: Self, x3: Self, x4: Self, p: Self, c: &[Self]) -> Self {
            let p = Self::poly5k(x, x2, x3, p, &c[..5]);
            Self::poly4k(x, x2, x3, x4, p, &c[5..])
        }
        #[inline]
        fn poly10k(x: Self, x2: Self, x3: Self, x4: Self, p: Self, c: &[Self]) -> Self {
            let p = Self::poly6k(x, x2, x3, p, &c[..6]);
            Self::poly4k(x, x2, x3, x4, p, &c[6..])
        }
        #[inline]
        fn poly11k(x: Self, x2: Self, x3: Self, x4: Self, p: Self, c: &[Self]) -> Self {
            let p = Self::poly7k(x, x2, x3, x4, p, &c[..7]);
            Self::poly4k(x, x2, x3, x4, p, &c[7..])
        }
        #[inline]
        fn poly12k(x: Self, x2: Self, x3: Self, x4: Self, p: Self, c: &[Self]) -> Self {
            let p = Self::poly8k(x, x2, x3, x4, p, &c[..8]);
            Self::poly4k(x, x2, x3, x4, p, &c[8..])
        }
        #[inline]
        fn poly13k(x: Self, x2: Self, x3: Self, x4: Self, p: Self, c: &[Self]) -> Self {
            let p = Self::poly9k(x, x2, x3, x4, p, &c[..9]);
            Self::poly4k(x, x2, x3, x4, p, &c[9..])
        }
        #[inline]
        fn poly14k(x: Self, x2: Self, x3: Self, x4: Self, p: Self, c: &[Self]) -> Self {
            let p = Self::poly10k(x, x2, x3, x4, p, &c[..10]);
            Self::poly4k(x, x2, x3, x4, p, &c[10..])
        }
        #[inline]
        fn poly15k(x: Self, x2: Self, x3: Self, x4: Self, p: Self, c: &[Self]) -> Self {
            let p = Self::poly11k(x, x2, x3, x4, p, &c[..11]);
            Self::poly4k(x, x2, x3, x4, p, &c[11..])
        }
        #[inline]
        fn poly16k(x: Self, x2: Self, x3: Self, x4: Self, p: Self, c: &[Self]) -> Self {
            let p = Self::poly12k(x, x2, x3, x4, p, &c[..12]);
            Self::poly4k(x, x2, x3, x4, p, &c[12..])
        }
        #[inline]
        fn poly17k(x: Self, x2: Self, x3: Self, x4: Self, p: Self, c: &[Self]) -> Self {
            let p = Self::poly13k(x, x2, x3, x4, p, &c[..13]);
            Self::poly4k(x, x2, x3, x4, p, &c[13..])
        }
        #[inline]
        fn poly18k(x: Self, x2: Self, x3: Self, x4: Self, p: Self, c: &[Self]) -> Self {
            let p = Self::poly14k(x, x2, x3, x4, p, &c[..14]);
            Self::poly4k(x, x2, x3, x4, p, &c[14..])
        }
    }
}

pub trait Polynom
where
    Self: poly::Poly + core::ops::Mul<Output = Self> + Copy,
    Self::Q: core::ops::AddAssign<(Self, Self)>,
{
    // Quire1 = 1
    #[inline]
    fn poly1(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 2);
        Self::poly1k(self, c[0], c[1])
    }
    // Quire1 + (x2=x*x) = 2
    #[inline]
    fn poly2(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 3);
        let x2 = self * self;
        Self::poly2k(self, x2, c[0], &c[1..])
    }
    // Quire1 + (x2, x3=x2*x) = 3, faster
    #[inline]
    fn poly3(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 4);
        let x2 = self * self;
        let x3 = x2 * self;
        Self::poly3k(self, x2, x3, c[0], &c[1..])
    }
    // Quire1 + (x2, x3, x4=x2*x2) = 4, faster
    #[inline]
    fn poly4(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 5);
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly4k(self, x2, x3, x4, c[0], &c[1..])
    }
    // Quire2 + (x2, x3) = 4
    #[inline]
    fn poly5(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 6);
        let x2 = self * self;
        let x3 = x2 * self;
        Self::poly5k(self, x2, x3, c[0], &c[1..])
    }
    // Quire2 + (x2, x3) = 4
    #[inline]
    fn poly6(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 7);
        let x2 = self * self;
        let x3 = x2 * self;
        Self::poly6k(self, x2, x3, c[0], &c[1..])
    }
    // Quire2 + (x2, x3, x4) = 5
    #[inline]
    fn poly7(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 8);
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly7k(self, x2, x3, x4, c[0], &c[1..])
    }
    // Quire2 + (x2, x3, x4) = 5
    #[inline]
    fn poly8(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 9);
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly8k(self, x2, x3, x4, c[0], &c[1..])
    }
    // Quire3 + (x2, x3, x4) = 6
    #[inline]
    fn poly9(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 10);
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly9k(self, x2, x3, x4, c[0], &c[1..])
    }
    // Quire3 + (x2, x3, x4) = 6
    #[inline]
    fn poly10(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 11);
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly10k(self, x2, x3, x4, c[0], &c[1..])
    }
    // Quire3 + (x2, x3, x4) = 6
    #[inline]
    fn poly11(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 12);
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly11k(self, x2, x3, x4, c[0], &c[1..])
    }
    // Quire3 + (x2, x3, x4) = 6
    #[inline]
    fn poly12(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 13);
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly12k(self, x2, x3, x4, c[0], &c[1..])
    }
    // Quire4 + (x2, x3, x4) = 7
    #[inline]
    fn poly13(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 14);
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly13k(self, x2, x3, x4, c[0], &c[1..])
    }
    // Quire4 + (x2, x3, x4) = 7
    #[inline]
    fn poly14(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 15);
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly14k(self, x2, x3, x4, c[0], &c[1..])
    }
    // Quire4 + (x2, x3, x4) = 7
    #[inline]
    fn poly15(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 16);
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly15k(self, x2, x3, x4, c[0], &c[1..])
    }
    // Quire4 + (x2, x3, x4) = 7
    #[inline]
    fn poly16(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 17);
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly16k(self, x2, x3, x4, c[0], &c[1..])
    }
    // Quire5 + (x2, x3, x4) = 8
    #[inline]
    fn poly17(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 18);
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly17k(self, x2, x3, x4, c[0], &c[1..])
    }
    // Quire5 + (x2, x3, x4) = 8
    #[inline]
    fn poly18(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 19);
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly18k(self, x2, x3, x4, c[0], &c[1..])
    }
    // Quire2 + (x2) = 3, more accurate
    #[inline]
    fn poly3a(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 4);
        let x2 = self * self;
        let p = Self::poly1k(self, c[0], c[1]);
        Self::poly2k(self, x2, p, &c[2..])
    }
    // Quire2 + (x2) = 3, more accurate
    #[inline]
    fn poly4a(self, c: &[Self]) -> Self {
        assert_eq!(c.len(), 5);
        let x2 = self * self;
        let p = Self::poly2k(self, x2, c[0], &c[1..3]);
        Self::poly2k(self, x2, p, &c[3..])
    }
}
