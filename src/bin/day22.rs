use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::io::BufRead;
use std::io;
use std::ops::Range;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Tile {
    Empty,
    Open,
    Wall,
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let repr = match self {
            Tile::Empty => " ",
            Tile::Open => ".",
            Tile::Wall => "#",
        };
        write!(f, "{repr}")
    }
}

struct Board {
    data: Vec<Tile>,
    row_len: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Player {
    dir: Dir,
    pos: Point,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Dir {
    Up, Right, Down, Left,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Move {
    Forward(usize),
    TurnRight,
    TurnLeft,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Side {
    Left, Right, Top, Bottom, Front, Back,
}

struct CubeTopology {
    side_len: usize,
    range_for: HashMap<Side, (Range<usize>, Range<usize>)>,
    neighbor_for: HashMap<(Side, Dir), (Side, Dir)>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
struct Point {
    x: usize,
    y: usize,
}

impl Board {
    fn read(s: &str) -> Result<Self, String> {
        let row_len = s.lines().map(|l| l.len()).max().ok_or("board should not be empty")?;
        let mut data: Vec<Tile> = Vec::new();
        for line in s.lines() {
            for c in line.chars() {
                let tile = match c {
                    ' ' => Ok(Tile::Empty),
                    '.' => Ok(Tile::Open),
                    '#' => Ok(Tile::Wall),
                    _ => Err("unexpected tile"),
                }?;
                data.push(tile);
            }
            if row_len > line.len() {
                for _ in 0..(row_len - line.len()) {
                    data.push(Tile::Empty);
                }
            }
        }
        Ok(Board { data, row_len })
    }

    fn rows(&self) -> Rows {
        Rows { board: self, i: 0 }
    }

    fn row_count(&self) -> usize {
        self.data.len() / self.row_len
    }

    fn line(&self, player: &Player) -> Line {
        Line { board: self, dir: player.dir, pos: player.pos }
    }

    fn get(&self, p: Point) -> Tile {
        self.data[p.y * self.row_len + p.x]
    }

    fn start_pos(&self) -> Point {
        let row = self.rows().next().unwrap();
        let first_open_tile = row.iter().position(|&t| t == Tile::Open).unwrap();
        Point::new(first_open_tile, 0)
    }

    fn move_player_part1(&self, player: Player, mv: Move) -> Player {
        use Dir::*;
        match mv {
            Move::Forward(n) => self.move_player_forward_wrapping(player, n),
            Move::TurnLeft => {
                let new_dir = match player.dir {
                    Up => Left,
                    Left => Down,
                    Down => Right,
                    Right => Up,
                };
                Player::new(new_dir, player.pos)
            },
            Move::TurnRight => {
                let new_dir = match player.dir {
                    Up => Right,
                    Right => Down,
                    Down => Left,
                    Left => Up,
                };
                Player::new(new_dir, player.pos)
            },
        }
    }

    fn move_player_forward_wrapping(&self, player: Player, n: usize) -> Player {
        // If the player tries to move into a wall they were already adjacent to, there won't be
        // anything to take from the iterator and we have to fallback to the original player.
        self.line(&player)
            .filter(|&p| self.get(p) != Tile::Empty)
            .take_while(|&p| self.get(p) != Tile::Wall)
            .take(n)
            .last()
            .map_or(player, |pos| Player::new(player.dir, pos))
    }

    fn move_player_part2(&self, player: Player, mv: Move, cube: &CubeTopology) -> Player {
        use Dir::*;
        match mv {
            Move::Forward(n) => self.move_player_forward_on_cube(player, n, cube),
            Move::TurnLeft => {
                let new_dir = match player.dir {
                    Up => Left,
                    Left => Down,
                    Down => Right,
                    Right => Up,
                };
                Player::new(new_dir, player.pos)
            },
            Move::TurnRight => {
                let new_dir = match player.dir {
                    Up => Right,
                    Right => Down,
                    Down => Left,
                    Left => Up,
                };
                Player::new(new_dir, player.pos)
            },
        }

    }

    fn move_player_forward_on_cube(&self, player: Player, n: usize, cube: &CubeTopology) -> Player {
        cube.ring(player)
            .filter(|&p| self.get(p.pos) != Tile::Empty)
            .take_while(|&p| self.get(p.pos) != Tile::Wall)
            .take(n)
            .last()
            .unwrap_or(player)  // Player tried to move into a wall they were already adjacent to.
    }
}

struct Rows<'a> {
    board: &'a Board,
    i: usize,
}

impl<'a> Iterator for Rows<'a> {
    type Item = &'a [Tile];

    fn next(&mut self) -> Option<Self::Item> {
        let row_len = self.board.row_len;
        if self.i * row_len >= self.board.data.len() {
            None
        } else {
            let row = &self.board.data[(self.i * row_len)..(self.i * row_len + row_len)];
            self.i += 1;
            Some(row)
        }
    }
}

struct Line<'a> {
    board: &'a Board,
    dir: Dir,
    pos: Point,
}

impl<'a> Iterator for Line<'a> {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        self.pos = match self.dir {
            Dir::Up => {
                let y = if self.pos.y == 0 { self.board.row_count() - 1 } else { self.pos.y - 1 };
                Point::new(self.pos.x, y)
            },
            Dir::Down => {
                let y = if self.pos.y == self.board.row_count() - 1 { 0 } else { self.pos.y + 1 };
                Point::new(self.pos.x, y)
            },
            Dir::Right => {
                let x = if self.pos.x == self.board.row_len - 1 { 0 } else { self.pos.x + 1 };
                Point::new(x, self.pos.y)
            },
            Dir::Left => {
                let x = if self.pos.x == 0 { self.board.row_len - 1 } else { self.pos.x - 1 };
                Point::new(x, self.pos.y)
            },
        };
        Some(self.pos)
    }
}


impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in self.rows() {
            let row_str: String = row.iter().map(|t| match t {
                Tile::Empty => ' ',
                Tile::Open => '.',
                Tile::Wall => '#',
            }).collect();
            writeln!(f, "{}", row_str.trim_end())?;
        }
        Ok(())
    }
}

