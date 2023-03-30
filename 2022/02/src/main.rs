use itertools::Itertools;
use std::io::{self, BufRead};

#[derive(Debug, Copy, Clone)]
enum Play {
    Rock,
    Paper,
    Scissor,
}

#[derive(Debug, Copy, Clone)]
enum GameOutcome {
    Win,
    Lose,
    Draw,
}

impl Play {
    fn score(&self) -> i32 {
        match *self {
            Play::Rock => 1,
            Play::Paper => 2,
            Play::Scissor => 3,
        }
    }
    fn get_outcome(&self, other: &Self) -> GameOutcome {
        use GameOutcome::*;
        match *self {
            Play::Rock => match *other {
                Play::Rock => Draw,
                Play::Paper => Lose,
                Play::Scissor => Win,
            },
            Play::Scissor => match *other {
                Play::Rock => Lose,
                Play::Paper => Win,
                Play::Scissor => Draw,
            },
            Play::Paper => match *other {
                Play::Rock => Win,
                Play::Paper => Draw,
                Play::Scissor => Lose,
            },
        }
    }
    fn find_winner(&self) -> Self {
        match *self {
            Self::Rock => Self::Paper,
            Self::Paper => Self::Scissor,
            Self::Scissor => Self::Rock,
        }
    }
    fn find_loser(&self) -> Self {
        match *self {
            Self::Rock => Self::Scissor,
            Self::Scissor => Self::Paper,
            Self::Paper => Self::Rock,
        }
    }
}

impl GameOutcome {
    fn score(&self) -> i32 {
        match *self {
            Self::Win => 6,
            Self::Lose => 0,
            Self::Draw => 3,
        }
    }
}

fn main() -> anyhow::Result<()> {
    let stdin = io::stdin();
    let stdin = stdin.lock();
    let games: Vec<(String, String)> = stdin
        .lines()
        .map(|line| -> anyhow::Result<(String, String)> {
            let line = line?;
            let foo @ (_, _) = line
                .split_whitespace()
                .map(str::to_string)
                .collect_tuple()
                .expect("Expect two words");
            Ok(foo)
        })
        .try_collect()?;
    println!("q1 result: {}", q1(&games));
    println!("q2 result: {}", q2(&games));
    Ok(())
}

fn q1(games: &[(String, String)]) -> i32 {
    games
        .iter()
        .map(|(opponent, player)| {
            let opponent = match opponent.as_str() {
                "A" => Play::Rock,
                "B" => Play::Paper,
                "C" => Play::Scissor,
                x => panic!("Unexpected input {x}"),
            };
            let player = match player.as_str() {
                "X" => Play::Rock,
                "Y" => Play::Paper,
                "Z" => Play::Scissor,
                x => panic!("Unexpected input {x}"),
            };
            player.score() + player.get_outcome(&opponent).score()
        })
        .sum()
}

fn q2(games: &[(String, String)]) -> i32 {
    games
        .iter()
        .map(|(opponent, expected_outcome)| {
            let opponent = match opponent.as_str() {
                "A" => Play::Rock,
                "B" => Play::Paper,
                "C" => Play::Scissor,
                x => panic!("Unexpected input {x}"),
            };
            let expected_outcome = match expected_outcome.as_str() {
                "X" => GameOutcome::Lose,
                "Y" => GameOutcome::Draw,
                "Z" => GameOutcome::Win,
                x => panic!("Unexpected input {x}"),
            };
            let my_play = match expected_outcome {
                GameOutcome::Lose => opponent.find_loser(),
                GameOutcome::Draw => opponent,
                GameOutcome::Win => opponent.find_winner(),
            };
            expected_outcome.score() + my_play.score()
        })
        .sum()
}
