use softposit::{P32, Q32};

fn main() {
    // c == 12.3*0.4 - 6.3*8.4 == -48;
    let mut q = Q32::new();
    q += (P32::from(12.3), P32::from(0.4));
    q -= (P32::from(6.3), P32::from(8.4));
    let c = q.roundp();
    println!("c = {0} or c = {0:?}", c);
}
