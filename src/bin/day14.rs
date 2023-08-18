use std::io::BufRead;
use std::fmt;
use std::ops::Range;
use std::cmp;

#[derive(Clone, Copy, PartialEq)]
enum Material {
    Air, Rock, Sand,
}

#[derive(Debug)]
enum FinalPosition {
    Rest(Point),
    Abyss,
}

struct Array2D {
    data: Vec<Material>,
    x_start: usize,
    cols: usize,
    bottom_row: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Point {
    x: usize,
    y: usize,
}

impl Point {
    fn new(x: usize, y: usize) -> Self {
        Point { x, y }
    }

    fn down(&self) -> Self {
        Point::new(self.x, self.y + 1)
    }

    fn down_left(&self) -> Self {
        Point::new(self.x - 1, self.y + 1)
    }

    fn down_right(&self) -> Self {
        Point::new(self.x + 1, self.y + 1)
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}, {}", self.x, self.y)?;
        Ok(())
    }
}

impl Array2D {
    fn new(cols: usize) -> Self {
        let rows = 200;
        let mut data: Vec<Material> = Vec::new();
        data.resize(rows * cols, Material::Air);
        Array2D { data, x_start: 500 - cols / 2, cols, bottom_row: 0 }
    }

    fn bottom_row(&self) -> usize {
        self.data.iter()
            .rposition(|&m| m == Material::Rock)
            .map(|i| self.point(i).y)
            .unwrap()
    }

    fn drop_sand(&mut self) -> FinalPosition {
        let final_pos = self.final_sand_pos(Point::new(500, 0));
        if let FinalPosition::Rest(p) = final_pos {
            self.set(&p, Material::Sand);
        }
        final_pos
    }

    fn final_sand_pos(&self, start: Point) -> FinalPosition {
        let mut cur = start;
        loop {
            let prev = cur;
            cur = self.next_sand_pos(cur);
            if cur == prev {
                return FinalPosition::Rest(cur);
            } else if cur.y >= self.bottom_row() {
                return FinalPosition::Abyss;
            }
        }
    }

    fn next_sand_pos(&self, p: Point) -> Point {
        if self.get(&p.down()) == Material::Air {
            p.down()
        } else if self.get(&p.down_left()) == Material::Air {
            p.down_left()
        } else if self.get(&p.down_right()) == Material::Air {
            p.down_right()
        } else {
            p
        }
    }

    fn read<T: BufRead>(r: T, cols: usize) -> Result<Self, String> {
        let mut array = Array2D::new(cols);
        for line in r.lines() {
            let line = line.map_err(|e| e.to_string())?;
            for (p1, p2) in PointPairs::new(&line) {
                array.set_line(p1, p2, Material::Rock);
            }
        }
        array.bottom_row = array.bottom_row();
        Ok(array)
    }

    pub fn col_range(&self) -> Range<usize> {
        self.x_start..(self.x_start + self.cols)
    }

    pub fn get(&self, p: &Point) -> Material {
        self.data[self.index(p)]
    }

    fn index(&self, p: &Point) -> usize {
        let col = p.x - self.x_start;
        if !(0..self.cols).contains(&col) {
            panic!("col index out of bounds: {p:?}");
        }
        p.y * self.cols + col
    }

    fn point(&self, i: usize) -> Point {
        Point::new(i % self.cols + self.x_start, i / self.cols)
    }

    pub fn set(&mut self, p: &Point, m: Material) {
        let index = self.index(p);
        self.data[index] = m
    }

    pub fn set_line(&mut self, p1: Point, p2: Point, m: Material) {
        let Point { x: x1, y: y1 } = p1;
        let Point { x: x2, y: y2 } = p2;
        if x1 == x2 {
            for y in cmp::min(y1, y2)..=cmp::max(y1, y2) {
                self.set(&Point::new(x1, y), m);
            }
        } else if y1 == y2 {
            for x in cmp::min(x1, x2)..=cmp::max(x1, x2) {
                self.set(&Point::new(x, y1), m);
            }
        } else {
            panic!("the line p1 -> p2 should not be diagonal");
        }
    }

    fn rows(&self) -> impl Iterator<Item=&[Material]> + '_ {
        self.data.chunks(self.cols)
    }

    fn active_box(&self) -> (Point, Point) {
        let start_col: Option<usize> = self.rows()
            .filter_map(|row| row.iter().position(|&m| m != Material::Air))
            .min();
        let end_col: Option<usize> = self.rows()
            .filter_map(|row| row.iter().rposition(|&m| m != Material::Air))
            .max();
        let start_row: Option<usize> = self.data.iter()
            .position(|&m| m != Material::Air)
            .map(|i| self.point(i).y);
        let end_row: Option<usize> = self.data.iter()
            .rposition(|&m| m != Material::Air)
            .map(|i| self.point(i).y);
        if let (Some(x1), Some(y1), Some(x2), Some(y2)) = (start_col, start_row, end_col, end_row) {
            // Add 1 to the end row/col so the range is half-open, so that we can represent an
            // empty active box.
            (
                Point::new(x1 + self.x_start, y1),
                Point::new(x2 + self.x_start + 1, y2 + 1)
            )
        } else {
            let p = Point::new(self.x_start, 0);
            (p, p)
        }
    }
}

impl fmt::Display for Array2D {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (p1, p2) = self.active_box();
        if p1 == p2 {
            writeln!(f, "empty")?;
            return Ok(());
        }

