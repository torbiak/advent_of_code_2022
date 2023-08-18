use std::collections::{HashSet, HashMap};
use std::error::Error;
use std::fmt;
use std::io::BufRead;
use std::ops::Range;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Point {
    x: i64,
    y: i64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Dir {
    N, NE, E, SE, S, SW, W, NW,
}

impl Point {
    fn new(x: i64, y: i64) -> Self {
        Self { x, y }
    }

    fn neighbor(&self, dir: Dir) -> Self {
        use Dir::*;
        match dir {
            N => Point::new(self.x, self.y + 1),
            NE => Point::new(self.x + 1, self.y + 1),
            E => Point::new(self.x + 1, self.y),
            SE => Point::new(self.x + 1, self.y - 1),
            S => Point::new(self.x, self.y - 1),
            SW => Point::new(self.x - 1, self.y - 1),
            W => Point::new(self.x - 1, self.y),
            NW => Point::new(self.x - 1, self.y + 1),
        }
    }
}

struct Board {
    elves: HashSet<Point>,
    round: usize,
}

impl Board {
    fn read(r: impl BufRead) -> Result<Board, Box<dyn Error>> {
        let mut elves: HashSet<Point> = HashSet::new();
        for (y, line) in r.lines().enumerate() {
            let line = line?;
            for (x, c) in line.chars().enumerate() {
                match c {
                    '.' => (),
                    // Reverse y so north can be y+1.
                    '#' => {
                        elves.insert(Point::new(x as i64, -(y as i64)));
                    },
                    c => return Err(format!("unexpected board char: {}", c).into()),
                };
            }
        }
        Ok(Board { elves, round: 0 })
    }

    fn is_alone(&self, elf: Point) -> bool {
        use Dir::*;
        [N, NE, E, SE, S, SW, W, NW]
            .iter()
            .all(|&d| !self.elves.contains(&elf.neighbor(d)))
    }

    // Return the number of elves that moved.
    fn play_round(&mut self) -> u64 {
        use Dir::*;
        let mut count_for: HashMap<Point, i64> = HashMap::new();
        let mut proposed: HashMap<Point, Point> = HashMap::new();
        let mut nmoved = 0;

        let dir_order = [N, S, W, E];
        let dir_order = dir_order.iter().cycle().skip(self.round % 4).take(4);

        for elf in self.elves.iter() {
            if self.is_alone(*elf) {
                continue;
            }
            for dir in dir_order.clone() {
                let dirs = match dir {
                    N => [N, NE, NW],
                    S => [S, SE, SW],
                    W => [W, NW, SW],
                    E => [E, NE, SE],
                    _ => panic!("unexpected dir"),
                };
                if dirs.iter().all(|&d| !self.elves.contains(&elf.neighbor(d))) {
                    let dst = elf.neighbor(*dir);
                    count_for.entry(dst).and_modify(|v| *v += 1).or_insert(1);
                    proposed.insert(*elf, dst);
                    break;
                }
            }
        }

        for (elf, dst) in proposed.iter() {
            if count_for[dst] == 1 {
                self.elves.remove(elf);
                self.elves.insert(*dst);
                nmoved += 1;
            }
        }

        self.round += 1;
        nmoved
    }

    fn open_spot_count(&self) -> u64 {
        let (x_range, y_range) = self.ranges();
        let mut count: u64 = 0;
        for x in x_range {
            for y in y_range.clone() {
                if !self.elves.contains(&Point::new(x, y)) {
                    count += 1;
                }
            }
        }
        count
    }

    fn ranges(&self) -> (Range<i64>, Range<i64>) {
        let mut min_x = i64::MAX;
        let mut max_x = i64::MIN;
        let mut min_y = i64::MAX;
        let mut max_y = i64::MIN;
        for elf in self.elves.iter() {
            min_x = min_x.min(elf.x);
            min_y = min_y.min(elf.y);
            max_x = max_x.max(elf.x);
            max_y = max_y.max(elf.y);
        }
        (min_x..max_x + 1, min_y..max_y + 1)
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (x_range, y_range) = self.ranges();
        for y in ((y_range.start - 2)..(y_range.end + 2)).rev() {
            for x in (x_range.start - 3)..(x_range.end + 3) {
                let c = match self.elves.contains(&Point::new(x, y)) {
                    true => '#',
                    false => '.',
                };
                write!(f, "{c}")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args[..] {
        ["part1"] => println!("{}", part1(std::io::stdin().lock())?),
        ["part2"] => println!("{}", part2(std::io::stdin().lock())?),
        _ => return Err("must specify part1|part2".into()),
    }
    Ok(())
}

fn part1(r: impl BufRead) -> Result<u64, Box<dyn Error>> {
    let mut board = Board::read(r)?;
    //println!("== Initial State==");
    //println!("{board}\n");
    for _ in 0..10 {
        board.play_round();
        //println!("== End of Round {} ==", board.round);
        //println!("{board}");
    }
    Ok(board.open_spot_count())
}

fn part2(r: impl BufRead) -> Result<usize, Box<dyn Error>> {
    let mut board = Board::read(r)?;
    let max_rounds = 1_000_000;
    for _ in 0..max_rounds {
        let nmoved = board.play_round();
        if nmoved == 0 {
            return Ok(board.round);
        }
    }
    Err(format!("Elves still moving after round {}", max_rounds).into())
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = "\
....#..
..###.#
#...#.#
.#...##
#.###..
##.#.##
.#..#..";

    #[test]
    fn test_part1() {
        assert_eq!(part1(EXAMPLE.as_bytes()).unwrap(), 110);
    }

    #[test]
    fn test_part2() {
        assert_eq!(part2(EXAMPLE.as_bytes()).unwrap(), 20);
    }
}
