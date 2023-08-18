use std::cmp::Ordering;
use std::io::BufRead;
use std::io;

struct NestedList<T>
where
    T: Iterator<Item=u8>,
{
    stack: Vec<ListItem>,
    bytes: T,
    peeked: Option<u8>,
}

enum ListItem {
    Int(i32),
    ListStart,
    ListEnd,
}

impl<T> NestedList<T>
where
    T: Iterator<Item=u8>,
{
    fn new(bytes: T) -> Self {
        Self { stack: Vec::new(), bytes, peeked: None }
    }

    fn push(&mut self, item: ListItem) {
        self.stack.push(item);
    }

    fn next_byte(&mut self) -> Option<u8> {
        self.peeked.take().or_else(|| self.bytes.next())
    }
}

fn nested_list(s: &str) -> NestedList<impl Iterator<Item=u8> + '_> {
    NestedList::new(s.bytes())
}

impl<T> Iterator for NestedList<T>
where
    T: Iterator<Item=u8>,
{
    type Item = ListItem;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.stack.pop() {
            return Some(item);
        }

        loop {
            match self.next_byte() {
                Some(b'[') => {
                    return Some(ListItem::ListStart);
                },
                Some(b']') => {
                    return Some(ListItem::ListEnd);
                },
                Some(c) if c.is_ascii_digit() => {
                    let mut buf = String::new();
                    buf.push(c as char);
                    while let Some(c) = self.next_byte() {
                        if !c.is_ascii_digit() {
                            self.peeked = Some(c);
                            break;
                        }
                        buf.push(c as char);
                    }
                    return Some(ListItem::Int(buf.parse::<i32>().unwrap()));
                },
                Some(b',') => continue,
                Some(c) => panic!("unexpected byte: {}", c as char),
                None => return None,
            };
        }
    }
}

struct PacketPair<T> 
where
    T: Iterator<Item=io::Result<String>>,
{
    lines: T,
}

impl<T> PacketPair<T>
where
    T: Iterator<Item=io::Result<String>>,
{
    fn new(lines: T) -> Self {
        PacketPair { lines }
    }
}

impl<T> Iterator for PacketPair<T>
where
    T: Iterator<Item=io::Result<String>>,
{
    type Item = Result<(String, String), String>;

    fn next(&mut self) -> Option<Self::Item> {
        let lines: Result<Vec<_>, _> = (&mut self.lines).take(3).collect();
        match lines {
            Err(e) => Some(Err(e.to_string())),
            Ok(lines) => {
                let mut lines = lines.into_iter().filter(|l| !l.is_empty());
                let a = lines.next();
                let b = lines.next();
                match (a, b) {
                    (Some(a), Some(b)) => Some(Ok((a, b))),
                    (None, None) => None,
                    _ => Some(Err("input should contain blank-line-delimited groups of 2 lines".to_string())),
                }
            }
        }
    }
}

#[allow(unused)]
fn cmp_lists<T>(mut a: NestedList<T>, mut b: NestedList<T>) -> Ordering
where
    T: Iterator<Item=u8>,
{
    use ListItem::*;
    loop {
        let (cur_a, cur_b) = (a.next().unwrap(), b.next().unwrap());
        match (cur_a, cur_b) {
            (ListStart, ListStart) => (),
            (ListEnd, ListEnd) => (),
            (ListEnd, _) => return Ordering::Less,
            (_, ListEnd) => return Ordering::Greater,
            (Int(a), Int(b)) => {
                if a != b {
                    return a.cmp(&b);
                }
            },
            (Int(a_int), ListStart) => {
                a.push(ListEnd);
                a.push(Int(a_int));
                return cmp_lists(a, b);
            },
            (ListStart, Int(b_int)) => {
                b.push(ListEnd);
                b.push(Int(b_int));
                return cmp_lists(a, b);
            },
        }
    }
}

fn part1<T: BufRead>(r: T) -> Result<u32, String> {
    let sum: Result<u32, String> = PacketPair::new(r.lines())
        .enumerate()
        .map(|(i, r)| {
            let (a, b) = r?;
            if cmp_lists(nested_list(&a), nested_list(&b)) == Ordering::Less {
                Ok((i + 1) as u32)
            } else {
                Ok(0)
            }
        })
        .sum();
    sum
}

fn part2<T: BufRead>(r: T) -> Result<usize, String> {
    let lines: Vec<String> = r.lines().collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;
    let mut lines: Vec<String> = lines.into_iter().filter(|l| !l.is_empty()).collect();
    let div1 = "[[2]]".to_string();
    let div2 = "[[6]]".to_string();
    lines.push(div1.clone());
    lines.push(div2.clone());
    lines.sort_by(|a, b| cmp_lists(nested_list(a), nested_list(b)));

    let div1_index = lines.iter()
        .position(|x| x == &div1)
        .ok_or("divider packet 1 not found".to_string())?;
    let div2_index = lines.iter()
        .position(|x| x == &div2)
        .ok_or("divider packet 2 not found".to_string())?;
    Ok((div1_index + 1) * (div2_index + 1))
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
[1,1,3,1,1]
[1,1,5,1,1]

[[1],[2,3,4]]
[[1],4]

[9]
[[8,7,6]]

[[4,4],4,4]
[[4,4],4,4,4]

[7,7,7,7]
[7,7,7]

[]
[3]

[[[]]]
[[]]

[1,[2,[3,[4,[5,6,7]]]],8,9]
[1,[2,[3,[4,[5,6,0]]]],8,9]";

    #[test]
    fn test_cmp_lists() {
        let a = NestedList::new("[1,2]".bytes());
        let b = NestedList::new("[1,[2],3]".bytes());
        assert_eq!(cmp_lists(a, b), Ordering::Less);
    }

    #[test]
    fn test_cmp_lists_long() {
        let a_str = "[[10,[0,7,[],3,[1,6]],[[2,4,5,4]],[]],[],[6,6,[[2,6,7],7,[5],[8,4,10,4,8],[0]],[10],[]],[[[],[6,0,9,10,2],8,[0]]]]";
        let b_str = "[[[6]],[[3],[[]],[[0,6,8,9,5],[7,9,10,2]]],[],[[[1],[9],5],9,[[],[0],5,1,[5,0]],5]]";
        let a = NestedList::new(a_str.bytes());
        let b = NestedList::new(b_str.bytes());
        assert_eq!(cmp_lists(a, b), Ordering::Greater);
    }

    #[test]
    fn test_cmp_lists_multi_promotion() {
        let a = NestedList::new("[[3]]".bytes());
        let b = NestedList::new("[[[[],[]]]]".bytes());
        assert_eq!(cmp_lists(a, b), Ordering::Greater);
    }

    #[test]
    fn test_cmp_lists_multi_promotion_long() {
        let a = NestedList::new("[[3,2,4],[1,[2,3,[5,1,8],7,9]],[[4,[]]]]".bytes());
        let b = NestedList::new("[[[[],[],6],3]]".bytes());
        assert_eq!(cmp_lists(a, b), Ordering::Greater);
    }

    #[test]
    fn test_part1() {
        assert_eq!(part1(EXAMPLE.as_bytes()).unwrap(), 13);
    }

    #[test]
    fn test_part2() {
        assert_eq!(part2(EXAMPLE.as_bytes()).unwrap(), 140);
    }
}
