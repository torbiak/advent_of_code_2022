use core::cmp::Reverse;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};
use std::error::Error;
use std::io;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Dir {
    Up, Right, Down, Left
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Tile {
    Wall,
    Open,
    Blizzard,
}

#[derive(Clone, Copy, Debug)]
struct Blizzard {
    start: Point,
    dir: Dir,
}

impl Blizzard {
    fn new(start: Point, dir: Dir) -> Self {
        Self { start, dir }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Point {
    x: usize,
    y: usize,
}

impl Point {
    fn new(x: usize, y: usize) -> Self {
        Point { x, y }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct State {
    lower_bound: usize,
    player: Point,
    round: usize,
}

impl State {
    fn new(lower_bound: usize, player: Point, round: usize) -> Self {
        Self { lower_bound, player, round }
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, o: &Self) -> Option<Ordering> {
        Some(self.lower_bound.cmp(&o.lower_bound))
    }
}

impl Ord for State {
    fn cmp(&self, o: &Self) -> Ordering {
        self.lower_bound.cmp(&o.lower_bound)
    }
}

struct Board {
    width: usize,  // including walls
    height: usize,  // including walls
    row_blizzards: Vec<Vec<Blizzard>>,
    col_blizzards: Vec<Vec<Blizzard>>,
    start_pos: Point,
    end_pos: Point,
}

enum Action {
    Move(Dir),
    Wait
}

impl Board {
    fn read(s: &str) -> Self {
        let height = s.lines().count();
        let width = s.lines().next().unwrap().len();
        let mut col_blizzards: Vec<Vec<Blizzard>> = Vec::new();
        col_blizzards.resize(width, Vec::default());
        let mut row_blizzards: Vec<Vec<Blizzard>> = Vec::new();
        row_blizzards.resize(height, Vec::default());
        for (y, line) in s.lines().enumerate() {
            for (x, c) in line.chars().enumerate() {
                match c {
                    '#' | '.' => (),
                    '^' => col_blizzards[x].push(Blizzard::new(Point::new(x, y), Dir::Up)),
                    '>' => row_blizzards[y].push(Blizzard::new(Point::new(x, y), Dir::Right)),
                    'v' => col_blizzards[x].push(Blizzard::new(Point::new(x, y), Dir::Down)),
                    '<' => row_blizzards[y].push(Blizzard::new(Point::new(x, y), Dir::Left)),
                    _ => panic!("unexpected tile: {}", c),
                };
            }
        }
        let start_pos: Point = s.lines().next().unwrap()
            .chars()
            .position(|c| c == '.')
            .map(|x| Point::new(x, 0))
            .expect("first row should have one Open tile");
        let end_pos: Point = s.lines().last().unwrap()
            .chars()
            .position(|c| c == '.')
            .map(|x| Point::new(x, height - 1))
            .expect("last row should have one Open tile");
        Board { width, height, row_blizzards, col_blizzards, start_pos, end_pos }
    }

    fn blizzard_position(&self, b: Blizzard, round: usize) -> Point {
        // #>....#
        use Dir::*;
        let open_width = self.width - 2;
        let open_height = self.height - 2;
        let round = round as isize;
        // We need to remove and add the walls back in when calculating blizzard positions.
        match b.dir {
            Up => Point::new(b.start.x, _mod(b.start.y - 1, -round, open_height) + 1),
            Right => Point::new(_mod(b.start.x - 1, round, open_width) + 1, b.start.y),
            Down => Point::new(b.start.x, _mod(b.start.y - 1, round, open_height) + 1),
            Left => Point::new(_mod(b.start.x - 1, -round, open_width) + 1, b.start.y),
        }
    }

    fn get(&self, p: Point, round: usize) -> Tile {
        if p == self.start_pos || p == self.end_pos {
            return Tile::Open;
        }

        let is_blizzard = self.row_blizzards[p.y].iter()
            .chain(self.col_blizzards[p.x].iter())
            .any(|&b| self.blizzard_position(b, round) == p);
        if is_blizzard {
            return Tile::Blizzard;
        }

        let is_wall = p.x == 0
            || p.x == self.width - 1
            || p.y == 0
            || p.y == self.height - 1;
        if is_wall {
            return Tile::Wall;
        }

        Tile::Open
    }

    fn move_player(&self, p: Point, dir: Dir) -> Option<Point> {
        match dir {
            Dir::Up if p.y == 0 => None,
            Dir::Up => Some(Point::new(p.x, p.y - 1)),
            Dir::Right if p.x == self.width - 1 => None,
            Dir::Right => Some(Point::new(p.x + 1, p.y)),
            Dir::Down if p.y == self.height - 1 => None,
            Dir::Down => Some(Point::new(p.x, p.y + 1)),
            Dir::Left if p.x == 0 => None,
            Dir::Left => Some(Point::new(p.x - 1, p.y)),
        }
    }
}

fn _mod(start: usize, change: isize, modulus: usize) -> usize {
    let modulus = modulus as isize;
    let mut rem = (start as isize + change) % modulus;
    if rem < 0 {
        rem += modulus;
    }
    rem as usize
}

fn find_min_actions(board: &Board, start: Point, end: Point, initial_round: usize) -> usize {
    use Dir::*;
    use Action::*;

    let mut best: usize = usize::MAX;

    let mut queue: BinaryHeap<Reverse<State>> = BinaryHeap::new();
    let initial_state = State::new(
        lower_bound(start, end, 0),
        start,
        initial_round);
    queue.push(Reverse(initial_state));

    let mut seen: HashSet<State> = HashSet::new();

    while let Some(Reverse(state)) = queue.pop() {
        if state.player == end {
            if state.round < best {
                best = state.round;
            }
            continue;
        }

        let round = state.round + 1;
        let branches = [Move(Up), Move(Right), Move(Down), Move(Left), Wait]
            .iter()
            .filter_map(|a| match a {
                Move(d) => board.move_player(state.player, *d),
                Wait => Some(state.player),
            })
            .filter_map(|p| {
                if board.get(p, round) != Tile::Open {
                    return None;
                }
                let lb = lower_bound(p, end, round);
                if lb >= best {
                    return None;
                }

                let child_state = State::new(lb, p, round);
                if seen.contains(&child_state) {
                    return None;
                } else {
                    seen.insert(child_state);
                }

                Some(child_state)
            });
        for b in branches {
            queue.push(Reverse(b));
        }
    }
    best - initial_round
}

fn lower_bound(a: Point, b: Point, round: usize) -> usize {
    manhattan_dist(a, b) + round
}

fn manhattan_dist(a: Point, b: Point) -> usize {
    a.x.abs_diff(b.x) + a.y.abs_diff(b.y)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args[..] {
        ["part1"] => {
            let input = io::read_to_string(io::stdin())?;
            println!("{}", part1(&input));
        },
        ["part2"] => {
            let input = io::read_to_string(io::stdin())?;
            println!("{}", part2(&input));
        },
        _ => return Err("must specify part1|part2".into()),
    }
    Ok(())
}

fn part1(board_str: &str) -> usize {
    let board = Board::read(board_str);
    find_min_actions(&board, board.start_pos, board.end_pos, 0)
}

fn part2(board_str: &str) -> usize {
    let board = Board::read(board_str);
    let phase1 = find_min_actions(&board, board.start_pos, board.end_pos, 0);
    let phase2 = find_min_actions(&board, board.end_pos, board.start_pos, phase1);
    let phase3 = find_min_actions(&board, board.start_pos, board.end_pos, phase1 + phase2);
    phase1 + phase2 + phase3
}


#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = "\
#.######
#>>.<^<#
#.<..<<#
#>v.><>#
#<^v^^>#
######.#";

    #[test]
    fn test_blizzard_position_horizontal() {
        let board = Board::read(EXAMPLE);
        let start = Point::new(6, 1);
        let blizzard = Blizzard::new(start, Dir::Right);
        assert_eq!(board.blizzard_position(blizzard, 0), start);
        assert_eq!(board.blizzard_position(blizzard, 1), Point::new(1, 1));
        assert_eq!(board.blizzard_position(blizzard, 2), Point::new(2, 1));
    }

    #[test]
    fn test_blizzard_position_vertical() {
        let board = Board::read(EXAMPLE);
        let start = Point::new(4, 1);
        let blizzard = Blizzard::new(start, Dir::Up);
        assert_eq!(board.blizzard_position(blizzard, 0), start);
        assert_eq!(board.blizzard_position(blizzard, 1), Point::new(4, 4));
        assert_eq!(board.blizzard_position(blizzard, 2), Point::new(4, 3));
    }

    #[test]
    fn test_part1() {
        assert_eq!(part1(EXAMPLE), 18);
    }

    #[test]
    fn test_part2() {
        assert_eq!(part2(EXAMPLE), 54);
    }
}