impl Player {
    fn new(dir: Dir, pos: Point) -> Self {
        Player { dir, pos }
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:?} {},{})", self.dir, self.pos.x, self.pos.y)
    }
}

impl Point {
    fn new(x: usize, y: usize) -> Self {
        Point { x, y }
    }
}

struct Moves<'a> {
    s: &'a str,
}

impl<'a> Moves<'a> {
    fn new(s: &'a str) -> Self {
        Moves { s: s.trim_end() }
    }
}

impl<'a> Iterator for Moves<'a> {
    type Item = Move;

    fn next(&mut self) -> Option<Move> {
        let first_char = self.s.as_bytes().first();
        match first_char {
            None | Some(b'\n') => None,
            Some(b'L') => {
                self.s = &self.s[1..];
                Some(Move::TurnLeft)
            },
            Some(b'R') => {
                self.s = &self.s[1..];
                Some(Move::TurnRight)
            },
            Some(c) if c.is_ascii_digit() => {
                let len = self.s.bytes().take_while(|&c| c.is_ascii_digit()).count();
                let n = self.s[0..len].parse().unwrap();
                self.s = &self.s[len..];
                Some(Move::Forward(n))
            },
            Some(c) => panic!("unexpected character on move line: {:?}", *c as char),
        }
    }
}

impl CubeTopology {
    fn side(&self, p: Point) -> Side {
        self.range_for.iter().find_map(|(&side, (ref x_range, ref y_range))| {
            if x_range.contains(&p.x) && y_range.contains(&p.y) {
                Some(side)
            } else {
                None
            }
        }).unwrap()
    }

    fn ring(&self, player: Player) -> Ring {
        Ring::new(self, player)
    }

