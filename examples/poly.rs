use softposit::{Polynom, P32};

fn p(x: f64) -> P32 {
    x.into()
}

fn gorner(x: P32, coefs: &[P32]) -> P32 {
    let mut s = coefs[0];
    for c in coefs.iter().skip(1) {
        s = s * x + *c;
    }
    s
}

fn main() {
    // x = 1.1000000014901161
    // 5.199999988079071×x^5−12.100000023841858×x^4−3.2999999970197678×x^3+0.6000000014901161×x^2+15 = 1.99274189651072
    let c = [p(5.2), p(-12.1), p(-3.3), p(0.6), p(0.), p(15.)];
    let x = p(1.1);
    let ex = 1.99274189651072;
    let y1 = x.poly5(&c);
    let y2 = gorner(x, &c);
    println!("Expected = {} ({})", ex, p(ex).to_bits());
    println!("Polynom = {} ({})", y1, y1.to_bits());
    println!("Gorner = {} ({})", y2, y2.to_bits());
}
