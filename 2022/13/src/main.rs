#[derive(Clone, Eq, PartialEq)]
enum Signal {
    Integer(i32),
    List(Vec<Signal>),
}

impl Ord for Signal {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::iter::once;
        use Signal::*;
        match (self, other) {
            (Integer(i), Integer(j)) => i.cmp(j),
            (List(v), List(u)) => v.cmp(u),
            (&Integer(i), List(u)) => once(&Signal::Integer(i)).cmp(u.iter()),
            (List(v), &Integer(j)) => v.iter().cmp(once(&Signal::Integer(j))),
        }
    }
}

impl PartialOrd for Signal {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}

mod parse {
    use super::*;
    use nom::{
        branch::alt,
        character::complete::{char, i32, multispace0, newline},
        combinator::map,
        multi::separated_list0,
        sequence::{delimited, pair, separated_pair, terminated},
        Finish, IResult,
    };

    type ParseResult<'a, T> = Result<T, nom::error::Error<String>>;

    fn integer(s: &str) -> IResult<&str, Signal> {
        map(i32, Signal::Integer)(s)
    }

    fn list(s: &str) -> IResult<&str, Signal> {
        map(
            delimited(char('['), separated_list0(char(','), signal), char(']')),
            Signal::List,
        )(s)
    }

    fn signal(s: &str) -> IResult<&str, Signal> {
        alt((integer, list))(s)
    }

    pub(super) fn parse_signal(s: &str) -> ParseResult<Signal> {
        signal(s)
            .map_err(|e| e.to_owned())
            .finish()
            .map(|(_, out)| out)
    }
    pub(super) fn parse(s: &str) -> ParseResult<Vec<(Signal, Signal)>> {
        terminated(
            separated_list0(
                pair(newline, newline),
                separated_pair(signal, newline, signal),
            ),
            multispace0,
        )(s)
        .map_err(|e| e.to_owned())
        .finish()
        .map(|(_, out)| out)
    }
}

mod my_parse {
    use super::Signal;

    #[derive(Debug, thiserror::Error)]
    #[error("Parse Error")]
    pub struct ParseError;

    pub type ParseResult<T> = Result<T, ParseError>;

    pub(super) fn parse_signal(s: &str) -> ParseResult<Signal> {
        todo!()
    }

    pub(super) fn parse(s: &str) -> ParseResult<Vec<(Signal, Signal)>> {
        todo!()
    }
}

fn main() -> anyhow::Result<()> {
    let input = std::io::read_to_string(std::io::stdin())?;
    let input = parse::parse(input.as_str())?;

    println!(
        "q1: {}",
        input
            .iter()
            .zip(1..)
            .filter_map(|((x, y), i)| (x <= y).then_some(i))
            .sum::<usize>()
    );
    let sentinels = [parse::parse_signal("[[2]]")?, parse::parse_signal("[[6]]")?];
    let mut input = input
        .into_iter()
        .flat_map(|(x, y)| [x, y])
        .chain(sentinels.clone())
        .collect::<Vec<_>>();
    input.sort();
    println!(
        "q2: {}",
        input
            .into_iter()
            .zip(1..)
            .filter_map(|(v, i)| sentinels.contains(&v).then_some(i))
            .product::<usize>()
    );
    Ok(())
}
