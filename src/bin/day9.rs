use std::cmp;
use std::str::FromStr;
use std::io::BufRead;
use std::collections::HashSet;

enum Dir { Up, Down, Left, Right }

impl FromStr for Dir {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "U" => Ok(Dir::Up),
            "D" => Ok(Dir::Down),
            "L" => Ok(Dir::Left),
            "R" => Ok(Dir::Right),
            _ => Err(format!("can't parse Dir: {}", s)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Pos {
    x: i32,
    y: i32,
}

impl Pos {
    pub fn new(x: i32, y: i32) -> Self {
        Pos { x, y }
    }

    pub fn chebyshev_distance(&self, o: &Self) -> u32 {
        cmp::max(self.x.abs_diff(o.x), self.y.abs_diff(o.y))
    }

    pub fn go(&self, dir: &Dir) -> Self {
        match dir {
            Dir::Up => Pos::new(self.x, self.y + 1),
            Dir::Down => Pos::new(self.x, self.y - 1),
            Dir::Left => Pos::new(self.x - 1, self.y),
            Dir::Right => Pos::new(self.x + 1, self.y),
        }
    }

    pub fn follow(&self, o: &Self) -> Self {
        if self.chebyshev_distance(o) > 1 {
            // Move straight or diagonally toward `o`, reducing Chebyshev distance by 1.
            Pos::new(one_closer(self.x, o.x), one_closer(self.y, o.y))
        } else {
            *self
        }
    }
}

fn one_closer(src: i32, tgt: i32) -> i32 {
    use cmp::Ordering::{Equal, Less, Greater};
    match src.cmp(&tgt) {
        Equal => src,
        Less => src + 1,
        Greater => src - 1,
    }
}

fn part1<T: BufRead>(r: T) -> Result<usize, String> {
    let mut head = Pos::new(0, 0);
    let mut tail = Pos::new(0, 0);

    let mut tail_positions: HashSet<Pos> = HashSet::new();
    tail_positions.insert(tail);
    for line in r.lines().map(|l| l.unwrap()) {
        if let [dir, count] = line.split_whitespace().collect::<Vec<&str>>()[..] {
            let dir = Dir::from_str(dir)?;
            let count: u32 = count.parse::<u32>().map_err(|e| e.to_string())?;
            for _ in 0..count {
                head = head.go(&dir);
                tail = tail.follow(&head);
                tail_positions.insert(tail);
            }
        } else {
            return Err(format!("unexpected line: {}", line));
        }
    }
    Ok(tail_positions.len())
}

fn part2<T: BufRead>(r: T) -> Result<usize, String> {
    let mut knots = [Pos::new(0, 0); 10];

    let mut tail_positions: HashSet<Pos> = HashSet::new();
    tail_positions.insert(knots[9]);
    for line in r.lines().map(|l| l.unwrap()) {
        if let [dir, count] = line.split_whitespace().collect::<Vec<&str>>()[..] {
            let dir = Dir::from_str(dir)?;
            let count: u32 = count.parse::<u32>().map_err(|e| e.to_string())?;
            for _ in 0..count {
                knots[0] = knots[0].go(&dir);
                for i in 1..knots.len() {
                    knots[i] = knots[i].follow(&knots[i-1]);
                }
                tail_positions.insert(knots[9]);
            }
        } else {
            return Err(format!("unexpected line: {}", line));
        }
    }
    Ok(tail_positions.len())
}

const USAGE: &str = "\
day9 <opts> part1|part2

-h|--help
    show help
";

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    if args.iter().any(|&a| a == "-h" || a == "--help") {
        print!("{}", USAGE);
        return Ok(());
    }
    match args[..] {
        ["part1"] => println!("{}", part1(std::io::stdin().lock())?),
        ["part2"] => println!("{}", part2(std::io::stdin().lock())?),
        _ => return Err("Must specify part1|part2".to_string()),
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_PART1: &str = "\
R 4
U 4
L 3
D 1
R 4
D 1
L 5
R 2";

    const EXAMPLE_PART2: &str = "\
R 5
U 8
L 8
D 3
R 17
D 10
L 25
U 20";

    #[test]
    fn test_chebyshev_distance() {
        let a = Pos::new(0, 0);
        let b = Pos::new(0, 1);
        assert_eq!(a.chebyshev_distance(&b), 1);
    }

    #[test]
    fn test_part1() {
        let count = part1(EXAMPLE_PART1.as_bytes()).unwrap();
        assert_eq!(count, 13);
    }

    #[test]
    fn test_part2() {
        let count = part2(EXAMPLE_PART2.as_bytes()).unwrap();
        assert_eq!(count, 36);
    }
}
