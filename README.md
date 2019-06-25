[![crates.io](https://img.shields.io/crates/v/softposit.svg)](https://crates.io/crates/softposit)
[![crates.io](https://img.shields.io/crates/d/softposit.svg)](https://crates.io/crates/softposit)

## `softposit-rs`

Rust port of [SoftPosit] repository.

[SoftPosit]: https://gitlab.com/cerlane/SoftPosit

Such types are supported now:

* P32E2 or P32: 32-bit posit with exponent size ES=2;
* P16E1 or P16: 16-bit posit with ES=1;
* P8E0 or P8: 8-bit posit without exponent bits.

## Examples

### Convert from f64, Add

```
use softposit::P32;

fn main() {
    let a = P32::from(12.3);
    let b = P32::from(154.);
    let c = a + b;
    println!("c = {0} or c = {0:?}", c);
}
```

## Quire

Each Posit type has correspondent Quire type (Q32, Q16, Q8).
These types support `AddAssign` and `SubAssign` operations.
For example:

```
use softposit::{P32, Q32};

// c == 12.3*0.4 - 6.3*8.4 == -48;
fn main() {
    let mut q = Q32::init();
    q += (P32::from(12.3), P32::from(0.4));
    q -= (P32::from(6.3), P32::from(8.4));
    let c = q.to_posit();
    println!("c = {0} or c = {0:?}", c);
}
```

## Math

Math functions, like trigonomentic, are partially implemented for P32E2 type.
For complex computations use `num::Complex` type.

## Linear algebra

You can also use matrix operations, defined in [nalgebra](https://crates.io/crates/nalgebra) crate
by using `linalg` feature:

```
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
```

```
cargo build --features="linalg,rand"
```

## Benchmarking

```
cargo bench
```

| Type  | Operation | i3-2310M @ 2.10GHz | i5-3470 @ 3.20GHz |
| ----- |:---------:|:------------------:|:-----------------:|
| P32E2 |    +      | 47 MPOPS           | 82 MPOPS          |
|       |    -      | 45 MPOPS           | 83 MPOPS          |
|       |    *      | 56 MPOPS           | 99 MPOPS          |
|       |    /      | 25 MPOPS           | 42 MPOPS          |
|       |   √⎺⎺     | 48 MPOPS           | 92 MPOPS          |
|       |  round    | 89 MPOPS           | 159 MPOPS         |
