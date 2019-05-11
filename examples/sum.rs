use softposit::P32;

fn main() {
    let a = P32::from(12.3);
    let b = P32::from(154.);
    let c = a + b;
    println!("c = {}", c);
}
