use std::error::Error;
use std::io;

#[derive(Clone, Copy, Debug)]
enum Dir {
    Left, Right,
}

#[derive(Clone, Copy, Debug)]
struct Point {
    x: usize,
    y: usize,
}

impl Point {
    pub fn new(x: usize, y: usize) -> Self {
        Point { x, y }
    }
}

struct Rock {
    shape: Vec<u8>,
    max_width: usize,
    max_height: usize,
}

fn rocks() -> Vec<Rock> {
    vec![
        Rock {
            shape: vec![0b11110000],
            max_width: 4,
            max_height: 1,
        },
        Rock {
            shape: vec![
                0b01000000,
                0b11100000,
                0b01000000,
            ],
            max_width: 3,
            max_height: 3,
        },
        Rock {
            // It looks upside down since the bottom row is first in the vec.
            shape: vec![
                0b11100000,
                0b00100000,
                0b00100000,
            ],
            max_width: 3,
            max_height: 3,
        },
        Rock {
            shape: vec![
                0b10000000,
                0b10000000,
                0b10000000,
                0b10000000,
            ],
            max_width: 1,
            max_height: 4,
        },
        Rock {
            shape: vec![
                0b11000000,
                0b11000000,
            ],
            max_width: 2,
            max_height: 2,
        },
    ]
}

const SHAFT_WIDTH: usize = 7;

struct Shaft {
    rows: Vec<u8>,
    highest: Option<usize>,
}

impl Shaft {
    pub fn new() -> Self {
        Shaft { rows: Vec::new(), highest: None }
    }

    pub fn place_rock(&mut self, rock: &Rock, rock_pos: Point) {
        for (dy, row) in rock.shape.iter().enumerate() {
            let y = rock_pos.y + dy;
            while self.rows.len() <= y {
                self.rows.push(0);
            }
            self.rows[y] |= row >> rock_pos.x;
            self.highest = Some(self.highest.map_or(y, |highest| highest.max(y)));
        }
    }

    fn rock_overlaps_walls(&self, rock: &Rock, rock_pos: Point, dir: Dir) -> bool {
        match dir {
            Dir::Left if rock_pos.x == 0 => true,
            Dir::Right if rock_pos.x + rock.max_width == SHAFT_WIDTH => true,
            _ => false,
        }
    }

    fn rock_overlaps_rocks(&self, rock: &Rock, rock_pos: Point) -> bool {
        for (dy, rock_row) in rock.shape.iter().enumerate() {
            let y = rock_pos.y + dy;
            if self.rows.get(y).map_or(false, |shaft_row| shaft_row & (rock_row >> rock_pos.x) > 0) {
                return true;
            }
        }
        false
    }

    fn print_falling_rock(&self, max_rows: usize, rock: &Rock, rock_pos: Point) {
        let start = (self.rows.len().saturating_sub(1)).max(rock_pos.y + rock.max_height);
        let end = start.saturating_sub(max_rows);
        for y in (end..=start).rev() {
            let rock_row = if y >= rock_pos.y {
                rock.shape.get(y - rock_pos.y)
            } else {
                None
            };
            print!("{:4} |", y);
            for x in 0..7 {
                let mask = 0b10000000 >> x;
                if matches!(rock_row, Some(row) if (row >> rock_pos.x) & mask > 0) {
                    print!("@");
                } else if matches!(self.rows.get(y), Some(row) if row & mask > 0) {
                    print!("#");
                } else {
                    print!(".");
                };
            }
            println!("|");
        }
        if end == 0 {
            println!("     +-------+");
        }
    }

    fn print(&self, max_rows: usize) {
        let start = self.rows.len().saturating_sub(1);
        let end = start.saturating_sub(max_rows);
        for y in (end..=start).rev() {
            print!("{:4} |", y);
            for x in 0..7 {
                let mask = 0b10000000 >> x;
                if matches!(self.rows.get(y), Some(row) if row & mask > 0) {
                    print!("#");
                } else {
                    print!(".");
                };
            }
            println!("|");
        }
        if end == 0 {
            println!("     +-------+");
        }
    }
}