    fn next_player(&self, p: Player) -> Player {
        let (x_mod, y_mod) = (p.pos.x % self.side_len, p.pos.y % self.side_len);
        let crossing_corner = (x_mod == 0 && p.dir == Dir::Left)
            || (x_mod == self.side_len - 1 && p.dir == Dir::Right)
            || (y_mod == 0 && p.dir == Dir::Up)
            || (y_mod == self.side_len - 1 && p.dir == Dir::Down);
        //println!("{p} x_mod={x_mod} y_mod={y_mod} cross={crossing_corner}");
        if crossing_corner {
            self.across_corner(p)
        } else {
            let pos = match p.dir {
                Dir::Up => Point::new(p.pos.x, p.pos.y - 1),
                Dir::Right => Point::new(p.pos.x + 1, p.pos.y),
                Dir::Down => Point::new(p.pos.x, p.pos.y + 1),
                Dir::Left => Point::new(p.pos.x - 1, p.pos.y),
            };
            Player::new(p.dir, pos)
        }
    }

    fn across_corner(&self, p: Player) -> Player {
        let src_side = self.side(p.pos);
        let (dst_side, dst_dir) = self.neighbor_for[&(src_side, p.dir)];
        let (x_mod, y_mod) = (p.pos.x % self.side_len, p.pos.y % self.side_len);
        let (x_mod, y_mod) = match p.dir.angle_between(dst_dir) {
            0 => (x_mod, y_mod),
            90 => (self.comp(y_mod), x_mod),
            180 => (self.comp(x_mod), self.comp(y_mod)),
            270 => (y_mod, self.comp(x_mod)),
            angle => panic!("unexpected angle: {}", angle),
        };
        let (x_range, y_range) = &self.range_for[&dst_side];
        let (x, y) = match dst_dir {
            Dir::Up => (x_range.start + x_mod, y_range.end - 1),
            Dir::Right => (x_range.start, y_range.start + y_mod),
            Dir::Down => (x_range.start + x_mod, y_range.start),
            Dir::Left => (x_range.end - 1, y_range.start + y_mod),
        };
        //println!("src_side={src_side:?} src_dir={:?} dst_side={dst_side:?} dst_dir={dst_dir:?} x_mod={x_mod} y_mod={y_mod} angle={} x_range={x_range:?} y_range={y_range:?}", p.dir, p.dir.angle_between(dst_dir));
        Player::new(dst_dir, Point::new(x, y))
    }

    // side complement
    fn comp(&self, i: usize) -> usize {
        self.side_len - 1 - i
    }

    #[allow(unused)]
    fn example() -> Self {
        //   1  back
        // 234  bottom left top
        //   56 front right

        use Side::*;

        let mut range_for: HashMap<Side, (Range<usize>, Range<usize>)> = HashMap::new();
        range_for.insert(Back, (8..12, 0..4));
        range_for.insert(Bottom, (0..4, 4..8));
        range_for.insert(Left, (4..8, 4..8));
        range_for.insert(Top, (8..12, 4..8));
        range_for.insert(Front, (8..12, 8..12));
        range_for.insert(Right, (12..16, 8..12));

        let mut neighbor_for: HashMap<(Side, Dir), (Side, Dir)> = HashMap::new();
        neighbor_for.insert((Back, Dir::Up), (Bottom, Dir::Down));
        neighbor_for.insert((Back, Dir::Right), (Right, Dir::Left));
        neighbor_for.insert((Back, Dir::Down), (Top, Dir::Down));
        neighbor_for.insert((Back, Dir::Left), (Left, Dir::Down));

        neighbor_for.insert((Bottom, Dir::Up), (Back, Dir::Down));
        neighbor_for.insert((Bottom, Dir::Right), (Left, Dir::Right));
        neighbor_for.insert((Bottom, Dir::Down), (Front, Dir::Up));
        neighbor_for.insert((Bottom, Dir::Left), (Right, Dir::Up));

        neighbor_for.insert((Left, Dir::Up), (Back, Dir::Right));
        neighbor_for.insert((Left, Dir::Right), (Top, Dir::Right));
        neighbor_for.insert((Left, Dir::Down), (Front, Dir::Right));
        neighbor_for.insert((Left, Dir::Left), (Bottom, Dir::Left));

        neighbor_for.insert((Top, Dir::Up), (Back, Dir::Up));
        neighbor_for.insert((Top, Dir::Right), (Right, Dir::Down));
        neighbor_for.insert((Top, Dir::Down), (Front, Dir::Down));
        neighbor_for.insert((Top, Dir::Left), (Left, Dir::Left));

        neighbor_for.insert((Front, Dir::Up), (Top, Dir::Up));
        neighbor_for.insert((Front, Dir::Right), (Right, Dir::Right));
        neighbor_for.insert((Front, Dir::Down), (Bottom, Dir::Up));
        neighbor_for.insert((Front, Dir::Left), (Left, Dir::Up));

        neighbor_for.insert((Right, Dir::Up), (Top, Dir::Left));
        neighbor_for.insert((Right, Dir::Right), (Back, Dir::Left));
        neighbor_for.insert((Right, Dir::Down), (Bottom, Dir::Right));
        neighbor_for.insert((Right, Dir::Left), (Front, Dir::Left));

        CubeTopology { side_len: 4, range_for, neighbor_for }
    }

