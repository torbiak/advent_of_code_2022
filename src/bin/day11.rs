#![allow(dead_code)]

use std::io::BufRead;
use std::str::FromStr;

#[derive(PartialEq, Eq, Debug)]
enum Op {
    Add(i32),
    Mul(i32),
    Square,
}

type Item = i64;

#[derive(PartialEq, Eq, Debug)]
struct Monkey {
    num: usize,
    items: Vec<Item>,
    op: Op,
    test: Item,
    success: usize,
    failure: usize,
}

impl Monkey {
    pub fn new(
        num: usize,
        items: Vec<Item>,
        op: Op,
        test: Item,
        success: usize,
        failure: usize,
    ) -> Self
    {
        Self { num, items, op, test, success, failure }
    }
}

impl FromStr for Monkey {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();

        let line = lines.next().ok_or("get monkey number line")?;
        let monkey_num: usize = line.split_whitespace()
            .nth(1).ok_or("get monkey number")
            .and_then(|v| v.trim_matches(':').parse().map_err(|_| "parse monkey number"))?;

        let line = lines.next().ok_or("get starting items")?;
        let items: Vec<Item> = line.replace(',', "")
            .split_whitespace().skip(2)
            .map(|v| v.parse())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| format!("parse items: {}", e))?;

        let line = lines.next().ok_or("get operation line")?;
        let op = match line.split_whitespace().skip(4).collect::<Vec<&str>>()[..] {
            ["*", "old"] => Op::Square,
            ["*", v] => Op::Mul(v.parse().map_err(|e| format!("parse op value: {}", e))?),
            ["+", v] => Op::Add(v.parse().map_err(|e| format!("parse op value: {}", e))?),
            _ => return Err(format!("unexpected operation line: {}", line)),
        };

        let line = lines.next().ok_or("get test line")?;
        let test: Item = line.split_whitespace()
            .nth(3).ok_or("get test value")
            .and_then(|v| v.parse().map_err(|_| "parse test value"))?;

        let line = lines.next().ok_or("get test success line")?;
        let success: usize = line.split_whitespace().nth(5)
            .ok_or("get test success monkey num")
            .and_then(|v| v.parse().map_err(|_| "parse success monkey num"))?;

        let line = lines.next().ok_or("get test failure line")?;
        let failure: usize = line.split_whitespace().nth(5)
            .ok_or("get test failure monkey num")
            .and_then(|v| v.parse().map_err(|_| "parse failure monkey num"))?;

        Ok(Monkey {
            num: monkey_num,
            items,
            op,
            test,
            success,
            failure,
        })
    }
}

struct Paragraphs<R> {
    r: R,
}

impl<R: BufRead> Paragraphs<R> {
    pub fn new(r: R) -> Self {
        Self { r }
    }
}

impl<R: BufRead> Iterator for Paragraphs<R> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = String::new();
        loop {
            match self.r.read_line(&mut buf) {
                Ok(0) if !buf.is_empty() => return Some(buf),
                Ok(0) => return None,
                Ok(_) if buf.ends_with("\n\n") => {
                    buf.pop();
                    return Some(buf);
                },
                Ok(_) => (),
                Err(e) => panic!("{}", e),
            }
        }
    }
}


fn part1<T: BufRead>(r: T) -> Result<u64, String> {
    let mut monkeys: Vec<Monkey> = Paragraphs::new(r)
        .map(|s| Monkey::from_str(&s))
        .collect::<Result<Vec<_>, _>>()?;
    let mut inspections: Vec<u64> = vec![0; monkeys.len()];
    for _round in 0..20 {
        for i in 0..monkeys.len() {
            let mut throws: Vec<(Item, usize)> = Vec::new();
            let monkey = &mut monkeys[i];
            while let Some(item) = monkey.items.pop() {
                inspections[monkey.num] += 1;
                let mut item = match monkey.op {
                    Op::Add(v) => item + v as Item,
                    Op::Mul(v) => item * v as Item,
                    Op::Square => item * item,
                };
                item /= 3;
                let throw_to = if item % monkey.test as Item == 0 {
                    monkey.success
                } else {
                    monkey.failure
                };
                throws.push((item, throw_to));
            }
            for (item, dst) in throws.into_iter() {
                monkeys[dst].items.push(item);
            }
        }
    }
    inspections.sort();
    let monkey_business = inspections.iter().rev().take(2).product();
    Ok(monkey_business)
}

