use nalgebra::{
    base::{
        allocator::Allocator,
        constraint::{AreMultipliable, ShapeConstraint},
        storage::Storage,
        Scalar,
    },
    DefaultAllocator, Dim, Matrix, MatrixMN,
};

use crate::AssociatedQuire;
use crate::Quire;

impl<'b, N, R1: Dim, C1: Dim, R2: Dim, C2: Dim, SA, SB> crate::QuireDot<&'b Matrix<N, R2, C2, SB>>
    for Matrix<N, R1, C1, SA>
where
    N: Scalar + AssociatedQuire<N>,
    SB: Storage<N, R2, C2>,
    SA: Storage<N, R1, C1>,
    DefaultAllocator: Allocator<N, R1, C2>,
    ShapeConstraint: AreMultipliable<R1, C1, R2, C2>,
{
    type Output = MatrixMN<N, R1, C2>;
    fn quire_dot(&self, rhs: &'b Matrix<N, R2, C2, SB>) -> Self::Output {
        let mut out =
            unsafe { Matrix::new_uninitialized_generic(self.data.shape().0, rhs.data.shape().1) };
        for (i, mut row) in out.row_iter_mut().enumerate() {
            for (j, elem) in row.iter_mut().enumerate() {
                let mut quire = N::Q::init();
                for (a, b) in self.row(i).iter().zip(rhs.column(j).iter()) {
                    quire.add_product(*a, *b);
                }
                *elem = quire.to_posit()
            }
        }
        out
    }
}
