use itertools::Itertools;
use std::io::Read;
fn find_substring_without_duplicates(s: &str, len: usize) -> Option<usize> {
    s.as_bytes()
        .windows(len)
        .enumerate()
        .find(|&(_, w)| w.iter().all_unique())
        .map(|(x, _)| x + len)
}

fn main() -> anyhow::Result<()> {
    let mut s = String::new();
    std::io::stdin().read_to_string(&mut s)?;
    println!(
        "q1: {}",
        find_substring_without_duplicates(&s, 4)
            .ok_or_else(|| anyhow::anyhow!("Can't find answer"))?
    );
    println!(
        "q2: {}",
        find_substring_without_duplicates(&s, 14)
            .ok_or_else(|| anyhow::anyhow!("Can't find answer"))?
    );
    Ok(())
}
