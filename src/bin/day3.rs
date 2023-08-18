use std::collections::HashSet;
use std::io;

const HELP: &str = "\
day3 <opts> part1|part2

-h|--help
    Show help
";

fn priority(c: &char) -> u32 {
    if c.is_lowercase() {
        *c as u32 - 'a' as u32 + 1
    } else if c.is_uppercase() {
        *c as u32 - 'A' as u32 + 26 + 1
    } else {
        panic!("unexpected character: {}", c);
    }
}

fn part1<T>(lines: T) -> u32
where
    T: Iterator,
    T::Item: AsRef<str>,
{
    let mut priority_sum = 0;
    for line in lines {
        let line = line.as_ref();
        let l = &line[..line.len()/2];
        let r = &line[line.len()/2..];
        let l_set: HashSet<char> = HashSet::from_iter(l.chars());
        let r_set: HashSet<char> = HashSet::from_iter(r.chars());
        let intersection = l_set.intersection(&r_set);
        for c in intersection.take(1) {
            priority_sum += priority(c);
        }
    }
    priority_sum
}

fn part2<T>(lines: T) -> u32
where
    T: Iterator,
    T::Item: AsRef<str>,
{
    let mut sum: u32 = 0;
    let chunks: Chunks<T> = Chunks::new(lines, 3);
    for chunk in chunks {
        let common = find_common_item(chunk.iter());
        sum += priority(&common);
    }
    sum
}

fn find_common_item<T>(lines: T) -> char
where
    T: Iterator,
    T::Item: AsRef<str>,
{
    let mut set: HashSet<char> = HashSet::new();
    for line in lines {
        let line_set = HashSet::from_iter(line.as_ref().chars());
        match set.len() {
            0 => set = line_set,
            _ => set = set.intersection(&line_set).cloned().collect(),
        }
    }
    if set.len() != 1 {
        panic!("expecting exactly one common item,set.len()={}", set.len());
    }
    set.into_iter().next().unwrap()
}

struct Chunks<T>
where
    T: Iterator
{
    inner: T,
    chunk_len: u32,
}

impl<T> Chunks<T>
where
    T: Iterator
{
    pub fn new(inner: T, chunk_len: u32) -> Self {
        Chunks { inner, chunk_len }
    }
}

impl<T, U> Iterator for Chunks<T>
where
    T: Iterator<Item=U>,
{
    type Item = Vec<U>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut chunk: Vec<U> = Vec::new();
        for i in 0..(self.chunk_len) {
            match self.inner.next() {
                Some(elem) => chunk.push(elem),
                None if i == 0 => return None,
                None => panic!("Not enough elements to fill chunk."),
            };
        }
        Some(chunk)
    }
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    if args.iter().any(|&s| s == "-h" || s == "--help") {
        print!("{}", HELP);
        return Ok(());
    }
    match args[..] {
        ["part1"] => println!("{}", part1(io::stdin().lines().map(|l| l.unwrap()))),
        ["part2"] => println!("{}", part2(io::stdin().lines().map(|l| l.unwrap()))),
        _ => {
            eprint!("{}", HELP);
            return Err("Must give part1|part2".to_owned());
        }
    };
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    fn lines() -> Vec<String> {
        let input = "\
vJrwpWtwJgWrhcsFMMfFFhFp
jqHRNqRjqzjGDLGLrsFMfFZSrLrFZsSL
PmmdzqPrVvPwwTWBwg
wMqvLMZHhHMvwLHjbvcjnnSBnvTQFn
ttgJtRGJQctTZtZT
CrZsJsPPZsGzwwsLwLmpwMDw".to_owned();
        input.lines().map(|l| l.to_owned()).collect()
    }

    #[test]
    fn test_part1() {
        let sum = part1(lines().iter());
        assert_eq!(sum, 157);
    }

    #[test]
    fn test_part2() {
        let sum = part2(lines().iter());
        assert_eq!(sum, 70);
    }
}
