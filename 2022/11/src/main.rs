use std::io::Read;


#[derive(Debug, Copy, Clone)]
struct Item(u32);

#[derive(Debug, Copy, Clone)]
enum Op {
    Add(u32),
    AddSelf,
    Mul(u32),
    MulSelf,
}

impl Op {
    fn apply(&self, val: u32) -> u32 {
        match *self {
            Op::Add(i) => val + i,
            Op::AddSelf => val + val,
            Op::Mul(i) => val * i,
            Op::MulSelf => val * val,
        }

    }
}

#[derive(Debug, Copy, Clone)]
struct Action {
    div: u32,
    if_true: usize,
    if_false: usize,
}

#[derive(Debug, Clone)]
struct Monkey {
    items: Vec<Item>,
    operation: Op,
    action: Action,
}

mod parse {
    use super::*;
    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::{char, line_ending, space0, space1, u32},
        combinator::{map, value},
        multi::separated_list1,
        sequence::{delimited, pair, preceded, terminated, tuple},
        IResult,
    };

    fn monkey(s: &str) -> IResult<&str, (usize, Monkey)> {
        fn monkey_number(s: &str) -> IResult<&str, usize> {
            map(
                delimited(tag("Monkey "), u32, preceded(char(':'), line_ending)),
                |n| n as usize,
            )(s)
        }

        fn items(s: &str) -> IResult<&str, Vec<Item>> {
            delimited(
                tag("  Starting items: "),
                separated_list1(terminated(char(','), space1), map(u32, Item)),
                line_ending,
            )(s)
        }
        fn op(s: &str) -> IResult<&str, Op> {
            #[derive(Copy, Clone)]
            enum Operator {
                Mul,
                Add,
            }
            fn operator(s: &str) -> IResult<&str, Operator> {
                terminated(
                    alt((
                        value(Operator::Mul, char('*')),
                        value(Operator::Add, char('+')),
                    )),
                    space0,
                )(s)
            }

            #[derive(Copy, Clone)]
            enum Rhs {
                Old,
                Num(u32),
            }

            fn rhs(s: &str) -> IResult<&str, Rhs> {
                terminated(
                    alt((value(Rhs::Old, tag("old")), map(u32, Rhs::Num))),
                    space0,
                )(s)
            }

            delimited(
                tag("  Operation: new = old "),
                map(pair(operator, rhs), |(op, rhs)| match (op, rhs) {
                    (Operator::Add, Rhs::Old) => Op::AddSelf,
                    (Operator::Add, Rhs::Num(i)) => Op::Add(i),
                    (Operator::Mul, Rhs::Old) => Op::MulSelf,
                    (Operator::Mul, Rhs::Num(i)) => Op::Mul(i),
                }),
                line_ending,
            )(s)
        }
        fn action(s: &str) -> IResult<&str, Action> {
            fn divisible(s: &str) -> IResult<&str, u32> {
                delimited(tag("  Test: divisible by "), u32, line_ending)(s)
            }
            fn if_true(s: &str) -> IResult<&str, usize> {
                map(
                    delimited(tag("    If true: throw to monkey "), u32, line_ending),
                    |n| n as usize,
                )(s)
            }
            fn if_false(s: &str) -> IResult<&str, usize> {
                map(
                    preceded(tag("    If false: throw to monkey "), u32),
                    |n| n as usize,
                )(s)
            }
            map(tuple((divisible, if_true, if_false)), |(div, t, f)| {
                Action {
                    div,
                    if_true: t,
                    if_false: f,
                }
            })(s)
        }
        map(
            tuple((monkey_number, items, op, action)),
            |(n, items, operation, action)| {
                (
                    n,
                    Monkey {
                        items,
                        operation,
                        action,
                    },
                )
            },
        )(s)
    }
    #[allow(unused)]
    pub(crate) fn monkeys(s: &str) -> IResult<&str, Vec<Monkey>> {
        map(separated_list1(pair(line_ending, line_ending), monkey), |monkeys| {
            monkeys
                .into_iter()
                .enumerate()
                .map(|(i, (j, m))| {
                    assert!(i == j);
                    m
                })
                .collect()
        })(s)
    }
}

fn largest_2<T: Ord>(vals: &[T]) -> Option<(&T, Option<&T>)> {
    fn divide_conquer<T: Ord>(vals: &[T]) -> (&T, Option<&T>) {
        match vals.len() {
            0 => unreachable!(),
            1 => (&vals[0], None),
            2 => {
                if vals[0] >= vals[1] { (&vals[0], Some(&vals[1])) } else {(&vals[1], Some(&vals[0]))}
            }
            _ => {
                let (left, right) = vals.split_at(vals.len() / 2);
                let (l1, l2) = divide_conquer(left);
                let (r1, r2) = divide_conquer(right);
                if l1 >= r1 {
                    (l1, Some(l2.map_or(r1, |l2| l2.max(r1))))

                } else {
                    (r1, Some(r2.map_or(l1, |r2| r2.max(l1))))
                }
            }
        }
    }
    (!vals.is_empty()).then(|| divide_conquer(vals))
}

fn q1(mut monkeys: Vec<Monkey>) -> usize {
    let mut inspect_count = vec![0; monkeys.len()];
    for _ in 0..20 {
        for i in 0..monkeys.len() {
            let Monkey{action, operation, ..} = monkeys[i];
            let items = std::mem::take(&mut monkeys[i].items);
            inspect_count[i] += items.len();
            for item in items {
                let Item(mut worry) = item;
                worry = operation.apply(worry);
                worry /= 3;
                let item = Item(worry);
                let Action{div, if_true, if_false} = action;
                if worry % div == 0 {
                    monkeys[if_true].items.push(item)
                } else {
                    monkeys[if_false].items.push(item);
                }
            }
        }
    }
    inspect_count.sort();
    inspect_count.into_iter().rev().take(2).product()

}

fn q2(mut monkeys: Vec<Monkey>) -> usize {
    let mut inspect_count = vec![0; monkeys.len()];
    let divisor = monkeys.iter().map(|m| m.action.div).reduce(num::integer::lcm).unwrap();
    for _ in 0..10000 {
        for i in 0..monkeys.len() {
            let Monkey{action, operation, ..} = monkeys[i];
            let items = std::mem::take(&mut monkeys[i].items);
            inspect_count[i] += items.len();
            for item in items {
                let Item(mut worry) = item;
                worry = operation.apply(worry);
                worry %= divisor;
                let item = Item(worry);
                let Action{div, if_true, if_false} = action;
                if worry % div == 0 {
                    monkeys[if_true].items.push(item)
                } else {
                    monkeys[if_false].items.push(item);
                }
            }
        }
    }
    inspect_count.sort();
    inspect_count.into_iter().rev().take(2).product()
}

fn main() -> anyhow::Result<()>{
    use nom::character::complete::line_ending;
    let mut input = String::new();
    std::io::stdin().lock().read_to_string(&mut input)?;
    let mut parser = nom::sequence::terminated(parse::monkeys, line_ending);
    let (_, input) = parser(input.as_str()).map_err(|e| e.to_owned())?;
    println!("q1: {}", q1(input.clone()));
    println!("q2: {}", q2(input));
    Ok(())
}