        write!(f, "{:4}", " ")?;  // Skip past row headings.
        for i in (p1.x)..(p2.x) {
            if i % 10 == 0 {
                write!(f, "|{:<9}", i)?;
            }
        }
        writeln!(f)?;
        for row in (p1.y)..(p2.y) {
            write!(f, "{:3} ", row)?;
            for col in (p1.x)..(p2.x) {
                let c = match self.get(&Point::new(col, row)) {
                    Material::Air => '.',
                    Material::Rock => '#',
                    Material::Sand => 'o',
                };
                write!(f, "{}", c)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

struct Scanner<'a> {
    s: &'a str,
    i: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(s: &'a str) -> Self {
        Self { s, i: 0 }
    }

    pub fn peek(&self) -> Option<char> {
        self.s.as_bytes().get(self.i).map(|b| *b as char)
    }

    pub fn next(&mut self) -> Option<char> {
        self.s.as_bytes().get(self.i).map(|b| {
            self.i += 1;
            *b as char
        })
    }

    pub fn is_done(&self) -> bool {
        self.i >= self.s.len()
    }

    pub fn take_while<P>(&mut self, predicate: P) -> &str
    where
        P: Fn(char) -> bool,
    {
        let mut range = self.i..self.i;
        while let Some(true) =  self.peek().map(&predicate) {
            self.next();
            range.end += 1;
        }
        &self.s[range]
    }

    pub fn expect(&mut self, expect: &str) -> Result<&str, String> {
        let got = self.s.get(self.i..(self.i + expect.len()));
        if got == Some(expect) {
            self.i += expect.len();
            Ok(got.unwrap())
        } else {
            Err(format!("expect={expect:?} got={got:?}"))
        }
    }
}

struct PointPairs<'a> {
    scanner: Scanner<'a>,
    p1: Option<Point>,
}

impl<'a> PointPairs<'a> {
    pub fn new(s: &'a str) -> Self {
        Self { scanner: Scanner::new(s), p1: None }
    }

    fn parse_point(&mut self) -> Result<Point, String> {
        let x = self.parse_int()?;
        self.scanner.expect(",")?;
        let y = self.parse_int()?;
        _ = self.scanner.expect(" -> ");
        Ok(Point::new(x, y))
    }

    fn parse_int(&mut self) -> Result<usize, String> {
        self.scanner
            .take_while(|c| c.is_ascii_digit())
            .parse::<usize>()
            .map_err(|e| e.to_string())
    }
}

impl Iterator for PointPairs<'_> {
    type Item = (Point, Point);

    fn next(&mut self) -> Option<Self::Item> {
        if self.scanner.is_done() {
            return None;
        }
        if self.p1.is_none() {
            self.p1 = self.parse_point().ok();
        }
        let p2 = self.parse_point().unwrap();
        let p1 = self.p1.replace(p2).unwrap();
        Some((p1, p2))
    }
}

fn part1<T: BufRead>(r: T) -> Result<usize, String> {
    let mut array = Array2D::read(r, 200)?;
    let mut i: usize = 0;
    loop {
        match array.drop_sand() {
            FinalPosition::Rest(_) => i +=1,
            FinalPosition::Abyss => {
                println!("{array}");
                return Ok(i)
            }, };
    }
}

fn part2<T: BufRead>(r: T) -> Result<usize, String> {
    let mut array = part2_array(r)?;
    let mut i: usize = 0;
    let sand_start = Point::new(500, 0);
    loop {
        match array.drop_sand() {
            FinalPosition::Rest(p) if p == sand_start => {
                println!("{array}");
                return Ok(i + 1);  // Include this last bit of sand in the result.
            },
            FinalPosition::Rest(_) => i += 1,
            FinalPosition::Abyss => panic!("sand should not go into the Abyss during part2"),
        }
    }
}

fn part2_array<T: BufRead>(r: T) -> Result<Array2D, String> {
    let mut array = Array2D::read(r, 400)?;
    let Range { start: first_col, end: last_col } = array.col_range();
    let row = array.bottom_row + 2;
    array.bottom_row = row;
    let p1 = Point::new(first_col, row);
    let p2 = Point::new(last_col - 1, row);
    array.set_line(p1, p2, Material::Rock);
    Ok(array)
}

// Use DFS to find all the points that sand can rest instead of simulating every move.
fn part2_fast<T: BufRead>(r: T) -> Result<usize, String> {
    let mut array = part2_array(r)?;
    let mut count = 0;
    let mut unvisited: Vec<Point> = Vec::new();
    unvisited.push(Point::new(500, 0));
    while let Some(p) = unvisited.pop() {
        count += 1;
        array.set(&p, Material::Sand);
        for child in &[p.down(), p.down_left(), p.down_right()] {
            if array.get(child) == Material::Air {
                unvisited.push(*child);
            }
        }
    }
    println!("{array}");
    Ok(count)
}

fn print<T: BufRead>(r: T) -> Result<(), String> {
    let array = Array2D::read(r, 200)?;
    println!("{array}");
    Ok(())
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args[..] {
        ["part1"] => Ok(println!("{}", part1(std::io::stdin().lock())?)),
        ["part2"] => Ok(println!("{}", part2(std::io::stdin().lock())?)),
        ["part2_fast"] => Ok(println!("{}", part2_fast(std::io::stdin().lock())?)),
        ["print"] => Ok(print(std::io::stdin().lock())?),
        _ => Err("must specify part1|part2|print".to_string()),
    }
}


#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = "\
498,4 -> 498,6 -> 496,6
503,4 -> 502,4 -> 502,9 -> 494,9";

    #[test]
    fn test_part1() {
        assert_eq!(part1(EXAMPLE.as_bytes()).unwrap(), 24);
    }

    #[test] #[ignore]  // Ignore: kinda slow.
    fn test_part2() {
        assert_eq!(part2(EXAMPLE.as_bytes()).unwrap(), 93);
    }

    #[test]
    fn test_part2_fast() {
        assert_eq!(part2_fast(EXAMPLE.as_bytes()).unwrap(), 93);
    }
}
