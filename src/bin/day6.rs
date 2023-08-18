use std::collections::{HashMap, VecDeque};

fn find_packet_marker(s: &str) -> Option<usize> {
    const WIN_LEN: usize = 4;
    for (i, win) in s.as_bytes().windows(WIN_LEN).enumerate() {
        // Just test every pair.
        if win[0] != win[1] 
            && win[0] != win[2]
            && win[0] != win[3]
            && win[1] != win[2]
            && win[1] != win[3]
            && win[2] != win[3]
        {
            return Some(i + WIN_LEN);
        }
    }
    None
}

fn find_message_marker(s: &str) -> Option<usize> {
    const WIN_LEN: usize = 14;
    let mut window: VecDeque<char> = VecDeque::new();
    let mut freq: HashMap<char, u32> = HashMap::new();

    for (i, c) in s.chars().enumerate() {
        window.push_back(c);
        *freq.entry(c).or_insert(0) += 1;

        while window.len() > WIN_LEN {
            let c = window.pop_front().unwrap();
            let count = freq.get_mut(&c).unwrap();
            *count -= 1;
            if *count == 0 {
                freq.remove(&c);
            }
        }

        if freq.len() == WIN_LEN {
            return Some(i + 1);
        }
    }
    None
}

const HELP: &str = "\
day6 <opts> part1|part2

-h|--help
    show help
";

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();
    if args.iter().any(|&s| s == "-h" || s == "--help") {
        print!("{}", HELP);
        return Ok(());
    }

    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();

    match args[..] {
        ["part1"] => println!("{}", find_packet_marker(&line).unwrap()),
        ["part2"] => println!("{}", find_message_marker(&line).unwrap()),
        _ => return Err("Must give part1|part2".to_owned()),
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_find_packet_marker() {
        assert_eq!(find_packet_marker("mjqjpqmgbljsphdztnvjfqwrcgsmlb"), Some(7));
        assert_eq!(find_packet_marker("bvwbjplbgvbhsrlpgdmjqwftvncz"), Some(5));
        assert_eq!(find_packet_marker("nppdvjthqldpwncqszvftbrmjlhg"), Some(6));
        assert_eq!(find_packet_marker("nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg"), Some(10));
        assert_eq!(find_packet_marker("zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw"), Some(11));
    }

    #[test]
    fn test_find_message_marker() {
        assert_eq!(find_message_marker("mjqjpqmgbljsphdztnvjfqwrcgsmlb"), Some(19));
        assert_eq!(find_message_marker("bvwbjplbgvbhsrlpgdmjqwftvncz"), Some(23));
        assert_eq!(find_message_marker("nppdvjthqldpwncqszvftbrmjlhg"), Some(23));
        assert_eq!(find_message_marker("nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg"), Some(29));
        assert_eq!(find_message_marker("zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw"), Some(26));
    }
}
