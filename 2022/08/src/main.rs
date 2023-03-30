use std::io::BufRead;

use itertools::Itertools;

fn read_input(x: impl BufRead) -> anyhow::Result<Vec<Vec<u8>>> {
    x.lines()
        .map(|line| -> anyhow::Result<Vec<u8>> {
            line.map_err(|e| e.into())
                .and_then(|line| -> anyhow::Result<Vec<u8>> {
                    line.chars()
                        .map(|c| -> anyhow::Result<u8> {
                            c.to_digit(10)
                                .and_then(|c| c.try_into().ok())
                                .ok_or_else(|| anyhow::anyhow!("Can't parse {c} as u8"))
                        })
                        .try_collect()
                })
        })
        .try_collect()
}

fn to_visible(input: &Vec<Vec<u8>>) -> Vec<Vec<bool>> {
    let height = input.len();
    assert!(height > 0);
    let width = input[0].len();
    assert!(width > 0);
    assert!(input.iter().all(|row| row.len() == width));
    let mut output = vec![vec![false; width]; height];
    // row
    output[0].iter_mut().for_each(|c| *c = true);
    output[height - 1].iter_mut().for_each(|c| *c = true);
    fn tag<'a>(mut src: impl Iterator<Item = (u8, &'a mut bool)>) {
        let (mut tree_height, outer) = src.next().unwrap();
        *outer = true;
        for (t, b) in src {
            if t > tree_height {
                tree_height = t;
                *b = true;
            }
        }
    }

    for (row, row_out) in input[0..height - 1]
        .iter()
        .zip(output[0..height - 1].iter_mut())
    {
        tag(row.iter().copied().zip(row_out.iter_mut()));
        tag(row.iter().copied().rev().zip(row_out.iter_mut().rev()))
    }

    fn iter_for_col<'a>(
        x: &'a Vec<Vec<u8>>,
        out: &'a mut Vec<Vec<bool>>,
        i: usize,
    ) -> impl Iterator<Item = (u8, &'a mut bool)> + std::iter::DoubleEndedIterator {
        x.iter()
            .map(move |v| v[i])
            .zip(out.iter_mut().map(move |v| &mut v[i]))
    }
    for i in 1..width - 1 {
        tag(iter_for_col(&input, &mut output, i));
        tag(iter_for_col(&input, &mut output, i).rev());
    }
    output
}

fn main() -> anyhow::Result<()> {
    let input = read_input(std::io::stdin().lock())?;
    let visibles = to_visible(&input);
    let q1_res = visibles.iter().flatten().filter(|&&x| x).count();
    println!("q1: {q1_res}");
    let q2_res = q2(input);
    println!("q2: {q2_res}");
    Ok(())
}

fn q2(input: Vec<Vec<u8>>) -> usize {
    fn scenic_score(input: impl Iterator<Item = u8>, h: u8) -> usize {
        let mut out = 0;
        for x in input {
            out += 1;
            if x >= h {
                break;
            }
        }
        out
    }
    let scenic_score_for_idx = |i: usize, j: usize| -> usize {
        let h = input[i][j];
        let left = scenic_score(input[i][..j].iter().copied().rev(), h);
        let right = scenic_score(input[i][j + 1..].iter().copied(), h);
        let up = scenic_score(input[..i].iter().rev().map(|v| v[j]), h);
        let down = scenic_score(input[i + 1..].iter().map(|v| v[j]), h);
        left * right * up * down
    };
    let height = input.len();
    let width = input[0].len();
    (1..height - 1)
        .cartesian_product(1..width - 1)
        .map(|(i, j)| scenic_score_for_idx(i, j))
        .max().unwrap_or(0)
}
