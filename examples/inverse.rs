use softposit::P32;

use nalgebra::{Dynamic, Matrix, VecStorage};
type DMatrix = Matrix<P32, Dynamic, Dynamic, VecStorage<P32, Dynamic, Dynamic>>;

fn main() {
    let a = DMatrix::new_random(5, 5);
    println!("Matrix A = {}", a);
    let decomp = &a.clone().lu();
    if let Some(c) = decomp.try_inverse() {
        println!("Inverse matrix Aˉ¹ = {}", c);
        println!("Check Identity");
        let ones = a * c;
        println!("A × Aˉ¹ = {}", ones);
    } else {
        println!("Matrix A can't be inverted");
    }
}
