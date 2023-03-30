use std::io::BufRead;
use std::str::FromStr;

use itertools::Itertools;

use nom::{
    IResult,
    character::complete::{digit1, char},
    combinator::{map, map_res, all_consuming},
    sequence::separated_pair,
};

fn parse_i32(s: &str) -> IResult<&str, i32> {
    map_res(
        digit1,
        |out: &str| out.parse::<i32>()
    )(s)
}

fn parse_assignment(input: &str) -> IResult<&str, Assignment> {
    map(
        separated_pair(parse_i32, char('-'), parse_i32),
        |(l, r)| Assignment(l, r)
    )(input)
}

fn parse_assignment_pair(input: &str) -> IResult<&str, AssignmentPair> {
    map(
        separated_pair(parse_assignment, char(','), parse_assignment),
        |(l, r)| AssignmentPair(l, r)
    )(input)
}

#[derive(Debug, Copy, Clone)]
struct Assignment(i32, i32);

#[derive(Debug, Copy, Clone)]
struct AssignmentPair(Assignment, Assignment);

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin().lock();
    let input: Vec<_> = stdin.lines()
        .map(|line| -> anyhow::Result<_> {
            let line = line?;
            let (_, this_pair) =
                all_consuming(parse_assignment_pair)(&line)
                .map_err(|e| e.to_owned())?;
            Ok(this_pair)
        }).try_collect()?;
    println!("q1: {}", q1(input.iter().copied()));
    println!("q2: {}", q2(input.into_iter()));
    Ok(())
}

fn q1(args: impl IntoIterator<Item=AssignmentPair>) -> usize {
    args.into_iter()
        .filter(|src| {
            let AssignmentPair(Assignment(lmin, lmax), Assignment(rmin, rmax)) = *src;
            (lmin <= rmin && lmax >= rmax) || (lmin >= rmin && lmax <= rmax)
        }).count()

}

fn q2(args: impl IntoIterator<Item=AssignmentPair>) -> usize {
    use std::cmp::{max, min};
    args.into_iter().filter(|&s| {
        let AssignmentPair(Assignment(lmin, lmax), Assignment(rmin, rmax)) = s;
        max(lmin, rmin) <= min(lmax, rmax)
    }).count()
}
