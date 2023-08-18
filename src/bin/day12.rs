#![allow(dead_code)]  // TODO

use std::fmt;
use std::io::BufRead;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Reverse;

#[derive(PartialEq, Eq, Hash, Debug, PartialOrd, Ord, Clone, Copy)]
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Point { x, y }
    }

    pub fn from_usize(x: usize, y: usize) -> Self {
        Point { x: x as i32, y: y as i32 }
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Point{{{}, {}}}", self.x, self.y)
    }
}

struct Map {
    data: Vec<u8>,
    cols: i32,
    rows: i32,
    start: Point,
    goal: Point,
}

impl Map {
    pub fn from_lines<T: BufRead>(r: T) -> Result<Self, String> {
        let mut data: Vec<u8> = Vec::new();
        let mut cols: Option<i32> = None;
        let mut rows: i32 = 0;
        let mut start: Option<Point> = None;
        let mut goal: Option<Point> = None;
        for (i, line) in r.lines().enumerate() {
            rows += 1;
            let line = line.map_err(|e| e.to_string())?;

            if let Some(len) = cols {
                if line.len() != len as usize {
                    return Err(format!("mismatched line length: line={}", i));
                }

            } else {
                cols = Some(line.len() as i32);
            }

            for (j, b) in line.as_bytes().iter().enumerate() {
                match *b as char {
                    'a'..='z' => data.push(Self::height(*b as char)),
                    'S' => {
                        if start.is_some() {
                            return Err("multiple start points found".to_string());
                        }
                        start = Some(Point::from_usize(i, j));
                        data.push(Self::height('a'));
                    },
                    'E' => {
                        if goal.is_some() {
                            return Err("multiple goal points found".to_string());
                        }
                        goal = Some(Point::from_usize(j, i));
                        data.push(Self::height('z'));
                    }
                    '\n' | '\r' => (),
                    _ => return Err(format!("unexpected char: {}", *b as char)),
                }
            }
        }

        match (cols, start, goal) {
            (None, _, _) => Err("no lines read".to_string()),
            (Some(cols), Some(start), Some(goal)) => Ok(Map { data, start, cols, rows, goal }),
            (_, None, _) => Err("no start point found".to_string()),
            (_, _, None) => Err("no goal point found".to_string()),
        }
    }

    pub fn height(c: char) -> u8 {
        let offset = match c {
            'a'..='z' => c as u8,
            'S' => b'a',
            'E' => b'z',
            _ => panic!("unexpected character: {}", c),
        };
        offset - b'a'
    }

    pub fn at(&self, p: &Point) -> u8 {
        self.data[(p.y * self.cols + p.x) as usize]
    }

    pub fn min_moves_to_goal(&self, start: Point) -> Option<u32> {
        // Dijkstra's algorithm.
        let mut frontier: BinaryHeap<Reverse<(u32, Point)>> = BinaryHeap::new();
        let mut prev: HashMap<Point, Point> = HashMap::new();
        let mut dist: HashMap<Point, u32> = HashMap::new();
        let mut visited: HashSet<Point> = HashSet::new();

        frontier.push(Reverse((0, start)));
        dist.insert(start, 0);

        while let Some(Reverse((_, p0))) = frontier.pop() {
            // Since values in a priority queue typically can't be cheaply updated, multiple tuples
            // might be inserted for the same point as the distance estimate changes. We want to
            // ignore all but the lowest estimate for a given point, though.
            if visited.contains(&p0) {
                continue;
            }

            let neighbors = Neighbors::new(p0, self.rows, self.cols);
            for p1 in neighbors {
                if self.at(&p1) > (self.at(&p0) + 1) || visited.contains(&p1) {
                    continue;
                }
                // Relax
                let d0 = dist[&p0];
                if !dist.contains_key(&p1) || d0 + 1 < dist[&p1] {
                    let d1 = d0 + 1;
                    dist.insert(p1, d1);
                    frontier.push(Reverse((d1, p1)));
                    prev.insert(p1, p0);
                }
            }
            visited.insert(p0);
        }

        dist.get(&self.goal).copied()
    }
}

struct Neighbors {
    start: Point,
    inner: std::slice::Iter<'static, (i32, i32)>,
    rows: i32,
    cols: i32,
}