struct SimConfig {
    print_shaft: bool,
    print_highest: bool,
}

fn simulate(jets: &str, nrocks: usize, config: SimConfig) -> Shaft {
    let rocks = rocks();
    let mut rocks = rocks.iter().cycle();
    let mut jets = jets.bytes()
        .map(|b| match b {
            b'<' => Dir::Left,
            b'>' => Dir::Right,
            _ => panic!("unexpected jet direction: {}", b),
        }).cycle();
    let mut shaft = Shaft::new();

    for i in 0..nrocks {
        let rock = rocks.next().unwrap();
        let mut rock_pos = Point::new(2, shaft.highest.map_or(3, |h| h + 4));

        if config.print_shaft {
            println!("New rock");
            shaft.print_falling_rock(10, rock, rock_pos);
            println!();
        }

        loop {
            // Move sideways.
            let dir = jets.next().unwrap();
            if !shaft.rock_overlaps_walls(rock, rock_pos, dir) {
                let new_pos = match dir {
                    Dir::Left => Point::new(rock_pos.x - 1, rock_pos.y),
                    Dir::Right => Point::new(rock_pos.x + 1, rock_pos.y),
                };
                if !shaft.rock_overlaps_rocks(rock, new_pos) {
                    rock_pos = new_pos;
                }
            }

            if config.print_shaft {
                println!("{dir:?}");
                shaft.print_falling_rock(10, rock, rock_pos);
                println!();
            }

            // Move down.
            if rock_pos.y == 0 {
                shaft.place_rock(rock, rock_pos);
                break;
            }
            let new_pos = Point::new(rock_pos.x, rock_pos.y - 1);
            if shaft.rock_overlaps_rocks(rock, new_pos) {
                shaft.place_rock(rock, rock_pos);
                break;
            }
            rock_pos = new_pos;

            if config.print_shaft {
                println!("Down");
                shaft.print_falling_rock(10, rock, rock_pos);
                println!();
            }
        }
        if config.print_shaft {
            println!("Placed");
            shaft.print(10);
            println!();
        }
        if config.print_highest {
            println!("{i},{}", shaft.highest.unwrap());
        }
    }
    shaft
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args[..] {
        ["print"] => _ = {
            let config = SimConfig { print_shaft: true, print_highest: false };
            simulate(">>><<><>><<<>><>>><<<>>><<<><<<>><>><<>>", 11, config)
        },
        ["part1"] => {
            let jets = io::read_to_string(io::stdin().lock())?;
            println!("{}", part1(jets.trim()));
        },
        ["part2"] => {
            let jets = io::read_to_string(io::stdin().lock())?;
            part2(jets.trim());
        }
        _ => return Err("must give print|part1|part2".into()),
    };
    Ok(())
}

fn part1(jets: &str) -> usize {
    let config = SimConfig { print_shaft: true, print_highest: false };
    let shaft = simulate(jets, 2022, config);
    shaft.highest.unwrap() + 1
}

// To answer part2 I wrote a CSV of the highest placed/settled rock position for the first 100k
// rocks, and loaded that up in pandas. I spoiled myself a bit, in that I saw on [Jukka Jylanki's
// page](http://clb.confined.space/aoc2022/#day17) that the input must be periodic. In pandas I
// plotted the height difference after each rock minus the mean difference, which showed a clear
// repeating signal. I then found where cumax().diff() was greater than normal to find the highest
// peak for each cycle. For my input the first peak was at 1652, and then every 1690 rocks after
// that, and the height increased by 2647 each cycle. And then we could just extrapolate
// out to a trillion:
//
//     >>> 1000000000000 // 1690 * 2647 + df.h.iat[1000000000000 % 1690]
//     1566272189352
//
fn part2(jets: &str) {
    let config = SimConfig { print_shaft: false, print_highest: true };
    _ = simulate(jets, 100_000, config);
}

#[cfg(test)]
mod test {
    use super::*;
    const EXAMPLE: &str = ">>><<><>><<<>><>>><<<>>><<<><<<>><>><<>>";

    #[test]
    fn test_part1() {
        assert_eq!(part1(EXAMPLE), 3068);
    }
}
