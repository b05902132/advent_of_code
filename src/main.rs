use aoc20_20;
use std::io::{stdin, Read};
fn main() {
    let stdin = stdin();
    let mut input = String::new();
    stdin.lock().read_to_string(&mut input).unwrap();
    let out = aoc20_20::solve_1(&input);
    println!("{}", out);
}
