use aoc20_20::*;
use std::io::{stdin, Read};
fn main() {
    let stdin = stdin();
    let mut input = String::new();
    stdin.lock().read_to_string(&mut input).unwrap();
    let sea_map = SeaMap::from_str(&input);
    let out1 : u64 = sea_map.corners().map(|i| i.id).product();
    println!("output1: {}", out1);
}