    fn part2() -> Self {
        //  12  back, right
        //  3   top
        // 45   front, left
        // 6    bottom

        use Side::*;

        let mut range_for: HashMap<Side, (Range<usize>, Range<usize>)> = HashMap::new();
        range_for.insert(Back, (50..100, 0..50));
        range_for.insert(Right, (100..150, 0..50));
        range_for.insert(Top, (50..100, 50..100));
        range_for.insert(Left, (0..50, 100..150));
        range_for.insert(Front, (50..100, 100..150));
        range_for.insert(Bottom, (0..50, 150..200));

        let mut neighbor_for: HashMap<(Side, Dir), (Side, Dir)> = HashMap::new();
        neighbor_for.insert((Back, Dir::Up), (Bottom, Dir::Right));
        neighbor_for.insert((Back, Dir::Right), (Right, Dir::Right));
        neighbor_for.insert((Back, Dir::Down), (Top, Dir::Down));
        neighbor_for.insert((Back, Dir::Left), (Left, Dir::Right));

        neighbor_for.insert((Right, Dir::Up), (Bottom, Dir::Up));
        neighbor_for.insert((Right, Dir::Right), (Front, Dir::Left));
        neighbor_for.insert((Right, Dir::Down), (Top, Dir::Left));
        neighbor_for.insert((Right, Dir::Left), (Back, Dir::Left));

        neighbor_for.insert((Top, Dir::Up), (Back, Dir::Up));
        neighbor_for.insert((Top, Dir::Right), (Right, Dir::Up));
        neighbor_for.insert((Top, Dir::Down), (Front, Dir::Down));
        neighbor_for.insert((Top, Dir::Left), (Left, Dir::Down));

        neighbor_for.insert((Left, Dir::Up), (Top, Dir::Right));
        neighbor_for.insert((Left, Dir::Right), (Front, Dir::Right));
        neighbor_for.insert((Left, Dir::Down), (Bottom, Dir::Down));
        neighbor_for.insert((Left, Dir::Left), (Back, Dir::Right));

        neighbor_for.insert((Front, Dir::Up), (Top, Dir::Up));
        neighbor_for.insert((Front, Dir::Right), (Right, Dir::Left));
        neighbor_for.insert((Front, Dir::Down), (Bottom, Dir::Left));
        neighbor_for.insert((Front, Dir::Left), (Left, Dir::Left));

        neighbor_for.insert((Bottom, Dir::Up), (Left, Dir::Up));
        neighbor_for.insert((Bottom, Dir::Right), (Front, Dir::Up));
        neighbor_for.insert((Bottom, Dir::Down), (Right, Dir::Down));
        neighbor_for.insert((Bottom, Dir::Left), (Back, Dir::Down));

        CubeTopology { side_len: 50, range_for, neighbor_for }
    }
}

impl Dir {
    fn angle_between(&self, o: Dir) -> u16 {
        let diff = (o.angle() as i16 - self.angle() as i16) % 360;
        if diff < 0 {
            (diff + 360) as u16
        } else {
            diff as u16
        }
    }

