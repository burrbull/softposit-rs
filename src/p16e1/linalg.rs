use nalgebra::{
    base::{
        allocator::Allocator,
        constraint::{AreMultipliable, ShapeConstraint},
        storage::Storage,
    },
    DefaultAllocator, Dim, Matrix, MatrixMN,
};

use super::{P16E1, Q16E1};

impl<'b, R1: Dim, C1: Dim, R2: Dim, C2: Dim, SA, SB> crate::QuireDot<&'b Matrix<P16E1, R2, C2, SB>>
    for Matrix<P16E1, R1, C1, SA>
where
    SB: Storage<P16E1, R2, C2>,
    SA: Storage<P16E1, R1, C1>,
    DefaultAllocator: Allocator<P16E1, R1, C2>,
    ShapeConstraint: AreMultipliable<R1, C1, R2, C2>,
{
    type Output = MatrixMN<P16E1, R1, C2>;
    fn quire_dot(&self, rhs: &'b Matrix<P16E1, R2, C2, SB>) -> Self::Output {
        let mut out =
            unsafe { Matrix::new_uninitialized_generic(self.data.shape().0, rhs.data.shape().1) };
        for (i, mut row) in out.row_iter_mut().enumerate() {
            for (j, elem) in row.iter_mut().enumerate() {
                let mut quire = Q16E1::new();
                for (a, b) in self.row(i).iter().zip(rhs.column(j).iter()) {
                    quire += (*a, *b);
                }
                *elem = quire.into()
            }
        }
        out
    }
}