fn part2<T: BufRead>(r: T) -> Result<u64, String> {
    let mut monkeys: Vec<Monkey> = Paragraphs::new(r)
        .map(|s| Monkey::from_str(&s))
        .collect::<Result<Vec<_>, _>>()?;

    let mut inspections: Vec<u64> = vec![0; monkeys.len()];

    // See https://en.wikipedia.org/wiki/Chinese_remainder_theorem
    let multimodulus: Item = monkeys.iter().map(|m| m.test as Item).product();

    for _round in 0..10_000 {
        for i in 0..monkeys.len() {
            let mut throws: Vec<(Item, usize)> = Vec::new();
            let monkey = &mut monkeys[i];
            while let Some(item) = monkey.items.pop() {
                inspections[monkey.num] += 1;

                // (a + b) mod m = ((a mod m) + (b mod m)) mod m
                // (a * b) mod m = ((a mod m) * (b mod m)) mod m
                let mut item = match monkey.op {
                    Op::Add(v) => item + v as Item,
                    Op::Mul(v) => item * v as Item,
                    Op::Square => item * item,
                };
                item %= multimodulus;
                let throw_to = if item % monkey.test == 0 { monkey.success } else { monkey.failure };
                throws.push((item, throw_to));
            }
            for (item, dst) in throws.into_iter() {
                monkeys[dst].items.push(item);
            }

        }
    }
    inspections.sort();
    let monkey_business = inspections.iter().rev().take(2).product();
    Ok(monkey_business)
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args[..] {
        ["part1"] => Ok(println!("{}", part1(std::io::stdin().lock())?)),
        ["part2"] => Ok(println!("{}", part2(std::io::stdin().lock())?)),
        _ => Err("Must specify part1|part2".to_string()),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = "\
Monkey 0:
  Starting items: 79, 98
  Operation: new = old * 19
  Test: divisible by 23
    If true: throw to monkey 2
    If false: throw to monkey 3

Monkey 1:
  Starting items: 54, 65, 75, 74
  Operation: new = old + 6
  Test: divisible by 19
    If true: throw to monkey 2
    If false: throw to monkey 0

Monkey 2:
  Starting items: 79, 60, 97
  Operation: new = old * old
  Test: divisible by 13
    If true: throw to monkey 1
    If false: throw to monkey 3

Monkey 3:
  Starting items: 74
  Operation: new = old + 3
  Test: divisible by 17
    If true: throw to monkey 0
    If false: throw to monkey 1";

    #[test]
    fn test_parse() {
        let got: Vec<Monkey> = Paragraphs::new(EXAMPLE.as_bytes())
            .map(|s| Monkey::from_str(&s))
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        let want: Vec<Monkey> = vec![
            Monkey::new(0, vec![79, 98], Op::Mul(19), 23, 2, 3),
            Monkey::new(1, vec![54, 65, 75, 74], Op::Add(6), 19, 2, 0),
            Monkey::new(2, vec![79, 60, 97], Op::Square, 13, 1, 3),
            Monkey::new(3, vec![74], Op::Add(3), 17, 0, 1),
        ];
        assert_eq!(got, want);
    }

    #[test]
    fn test_part1() {
        assert_eq!(part1(EXAMPLE.as_bytes()), Ok(10605));
    }

    #[test]
    fn test_part2() {
        assert_eq!(part2(EXAMPLE.as_bytes()), Ok(2713310158));
    }
}
