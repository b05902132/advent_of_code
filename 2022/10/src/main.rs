use std::{collections::HashSet, io::Read};

#[derive(Copy, Clone, Debug)]
enum Instruction {
    Noop,
    Addx(i32),
}

mod parse {
    use super::*;
    use chumsky::error::Simple;
    use chumsky::prelude::*;
    use chumsky::text::newline;
    fn noop() -> impl Parser<char, Instruction, Error = Simple<char>> {
        just("noop").to(Instruction::Noop)
    }
    fn int() -> impl Parser<char, i32, Error = Simple<char>> {
        just('-')
            .to(())
            .or_not()
            .map(|neg| neg.is_some())
            .then(chumsky::text::int(10))
            .try_map(|(is_neg, s), span| {
                s.parse::<i32>()
                    .map(|n| if is_neg { -n } else { n })
                    .map_err(|e| Simple::custom(span, format!("{e}")))
            })
    }
    fn addx() -> impl Parser<char, Instruction, Error = Simple<char>> {
        just("addx")
            .then(just(' '))
            .ignore_then(int())
            .map(Instruction::Addx)
    }
    pub(crate) fn instructions() -> impl Parser<char, Vec<Instruction>, Error = Simple<char>> {
        noop().or(addx()).separated_by(newline()).allow_trailing().then_ignore(end())
    }
}

#[derive(Clone, Debug)]
struct CPU {
    register: i32,
}

impl CPU {
    fn new() -> Self {
        Self { register: 1 }
    }

    fn run_instruction(&mut self, ins: Instruction) -> (u8, i32) {
        match ins {
            Instruction::Addx(i) => {
                self.register += i;
                (2, self.register)
            }
            Instruction::Noop => (1, self.register),
        }
    }

    fn simulate(
        self,
        instructions: impl IntoIterator<Item = Instruction>,
    ) -> impl Iterator<Item = i32> {
        let mut cpu = self;
        let instructions = instructions.into_iter();
        std::iter::once(1).chain(instructions.flat_map(move |ins| {
            let old_register = cpu.register;
            let (duration, new_register) = cpu.run_instruction(ins);
            std::iter::repeat(old_register)
                .take(duration as usize - 1)
                .chain(std::iter::once(new_register))
        }))
    }
}

fn q1(instructions: impl IntoIterator<Item = Instruction>) -> i32 {
    let cpu = CPU::new();
    let target_cycle = [20, 60, 100, 140, 180, 220]
        .into_iter()
        .collect::<HashSet<_>>();
    let mut out = 0;
    cpu.simulate(instructions)
        .enumerate()
        .map(|(i, x)| (i + 1, x))
        .for_each(|(cycle, reg_value)| {
            if target_cycle.contains(&cycle) {
                out += (cycle as i32) * reg_value;
            }
        });
    out
}

fn q2(instructions: impl IntoIterator<Item = Instruction>) {
    let mut display = vec![vec!['.'; 40]; 6];
    let mut execs = CPU::new().simulate(instructions);
    for row in &mut display {
        for (i, sprite_center) in execs.by_ref().take(40).enumerate() {
            if i as i32 >= sprite_center - 1 && i as i32 <= sprite_center + 1 {
                row[i] = '#';
            }
        }
    }
    for row in display {
        println!("{}", row.into_iter().collect::<String>())
    }
}

fn main() -> anyhow::Result<()> {
    use chumsky::Parser;
    let mut input = String::new();
    std::io::stdin().lock().read_to_string(&mut input)?;
    let instructions = parse::instructions()
        .parse(input)
        .map_err(|e| anyhow::anyhow!("{}", &e[0]))?;
    println!("q1: {}", q1(instructions.clone()));
    q2(instructions);
    Ok(())
}
