use std::io::{Read, stdin};
use aoc20;
fn main() {
    let stdin = stdin();
    let mut input = String::new();
    stdin.lock().read_to_string(&mut input).unwrap();
    let out = aoc20::solve(&input);
    println!("{}", out);
}
