use std::io::BufRead;
use std::error::Error;
use std::collections::HashMap;


struct Monkeys {
    job_for: HashMap<String, Job>,
    parent_for: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
enum Job {
    Constant(i64),
    Expression(String, Op, String)
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
struct MonkeyIndex(u16);

#[derive(Debug, PartialEq, Clone, Copy)]
enum Op {
    Add, Sub, Mul, Div,
}

impl Monkeys {
    fn new() -> Self {
        Self {
            job_for: HashMap::new(),
            parent_for: HashMap::new(),
        }
    }

    fn read(r: impl BufRead) -> Result<Self, Box<dyn Error>> {
        let mut monkeys = Self::new();
        for line in r.lines() {
            let line = line?;
            let fields: Vec<&str> = line.split_whitespace().collect();
            match fields.len() {
                2 => {
                    let name = fields[0]
                        .strip_suffix(':')
                        .ok_or("no trailing colon on name")?
                        .to_string();
                    let constant = fields[1].parse()?;
                    monkeys.job_for.insert(name, Job::Constant(constant));
                },
                4 => {
                    let name = fields[0]
                        .strip_suffix(':')
                        .ok_or("no trailing colon on name")?
                        .to_string();
                    let left_name = fields[1].to_string();
                    let op: Option<Op> = match fields[2] {
                        "+" => Some(Op::Add),
                        "-" => Some(Op::Sub),
                        "*" => Some(Op::Mul),
                        "/" => Some(Op::Div),
                        _ => None,
                    };
                    let op = op.ok_or("unexpected operation")?;
                    let right_name = fields[3].to_string();

                    monkeys.job_for.insert(name.clone(), Job::Expression(left_name.clone(), op, right_name.clone()));
                    monkeys.parent_for.insert(left_name, name.clone());
                    monkeys.parent_for.insert(right_name, name);
                },
                _ => return Err("lines should have 2 or 4 words".into()),
            };
        }
        Ok(monkeys)
    }

    fn eval(&self, name: &str) -> i64 {
        match &self.job_for[name] {
            Job::Constant(n) => *n,
            Job::Expression(left_name, op, right_name) => {
                let left = self.eval(left_name);
                let right = self.eval(right_name);
                match op {
                    Op::Add => left + right,
                    Op::Sub => left - right,
                    Op::Mul => left * right,
                    Op::Div => left / right,
                }
            }
        }
    }

    fn find_path<'a>(&'a self, name: &'a str) -> Vec<&'a str> {
        let mut cur = name;
        let mut path = vec![name];
        while let Some(parent) = self.parent_for.get(cur) {
            path.push(parent);
            cur = parent;
        }
        path.reverse();
        path
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

fn part1(r: impl BufRead) -> Result<i64, Box<dyn Error>> {
    let monkeys = Monkeys::read(r)?;
    Ok(monkeys.eval("root"))
}

fn part2(r: impl BufRead) -> Result<i64, Box<dyn Error>> {
    let monkeys = Monkeys::read(r)?;
    let target_name = "humn";
    let path = monkeys.find_path(target_name);
    let mut path = path.iter().skip(1);  // Skip root.

    let human_side: &str = path.next().ok_or("should still have path left")?;
    let Job::Expression(ref l, _, ref r) = monkeys.job_for["root"] else {
        return Err("root monkey should have an Expression job".into());
    };
    let mut upper: i64 = monkeys.eval(if l == human_side { r } else { l });
    let mut cur = human_side;
    //println!("cur={cur} upper={upper} l={l} r={r}");

    while cur != target_name {
        let Job::Expression(ref l, op, ref r) = monkeys.job_for[cur] else {
            return Err("monkey should have an Expression job".into());
        };
        let human_side: &str = path.next().ok_or("should still have path left")?;
        //println!("cur={cur} upper={upper} l={l} op={op:?} r={r}");
        upper = match (l == human_side, op) {
            //upper = l + r, l = upper - r, r = upper - l
            (true, Op::Add) => upper - monkeys.eval(r),
            (false, Op::Add) => upper - monkeys.eval(l),
            // upper = l - r, l = upper + r, r = l - upper
            (true, Op::Sub) => upper + monkeys.eval(r),
            (false, Op::Sub) => monkeys.eval(l) - upper,
            // upper = l * r, l = upper / r, r = upper / l
            (true, Op::Mul) => upper / monkeys.eval(r),
            (false, Op::Mul) => upper / monkeys.eval(l),
            // upper = l / r, l = upper * r, r = l / upper
            (true, Op::Div) => upper * monkeys.eval(r),
            (false, Op::Div) => monkeys.eval(l) / upper,
        };
        cur = human_side;
    }
    Ok(upper)
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = "\
root: pppw + sjmn
dbpl: 5
cczh: sllz + lgvd
zczc: 2
ptdq: humn - dvpt
dvpt: 3
lfqf: 4
humn: 5
ljgn: 2
sjmn: drzm * dbpl
sllz: 4
pppw: cczh / lfqf
lgvd: ljgn * ptdq
drzm: hmdt - zczc
hmdt: 32";

    #[test]
    fn test_monkeys_read_root() {
        let monkeys = Monkeys::read(EXAMPLE.as_bytes()).unwrap();
        let job = &monkeys.job_for["root"];
        let Job::Expression(left_name, op, right_name) = job else {
            panic!("unexpected job: {:?}", job);
        };
        assert_eq!(left_name, "pppw");
        assert_eq!(right_name, "sjmn");
        assert_eq!(op, &Op::Add);
    }

    #[test]
    fn test_monkeys_read_dvpt() {
        let monkeys = Monkeys::read(EXAMPLE.as_bytes()).unwrap();
        assert_eq!(monkeys.job_for["dvpt"], Job::Constant(3));
    }

    #[test]
    fn test_part1() {
        assert_eq!(part1(EXAMPLE.as_bytes()).unwrap(), 152);
    }

    #[test]
    fn test_part2() {
        assert_eq!(part2(EXAMPLE.as_bytes()).unwrap(), 301);
    }
}
