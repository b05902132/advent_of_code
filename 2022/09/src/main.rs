use std::io::Read;

use itertools::Itertools;

#[derive(Copy, Clone, Debug)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Copy, Clone, Debug)]
struct Command(Direction, u8);

mod parse {
    use super::*;
    use chumsky::prelude::*;

    use chumsky::error::Simple;
    use chumsky::text::newline;

    fn command() -> impl Parser<char, Command, Error = Simple<char>> {
        choice((
            just('L').to(Direction::Left),
            just('R').to(Direction::Right),
            just('U').to(Direction::Up),
            just('D').to(Direction::Down),
        ))
        .padded()
        .then(text::int(10).try_map(|s: String, span| {
            s.parse::<u8>()
                .map_err(|e| Simple::custom(span, format!("{e}")))
        }))
        .map(|(dir, cnt)| Command(dir, cnt))
    }

    pub(crate) fn commands() -> impl Parser<char, Vec<Command>, Error = Simple<char>> {
        command()
            .separated_by(newline())
            .allow_trailing()
            .then_ignore(end())
    }
}

fn tail_pos(head_pos: (i32, i32), mut tail_pos: (i32, i32)) -> (i32, i32) {
    let delta = (
        head_pos.0 - tail_pos.0,
        head_pos.1 - tail_pos.1,
    );

    if delta.0.abs() > 1 || delta.1.abs() > 1 {
        tail_pos.0 += match delta.0.cmp(&0) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        };
        tail_pos.1 += match delta.1.cmp(&0) {
            std::cmp::Ordering::Less => -1,
            std::cmp::Ordering::Equal => 0,
            std::cmp::Ordering::Greater => 1,
        };
    }
    tail_pos

}

#[derive(Clone, Debug)]
struct Rope {
    head_pos: (i32, i32),
    tail_pos: Vec<(i32, i32)>,
}

impl Rope {
    pub(crate) fn new(tail_cnt: usize) -> Self {
        assert!(tail_cnt > 0);
        Self {
            head_pos: (0, 0),
            tail_pos: vec![(0, 0); tail_cnt],
        }
    }
    pub(crate) fn move_to(&mut self, dir: Direction) {
        match dir {
            Direction::Left => self.head_pos.0 -= 1,
            Direction::Right => self.head_pos.0 += 1,
            Direction::Up => self.head_pos.1 += 1,
            Direction::Down => self.head_pos.1 -= 1,
        }
        let mut prev_pos = self.head_pos;
        self.tail_pos.iter_mut()
            .for_each(|tail| {
                *tail = tail_pos(prev_pos, *tail);
                prev_pos = *tail;
            })

    }
}

use chumsky::error::Error;
use chumsky::prelude::Parser;
fn main() -> anyhow::Result<()> {
    let mut input = String::new();
    std::io::stdin().lock().read_to_string(&mut input)?;
    let input = parse::commands().parse(input).map_err(|e| {
        let error = e.into_iter().reduce(|lhs, rhs| lhs.merge(rhs)).unwrap();
        anyhow::anyhow!("{error}")
    })?;
    println!("q1: {}", q1(input.iter().cloned()));
    println!("q2: {}", q2(input.into_iter()));
    Ok(())
}

fn q1(input: impl Iterator<Item = Command>) -> usize {
    let mut rope = Rope::new(1);
    std::iter::once((0, 0))
        .chain(
            input
                .flat_map(|Command(dir, repeat)| std::iter::repeat(dir).take(repeat as usize))
                .map(|dir| {
                    rope.move_to(dir);
                    rope.tail_pos.last().copied().unwrap()
                }),
        )
        .unique()
        .count()
}

fn q2(input: impl Iterator<Item = Command>) -> usize {
    let mut rope = Rope::new(9);
    std::iter::once((0, 0))
        .chain(
            input
                .flat_map(|Command(dir, repeat)| std::iter::repeat(dir).take(repeat as usize))
                .map(|dir| {
                    rope.move_to(dir);
                    rope.tail_pos.last().copied().unwrap()
                }),
        )
        .unique()
        .count()
}