    fn angle(&self) -> u16 {
        match self {
            Dir::Up => 0,
            Dir::Right => 90,
            Dir::Down => 180,
            Dir::Left => 270,
        }
    }
}

// A ring-like path around the perimeter of the given cube.
struct Ring<'a> {
    cube_topology: &'a CubeTopology,
    player: Player,
}

impl<'a> Ring<'a> {
    fn new(cube_topology: &'a CubeTopology, player: Player) -> Self {
        Ring { player, cube_topology }
    }
}

impl<'a> Iterator for Ring<'a> {
    type Item = Player;

    fn next(&mut self) -> Option<Self::Item> {
        self.player = self.cube_topology.next_player(self.player);
        Some(self.player)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args[..] {
        ["part1"] => println!("{}", part1(io::stdin().lock())?),
        ["part2"] => {
            let password = part2(io::stdin().lock(), CubeTopology::part2())?;
            println!("{}", password);
        },
        _ => return Err("must specify part1|part2".into()),
    }
    Ok(())

}

fn part1(r: impl BufRead) -> Result<usize, Box<dyn Error>> {
    let input = io::read_to_string(r)?;
    let Some((board_str, moves_str)) = input.split_once("\n\n") else {
        return Err("input should consist of two paragraphs".into());
    };
    let board = Board::read(board_str)?;
    let moves = Moves::new(moves_str);
    let mut player = Player::new(Dir::Right, board.start_pos());
    for mv in moves {
        player = board.move_player_part1(player, mv);
    }
    Ok(password(player))
}

fn password(player: Player) -> usize {
    (player.pos.y + 1) * 1000 + (player.pos.x + 1) * 4 + match player.dir {
        Dir::Right => 0,
        Dir::Down => 1,
        Dir::Left => 2,
        Dir::Up => 3,
    }
}

fn part2(r: impl BufRead, cube: CubeTopology) -> Result<usize, Box<dyn Error>> {
    let input = io::read_to_string(r)?;
    let Some((board_str, moves_str)) = input.split_once("\n\n") else {
        return Err("input should consist of two paragraphs".into());
    };
    let board = Board::read(board_str)?;
    let moves = Moves::new(moves_str);
    let mut player = Player::new(Dir::Right, board.start_pos());
    for mv in moves {
        player = board.move_player_part2(player, mv, &cube);
    }
    Ok(password(player))
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_MOVES: &str = "10R5L5R10L4R5L5";

    fn example_board() -> &'static str {
        static BOARD: &str = "
        ...#
        .#..
        #...
        ....
...#.......#
........#...
..#....#....
..........#.
        ...#....
        .....#..
        .#......
        ......#.
";
        BOARD.trim_start_matches('\n')
    }


    #[test]
    fn test_board_read() {
        let board = Board::read(example_board()).unwrap();
        assert_eq!(format!("{board}"), example_board());
    }

    #[test]
    fn test_board_start_pos() {
        let board = Board::read(example_board()).unwrap();
        assert_eq!(board.start_pos(), Point::new(8, 0));
    }

    #[test]
    fn test_moves() {
        let mut moves = Moves::new(EXAMPLE_MOVES);
        assert_eq!(moves.next(), Some(Move::Forward(10)));
        assert_eq!(moves.next(), Some(Move::TurnRight));
        assert_eq!(moves.next(), Some(Move::Forward(5)));
        assert_eq!(moves.next(), Some(Move::TurnLeft));
        assert_eq!(moves.next(), Some(Move::Forward(5)));
        assert_eq!(moves.next(), Some(Move::TurnRight));
        assert_eq!(moves.next(), Some(Move::Forward(10)));
        assert_eq!(moves.next(), Some(Move::TurnLeft));
        assert_eq!(moves.next(), Some(Move::Forward(4)));
        assert_eq!(moves.next(), Some(Move::TurnRight));
        assert_eq!(moves.next(), Some(Move::Forward(5)));
        assert_eq!(moves.next(), Some(Move::TurnLeft));
        assert_eq!(moves.next(), Some(Move::Forward(5)));
        assert_eq!(moves.next(), None);
    }

