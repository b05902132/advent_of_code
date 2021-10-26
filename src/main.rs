use std::io::{stdin, Read};

use itertools::Itertools;

use aoc20_20::*;

const SEA_MONSTER: &str = r"                  # 
#    ##    ##    ###
 #  #  #  #  #  #   ";
fn main() {
    use std::ops::Deref;
    let stdin = stdin();
    let mut input = String::new();
    stdin.lock().read_to_string(&mut input).unwrap();
    let sea_map = SeaMap::from_str(&input);
    let out1: u64 = sea_map.corners().map(|i| i.id).product();
    println!("output1: {}", out1);
    let sea_monster = SEA_MONSTER.lines().map(|s| {
        s.replace(' ', ".")
    }).collect_vec();
    let sea_monster = aoc20_20::strs_to_image(sea_monster.iter().map(Deref::deref));
    let mut image = sea_map.image.clone();
    remove_subimage(&mut image, &sea_monster);
    let out2 = image
        .iter()
        .map(|v| v.iter())
        .flatten()
        .copied()
        .filter(|&x| x)
        .count();
    println!("output2: {}", out2);
}
