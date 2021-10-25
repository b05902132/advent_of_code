use lazy_static::lazy_static;
use regex::Regex;
use std::collections::{HashMap, HashSet};

type Pixel = bool;
type Image = Vec<Vec<Pixel>>;
type TileId = u64;
type Border = u16;

fn char_to_pixel(c: char) -> Pixel {
    match c {
        '#' => true,
        '.' => false,
        c => panic!("Unknown character {}", c),
    }
}

fn str_to_image_row(s: &str) -> Vec<Pixel> {
    s.chars().map(char_to_pixel).collect()
}

fn strs_to_image<'a>(s: impl IntoIterator<Item = &'a str>) -> Image {
    s.into_iter().map(str_to_image_row).collect()
}

fn bools_to_border(it: impl IntoIterator<Item = bool>) -> Border {
    it.into_iter().fold(0, |mut accu, b| {
        accu *= 2;
        if b {
            accu += 1;
        }
        accu
    })
}

#[derive(Clone, Debug)]
pub struct Tile {
    id: TileId,
    borders: [Border; 4],
    neighbors: Vec<TileId>,
    image: Image,
}
impl Tile {
    fn new(s: &[&str]) -> Self {
        lazy_static! {
            static ref TITLE_RE: Regex = Regex::new(r"^\s*Tile (\d*):\s*$").unwrap();
        };
        let title_line = s[0];
        let cap = &TITLE_RE.captures(title_line).unwrap()[1];
        let id = cap.parse::<u64>().unwrap();
        let image: Image = strs_to_image(s[1..].iter().copied());
        let border1 = bools_to_border(image[0].iter().copied());
        let border2 = bools_to_border(image.iter().map(|x| *x.last().unwrap()));
        let border3 = bools_to_border(image.last().unwrap().iter().copied().rev());
        let border4 = bools_to_border(image.iter().map(|x| x[0]).rev());
        let border = [border1, border2, border3, border4];
        Self {
            id,
            image,
            borders: border,
            neighbors: Default::default(),
        }
    }
    fn borders_candidates<'a>(&'a self) -> impl Iterator<Item = Border> + 'a {
        self.borders
            .iter()
            .copied()
            .flat_map(|b| [b, complement(b)])
    }
    fn flip(&mut self) {
        self.image.reverse();
        self.borders.swap(0, 2);
        self.borders.iter_mut().for_each(|i| *i = complement(*i));
    }
    fn rotate(&mut self) {
        // rotate clockwise
        use std::{iter, mem};
        let mut rows: Vec<_> = mem::take(&mut self.image)
            .into_iter()
            .rev()
            .map(|row| row.into_iter())
            .collect();
        self.image = iter::from_fn(move || rows.iter_mut().map(|r| r.next()).collect()).collect();
        let new_borders = [
            self.borders[3],
            self.borders[0],
            self.borders[1],
            self.borders[2],
        ];
        self.borders = new_borders;
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

fn complement(mut i: Border) -> Border {
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
            .flat_map(|(id, t)| t.borders_candidates().map(|x| (*id, x)))
        {
            s.entry(border).or_insert(HashSet::new()).insert(id);
        }
        s
    };
    loop {
        fn add_neighbor(tile_map: &mut HashMap<u64, Tile>, lid: u64, rid: u64) -> usize {
            let l = tile_map.get_mut(&lid).unwrap();
            l.neighbors.push(rid);
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
            for border in tile_by_id[&tid].borders_candidates() {
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
    const SAMPLE_TILE: &str = r"Tile 2311:
..##.#..#.
##..#.....
#...##..#.
####.#...#
##.##.###.
##...#.###
.#.#.#..##
..#....#..
###...#.#.
..###..###";
    fn make_sample_tile() -> Tile {
        let lines: Vec<_> = SAMPLE_TILE.lines().collect();
        Tile::new(&lines)
    }

    #[test]
    fn test_border_seq() {
        let tile = make_sample_tile();
        assert_eq!(tile.borders[0], 0b0011010010);
        assert_eq!(tile.borders[1], 0b0001011001);
        assert_eq!(tile.borders[2], 0b1110011100);
        assert_eq!(tile.borders[3], 0b0100111110);
    }

    #[test]
    fn test_border_flip() {
        let mut tile = make_sample_tile();
        tile.flip();
        assert_eq!(tile.borders[0], 0b0011100111);
        assert_eq!(tile.borders[1], 0b1001101000);
        assert_eq!(tile.borders[2], 0b0100101100);
        assert_eq!(tile.borders[3], 0b0111110010);
    }
    #[test]
    fn test_border_rotate() {
        let mut tile = make_sample_tile();
        tile.rotate();
        assert_eq!(tile.borders[0], 0b0100111110);
        assert_eq!(tile.borders[1], 0b0011010010);
        assert_eq!(tile.borders[2], 0b0001011001);
        assert_eq!(tile.borders[3], 0b1110011100);
    }
    fn make_small_tile() -> Tile {
        const TILE: &str = r"Tile 0:
..#.
#.##
####
#...
";
        let lines: Vec<_> = TILE.lines().collect();
        Tile::new(&lines)
    }
    fn make_small_tile_rotated() -> Tile {
        const TILE: &str = r"Tile 0:
###.
.#..
.###
.##.
";
        let lines: Vec<_> = TILE.lines().collect();
        Tile::new(&lines)
    }
    #[test]
    fn test_rotate_image() {
        let mut tile = make_small_tile();
        tile.rotate();
        let target_tile = make_small_tile_rotated();
        assert_eq!(&tile.image, &target_tile.image)
    }
    #[test]
    fn test_rotate_border_consistent() {
        let mut tile = make_small_tile();
        tile.rotate();
        let target_tile = make_small_tile_rotated();
        assert_eq!(&tile.borders, &target_tile.borders)
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