impl Neighbors {
    fn new(start: Point, rows: i32, cols: i32) -> Self {
        Neighbors { start, inner: NEIGHBOR_OFFSETS.iter(), rows, cols }
    }
}

// Origin is at the upper left.
const NEIGHBOR_OFFSETS: [(i32, i32); 4] = [
    (0, -1),  // up
    (1, 0),  // right
    (0, 1),  // down
    (-1, 0),  // left
];

impl Iterator for Neighbors {
    type Item = Point;

    fn next(&mut self) -> Option<Point> {
        for (dx, dy) in &mut self.inner {
            let x = self.start.x + dx;
            let y = self.start.y + dy;
            if (0..self.cols).contains(&x) && (0..self.rows).contains(&y) {
                return Some(Point::new(x, y));
            }
        }
        None
    }
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args[..] {
        ["part1"] => Ok(println!("{}", part1(std::io::stdin().lock())?)),
        ["part2"] => Ok(println!("{}", part2(std::io::stdin().lock())?)),
        _ => Err("Must specify part1|part2".to_string()),
    }
}

fn part1<T: BufRead>(r: T) -> Result<u32, String> {
    let map = Map::from_lines(r)?;
    map.min_moves_to_goal(map.start).ok_or_else(|| "no path to goal found".to_string())
}

fn part2<T: BufRead>(r: T) -> Result<u32, String> {
    let map = Map::from_lines(r)?;

    let mut points: Vec<Point> = Vec::new();
    for col in 0..map.cols {
        for row in 0..map.rows {
            points.push(Point::new(col, row));
        }
    }

    points.iter()
        .filter(|p| map.at(p) == 0)
        .filter_map(|p| map.min_moves_to_goal(*p))
        .min()
        .ok_or_else(|| "no paths to the goal were found".to_string())
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = "\
Sabqponm
abcryxxl
accszExk
acctuvwj
abdefghi";

    fn map() -> Map {
        Map::from_lines(EXAMPLE.as_bytes()).unwrap()
    }

    #[test]
    fn test_part1() {
        assert_eq!(part1(EXAMPLE.as_bytes()).unwrap(), 31);
    }

    #[test]
    fn test_part2() {
        assert_eq!(part2(EXAMPLE.as_bytes()).unwrap(), 29);
    }

    #[test]
    fn test_map_from_lines() {
        let map = Map::from_lines(EXAMPLE.as_bytes()).unwrap();
        assert_eq!(&map.start, &Point::new(0, 0));
        assert_eq!(&map.goal, &Point::new(5, 2));
        assert_eq!(map.data.last(), Some(&8));
        assert_eq!(map.cols, 8);
        assert_eq!(map.rows, 5);
    }

    #[test]
    fn test_at() {
        let map = map();
        assert_eq!(map.at(&Point::new(4, 1)), Map::height('y'));
    }

    #[test]
    fn test_neighbors_upper_left() {
        let mut it = Neighbors::new(Point::new(0, 0), 2, 2);
        assert_eq!(it.next(), Some(Point::new(1, 0)));
        assert_eq!(it.next(), Some(Point::new(0, 1)));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_neighbors_upper_right() {
        let mut it = Neighbors::new(Point::new(1, 0), 2, 2);
        assert_eq!(it.next(), Some(Point::new(1, 1)));
        assert_eq!(it.next(), Some(Point::new(0, 0)));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_neighbors_bottom_right() {
        let mut it = Neighbors::new(Point::new(1, 1), 2, 2);
        assert_eq!(it.next(), Some(Point::new(1, 0)));
        assert_eq!(it.next(), Some(Point::new(0, 1)));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_neighbors_bottom_left() {
        let mut it = Neighbors::new(Point::new(0, 1), 2, 2);
        assert_eq!(it.next(), Some(Point::new(0, 0)));
        assert_eq!(it.next(), Some(Point::new(1, 1)));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_neighbors_middle() {
        let mut it = Neighbors::new(Point::new(1, 1), 3, 3);
        assert_eq!(it.next(), Some(Point::new(1, 0)));
        assert_eq!(it.next(), Some(Point::new(2, 1)));
        assert_eq!(it.next(), Some(Point::new(1, 2)));
        assert_eq!(it.next(), Some(Point::new(0, 1)));
        assert_eq!(it.next(), None);
    }
}
