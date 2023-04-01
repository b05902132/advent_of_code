#![allow(unused)]

type Height = u8;
type Position = (usize, usize);
type HeightMap = Vec<Vec<u8>>;

#[derive(thiserror::Error, Debug)]
enum ParseError {
    #[error("Bad input on row {row_no}")]
    BadRow { row_no: usize },
    #[error("The width of row {row_no} is different from its predecessors")]
    InconsistentRowSize { row_no: usize },
    #[error("Can't find the start point")]
    NoStart,
    #[error("There are multiple start points")]
    DuplicatedStart,
    #[error("Can't find the target")]
    NoTarget,
    #[error("There are multiple targets")]
    DuplicatedTarget,
}
#[derive(Debug)]
struct MapState {
    target: Position,
    start: Position,
    map: HeightMap,
    max_rows: usize,
    max_cols: usize,
}

impl TryFrom<&'_ str> for MapState {
    type Error = ParseError;

    fn try_from(value: &'_ str) -> Result<Self, Self::Error> {
        let mut lines = value
            .lines()
            .map(|s| s.as_bytes().iter().copied())
            .peekable();
        let mut target: Option<Position> = None;
        let mut start: Option<Position> = None;
        let max_cols = lines.peek().ok_or(ParseError::BadRow { row_no: 0 })?.len();
        #[derive(Default)]
        struct ParseState {
            target: Option<Position>,
            start: Option<Position>,
            row_cnt: usize,
            col_size: usize,
        }

        impl ParseState {
            fn parse_row(
                &mut self,
                row_no: usize,
                row: impl Iterator<Item = u8>,
            ) -> Result<Vec<u8>, ParseError> {
                let out = row
                    .enumerate()
                    .map(|(col_no, mut col)| {
                        if col == b'S' {
                            self.start = match self.start {
                                None => Some((row_no, col_no)),
                                Some(_) => return Err(ParseError::DuplicatedStart),
                            };
                            col = b'a';
                        } else if col == b'E' {
                            self.target = match self.target {
                                None => Some((row_no, col_no)),
                                Some(_) => return Err(ParseError::DuplicatedTarget),
                            };
                            col = b'z';
                        }
                        match col {
                            b'a'..=b'z' => Ok(col - b'a'),
                            _ => Err(ParseError::BadRow { row_no }),
                        }
                    })
                    .collect::<Result<Vec<u8>, ParseError>>()?;
                if out.len() != self.col_size {
                    return Err(ParseError::BadRow { row_no });
                }
                self.row_cnt += 1;
                Ok(out)
            }
            fn parse_map<T: Iterator<Item = u8>>(
                &mut self,
                input: impl Iterator<Item = T>,
            ) -> Result<HeightMap, ParseError> {
                input
                    .enumerate()
                    .map(|(row_no, row)| self.parse_row(row_no, row))
                    .collect()
            }
        }
        let mut parser = ParseState {
            col_size: max_cols,
            ..ParseState::default()
        };
        let map = parser.parse_map(lines)?;
        let start = parser.start.ok_or(ParseError::NoStart)?;
        let target = parser.target.ok_or(ParseError::NoTarget)?;
        Ok(Self {
            max_cols,
            max_rows: parser.row_cnt,
            map,
            target,
            start,
        })
    }
}

#[derive(Debug)]
struct BFSOutput {
    dest: (usize, usize),
    steps: usize,
}

impl MapState {
    fn bfs<E, F>(
        &self,
        start: (usize, usize),
        mut is_connected: E,
        mut is_end: F,
    ) -> Option<BFSOutput>
    where
        E: FnMut((usize, usize), (usize, usize)) -> bool,
        F: FnMut((usize, usize)) -> bool,
    {
        let mut pos = vec![start];
        let mut visited = vec![vec![false; self.max_cols]; self.max_rows];
        visited[start.0][start.1] = true;
        let mut steps = 0;
        while !pos.is_empty() {
            steps += 1;
            let mut new_pos = vec![];
            for (x, y) in pos {
                let h = self.map[x][y];
                let neighbors = [
                    x.checked_sub(1).map(|x| (x, y)),
                    y.checked_sub(1).map(|y| (x, y)),
                    ((x + 1) < self.max_rows).then_some((x + 1, y)),
                    ((y + 1) < self.max_cols).then_some((x, y + 1)),
                ];
                for (x2, y2) in neighbors
                    .into_iter()
                    .filter_map(|i| i)
                    .filter(|&(x2, y2)| is_connected((x, y), (x2, y2)))
                {
                    if visited[x2][y2] {
                        continue;
                    }
                    visited[x2][y2] = true;
                    let h2 = self.map[x2][y2];
                    if is_end((x2, y2)) {
                        return Some(BFSOutput {
                            steps,
                            dest: (x2, y2),
                        });
                    }
                    new_pos.push((x2, y2));
                }
            }
            pos = new_pos;
        }
        None
    }

    fn climb(&self) -> usize {
        self.bfs(
            self.start,
            |(x1, y1), (x2, y2)| self.map[x1][y1] + 1 >= self.map[x2][y2],
            |pos| pos == self.target,
        )
        .expect("BFS cannot reach the target")
        .steps
    }
    fn min_steps_to_lowest(&self) -> usize {
        self.bfs(
            self.target,
            |(x1, y1), (x2, y2)| self.map[x2][y2] + 1 >= self.map[x1][y1],
            |(x, y)| self.map[x][y] == 0,
        )
        .expect("BFS cannot reach the lowest point")
        .steps
    }
}

fn main() -> anyhow::Result<()> {
    let input = std::io::read_to_string(std::io::stdin())?;
    let state = MapState::try_from(input.as_str())?;
    println!("q1: {}", state.climb());
    println!("q2: {}", state.min_steps_to_lowest());
    Ok(())
}
