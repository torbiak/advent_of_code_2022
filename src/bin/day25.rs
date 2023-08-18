use std::io::BufRead;
use std::error::Error;

fn from_snafu_digit(c: char) -> i64 {
    match c {
        '2' => 2,
        '1' => 1,
        '0' => 0,
        '-' => -1,
        '=' => -2,
        _ => panic!("unexpected char: {c}"),
    }
}

fn to_snafu_digit(digit: i64) -> char {
    match digit {
        2 => '2',
        1 => '1',
        0 => '0',
        -1 => '-',
        -2 => '=',
        _ => panic!("unexpected digit: {digit}"),
    }
}

fn from_snafu(s: &str) -> i64 {
    let place_values = (0..).map(|i| 5i64.pow(i));
    let digit_values = s.chars().rev().map(from_snafu_digit);
    place_values.zip(digit_values).map(|(pv, dv)| pv * dv).sum()
}

fn to_snafu(n: i64) -> String {
    let mut snafu: String = String::new();
    let mut n = n;

    let mut place_value = 1;
    while place_value * 2 < n {
        place_value *= 5;
    }

    while place_value > 0 {
        //let orig_n = n;
        let mut digit = 0;
        if n > 0 {
            // `place_value / 2` is the max value representable by subsequent digits.
            while n > place_value / 2 {
                n -= place_value;
                digit += 1;
            }
        } else {
            while n < -place_value / 2 {
                n += place_value;
                digit -= 1;
            }
        }
        //println!("orig_n={orig_n} n={n} pv={place_value} digit={digit}");
        snafu.push(to_snafu_digit(digit));
        place_value /= 5;
    }
    snafu
}

fn part1(r: impl BufRead) -> Result<String, Box<dyn Error>> {
    let sum = r.lines()
        .map(|line| Ok::<i64, Box<dyn Error>>(from_snafu(&line?)))
        .sum::<Result<i64, _>>()?;
    Ok(to_snafu(sum))
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    match args[..] {
        ["part1"] => println!("{}", part1(std::io::stdin().lock())?),
        _ => return Err("must specify part1|part2".into()),
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = "\
1=-0-2
12111
2=0=
21
2=01
111
20012
112
1=-1=
1-12
12
1=
122";

    #[test]
    fn test_part1() {
        assert_eq!(part1(EXAMPLE.as_bytes()).unwrap(), "2=-1=0");
    }

    #[test]
    fn test_from_snafu() {
        assert_eq!(from_snafu("1"), 1);
        assert_eq!(from_snafu("2"), 2);
        assert_eq!(from_snafu("1="), 3);
        assert_eq!(from_snafu("1-"), 4);
        assert_eq!(from_snafu("10"), 5);
        assert_eq!(from_snafu("11"), 6);
        assert_eq!(from_snafu("12"), 7);
        assert_eq!(from_snafu("2="), 8);
        assert_eq!(from_snafu("2-"), 9);
        assert_eq!(from_snafu("20"), 10);
        assert_eq!(from_snafu("1=0"), 15);
        assert_eq!(from_snafu("1-0"), 20);
        assert_eq!(from_snafu("1=11-2"), 2022);
        assert_eq!(from_snafu("1-0---0"), 12345);
        assert_eq!(from_snafu("1121-1110-1=0"), 314159265);
    }

    #[test]
    fn test_to_snafu() {
        assert_eq!(to_snafu(1), "1");
        assert_eq!(to_snafu(2), "2");
        assert_eq!(to_snafu(3), "1=");
        assert_eq!(to_snafu(4), "1-");
        assert_eq!(to_snafu(5), "10");
        assert_eq!(to_snafu(6), "11");
        assert_eq!(to_snafu(7), "12");
        assert_eq!(to_snafu(8), "2=");
        assert_eq!(to_snafu(9), "2-");
        assert_eq!(to_snafu(10), "20");
        assert_eq!(to_snafu(13), "1==");
        assert_eq!(to_snafu(15), "1=0");
        assert_eq!(to_snafu(20), "1-0");
        assert_eq!(to_snafu(2022), "1=11-2");
        assert_eq!(to_snafu(12345), "1-0---0");
        assert_eq!(to_snafu(314159265), "1121-1110-1=0");
    }
}
