use std::collections::{HashMap, HashSet};
use std::iter;
use std::mem;

use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;

pub type Pixel = bool;
pub type Image = Vec<Vec<Pixel>>;
pub type TileId = u64;
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

pub fn strs_to_image<'a>(s: impl IntoIterator<Item = &'a str>) -> Image {
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

pub fn flip_image(image: &mut Image) {
    image.reverse()
}

pub fn rotate_image(image: &mut Image) {
    let mut rows = mem::take(image);
    rows.reverse();
    let new_image = iter::from_fn({
        let mut row_fronts: Vec<_> = rows.into_iter().map(|r| r.into_iter()).collect();
        move || row_fronts.iter_mut().map(|it| it.next()).collect()
    })
    .collect();
    *image = new_image;
}

pub fn remove_subimage(image: &mut Image, target: &Image) {
    fn size_check(image: &Image) {
        assert!(image.iter().map(|x| x.len()).all_equal())
    }
    size_check(image);
    size_check(target);
    fn run(image: &mut Image, target: &Image) {
        let ih = image.len();
        if ih == 0 {
            return;
        }
        let iw = image[0].len();
        let th = target.len();
        if th == 0 {
            return;
        }
        let tw = target[0].len();
        let true_loc = || {
            (0..th)
                .cartesian_product(0..tw)
                .filter(|&(p, q)| target[p][q])
        };
        for (i, j) in (0..ih - th).cartesian_product(0..iw - tw) {
            if true_loc().map(|(p, q)| image[i + p][j + q]).all(|x| x) {
                true_loc().for_each(|(p, q)| image[i + p][j + q] = false);
            }
        }
    }
    for _ in 0..4 {
        run(image, target);
        rotate_image(image);
    }
    flip_image(image);
    for _ in 0..4 {
        run(image, target);
        rotate_image(image);
    }
}
pub fn image_string(image: &Image) -> String {
    let mut out = String::new();
    for row in image.iter() {
        let row_chars = row.iter().copied().map(|p| if p { '#' } else { '.' });
        out.extend(row_chars);
        out.push('\n');
    }
    out
}

#[derive(Clone, Debug)]
pub struct Tile {
    pub id: TileId,
    borders: [Border; 4],
    neighbors: [Option<TileId>; 4],
    pub image: Image,
}
impl Tile {
    fn new(s: &[&str]) -> Self {
        lazy_static! {
            static ref TITLE_RE: Regex = Regex::new(r"^\s*Tile (\d*):\s*$").unwrap();
        };
        let title_line = s[0];
        let cap = &TITLE_RE.captures(title_line).unwrap()[1];
        let id = cap.parse::<TileId>().unwrap();
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

    fn borders_candidates(&self) -> impl Iterator<Item = Border> {
        self.borders
            .clone()
            .into_iter()
            .flat_map(|b| [b, complement(b)])
    }

    fn flip(&mut self) {
        flip_image(&mut self.image);
        self.borders.swap(0, 2);
        self.neighbors.swap(0, 2);
        self.borders.iter_mut().for_each(|i| *i = complement(*i));
    }

    fn rotate(&mut self) {
        // rotate clockwise
        rotate_image(&mut self.image);
        self.borders.rotate_right(1);
        self.neighbors.rotate_right(1);
    }

    fn neighbors(&self) -> impl Iterator<Item = TileId> {
        self.neighbors.clone().into_iter().filter_map(|x| x)
    }

    fn match_adj_border(&mut self, direction: usize, target_border: Border) {
        let my_target_border = complement(target_border);
        if !self.borders.contains(&my_target_border) {
            self.flip();
        }
        assert!(self.borders.contains(&my_target_border));
        while self.borders[direction] != my_target_border {
            self.rotate();
        }
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
    tiles: Vec<Vec<Tile>>,
    pub image: Vec<Vec<bool>>,
}

impl SeaMap {
    pub fn from_str(s: &str) -> Self {
        Self::new(input_to_tiles(s))
    }
    pub fn new(tile: impl Iterator<Item = Tile>) -> Self {
        let mut tile_by_id: HashMap<_, _> = tile.map(|t| (t.id, t)).collect();
        connect_tiles(&mut tile_by_id);
        fn build_row(first_tile: Tile, tile_by_id: &mut HashMap<TileId, Tile>) -> Vec<Tile> {
            let mut row = vec![];
            let mut last_tile = first_tile;
            while let (Some(next_id), last_border) = (last_tile.neighbors[1], last_tile.borders[1])
            {
                let mut next_tile = tile_by_id.remove(&next_id).unwrap();
                let this_border = complement(last_border);
                if !next_tile.borders.contains(&this_border) {
                    next_tile.flip();
                }
                while next_tile.borders[3] != this_border {
                    next_tile.rotate();
                }
                row.push(last_tile);
                last_tile = next_tile;
            }
            row.push(last_tile);
            row
        }
        let lu_id = tile_by_id
            .iter()
            .find_map(|(id, t)| (t.neighbors().count() == 2).then(|| *id))
            .unwrap();
        let mut row_begin = tile_by_id.remove(&lu_id).unwrap();
        while row_begin.neighbors[0].is_some() || row_begin.neighbors[3].is_some() {
            row_begin.rotate();
        }
        let mut tiles = vec![];
        loop {
            let row = build_row(row_begin, &mut tile_by_id);
            let next_row_begin_id = row[0].neighbors[2];
            let prev_row_begin_down_border = row[0].borders[2];
            tiles.push(row);
            let mut next_row_begin = match next_row_begin_id {
                None => break,
                Some(id) => tile_by_id.remove(&id).unwrap(),
            };
            next_row_begin.match_adj_border(0, prev_row_begin_down_border);
            row_begin = next_row_begin;
        }
        assert!(tile_by_id.is_empty());
        let mut rows: Vec<Vec<Pixel>> = vec![];
        fn trimmed_pixel_rows_from_tile(t: &Tile) -> impl Iterator<Item = &[Pixel]> {
            let image = &t.image;
            image[1..image.len() - 1]
                .iter()
                .map(|row| &row[1..row.len() - 1])
        }
        for tile_row in &tiles {
            let mut pixel_row_iters: Vec<_> =
                tile_row.iter().map(trimmed_pixel_rows_from_tile).collect();
            let new_rows_iter = std::iter::from_fn(move || {
                pixel_row_iters.iter_mut().map(|it| it.next()).try_fold(
                    vec![],
                    |mut accu: Vec<Pixel>, v: Option<&[Pixel]>| {
                        accu.extend(v?);
                        Some(accu)
                    },
                )
            });
            rows.extend(new_rows_iter)
        }

        Self { tiles, image: rows }
    }
    pub fn tiles(&self) -> impl Iterator<Item = &Tile> {
        self.tiles.iter().flat_map(|i| i.iter())
    }

    pub fn corners<'a>(&'a self) -> impl Iterator<Item = &'a Tile> + 'a {
        std::iter::from_fn({
            let (x_min, x_max) = (0, self.tiles.len() - 1);
            let (y_min, y_max) = (0, self.tiles[0].len() - 1);
            let pos = [
                (x_min, y_min),
                (x_min, y_max),
                (x_max, y_min),
                (x_max, y_max),
            ];
            let mut pos_iter = pos.into_iter();
            move || {
                let (x, y) = pos_iter.next()?;
                Some(&self.tiles[x][y])
            }
        })
    }

    pub fn image(&self) -> &Image {
        &self.image
    }
}

fn connect_tiles(tile_by_id: &mut HashMap<TileId, Tile>) {
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
        fn add_neighbor(
            tile_map: &mut HashMap<TileId, Tile>,
            lid: TileId,
            lborder: Border,
            rid: TileId,
        ) -> usize {
            let l = tile_map.get_mut(&lid).unwrap();
            let border_idx = l
                .borders
                .iter()
                .position(|&p| p == lborder || complement(p) == lborder)
                .unwrap();
            l.neighbors[border_idx] = Some(rid);
            return l.neighbors().count();
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
            let l_neighbor_count = add_neighbor(tile_by_id, lid, border, rid);
            let r_neighbor_count = add_neighbor(tile_by_id, rid, border, lid);
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
    sea_map.corners().map(|t| t.id).product()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        assert_eq!(solve_1(SAMPLE_IN), 20899048083289)
    }

    #[test]
    fn test_run_2() {
        const MONSTER: &str = r"                  # 
#    ##    ##    ###
 #  #  #  #  #  #   ";
        let monster: Image = MONSTER
            .lines()
            .map(|s| {
                s.chars()
                    .map(|c| if c == '#' { true } else { false })
                    .collect()
            })
            .collect();
        let mut sea_map = SeaMap::from_str(SAMPLE_IN);
        remove_subimage(&mut sea_map.image, &monster);
        assert_eq!(
            sea_map
                .image
                .iter()
                .flatten()
                .copied()
                .filter(|&x| x)
                .count(),
            273
        );
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
    #[test]
    fn test_combine_image() {
        let x = SeaMap::from_str(SAMPLE_IN);
        print!("{}", image_string(&x.image))
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
