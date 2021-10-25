use lazy_static::lazy_static;
use regex::Regex;
use std::collections::{HashMap, HashSet};
#[derive(Clone, Debug)]
pub struct Tile {
    id: u64,
    borders: [u64; 4],
    image: Vec<String>,
    neighbors: HashSet<u64>,
}
impl Tile {
    fn chars_to_u64(s: impl Iterator<Item = char>) -> u64 {
        s.fold(0, |mut accu, c| {
            accu *= 2;
            if c == '#' {
                accu += 1;
            } else {
                assert!(c == '.');
            }
            accu
        })
    }
    fn new(s: &[&str]) -> Self {
        lazy_static! {
            static ref TITLE_RE: Regex = Regex::new(r"^\s*Tile (\d*):\s*$").unwrap();
        };
        let title_line = s[0];
        let cap = &TITLE_RE.captures(title_line).unwrap()[1];
        let id = cap.parse::<u64>().unwrap();
        let image: Vec<_> = s[1..].iter().map(|s| s.to_string()).collect();
        let border1 = Self::chars_to_u64(image[0].chars());
        let border2 = Self::chars_to_u64(image.iter().map(|x| x.chars().next().unwrap()));
        let border3 = Self::chars_to_u64(image.last().unwrap().chars().rev());
        let border4 =
            Self::chars_to_u64(image.iter().map(|x| x.chars().rev().next().unwrap()).rev());
        let border = [border1, border2, border3, border4];
        Self {
            id,
            image,
            borders: border,
            neighbors: HashSet::new(),
        }
    }
    fn borders<'a>(&'a self) -> impl Iterator<Item = u64> + 'a {
        self.borders
            .iter()
            .copied()
            .flat_map(|b| [b, complement(b)])
    }
}

pub fn input_to_tiles<'a>(s: &'a str) -> impl Iterator<Item = Tile> + 'a {
    fn sep_empty_line<'a>(
        mut it: impl Iterator<Item = &'a str>,
    ) -> impl Iterator<Item = Vec<&'a str>> {
        return std::iter::from_fn(move || {
            let mut v = vec![];
            while let Some(s) = it.next() {
                if s.trim().is_empty() {
                    return Some(v);
                }
                v.push(s);
            }
            if v.is_empty() {
                None
            } else {
                Some(v)
            }
        });
    }
    sep_empty_line(s.lines()).map(|v| Tile::new(&v))
}

fn complement(mut i: u64) -> u64 {
    let mut out = 0;
    for _ in 0..10 {
        out <<= 1;
        out += i % 2;
        i /= 2;
    }
    out
}

pub struct SeaMap {
    tile_by_id: HashMap<u64, Tile>,
    image: Vec<Vec<bool>>,
}

impl SeaMap {
    pub fn new(tile: impl Iterator<Item = Tile>) -> Self {
        let mut tile_by_id: HashMap<_, _> = tile.map(|t| (t.id, t)).collect();
        connect_tiles(&mut tile_by_id);
        Self {
            tile_by_id,
            image: Default::default(), //TODO
        }
    }
    pub fn tiles(&self) -> impl Iterator<Item = &Tile> {
        self.tile_by_id.values()
    }
    pub fn tiles_mut(&mut self) -> impl Iterator<Item = &mut Tile> {
        self.tile_by_id.values_mut()
    }
}

fn connect_tiles(tile_by_id: &mut HashMap<u64, Tile>) {
    let mut border_to_id = {
        let mut s: HashMap<_, _> = HashMap::new();
        for (id, border) in tile_by_id
            .iter()
            .flat_map(|(id, t)| t.borders().map(|x| (*id, x)))
        {
            s.entry(border).or_insert(HashSet::new()).insert(id);
        }
        s
    };
    loop {
        fn add_neighbor(tile_map: &mut HashMap<u64, Tile>, lid: u64, rid: u64) -> usize {
            let l = tile_map.get_mut(&lid).unwrap();
            l.neighbors.insert(rid);
            return l.neighbors.len();
        }
        let mut done = true;
        let mut border_to_remove = HashSet::new();
        let mut tile_id_to_remove = HashSet::new();
        for (&border, ids) in border_to_id.iter().filter(|(_border, s)| s.len() == 2) {
            done = false;
            if border_to_remove.contains(&complement(border)) {
                continue;
            }
            let mut id_iter = ids.iter();
            let lid = *id_iter.next().unwrap();
            let rid = *id_iter.next().unwrap();
            let l_neighbor_count = add_neighbor(tile_by_id, lid, rid);
            let r_neighbor_count = add_neighbor(tile_by_id, rid, lid);
            border_to_remove.extend([border, complement(border)]);
            if l_neighbor_count == 4 {
                tile_id_to_remove.insert(lid);
            }
            if r_neighbor_count == 4 {
                tile_id_to_remove.insert(rid);
            }
        }
        border_to_remove.into_iter().for_each(|b| {
            border_to_id.remove(&b).unwrap();
        });
        tile_id_to_remove.into_iter().for_each(|tid| {
            for border in tile_by_id[&tid].borders() {
                if let Some(s) = border_to_id.get_mut(&border) {
                    s.remove(&tid);
                }
            }
        });
        if done {
            break;
        }
    }
}

pub fn solve_1(s: &str) -> u64 {
    let sea_map = SeaMap::new(input_to_tiles(s));
    sea_map
        .tiles()
        .filter_map(|t| (t.neighbors.len() == 2).then(|| t.id))
        .product()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        assert_eq!(solve_1(SAMPLE_IN), 20899048083289)
    }

    #[test]
    fn test_complement() {
        for i in 0..1024 {
            assert_eq!(
                i,
                complement(complement(i)),
                "test failed while computing complement {}",
                i
            )
        }
    }
    const SAMPLE_IN: &str = r"Tile 2311:
..##.#..#.
##..#.....
#...##..#.
####.#...#
##.##.###.
##...#.###
.#.#.#..##
..#....#..
###...#.#.
..###..###

Tile 1951:
#.##...##.
#.####...#
.....#..##
#...######
.##.#....#
.###.#####
###.##.##.
.###....#.
..#.#..#.#
#...##.#..

Tile 1171:
####...##.
#..##.#..#
##.#..#.#.
.###.####.
..###.####
.##....##.
.#...####.
#.##.####.
####..#...
.....##...

Tile 1427:
###.##.#..
.#..#.##..
.#.##.#..#
#.#.#.##.#
....#...##
...##..##.
...#.#####
.#.####.#.
..#..###.#
..##.#..#.

Tile 1489:
##.#.#....
..##...#..
.##..##...
..#...#...
#####...#.
#..#.#.#.#
...#.#.#..
##.#...##.
..##.##.##
###.##.#..

Tile 2473:
#....####.
#..#.##...
#.##..#...
######.#.#
.#...#.#.#
.#########
.###.#..#.
########.#
##...##.#.
..###.#.#.

Tile 2971:
..#.#....#
#...###...
#.#.###...
##.##..#..
.#####..##
.#..####.#
#..#.#..#.
..####.###
..#.#.###.
...#.#.#.#

Tile 2729:
...#.#.#.#
####.#....
..#.#.....
....#..#.#
.##..##.#.
.#.####...
####.#.#..
##.####...
##..#.##..
#.##...##.

Tile 3079:
#.#.#####.
.#..######
..#.......
######....
####.#..#.
.#...#.##.
#.#####.##
..#.###...
..#.......
..#.###...";
}
