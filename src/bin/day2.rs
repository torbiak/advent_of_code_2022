use std::str::FromStr;
use std::io;

#[derive(Clone)]
pub enum Move {
    Rock,
    Paper,
    Scissors,
}

impl FromStr for Move {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "A" | "X" => Ok(Move::Rock),
            "B" | "Y" => Ok(Move::Paper),
            "C" | "Z" => Ok(Move::Scissors),
            bad => Err(format!("bad move: {}", bad)),
        }
    }
}

impl Move {
    pub fn from_intent(them: &Self, intent: &Intent) -> Self {
        use Move::*;
        use Intent::*;
        match (them, intent) {
            (them, Draw) => them.clone(),
            (Rock, Win) => Paper,
            (Rock, Lose) => Scissors,
            (Paper, Win) => Scissors,
            (Paper, Lose) => Rock,
            (Scissors, Win) => Rock,
            (Scissors, Lose) => Paper,
        }
    }
}

pub enum Intent {
    Win,
    Draw,
    Lose,
}

impl FromStr for Intent {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Intent::*;
        match s {
            "X" => Ok(Lose),
            "Y" => Ok(Draw),
            "Z" => Ok(Win),
            s => Err(format!("bad intent: {}", s)),
        }
    }
}

// score = shape_score + win_score
// shape_score:
//     Rock -> 1
//     Paper -> 2
//     Scissors -> 3
//
// win_score:
//     win -> 6
//     draw -> 3
//     lose -> 0
fn our_score(them: Move, us: Move) -> u32 {
    use Move::{Rock,Paper,Scissors};
    let win_score = match (&them, &us) {
        (Rock, Rock) => 3,
        (Rock, Paper) => 6,
        (Rock, Scissors) => 0,
        (Paper, Rock) => 0,
        (Paper, Paper) => 3,
        (Paper, Scissors) => 6,
        (Scissors, Rock) => 6,
        (Scissors, Paper) => 0,
        (Scissors, Scissors) => 3,
    };
    let shape_score = match &us {
        Rock => 1,
        Paper => 2,
        Scissors => 3,
    };
    win_score + shape_score
}

pub fn sum_line_scores<T, F>(lines: T, move_converter: F) -> u32
where
    T: Iterator,
    T::Item: AsRef<str>,
    F: Fn(&str) -> Result<(Move, Move), String>,
{
    lines.map(|line| {
        match move_converter(line.as_ref()) {
            Ok((them, us)) => our_score(them, us),
            Err(e) => {
                eprintln!("{}", e);
                0
            }
        }
    }).sum()
}

fn line_to_moves_part1(line: &str) -> Result<(Move, Move), String> {
    if let [them, us] = line.split(' ').collect::<Vec<_>>()[..] {
        let them = Move::from_str(them)?;
        let us = Move::from_str(us)?;
        Ok((them, us))
    } else {
        Err(format!("bad line: {}", line))
    }
}

fn line_to_moves_part2(line: &str) -> Result<(Move, Move), String> {
    if let [them, intent] = line.split(' ').collect::<Vec<_>>()[..] {
        let them = Move::from_str(them)?;
        let intent = Intent::from_str(intent)?;
        let us = Move::from_intent(&them, &intent);
        Ok((them, us))
    } else {
        Err(format!("bad line: {}", line))
    }
}

const HELP: &str = "\
day2 <opts> part1|part2

-h|--help
    Show help
";

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    if args.contains(&"-h") || args.contains(&"--help") {
        print!("{}", HELP);
        return Ok(());
    }
    let stdin = io::stdin().lines().map(|line| line.unwrap());
    match args[..] {
        ["part1"] => println!("{}", sum_line_scores(stdin, line_to_moves_part1)),
        ["part2"] => println!("{}", sum_line_scores(stdin, line_to_moves_part2)),
        _ => {
            eprint!("{}", HELP);
            return Err("No part specified".to_owned());
        },
    };
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn part1() {
        let lines = vec!["A Y", "B X", "C Z"];
        let score = sum_line_scores(lines.iter(), line_to_moves_part1);
        assert_eq!(score, 15);
    }

    #[test]
    fn part2() {
        let lines = vec!["A Y", "B X", "C Z"];
        let score = sum_line_scores(lines.iter(), line_to_moves_part2);
        assert_eq!(score, 12);

    }
}
