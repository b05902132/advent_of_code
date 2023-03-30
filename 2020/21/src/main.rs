use itertools::Itertools;

use aoc20_21::q1;
use std::io::prelude::*;
fn main() {
    let mut stdin = std::io::stdin();
    let mut s = String::new();
    stdin.read_to_string(&mut s).unwrap();
    let (q1_res, q2_res) = q1(s.lines());
    println!("q1: {}", q1_res);
    let q2_res = q2_res.values().join(",");
    println!("q2: {}", q2_res);
}
