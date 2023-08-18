use std::io::BufRead;
use std::error::Error;
use std::ops::Range;
use std::collections::HashSet;

use once_cell::unsync::Lazy;
use regex_lite::Regex;


#[derive(PartialEq, Eq, Hash, Debug)]
struct Point {
    x: i64,
    y: i64,
}

impl Point {
    fn new(x: i64, y: i64) -> Self {
        Point { x, y }
    }
}

#[derive(PartialEq, Debug)]
struct Pair {
    sensor: Point,
    beacon: Point,
}


impl Pair {
    pub fn from_coords(sensor_x: i64, sensor_y: i64, beacon_x: i64, beacon_y: i64) -> Self {
        Self {
            sensor: Point { x: sensor_x, y: sensor_y },
            beacon: Point { x: beacon_x, y: beacon_y },
        }
    }

    pub fn distance_to_beacon(&self) -> u64 {
        self.sensor.x.abs_diff(self.beacon.x) + self.sensor.y.abs_diff(self.beacon.y)
    }

    pub fn range_covered_at_row(&self, row: i64) -> Option<Range<i64>> {
        let ydist = self.sensor.y.abs_diff(row);
        let beacon_dist = self.distance_to_beacon();
        if ydist > beacon_dist {
            return None;
        }
        let xlen_at_row: i64 = (beacon_dist - ydist) as i64;
        let mid = self.sensor.x;
        let range = (mid - xlen_at_row)..(mid + xlen_at_row + 1);
        Some(range)
    }
}


fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args[..] {
        ["part1"] => Ok(println!("{}", part1(std::io::stdin().lock(), 2000000)?)),
        ["part2"] => Ok(println!("{}", part2(std::io::stdin().lock(), 4000000, 4000000)?)),
        _ => Err("must specify part1|part2".into()),
    }
}

fn part1(r: impl BufRead, row: i64) -> Result<u64, Box<dyn Error>> {
    let pairs = read_pairs(r)?;
    let ranges = merged_ranges_for_row(&pairs, row);
    let covered_spots: u64 = ranges.iter().map(|r| r.end.abs_diff(r.start)).sum();
    let beacon_spots: u64 = pairs.iter()
        .map(|p| &p.beacon)
        .filter(|b| b.y == row)
        .collect::<HashSet<_>>()
        .len().try_into().unwrap();
    Ok(covered_spots - beacon_spots)
}

fn part2(r: impl BufRead, x_max: i64, y_max: i64) -> Result<i64, Box<dyn Error>> {
    let pairs = read_pairs(r)?;
    let Some(p) = first_uncovered_point(&pairs, x_max, y_max) else {
        return Err("no uncovered point found".into());
    };
    let tuning_freq = 4000000 * p.x + p.y;
    Ok(tuning_freq)
}

