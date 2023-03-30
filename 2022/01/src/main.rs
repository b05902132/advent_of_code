use aoc_util::read_stdin_by_section;
use std::io::{self};

fn main() -> std::io::Result<()> {
    let calories: io::Result<Vec<i32>> = read_stdin_by_section()
        .try_fold(vec![], |mut v: Vec<i32>, s: io::Result<Vec<String>>| {
            let s = s?;
            v.push(s.into_iter().fold(0i32, |accu, s| { accu + s.parse::<i32>().unwrap() }));
            Ok(v)
        });
    let mut calories = calories?;
    let m = calories.iter().max().unwrap();
    println!("{m}");
    calories.sort();
    let n = calories.into_iter().rev().take(3).sum::<i32>();
    println!("{n}");
    Ok(())
}
