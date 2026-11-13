// examples/dump_seq.rs
use fcr_rsa::fib::generate;
use num_bigint::BigInt;
use num_traits::FromPrimitive;

fn main() {
    let q = BigInt::from_u64(1_000_000_007).unwrap();
    let seed = b"fcr-rsa-demo-seed";
    let seq = generate(10, seed, &q);
    println!("Rust seq: {:?}", seq.iter().map(|x| x.to_string()).collect::<Vec<_>>());
}