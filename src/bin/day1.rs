use std::io;
use std::fmt::Debug;
use std::collections::VecDeque;

#[derive(Clone,Debug,PartialEq)]
struct Elf {
    i: i32,
    calories: i32,
}

struct ElfReader<T>
where
    T: Iterator<Item=io::Result<String>>,
{
    lines: T,
    elf: Option<Elf>,
}

impl<T> ElfReader<T> 
where
    T: Iterator<Item=io::Result<String>>,
{
    pub fn new(lines: T) -> ElfReader<T> {
        ElfReader {
            lines,
            elf: Some(Elf { i: 0, calories: 0 }),
        }
    }
}

impl<T> Iterator for ElfReader<T>
where
    T: Iterator<Item=io::Result<String>>,
{
    type Item = Elf;
    fn next(&mut self) -> Option<Self::Item> {
        for line in self.lines.by_ref() {
            match line {
                Err(msg) => {
                    eprintln!("read stdin: {}", msg);
                },
                Ok(line) if line.as_str() == "" => {
                    let elf = self.elf.clone();
                    self.elf = self.elf.as_ref().map(|prev| Elf {
                        i: prev.i + 1,
                        calories: 0,
                    });
                    return elf;
                },
                Ok(line) => {
                    let calories = line.parse::<i32>().map_err(|err| {
                        eprintln!("bad line,err={},line={}", err, line);
                        err
                    });
                    if let Ok(cals) = calories {
                        self.elf.as_mut().unwrap().calories += cals;
                    }
                }
            }
        }
        self.elf.take()
    }
}

#[derive(Debug)]
struct TopN<T> {
    n: usize,
    vec: VecDeque<T>,
}

impl<T> TopN<T>
where
    T: Debug,
{
    pub fn new(n: usize) -> Self {
        if n == 0 {
            panic!("n must be greater than 0");
        }
        TopN { n, vec: VecDeque::new() }
    }

    fn add<U, F>(&mut self, val: T, key_func: F)
    where
        U: PartialOrd + Debug,
        F: Fn(&T) -> &U,
    {
        let val_key = key_func(&val);
        let mut insert_index = self.vec.len();
        for (i, el) in self.vec.iter().enumerate() {
            if val_key < key_func(el) {
                insert_index = i;
                break;
            }
        }
        self.vec.insert(insert_index, val);
        while self.vec.len() > self.n {
            self.vec.pop_front();
        }
    }

    fn iter(&self) -> std::collections::vec_deque::Iter<'_, T> {
        self.vec.iter()
    }
}

fn calories_for_top_elf() {
    let reader = ElfReader::new(io::stdin().lines());
    let max = reader.max_by_key(|e| e.calories).unwrap();
    println!("max: {:?}", max);
}

fn calories_for_top_3_elves() {
    let reader = ElfReader::new(io::stdin().lines());
    let mut topn = TopN::new(3);
    for elf in reader {
        topn.add(elf, |e| &e.calories);
    }
    let sum: i32 = topn.iter().map(|e| e.calories).sum();
    dbg!(&topn);
    println!("{}", sum);
}

const HELP: &str = "\
day1 <opts> part1|part2

-h|--help
    Show help
";

pub fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    if args.iter().any(|&a| a == "-h" || a == "--help") {
        print!("{}", HELP);
        return Ok(());
    }
    match args[..] {
        ["part1"] => calories_for_top_elf(),
        ["part2"] => calories_for_top_3_elves(),
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

    #[test]
    fn basic() {
        let input = vec!["23", "1", "", "1", "2"];
        let lines = input.iter().map(|v| io::Result::Ok(String::from(*v)));
        let mut reader = ElfReader::new(lines);
        assert_eq!(reader.next(), Some(Elf { i: 0, calories: 24 }));
        assert_eq!(reader.next(), Some(Elf { i: 1, calories: 3 }));

    }

    #[test]
    fn topn() {
        let input = vec![3, 5, 8, 2, 9, 12, 3];
        let mut topn = TopN::new(3);
        for v in input {
            topn.add(v, |v| v);
        }
        dbg!(&topn);
        let mut iter = topn.iter();
        assert_eq!(iter.next(), Some(&8));
        assert_eq!(iter.next(), Some(&9));
        assert_eq!(iter.next(), Some(&12));
        assert_eq!(iter.next(), None);
    }
}
