use std::io;

const MAX_STACKS: usize = 9;

type Stacks = [Vec<char>; MAX_STACKS];

struct Move {
    n: usize,
    src: usize,
    dst: usize,
}

fn part1<T>(mut lines: T) -> String
where
    T: Iterator,
    T::Item: AsRef<str>,
{
    let mut stacks = parse_stacks(&mut lines);
    parse_moves(&mut lines, &mut stacks, move_lifo);
    stacks.iter().map(|s| s.last().unwrap_or(&' ')).collect()
}

fn part2<T>(mut lines: T) -> String
where
    T: Iterator,
    T::Item: AsRef<str>,
{
    let mut stacks = parse_stacks(&mut lines);
    parse_moves(&mut lines, &mut stacks, move_fifo);
    stacks.iter().map(|s| s.last().unwrap_or(&' ')).collect()
}

fn parse_stacks<T>(lines: T) -> Stacks
where
    T: Iterator,
    T::Item: AsRef<str>,
{
    let mut stacks: Stacks = core::array::from_fn(|_| Vec::new());
    for line in lines.take_while(|l| l.as_ref().trim().starts_with('[')) {
        for (i, c) in line.as_ref().chars().enumerate() {
            if !c.is_ascii_alphabetic() {
                continue;
            }
            stacks[i / 4].insert(0, c);
        }
    }
    stacks
}

fn parse_moves<T, F>(lines: T, stacks: &mut Stacks, mut move_fn: F)
where
    T: Iterator,
    T::Item: AsRef<str>,
    F: FnMut(&mut Stacks, Move),
{
    for line in lines.skip_while(|l| !l.as_ref().starts_with("move")) {
        let line = line.as_ref();
        let fields = line.split(' ').collect::<Vec<&str>>();
        if let [_, n, _, src, _, dst] = fields[..] {
            let n = n.parse::<usize>().unwrap();
            let src = src.parse::<usize>().unwrap() - 1;
            let dst = dst.parse::<usize>().unwrap() - 1;
            move_fn(stacks, Move { n, src, dst });
        } else {
            panic!("unexpected line: {}", line);
        }
    }
}

fn move_lifo(stacks: &mut Stacks, mv: Move) {
    for _ in 0..mv.n {
        let item = stacks[mv.src].pop().unwrap();
        stacks[mv.dst].push(item);
    }
}

fn move_fifo(stacks: &mut Stacks, mv: Move) {
    let mut scratch: Vec<char> = Vec::new();
    for _ in 0..mv.n {
        let item = stacks[mv.src].pop().unwrap();
        scratch.push(item);
    }
    while let Some(item) = scratch.pop() {
        stacks[mv.dst].push(item);
    }
}

const HELP: &str = "\
day5 <opts> part1|part2

-h|--help
    show help
";

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    if args.iter().any(|&a| a == "-h" || a == "--help") {
        print!("{}", HELP);
        return Ok(());
    }
    match args[..] {
        ["part1"] => println!("{}", part1(io::stdin().lines().map(|l| l.unwrap()))),
        ["part2"] => println!("{}", part2(io::stdin().lines().map(|l| l.unwrap()))),
        _ => return Err("Must specify part1|part2".to_owned()),
    };
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fmt::Debug;

    fn lines() -> Vec<String> {
        // `input` starts with an extra empty line which we remove below.
        let input = "
    [D]
[N] [C]
[Z] [M] [P]
 1   2   3

move 1 from 2 to 1
move 3 from 1 to 3
move 2 from 2 to 1
move 1 from 1 to 2";
        input.trim_start_matches('\n').lines().map(|l| l.to_owned()).collect()
    }

    fn assert_slices_eq<T>(a: &[T], b: &[T])
    where
        T: Eq + Debug,
    {
        let (mut it_a, mut it_b) = (a.iter(), b.iter());
        loop {
            let (i, j) = (it_a.next(), it_b.next());
            if let (None, None) = (i, j) {
                return;
            }
            if i != j {
                dbg!(a);
                dbg!(b);
                if let (Some(i), Some(j)) = (i, j) {
                    assert_eq!(i, j);
                } else {
                    assert_eq!(i, j);
                }
            }
        }
    }

    #[test]
    fn test_parse_stacks() {
        let stacks = parse_stacks(lines().iter());
        assert_slices_eq(&stacks[0], &vec!['Z', 'N']);
        assert_slices_eq(&stacks[1], &vec!['M', 'C', 'D']);
        assert_slices_eq(&stacks[2], &vec!['P']);
    }

    #[test]
    fn test_parse_moves_lifo() {
        let lines = lines();
        let mut lines = lines.iter();
        let mut stacks = parse_stacks(&mut lines);
        parse_moves(&mut lines, &mut stacks, move_lifo);
        assert_slices_eq(&stacks[0], &vec!['C']);
        assert_slices_eq(&stacks[1], &vec!['M']);
        assert_slices_eq(&stacks[2], &vec!['P', 'D', 'N', 'Z']);
    }

    #[test]
    fn test_parse_moves_fifo() {
        let lines = lines();
        let mut lines = lines.iter();
        let mut stacks = parse_stacks(&mut lines);
        parse_moves(&mut lines, &mut stacks, move_fifo);
        assert_slices_eq(&stacks[0], &vec!['M']);
        assert_slices_eq(&stacks[1], &vec!['C']);
        assert_slices_eq(&stacks[2], &vec!['P', 'Z', 'N', 'D']);
    }
}
