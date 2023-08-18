use std::io;
use std::str::FromStr;

struct Range {
    start: i32,
    end: i32,
}

impl Range {
    pub fn new(start: i32, end: i32) -> Self {
        Range { start, end }
    }

    pub fn contains(&self, o: &Self) -> bool {
        o.start >= self.start && o.end <= self.end
    }

    pub fn either_contains_other(a: &Self, b: &Self) -> bool {
        a.contains(b) || b.contains(a)
    }

    pub fn overlaps(&self, o: &Self) -> bool {
        !(o.start > self.end || o.end < self.start)
    }
}

impl FromStr for Range {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split('-').collect::<Vec<&str>>()[..] {
            [start, end] => {
                let start = start.parse::<i32>().map_err(|e| format!("parse range: {}", e))?;
                let end = end.parse::<i32>().map_err(|e| format!("parse range: {}", e))?;
                Ok(Range::new(start, end))
            },
            _ => Err("parse range: unexpected number of fields".to_owned()),
        }
    }
}

fn line_to_ranges(line: &str) -> Result<(Range, Range), String> {
    let ranges: Vec<&str> = line.split(',').collect();
    if let [a, b] = ranges[..] {
        let a = Range::from_str(a).unwrap();
        let b = Range::from_str(b).unwrap();
        Ok((a, b))
    } else {
        Err("unexpected number of ranges on line".to_owned())
    }
}

fn part1<T>(lines: T) -> u32
where
    T: Iterator,
    T::Item: AsRef<str>,
{
    lines.map(|l| {
        let (a, b) = line_to_ranges(l.as_ref()).unwrap();
        if Range::either_contains_other(&a, &b) { 1 } else { 0 }
    }).sum()
}

fn part2<T>(lines: T) -> u32
where
    T: Iterator,
    T::Item: AsRef<str>,
{
    lines.map(|l| {
        let (a, b) = line_to_ranges(l.as_ref()).unwrap();
        if a.overlaps(&b) { 1 } else { 0 }
    }).sum()
}

const HELP: &str = "\
day4 <opts> part1|part2

-h|--help
    show help
";

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    if args.iter().any(|&v| v == "-h" || v == "--help") {
        print!("{}", HELP);
        return Ok(());
    }
    match args[..] {
        ["part1"] => println!("{}", part1(io::stdin().lines().map(|l| l.unwrap()))),
        ["part2"] => println!("{}", part2(io::stdin().lines().map(|l| l.unwrap()))),
        _ => {
            eprint!("{}", HELP);
            return Err("Must give part1|part2".to_owned())
        }
    };
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    fn lines() -> Vec<String> {
        let input = "\
2-4,6-8
2-3,4-5
5-7,7-9
2-8,3-7
6-6,4-6
2-6,4-8";
        input.lines().map(&str::to_string).collect()
    }

    #[test]
    fn test_part1() {
        let sum = part1(lines().iter());
        assert_eq!(sum, 2);
    }

    #[test]
    fn test_part2() {
        let sum = part2(lines().iter());
        assert_eq!(sum, 4);
    }
}
