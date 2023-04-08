use std::collections::HashMap;

use nohash_hasher::BuildNoHashHasher;

type ColTops = HashMap<usize, Vec<bool>, BuildNoHashHasher<usize>>;

#[derive(Default, Debug, Clone)]
struct Cave {
    col_tops: ColTops,
}

enum SandState {
    Blocked,
    Lost,
    Stopped,
}

impl Cave {
    fn from_paths(paths: impl IntoIterator<Item = RockPath>) -> Self {
        let mut col_tops = ColTops::default();
        let mut row_cnt = 0;
        fn min_max<T: Ord>(x: T, y: T) -> (T, T) {
            if x <= y {
                (x, y)
            } else {
                (y, x)
            }
        }
        for RockPath(vertices) in paths.into_iter() {
            assert!(vertices.len() > 1);
            let (col, h) = vertices[0];
            {
                let v = col_tops.entry(col).or_default();
                v.resize(v.len().max(h + 1), false);
                v[h] = true;
                row_cnt = row_cnt.max(h + 1);
            }
            for (&(col1, h1), &(col2, h2)) in vertices.iter().zip(&vertices[1..]) {
                if h1 == h2 {
                    let (beg, end) = min_max(col1, col2);
                    for c in beg..=end {
                        let v = col_tops.entry(c).or_default();
                        v.resize(row_cnt, false);
                        v[h1] = true;
                    }
                } else {
                    let (beg, end) = min_max(h1, h2);
                    row_cnt = row_cnt.max(end + 1);
                    let col = col_tops
                        .get_mut(&col1)
                        .expect("col1 should already been built");
                    col.resize(row_cnt, false);
                    for v in &mut col[beg..=end] {
                        *v = true;
                    }
                }
            }
        }
        col_tops.values_mut().for_each(|v| v.resize(row_cnt, false));
        Self { col_tops }
    }

    fn add_floor(mut self) -> Self {
        let row_cnt = self.col_tops[&500].len();
        let floor_level = row_cnt + 1;
        let leftmost = 500 - floor_level;
        let rightmost = 500 + floor_level;
        for i in leftmost..=rightmost {
            self.col_tops.entry(i).or_default();
        }
        for col in self.col_tops.values_mut() {
            col.resize(floor_level, false);
            col.push(true);
        }
        self
    }

    fn accumulate_2(&mut self) -> usize {
        let mut count = 0;
        loop {
            match self.drop_on(500, 0) {
                SandState::Lost => panic!("Should not happen"),
                SandState::Blocked => break,
                SandState::Stopped => count += 1,
            }
        }
        count
    }

    fn accumulate(&mut self) -> usize {
        let mut count = 0;
        loop {
            match self.drop_on(500, 0) {
                SandState::Lost => break,
                SandState::Blocked => panic!("Should not happen"),
                SandState::Stopped => count += 1,
            }
        }
        count
    }

    fn drop_on(&mut self, col_no: usize, depth: usize) -> SandState {
        let col = match self.col_tops.get_mut(&col_no) {
            Some(col) => col,
            None => return SandState::Lost,
        };
        let last_unblocked = match col[depth..]
            .iter()
            .take_while(|&x| *x == false)
            .count()
            .checked_sub(1)
        {
            Some(x) => depth + x,
            None => return SandState::Blocked, // blocked
        };
        match self.drop_on(col_no - 1, last_unblocked + 1) {
            SandState::Blocked => (),
            others => return others,
        }
        match self.drop_on(col_no + 1, last_unblocked + 1) {
            SandState::Blocked => (),
            others => return others,
        }
        self.col_tops.get_mut(&col_no).unwrap()[last_unblocked] = true;
        return SandState::Stopped;
    }
}

struct RockPath(Vec<(usize, usize)>);

#[derive(Debug)]
struct ParseError;

impl std::str::FromStr for RockPath {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inner = s
            .split("->")
            .map(|seg| {
                let (l, r) = seg.split_once(',').ok_or(ParseError)?;
                let l = l.trim().parse().map_err(|_| ParseError)?;
                let r = r.trim().parse().map_err(|_| ParseError)?;
                Ok((l, r))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(RockPath(inner))
    }
}

fn main() -> std::io::Result<()> {
    let input = std::io::read_to_string(std::io::stdin())?;
    let cave = Cave::from_paths(input.lines().map(|s| s.parse::<RockPath>().unwrap()));
    println!("q1: {}", cave.clone().accumulate());
    println!("q2: {}", cave.add_floor().accumulate_2());
    Ok(())
}