fn read_pairs(r: impl BufRead) -> Result<Vec<Pair>, Box<dyn Error>> {
    let line_re = Lazy::new(|| {
        Regex::new(r#"Sensor at x=([-0-9]+), y=([-0-9]+): closest beacon is at x=([-0-9]+), y=([-0-9]+)"#).unwrap()
    });
    let mut pairs: Vec<Pair> = Vec::new();
    for line in r.lines() {
        let line = line?;
        let Some(caps) = line_re.captures(&line) else {
            return Err(format!("unexpected line format: {line}").into());
        };
        let coords: Vec<i64> = caps.iter().skip(1)
            .map(|s| s.unwrap().as_str().parse::<i64>())
            .collect::<Result<Vec<_>, _>>()?;
        pairs.push(Pair::from_coords(coords[0], coords[1], coords[2], coords[3]));
    }
    Ok(pairs)
}

fn merged_ranges_for_row(pairs: &[Pair], row: i64) -> Vec<Range<i64>> {
    let mut ranges: Vec<Range<i64>> = pairs.iter()
        .filter_map(|p| p.range_covered_at_row(row))
        .collect();
    merged_ranges(&mut ranges)
}

fn merged_ranges(ranges: &mut [Range<i64>]) -> Vec<Range<i64>> {
    ranges.sort_by_key(|r| r.start);
    let mut stack: Vec<Range<i64>> = Vec::new();
    for r in ranges {
        let Some(prev) = stack.pop() else {
            stack.push(r.clone());
            continue;
        };
        // There's three cases. r can be: outside prev, overlap prev's end, or inside prev.
        // Since we're sorted by start, r's start can't be less than prev's start.
        if r.end < prev.end {  // inside prev
            stack.push(prev);
        } else if r.start <= prev.end {  // overlaps prev's end
            stack.push(prev.start..r.end);
        } else {  // outside prev
            stack.push(prev);
            stack.push(r.clone());
        }
    }
    stack
}

fn first_uncovered_point(pairs: &[Pair], x_max: i64, y_max: i64) -> Option<Point> {
    for row in 0..y_max {
        let ranges = merged_ranges_for_row(pairs, row);
        if let Some(x) = first_uncovered_x(&ranges, x_max) {
            return Some(Point::new(x, row));
        }
    }
    None
}

fn first_uncovered_x(merged_ranges: &[Range<i64>], max: i64) -> Option<i64> {
    let mut cur = 0;
    for r in merged_ranges {
        if cur < r.start && cur <= max {  // before
            return Some(cur);
        } else if r.contains(&cur) {  // inside
            cur = r.end;
        } else {  // after
            continue;
        }
    }
    if cur <= max {
        Some(cur)
    } else {
        None
    }
}


#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = "\
Sensor at x=2, y=18: closest beacon is at x=-2, y=15
Sensor at x=9, y=16: closest beacon is at x=10, y=16
Sensor at x=13, y=2: closest beacon is at x=15, y=3
Sensor at x=12, y=14: closest beacon is at x=10, y=16
Sensor at x=10, y=20: closest beacon is at x=10, y=16
Sensor at x=14, y=17: closest beacon is at x=10, y=16
Sensor at x=8, y=7: closest beacon is at x=2, y=10
Sensor at x=2, y=0: closest beacon is at x=2, y=10
Sensor at x=0, y=11: closest beacon is at x=2, y=10
Sensor at x=20, y=14: closest beacon is at x=25, y=17
Sensor at x=17, y=20: closest beacon is at x=21, y=22
Sensor at x=16, y=7: closest beacon is at x=15, y=3
Sensor at x=14, y=3: closest beacon is at x=15, y=3
Sensor at x=20, y=1: closest beacon is at x=15, y=3";

    #[test]
    fn test_read_pairs() {
        let pairs = read_pairs(EXAMPLE.as_bytes()).unwrap();
        assert_eq!(pairs.len(), 14);
        assert_eq!(pairs[0], Pair::from_coords(2, 18, -2, 15))
    }

    #[test]
    fn test_range_covered_at_row() {
        let pair = Pair::from_coords(8, 7, 2, 10);
        assert_eq!(pair.range_covered_at_row(10), Some(2..15i64));
    }

    #[test]
    fn test_part1() {
        assert_eq!(part1(EXAMPLE.as_bytes(), 10).unwrap(), 26);
    }

    #[test]
    fn test_first_uncovered_x_all_covered() {
        let ranges = vec![-3..-2, 0..11];
        assert_eq!(first_uncovered_x(&ranges, 10), None);
    }

    #[test]
    fn test_first_uncovered_x_at_start() {
        let ranges = vec![-3..-2, 1..11];
        assert_eq!(first_uncovered_x(&ranges, 10), Some(0));
    }

    #[test]
    fn test_first_uncovered_x_at_middle() {
        let ranges = vec![-3..-2, 0..5, 6..11];
        assert_eq!(first_uncovered_x(&ranges, 10), Some(5));
    }

    #[test]
    fn test_first_uncovered_x_at_end() {
        let ranges = vec![-3..-2, 0..10];
        assert_eq!(first_uncovered_x(&ranges, 10), Some(10));
    }

    #[test]
    fn test_part2() {
        assert_eq!(part2(EXAMPLE.as_bytes(), 20, 20).unwrap(), 56000011);
    }
}