    #[test]
    fn test_moving_player_part1() {
        let board = Board::read(example_board()).unwrap();
        let moves = Moves::new(EXAMPLE_MOVES);
        let mut player = Player::new(Dir::Right, board.start_pos());
        let wants = vec![
            Player::new(Dir::Right, Point::new(10, 0)),
            Player::new(Dir::Down, Point::new(10, 0)),
            Player::new(Dir::Down, Point::new(10, 5)),
            Player::new(Dir::Right, Point::new(10, 5)),
            Player::new(Dir::Right, Point::new(3, 5)),
            Player::new(Dir::Down, Point::new(3, 5)),
            Player::new(Dir::Down, Point::new(3, 7)),
            Player::new(Dir::Right, Point::new(3, 7)),
            Player::new(Dir::Right, Point::new(7, 7)),
            Player::new(Dir::Down, Point::new(7, 7)),
            Player::new(Dir::Down, Point::new(7, 5)),
            Player::new(Dir::Right, Point::new(7, 5)),
            Player::new(Dir::Right, Point::new(7, 5)),
        ];

        for (i, (mv, want)) in moves.zip(wants).enumerate() {
            player = board.move_player_part1(player, mv);
            assert_eq!(player, want, "mismatch at move {i}: {mv:?}");
        }
    }

    #[test]
    fn test_part1() {
        let input = format!("{}\n{}\n", example_board(), EXAMPLE_MOVES);
        assert_eq!(part1(input.as_bytes()).unwrap(), 6032);
    }

    #[test]
    fn test_dir_angle_between() {
        assert_eq!(Dir::Up.angle_between(Dir::Up), 0);
        assert_eq!(Dir::Up.angle_between(Dir::Right), 90);
        assert_eq!(Dir::Up.angle_between(Dir::Down), 180);
        assert_eq!(Dir::Up.angle_between(Dir::Left), 270);

        assert_eq!(Dir::Left.angle_between(Dir::Up), 90);
        assert_eq!(Dir::Left.angle_between(Dir::Right), 180);
        assert_eq!(Dir::Left.angle_between(Dir::Down), 270);
        assert_eq!(Dir::Left.angle_between(Dir::Left), 0);
    }

    #[test]
    fn test_part2() {
        let input = format!("{}\n{}\n", example_board(), EXAMPLE_MOVES);
        assert_eq!(part2(input.as_bytes(), CubeTopology::example()).unwrap(), 5031);
    }

    #[test]
    fn test_moving_player_part2() {
        let board = Board::read(example_board()).unwrap();
        let moves = Moves::new(EXAMPLE_MOVES);
        let mut player = Player::new(Dir::Right, board.start_pos());
        let wants = vec![
            Player::new(Dir::Right, Point::new(10, 0)),
            Player::new(Dir::Down, Point::new(10, 0)),
            Player::new(Dir::Down, Point::new(10, 5)),
            Player::new(Dir::Right, Point::new(10, 5)),
            Player::new(Dir::Down, Point::new(14, 10)),
            Player::new(Dir::Left, Point::new(14, 10)),
            Player::new(Dir::Left, Point::new(10, 10)),
            Player::new(Dir::Down, Point::new(10, 10)),
            Player::new(Dir::Up, Point::new(1, 5)),
            Player::new(Dir::Right, Point::new(1, 5)),
            Player::new(Dir::Right, Point::new(6, 5)),
            Player::new(Dir::Up, Point::new(6, 5)),
            Player::new(Dir::Up, Point::new(6, 4)),
        ];

        let cube = CubeTopology::example();
        for (i, (mv, want)) in moves.zip(wants).enumerate() {
            let new_player = board.move_player_part2(player, mv, &cube);
            assert_eq!(new_player, want, "mismatch at move {i}: {player} {mv:?}");
            player = new_player;
        }
    }

    #[test]
    fn test_ring() {
        let cube = CubeTopology::example();
        let player = Player::new(Dir::Right, Point::new(11, 4));
        assert_eq!(cube.next_player(player), Player::new(Dir::Down, Point::new(15, 8)));
    }
}
