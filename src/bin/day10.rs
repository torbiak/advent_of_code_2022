use std::str::FromStr;
use std::io::BufRead;

enum Op {
    Noop,
    AddX(i32),
}

impl Op {
    pub fn ticks(&self) -> i32 {
        match self {
            Op::Noop => 1,
            Op::AddX(_) => 2,
        }
    }
}

impl FromStr for Op {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let fields = s.split_whitespace().collect::<Vec<&str>>();
        match fields[..] {
            ["noop"] => Ok(Op::Noop),
            ["addx", v] => {
                let v = v.parse::<i32>().map_err(|e| e.to_string())?;
                Ok(Op::AddX(v))
            },
            _ => Err(format!("can't parse op from: {}", s)),
        }

    }
}

fn part1<T: BufRead>(r: T) -> i32 {
    let mut x: i32 = 1;
    let mut total_signal_strength: i32 = 0;
    let mut ticks_left: i32 = 0;
    let mut op: Op = Op::Noop;

    let mut ops = r.lines().map(|s| Op::from_str(&s.unwrap()).unwrap());

    for tick in 1..=220 {
        if ticks_left == 0 {
            op = ops.next().unwrap();
            ticks_left = op.ticks();
        }

        if tick % 40 == 20 {
            let signal_strength = x * tick;
            total_signal_strength += signal_strength;
        }

        ticks_left -= 1;
        if ticks_left == 0 {
            match op {
                Op::Noop => (),
                Op::AddX(v) => x += v,
            }
        }
    }
    total_signal_strength
}

fn part2<T: BufRead>(r: T) -> String {
    let mut x: i32 = 1;
    let mut ticks_left: i32 = 0;
    let mut op: Op = Op::Noop;
    let mut pixels: String = String::new();

    let mut ops = r.lines().map(|s| Op::from_str(&s.unwrap()).unwrap());

    for tick in 1..=240i32 {
        if ticks_left == 0 {
            op = ops.next().unwrap();
            ticks_left = op.ticks();
        }

        let pos = (tick - 1) % 40;  // tick=1 -> pos=0, tick=41 -> pos=0
        let pixel = if pos.abs_diff(x) < 2 {
            '#'
        } else {
            '.'
        };
        pixels.push(pixel);

        if tick % 40 == 0 {
            pixels.push('\n');
        }

        ticks_left -= 1;
        if ticks_left == 0 {
            match op {
                Op::Noop => (),
                Op::AddX(v) => x += v,
            }
        }
    }
    pixels
}

fn main() -> Result<(), String> {
    match std::env::args().nth(1).unwrap().as_str() {
        "part1" => println!("{}", part1(std::io::stdin().lock())),
        "part2" => println!("{}", part2(std::io::stdin().lock())),
        _ => return Err("Must specify part1|part2".to_string()),
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = "\
addx 15
addx -11
addx 6
addx -3
addx 5
addx -1
addx -8
addx 13
addx 4
noop
addx -1
addx 5
addx -1
addx 5
addx -1
addx 5
addx -1
addx 5
addx -1
addx -35
addx 1
addx 24
addx -19
addx 1
addx 16
addx -11
noop
noop
addx 21
addx -15
noop
noop
addx -3
addx 9
addx 1
addx -3
addx 8
addx 1
addx 5
noop
noop
noop
noop
noop
addx -36
noop
addx 1
addx 7
noop
noop
noop
addx 2
addx 6
noop
noop
noop
noop
noop
addx 1
noop
noop
addx 7
addx 1
noop
addx -13
addx 13
addx 7
noop
addx 1
addx -33
noop
noop
noop
addx 2
noop
noop
noop
addx 8
noop
addx -1
addx 2
addx 1
noop
addx 17
addx -9
addx 1
addx 1
addx -3
addx 11
noop
noop
addx 1
noop
addx 1
noop
noop
addx -13
addx -19
addx 1
addx 3
addx 26
addx -30
addx 12
addx -1
addx 3
addx 1
noop
noop
noop
addx -9
addx 18
addx 1
addx 2
noop
noop
addx 9
noop
noop
noop
addx -1
addx 2
addx -37
addx 1
addx 3
noop
addx 15
addx -21
addx 22
addx -6
addx 1
noop
addx 2
addx 1
noop
addx -10
noop
noop
addx 20
addx 1
addx 2
addx 2
addx -6
addx -11
noop
noop
noop";

    const PIXELS: &str = "\
##..##..##..##..##..##..##..##..##..##..
###...###...###...###...###...###...###.
####....####....####....####....####....
#####.....#####.....#####.....#####.....
######......######......######......####
#######.......#######.......#######.....
";

    #[test]
    fn test_part1() {
        assert_eq!(part1(EXAMPLE.as_bytes()), 13140);
    }

    #[test]
    fn test_part2() {
        let got = part2(EXAMPLE.as_bytes());
        assert_eq!(got, PIXELS);
    }
}
