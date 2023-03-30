use std::collections::HashSet;
use std::io::BufRead;

use itertools::Itertools;

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct Item(u8);

impl Item {
    fn score(&self) -> i32 {
        match self.0 {
            n @ b'a'..=b'z' => (n - b'a' + 1) as i32,
            n @ b'A'..=b'Z' => (n - b'A' + 27) as i32,
            _ => unreachable!("precondition violated"),
        }
    }
}

impl std::fmt::Debug for Item {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        fmt.debug_tuple("Item").field(&(self.0 as char)).finish()
    }
}

impl TryFrom<char> for Item {
    type Error = anyhow::Error;
    fn try_from(src: char) -> Result<Self, Self::Error> {
        let code: u8 = src.try_into()?;
        match code {
            b'a'..=b'z' | b'A'..=b'Z' => Ok(Item(code)),
            _ => {
                anyhow::bail!("unexpected character: {code}")
            }
        }
    }
}

struct Sack {
    items: HashSet<Item>,
    common: Item,
}

impl Sack {
    fn common(&self) -> Item {
        self.common
    }
    fn items(&self) -> &HashSet<Item> {
        &self.items
    }
}

impl<'a> TryFrom<&'a str> for Sack {
    type Error = anyhow::Error;
    fn try_from(src: &'a str) -> Result<Self, Self::Error> {
        let total_len = src.len();
        let mut items = src.chars().map(Item::try_from);
        let left: HashSet<_> = items.by_ref().take(total_len / 2).try_collect()?;
        let right: HashSet<_> = items.try_collect()?;
        let intersection = left.intersection(&right).copied().exactly_one().map_err(|_| anyhow::anyhow!("duplicated items"))?;
        let mut union = left;
        union.extend(right.into_iter());
        Ok(Self{
            items: union,
            common: intersection,
        })
    }
}

fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin().lock();
    let sacks: Vec<_> = stdin
        .lines()
        .map(|s| -> anyhow::Result<_> {
            let s = s?;
            Sack::try_from(s.as_str())
        })
        .try_collect()?;
    let total_common_score = sacks.iter().map(|s| s.common().score()).sum::<i32>();
    println!("q1: {total_common_score}");
    let badges: Vec<Item> = sacks
        .into_iter()
        .map(|s| s.items)
        .chunks(3)
        .into_iter()
        .map(|chunk| {
            let badge : HashSet<Item> = chunk.reduce(|left, right: HashSet<_>| {
                left.into_iter().filter(|i| right.contains(&i)).collect()
            }).expect("It's impossible to have empty chunks");
            badge.into_iter().exactly_one()
        })
        .try_collect()?;
    let total_group_score = badges.into_iter()
        .map(|s| s.score())
        .sum::<i32>();
    println!("q2: {total_group_score}");
    Ok(())
}
