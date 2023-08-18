use std::fmt;
use std::io::BufRead;
use std::fmt::{Display, Formatter};
use std::cmp;

struct Array {
    rows: usize,
    cols: usize,
    data: Vec<u8>,  // Stored in row-major order.
}

struct Coords {
    start: (usize, usize),
    end: (usize, usize),
    done: bool,
}

impl Coords {
    // start and end are inclusive.
    pub fn new(start: (usize, usize), end: (usize, usize)) -> Self {
        Coords { start, end, done: false}
    }

    // Return a new start that's one step closer to `end`.
    fn forward(start: usize, end: usize) -> usize {
        use cmp::Ordering::{Equal, Less, Greater};
        match start.cmp(&end) {
            Equal => start,
            Less => start.saturating_add(1),
            Greater => start.saturating_sub(1),
        }
    }
}

impl Iterator for Coords {
    type Item = (usize, usize);

    fn next(&mut self) -> Option<(usize, usize)> {
        if self.done {
            return None;
        }
        let this = self.start;
        self.start = (
            Self::forward(self.start.0, self.end.0),
            Self::forward(self.start.1, self.end.1),
        );
        if this == self.start {
            self.done = true;
        }
        Some(this)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl ExactSizeIterator for Coords {
    fn len(&self) -> usize {
        if self.done {
            0
        } else {
            let max_delta = cmp::max(
                self.end.0.abs_diff(self.start.0),
                self.end.1.abs_diff(self.start.1));
            max_delta + 1
        }
    }
}

impl Array {
    pub fn new(rows: usize, cols: usize) -> Self {
        let mut data: Vec<u8> = Vec::new();
        data.resize_with(rows * cols, Default::default);
        Array { rows, cols, data }
    }

    pub fn from_lines<U: BufRead>(r: U) -> Result<Self, String> {
        let mut data: Vec<u8> = Vec::new();
        let mut row_len: Option<usize> = None;
        let mut nlines: usize = 0;
        for (line_num, line) in r.lines().enumerate() {
            let fields = line.map_err(|e| e.to_string());
            let mut nfields: usize = 0;
            for (col, height) in fields?.chars().enumerate() {
                let height: u8 = height.to_digit(10)
                    .ok_or(format!("parse height at {},{})", line_num, col))?
                    as u8;
                data.push(height);
                nfields += 1;
            }
            if nfields != *row_len.get_or_insert(nfields) {
                return Err(format!("wrong number of fields,line_num={}", line_num));
            }
            nlines += 1;
        }
        Ok(Array { rows: nlines, cols: row_len.unwrap(), data })
    }

    pub fn get(&self, row: usize, col: usize) -> &u8 {
        &self.data[row * self.cols + col]
    }

    pub fn get_mut(&mut self, row: usize, col: usize) -> &mut u8 {
        &mut self.data[row * self.cols + col]
    }

    pub fn row(&self, row: usize) -> Coords {
        Coords::new((row, 0), (row, self.cols - 1))
    }

    pub fn row_rev(&self, row: usize) -> Coords {
        Coords::new((row, self.cols - 1), (row, 0))
    }

    pub fn col(&self, col: usize) -> Coords {
        Coords::new((0, col), (self.rows - 1, col))
    }

    pub fn col_rev(&self, col: usize) -> Coords {
        Coords::new((self.rows - 1, col), (0, col))
    }
}

impl Display for Array {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for row in 0..self.rows {
            for (row, col) in self.row(row) {
                write!(f, "{}", self.get(row, col))?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

struct Visibles<'a> {
    height_map: &'a Array,
    coords: Coords,
    max: u8,
    first: bool,
}

impl<'a> Visibles<'a> {
    pub fn new(height_map: &'a Array, coords: Coords) -> Self {
        Visibles { height_map, coords, max: 0, first: true }
    }
}

impl<'a> Iterator for Visibles<'a> {
    type Item = (usize, usize, u8);

    fn next(&mut self) -> Option<Self::Item> {
        let mut visible: bool = false;

        let (row, col) = self.coords.next()?;
        let height = self.height_map.get(row, col);
        if height > &self.max {
            self.max = *height;
            visible = true;
        }

        if self.first {
            visible = true;
            self.first = false;
        }

        // At the end of the row or col.
        if self.coords.len() == 0 {
            visible = true;
        }

        Some((row, col, if visible { 1 } else { 0 }))
    }
}

fn visibility(height_map: &Array) -> Array {
    let mut vis_map: Array = Array::new(height_map.rows, height_map.cols);
    for row in 0..height_map.rows {
        for (row, col, is_visible) in Visibles::new(height_map, height_map.row(row)) {
            if is_visible  == 1 {
                *vis_map.get_mut(row, col) = is_visible;
            }
        }
        for (row, col, is_visible) in Visibles::new(height_map, height_map.row_rev(row)) {
            if is_visible  == 1 {
                *vis_map.get_mut(row, col) = is_visible;
            }
        }
    }
    for col in 0..height_map.cols {
        for (row, col, is_visible) in Visibles::new(height_map, height_map.col(col)) {
            if is_visible  == 1 {
                *vis_map.get_mut(row, col) = is_visible;
            }
        }
        for (row, col, is_visible) in Visibles::new(height_map, height_map.col_rev(col)) {
            if is_visible  == 1 {
                *vis_map.get_mut(row, col) = is_visible;
            }
        }
    }
    vis_map
}

fn part1<T: BufRead>(r: T) -> Result<usize, String> {
    let height_map: Array = Array::from_lines(r)?;
    Ok(visible_tree_count(&height_map))
}

fn visible_tree_count(height_map: &Array) -> usize {
    let vis_map = visibility(height_map);
    vis_map.data.iter().map(|v| *v as usize).sum()
}

fn part2<T: BufRead>(r: T) -> Result<usize, String> {
    let height_map: Array = Array::from_lines(r)?;
    Ok(highest_scenic_score(height_map))
}

fn highest_scenic_score(height_map: Array) -> usize {
    // Edges have a scenic score of 0, so skip them by starting at one and ending one space before
    // the end of each row or col.
    let mut max_score = 0;
    for row in 1..(height_map.rows - 1) {
        for col in 1..(height_map.cols - 1) {
            let score = scenic_score(&height_map, (row, col));
            max_score = cmp::max(max_score, score);
        }
    }
    max_score
}

fn scenic_score(height_map: &Array, tree: (usize, usize)) -> usize {
    let (row, col) = tree;
    if row == 0 || col == 0 || row == height_map.rows - 1 || col == height_map.cols - 1 {
        return 0;
    }
    let dsts = [
        (0, col),
        (height_map.rows - 1, col),
        (row, 0),
        (row, height_map.cols - 1),
    ];
    let a = height_map.get(row, col);
    let mut score = 1;
    for dst in dsts.iter() {
        let mut dist = 0;
        for (row, col) in Coords::new(tree, *dst).skip(1) {
            dist += 1;
            let b = height_map.get(row, col);
            if b >= a {
                break;
            }
        }
        if dist == 0 {
            return 0;
        }
        score *= dist;
    }
    score
}

const USAGE: &str = "\
day8 <opts> part1|part2

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

    const EXAMPLE: &str = "\
30373
25512
65332
33549
35390";

    #[test]
    fn from_lines() {
        let ar: Array = Array::from_lines(EXAMPLE.as_bytes()).unwrap();
        for (row, line) in EXAMPLE.lines().enumerate() {
            for (col, c) in line.chars().enumerate() {
                let height: u8 = c.to_digit(10).unwrap() as u8;
                assert_eq!(height, *ar.get(row, col), "mismatch at row={} col={}", row, col);
            }
        }
    }

    #[test]
    fn visibility_count() {
        let height_map = Array::from_lines(EXAMPLE.as_bytes()).unwrap();
        let count = visible_tree_count(&height_map);
        assert_eq!(count, 21);
    }

    #[test]
    fn coords_forward() {
        let mut coords = Coords::new((0, 3), (2, 3));
        assert_eq!(coords.next(), Some((0usize, 3usize)));
        assert_eq!(coords.next(), Some((1usize, 3usize)));
        assert_eq!(coords.next(), Some((2usize, 3usize)));
        assert_eq!(coords.next(), None);
    }

    #[test]
    fn coords_backward() {
        let mut coords = Coords::new((3, 3), (3, 0));
        assert_eq!(coords.next(), Some((3usize, 3usize)));
        assert_eq!(coords.next(), Some((3usize, 2usize)));
        assert_eq!(coords.next(), Some((3usize, 1usize)));
        assert_eq!(coords.next(), Some((3usize, 0usize)));
        assert_eq!(coords.next(), None);
    }

    #[test]
    fn coords_len() {
        let coords = Coords::new((3, 3), (3, 0));
        assert_eq!(coords.len(), 4);

    }

    #[test]
    fn test_scenic_score() {
        let height_map: Array = Array::from_lines(EXAMPLE.as_bytes()).unwrap();
        assert_eq!(highest_scenic_score(height_map), 8);
    }
}
