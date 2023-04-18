use std::collections::BTreeSet;

use once_cell::sync::Lazy;
use regex::Regex;

type Coordinate = (i64, i64);

const TARGET_ROW: i64 = 2000_000;
const MAX_BEACON_VAL: i64 = 4000_000;

#[derive(Debug, thiserror::Error)]
enum ParseError {
    #[error("Cannot parse the string")]
    NoParse,
}

fn parse(s: &str) -> anyhow::Result<(Coordinate, Coordinate)> {
    static REGEX: Lazy<regex::Regex> = Lazy::new(|| {
        Regex::new(r"x=(.+),.*y=(.+):.*x=(.+),.*y=(.*)$").expect("The regex should be valid")
    });
    let caps = REGEX.captures(s).ok_or(ParseError::NoParse)?;
    let mut caps = caps.iter().map(|s| -> anyhow::Result<_> {
        let s = s.ok_or(ParseError::NoParse)?;
        let out = s.as_str().parse()?;
        Ok(out)
    });
    let _ = caps.next();
    let x1 = caps.next().ok_or(ParseError::NoParse)??;
    let y1 = caps.next().ok_or(ParseError::NoParse)??;
    let x2 = caps.next().ok_or(ParseError::NoParse)??;
    let y2 = caps.next().ok_or(ParseError::NoParse)??;
    Ok(((x1, y1), (x2, y2)))
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
struct Intervals(BTreeSet<(i64, i64)>);

impl Intervals {
    fn new() -> Self {
        Self::default()
    }

    fn size(&self) -> usize {
        self.0
            .iter()
            .map(|&(lower, upper)| (upper - lower + 1) as usize)
            .sum()
    }

    fn insert(&mut self, (mut left, mut right): (i64, i64)) {
        if let Some(prev @ (min, max)) = self.0.range(..(left, right)).copied().next_back() {
            // `min` <= `left`
            // check the position of `max` inside the interval [`left`, `right`].
            if max >= right {
                return; // already covered
            } else if max >= left {
                // merge the two intervals
                self.0.remove(&prev);
                left = min;
                // we do not update right since max < right in this branch
            } else {
                // do nothing, there's no overlap.
            }
        }
        if let Some(next @ (min, max)) = self.0.range((left, right)..).copied().next() {
            // min >= left
            // check the position of `min` inside the interval [`left`, `right`]
            if min <= right {
                // merge the two intervals
                self.0.remove(&next);
                right = right.max(max);
            } else {
                // do nothing, there's no overlap.
            }
        }
        self.0.insert((left, right));
    }

    fn remove(&mut self, (lower, upper): (i64, i64)) {
        if let Some(leftmost @ (min, max)) = self.0.range(..(lower, i64::MIN)).copied().next_back()
        {
            if lower <= max {
                self.0.remove(&leftmost);
                if min < lower {
                    self.0.insert((min, lower - 1));
                }
                if max > upper {
                    self.0.insert((upper + 1, max));
                }
            }
        }
        let mut remove_candidate = self
            .0
            .range((lower, i64::MIN)..(upper + 1, i64::MIN))
            .copied()
            .collect::<Vec<_>>();
        if let Some(rightmost @ (_, max)) = remove_candidate.pop() {
            self.0.remove(&rightmost);
            if max > upper {
                self.0.insert((upper + 1, max));
            }
        }
        for mid in remove_candidate {
            self.0.remove(&mid);
        }
    }
}

impl std::iter::FromIterator<(i64, i64)> for Intervals {
    fn from_iter<T: IntoIterator<Item = (i64, i64)>>(iter: T) -> Self {
        let inner = iter.into_iter()
            .inspect(|&(a, b)| assert!(a <= b))
            .collect::<BTreeSet<_>>();
        Self(inner)
    }
}


fn distance((x1, y1): Coordinate, (x2, y2): Coordinate) -> u64 {
    x1.abs_diff(x2) + y1.abs_diff(y2)
}

fn disallowed_x_at_y(sensor: Coordinate, beacon: Coordinate, y: i64) -> Option<(i64, i64)> {
    let dis = distance(sensor, beacon);
    let allowed_x_dis = match dis.checked_sub(sensor.1.abs_diff(y)) {
        Some(i) => i,
        None => return None,
    };
    let x_delta = i64::try_from(allowed_x_dis).expect("distance cannot fit in an `i64`");
    Some((sensor.0 - x_delta, sensor.0 + x_delta))
}

fn count_impossible_positions_at_row(input: &[(Coordinate, Coordinate)], row: i64) -> usize {
    let mut disallowed_interval = Intervals::new();
    for &(sensor, beacon) in input {
        if let Some(interval) = disallowed_x_at_y(sensor, beacon, row) {
            disallowed_interval.insert(interval);
        }
    }
    let possible_location_count = disallowed_interval.size();
    let occupied_count = {
        let beacons = input
            .iter()
            .copied()
            .filter_map(|(_sensor, _beacon @ (x, y))| (y == TARGET_ROW).then_some(x))
            .collect::<std::collections::HashSet<_>>();
        beacons.len()
    };
    possible_location_count - occupied_count
}

fn find_beacon(input: &[(Coordinate, Coordinate)], max_val: i64) -> (i64, i64) {
    let mut candidates = vec![];
    for row in 0..=max_val {
        let mut interval = Intervals::from_iter([(0, max_val)]);
        for &(sensor, beacon) in input {
            if let Some(forbidden) = disallowed_x_at_y(sensor, beacon, row) {
                interval.remove(forbidden);
            }
        }
        for (min, max) in interval.0 {
            (min..=max).for_each(|x| candidates.push((x, row)));
        }
    }
    assert!(!candidates.is_empty(), "Cannot find beacon!");
    assert!(candidates.len() == 1, "Too many beacons: {:?}", &candidates);
    candidates.into_iter().next().expect("Impossible")
}

fn main() -> anyhow::Result<()> {
    let input = std::io::read_to_string(std::io::stdin())?;
    let input = input
        .lines()
        .map(parse)
        .collect::<anyhow::Result<Vec<_>>>()?;
    println!(
        "q1: {}",
        count_impossible_positions_at_row(&input, TARGET_ROW),
    );
    let (q2_x, q2_y) = find_beacon(&input, MAX_BEACON_VAL);
    println!("q2: {}", q2_x * 4000000 + q2_y);
    Ok(())
}

#[cfg(test)]
mod test_interval {
    use super::Intervals;
    #[test]
    fn disjoint() {
        let left = (1, 2);
        let right = (5, 6);
        let target = Intervals::from_iter([(1, 2), (5, 6)]);
        {
            let mut obj = Intervals::new();
            obj.insert(left);
            obj.insert(right);
            assert_eq!(obj, target);
        }
        {
            let mut obj = Intervals::new();
            obj.insert(right);
            obj.insert(left);
            assert_eq!(obj, target);
        }
    }
    #[test]
    fn some_overlap() {
        let left = (10, 20);
        let right = (15, 25);
        let target = Intervals::from_iter([(10, 25)]);
        {
            let mut obj = Intervals::new();
            obj.insert(left);
            obj.insert(right);
            assert_eq!(obj, target);
        }
        {
            let mut obj = Intervals::new();
            obj.insert(right);
            obj.insert(left);
            assert_eq!(obj, target);
        }
    }
    #[test]
    fn full_covered() {
        let outer = (10, 20);
        let inner = (13, 15);
        let target = Intervals::from_iter([(10, 20)]);
        {
            let mut obj = Intervals::new();
            obj.insert(outer);
            obj.insert(inner);
            assert_eq!(obj, target);
        }
        {
            let mut obj = Intervals::new();
            obj.insert(inner);
            obj.insert(outer);
            assert_eq!(obj, target);
        }
    }
    #[test]
    fn adjacent() {
        let left = (10, 15);
        let right = (15, 20);
        let target = Intervals::from_iter([(10, 20)]);
        {
            let mut obj = Intervals::new();
            obj.insert(left);
            obj.insert(right);
            assert_eq!(obj, target);
        }
        {
            let mut obj = Intervals::new();
            obj.insert(right);
            obj.insert(left);
            assert_eq!(obj, target);
        }
    }
    #[test]
    fn unit_size() {
        let left = (10, 15);
        let right = (15, 15);
        let target = Intervals::from_iter([(10, 15)]);
        {
            let mut obj = Intervals::new();
            obj.insert(left);
            obj.insert(right);
            assert_eq!(obj, target);
        }
        {
            let mut obj = Intervals::new();
            obj.insert(right);
            obj.insert(left);
            assert_eq!(obj, target);
        }
        let left = (10, 15);
        let right = (10, 10);
        let target = Intervals::from_iter([(10, 15)]);
        {
            let mut obj = Intervals::new();
            obj.insert(left);
            obj.insert(right);
            assert_eq!(obj, target);
        }
        {
            let mut obj = Intervals::new();
            obj.insert(right);
            obj.insert(left);
            assert_eq!(obj, target);
        }
        let outer = (10, 15);
        let inner = (12, 12);
        let target = Intervals::from_iter([(10, 15)]);
        {
            let mut obj = Intervals::new();
            obj.insert(outer);
            obj.insert(inner);
            assert_eq!(obj, target);
        }
        {
            let mut obj = Intervals::new();
            obj.insert(inner);
            obj.insert(outer);
            assert_eq!(obj, target);
        }
    }
    #[test]
    fn insert_middle() {
        let original = Intervals::from_iter([(10, 20), (30, 40)]);
        {
            let mut interval = original.clone();
            interval.insert((25, 25));
            let target = Intervals::from_iter([(10, 20), (25, 25), (30, 40)]);
            assert_eq!(interval, target);
        }
        {
            let mut interval = original.clone();
            interval.insert((15, 25));
            let target = Intervals::from_iter([(10, 25), (30, 40)]);
            assert_eq!(interval, target);
        }
        {
            let mut interval = original.clone();
            interval.insert((25, 35));
            let target = Intervals::from_iter([(10, 20), (25, 40)]);
            assert_eq!(interval, target);
        }
        {
            let mut interval = original.clone();
            interval.insert((15, 35));
            let target = Intervals::from_iter([(10, 40)]);
            assert_eq!(interval, target);
        }
    }

    #[test]
    fn remove_identical() {
        let mut interval = Intervals::from_iter([(1, 1)]);
        interval.remove((1, 1));
        assert_eq!(interval, Intervals::new());
    }

    #[test]
    fn remove_nonoverlap() {
        let mut interval = Intervals::from_iter([(1, 1)]);
        let target = interval.clone();
        interval.remove((0, 0));
        interval.remove((2, 2));
        assert_eq!(interval, target);
    }

    #[test]
    fn remove_overlap_left() {
        let mut interval = Intervals::from_iter([(-10, -5), (1, 10)]);
        interval.remove((5, 11));
        assert_eq!(
            interval,
            Intervals::from_iter([(-10, -5), (1, 4)]),
        );
    }

    #[test]
    fn remove_overlap_right() {
        let mut interval = Intervals::from_iter([(1, 10), (20, 30)]);
        interval.remove((-5, 5));
        assert_eq!(
            interval,
            Intervals::from_iter([(6, 10), (20, 30)])
        );
    }
    #[test]
    fn remove_covered() {
        let mut interval = Intervals::from_iter([
            (1, 1),
            (3, 3),
            (5, 5),
            (7, 7),
            (9, 9),
        ]);
        interval.remove((3, 7));
        assert_eq!(interval, Intervals::from_iter([(1, 1), (9, 9)]))
    }
}
