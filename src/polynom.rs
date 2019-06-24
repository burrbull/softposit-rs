pub(crate) mod poly {
    use crate::{AssociatedQuire, Quire};

    pub trait Poly<T>
    where
        Self: Copy
            + AssociatedQuire<Self>
            + core::convert::From<<Self as AssociatedQuire<Self>>::Q>
            + num_traits::One,
        Self::Q: core::ops::AddAssign<(Self, T)> + core::ops::AddAssign<(Self, Self)>,
        T: Copy,
    {
        #[inline]
        fn poly1k(x: Self, c0: T, c1: T) -> Self {
            let mut q = Self::Q::init(); // QCLR.S
            q += (Self::one(), c1); // QMADD.S
            q += (x, c0); // QMADD.S
            q.into() // QROUND.S
        }
        #[inline]
        fn poly2k(x: Self, x2: Self, c0: Self, c: &[T]) -> Self {
            let mut q = Self::Q::init();
            q += (Self::one(), c[1]);
            q += (x, c[0]);
            q += (x2, c0);
            q.into()
        }
        #[inline]
        fn poly2kt(x: Self, x2: Self, c0: T, c: &[T]) -> Self {
            let mut q = Self::Q::init();
            q += (Self::one(), c[1]);
            q += (x, c[0]);
            q += (x2, c0);
            q.into()
        }
        #[inline]
        fn poly3kt(x: Self, x2: Self, x3: Self, c0: T, c: &[T]) -> Self {
            let mut q = Self::Q::init();
            q += (Self::one(), c[2]);
            q += (x, c[1]);
            q += (x2, c[0]);
            q += (x3, c0);
            q.into()
        }
        #[inline]
        fn poly3k(x: Self, x2: Self, x3: Self, c0: Self, c: &[T]) -> Self {
            let mut q = Self::Q::init();
            q += (Self::one(), c[2]);
            q += (x, c[1]);
            q += (x2, c[0]);
            q += (x3, c0);
            q.into()
        }
        #[inline]
        fn poly4k(x: Self, x2: Self, x3: Self, x4: Self, c0: Self, c: &[T]) -> Self {
            let mut q = Self::Q::init();
            q += (Self::one(), c[3]);
            q += (x, c[2]);
            q += (x2, c[1]);
            q += (x3, c[0]);
            q += (x4, c0);
            q.into()
        }
        #[inline]
        fn poly4kt(x: Self, x2: Self, x3: Self, x4: Self, c0: T, c: &[T]) -> Self {
            let mut q = Self::Q::init();
            q += (Self::one(), c[3]);
            q += (x, c[2]);
            q += (x2, c[1]);
            q += (x3, c[0]);
            q += (x4, c0);
            q.into()
        }
        #[inline]
        fn poly5k(x: Self, x2: Self, x3: Self, p: T, c: &[T]) -> Self {
            let p = Self::poly2kt(x, x2, p, &c[..2]);
            Self::poly3k(x, x2, x3, p, &c[2..])
        }
        #[inline]
        fn poly6k(x: Self, x2: Self, x3: Self, p: T, c: &[T]) -> Self {
            let p = Self::poly3kt(x, x2, x3, p, &c[..3]);
            Self::poly3k(x, x2, x3, p, &c[3..])
        }
        #[inline]
        fn poly7k(x: Self, x2: Self, x3: Self, x4: Self, p: T, c: &[T]) -> Self {
            let p = Self::poly3kt(x, x2, x3, p, &c[..3]);
            Self::poly4k(x, x2, x3, x4, p, &c[3..])
        }
        #[inline]
        fn poly8k(x: Self, x2: Self, x3: Self, x4: Self, p: T, c: &[T]) -> Self {
            let p = Self::poly4kt(x, x2, x3, x4, p, &c[..4]);
            Self::poly4k(x, x2, x3, x4, p, &c[4..])
        }
        #[inline]
        fn poly9k(x: Self, x2: Self, x3: Self, x4: Self, p: T, c: &[T]) -> Self {
            let p = Self::poly5k(x, x2, x3, p, &c[..5]);
            Self::poly4k(x, x2, x3, x4, p, &c[5..])
        }
        #[inline]
        fn poly10k(x: Self, x2: Self, x3: Self, x4: Self, p: T, c: &[T]) -> Self {
            let p = Self::poly6k(x, x2, x3, p, &c[..6]);
            Self::poly4k(x, x2, x3, x4, p, &c[6..])
        }
        #[inline]
        fn poly11k(x: Self, x2: Self, x3: Self, x4: Self, p: T, c: &[T]) -> Self {
            let p = Self::poly7k(x, x2, x3, x4, p, &c[..7]);
            Self::poly4k(x, x2, x3, x4, p, &c[7..])
        }
        #[inline]
        fn poly12k(x: Self, x2: Self, x3: Self, x4: Self, p: T, c: &[T]) -> Self {
            let p = Self::poly8k(x, x2, x3, x4, p, &c[..8]);
            Self::poly4k(x, x2, x3, x4, p, &c[8..])
        }
        #[inline]
        fn poly13k(x: Self, x2: Self, x3: Self, x4: Self, p: T, c: &[T]) -> Self {
            let p = Self::poly9k(x, x2, x3, x4, p, &c[..9]);
            Self::poly4k(x, x2, x3, x4, p, &c[9..])
        }
        #[inline]
        fn poly14k(x: Self, x2: Self, x3: Self, x4: Self, p: T, c: &[T]) -> Self {
            let p = Self::poly10k(x, x2, x3, x4, p, &c[..10]);
            Self::poly4k(x, x2, x3, x4, p, &c[10..])
        }
        #[inline]
        fn poly15k(x: Self, x2: Self, x3: Self, x4: Self, p: T, c: &[T]) -> Self {
            let p = Self::poly11k(x, x2, x3, x4, p, &c[..11]);
            Self::poly4k(x, x2, x3, x4, p, &c[11..])
        }
        #[inline]
        fn poly16k(x: Self, x2: Self, x3: Self, x4: Self, p: T, c: &[T]) -> Self {
            let p = Self::poly12k(x, x2, x3, x4, p, &c[..12]);
            Self::poly4k(x, x2, x3, x4, p, &c[12..])
        }
        #[inline]
        fn poly17k(x: Self, x2: Self, x3: Self, x4: Self, p: T, c: &[T]) -> Self {
            let p = Self::poly13k(x, x2, x3, x4, p, &c[..13]);
            Self::poly4k(x, x2, x3, x4, p, &c[13..])
        }
        #[inline]
        fn poly18k(x: Self, x2: Self, x3: Self, x4: Self, p: T, c: &[T]) -> Self {
            let p = Self::poly14k(x, x2, x3, x4, p, &c[..14]);
            Self::poly4k(x, x2, x3, x4, p, &c[14..])
        }
    }
}

pub trait Polynom<T>
where
    Self: poly::Poly<T> + core::ops::Mul<Output = Self> + Copy,
    Self::Q: core::ops::AddAssign<(Self, T)> + core::ops::AddAssign<(Self, Self)>,
    T: Copy,
{
    // Quire1 = 1
    #[inline]
    fn poly1(self, c: &[T; 2]) -> Self {
        Self::poly1k(self, c[0], c[1])
    }
    // Quire1 + (x2=x*x) = 2
    #[inline]
    fn poly2(self, c: &[T; 3]) -> Self {
        let x2 = self * self;
        Self::poly2kt(self, x2, c[0], &c[1..])
    }
    // Quire1 + (x2, x3=x2*x) = 3, faster
    #[inline]
    fn poly3(self, c: &[T; 4]) -> Self {
        let x2 = self * self;
        let x3 = x2 * self;
        Self::poly3kt(self, x2, x3, c[0], &c[1..])
    }
    // Quire1 + (x2, x3, x4=x2*x2) = 4, faster
    #[inline]
    fn poly4(self, c: &[T; 5]) -> Self {
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly4kt(self, x2, x3, x4, c[0], &c[1..])
    }
    // Quire2 + (x2, x3) = 4
    #[inline]
    fn poly5(self, c: &[T; 6]) -> Self {
        let x2 = self * self;
        let x3 = x2 * self;
        Self::poly5k(self, x2, x3, c[0], &c[1..])
    }
    // Quire2 + (x2, x3) = 4
    #[inline]
    fn poly6(self, c: &[T; 7]) -> Self {
        let x2 = self * self;
        let x3 = x2 * self;
        Self::poly6k(self, x2, x3, c[0], &c[1..])
    }
    // Quire2 + (x2, x3, x4) = 5
    #[inline]
    fn poly7(self, c: &[T; 8]) -> Self {
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly7k(self, x2, x3, x4, c[0], &c[1..])
    }
    // Quire2 + (x2, x3, x4) = 5
    #[inline]
    fn poly8(self, c: &[T; 9]) -> Self {
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly8k(self, x2, x3, x4, c[0], &c[1..])
    }
    // Quire3 + (x2, x3, x4) = 6
    #[inline]
    fn poly9(self, c: &[T; 10]) -> Self {
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly9k(self, x2, x3, x4, c[0], &c[1..])
    }
    // Quire3 + (x2, x3, x4) = 6
    #[inline]
    fn poly10(self, c: &[T; 11]) -> Self {
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly10k(self, x2, x3, x4, c[0], &c[1..])
    }
    // Quire3 + (x2, x3, x4) = 6
    #[inline]
    fn poly11(self, c: &[T; 12]) -> Self {
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly11k(self, x2, x3, x4, c[0], &c[1..])
    }
    // Quire3 + (x2, x3, x4) = 6
    #[inline]
    fn poly12(self, c: &[T; 13]) -> Self {
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly12k(self, x2, x3, x4, c[0], &c[1..])
    }
    // Quire4 + (x2, x3, x4) = 7
    #[inline]
    fn poly13(self, c: &[T; 14]) -> Self {
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly13k(self, x2, x3, x4, c[0], &c[1..])
    }
    // Quire4 + (x2, x3, x4) = 7
    #[inline]
    fn poly14(self, c: &[T; 15]) -> Self {
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly14k(self, x2, x3, x4, c[0], &c[1..])
    }
    // Quire4 + (x2, x3, x4) = 7
    #[inline]
    fn poly15(self, c: &[T; 16]) -> Self {
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly15k(self, x2, x3, x4, c[0], &c[1..])
    }
    // Quire4 + (x2, x3, x4) = 7
    #[inline]
    fn poly16(self, c: &[T; 17]) -> Self {
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly16k(self, x2, x3, x4, c[0], &c[1..])
    }
    // Quire5 + (x2, x3, x4) = 8
    #[inline]
    fn poly17(self, c: &[T; 18]) -> Self {
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly17k(self, x2, x3, x4, c[0], &c[1..])
    }
    // Quire5 + (x2, x3, x4) = 8
    #[inline]
    fn poly18(self, c: &[T; 19]) -> Self {
        let x2 = self * self;
        let x3 = x2 * self;
        let x4 = x2 * x2;
        Self::poly18k(self, x2, x3, x4, c[0], &c[1..])
    }
    // Quire2 + (x2) = 3, more accurate
    #[inline]
    fn poly3a(self, c: &[T; 4]) -> Self {
        let x2 = self * self;
        let p = Self::poly1k(self, c[0], c[1]);
        Self::poly2k(self, x2, p, &c[2..])
    }
    // Quire2 + (x2) = 3, more accurate
    #[inline]
    fn poly4a(self, c: &[T; 5]) -> Self {
        let x2 = self * self;
        let p = Self::poly2kt(self, x2, c[0], &c[1..3]);
        Self::poly2k(self, x2, p, &c[3..])
    }
}
