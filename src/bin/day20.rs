#![allow(dead_code)]

use std::io::BufRead;
use std::error::Error;
use std::fmt;

struct CircularList {
    head_idx: Option<ListIndex>,
    nodes: Vec<Node>,
}

type Int = i64;

#[derive(Clone, Copy)]
struct ListIndex(usize);

struct Node {
    val: Int,
    prev: ListIndex,
    next: ListIndex,
}

impl Node {
    fn new(val: Int, prev: ListIndex, next: ListIndex) -> Self {
        Self { val, prev, next }
    }
}

impl CircularList {
    fn new() -> Self {
        CircularList {
            head_idx: None,
            nodes: Vec::new(),
        }
    }

    fn len(&self) -> usize {
        self.nodes.len()
    }

    fn get(&self, idx: ListIndex) -> &Node {
        &self.nodes[idx.0]
    }

    fn get_mut(&mut self, idx: ListIndex) -> &mut Node {
        &mut self.nodes[idx.0]
    }

    fn push(&mut self, val: Int) {
        match self.head_idx {
            Some(head_idx) => {
                let head = self.get(head_idx);
                let tail_idx = head.prev;
                let new_idx = ListIndex(self.len());
                let new = Node::new(val, tail_idx, head_idx);
                self.nodes.push(new);
                let head = self.get_mut(head_idx);
                head.prev = new_idx;
                let tail = self.get_mut(tail_idx);
                tail.next = new_idx;
            },
            _ => {
                let new_idx = ListIndex(self.len());
                let new = Node::new(val, new_idx, new_idx);
                self.nodes.push(new);
                self.head_idx = Some(new_idx);
            }
        }
    }

    fn next_nodes(&self, idx: ListIndex) -> NextNodes {
        NextNodes { list: self, cur: self.get(idx) }
    }

    fn prev_nodes(&self, idx: ListIndex) -> PrevNodes {
        PrevNodes { list: self, cur: self.get(idx) }
    }

    fn nodes_from_zero(&self) -> Option<NodesFromZero> {
        let zero = self.next_nodes(self.head_idx?).find(|n| n.val == 0)?;
        Some(NodesFromZero::new(self, zero))
    }

    fn as_vec(&self) -> Option<Vec<Int>> {
        let iter = self.nodes_from_zero()?;
        Some(iter.map(|n| n.val).collect())
    }

    fn mix(&mut self) {
        for idx in 0..self.len() {
            self.mix_one(ListIndex(idx));
        }
    }

    fn mix_one(&mut self, start_idx: ListIndex) {
        // Disconnect `start` from its neighbors and connect the neighbors.
        {
            let start = self.get(start_idx);
            let next_idx = start.next;
            let prev_idx = start.prev;
            let next = self.get_mut(next_idx);
            next.prev = prev_idx;
            let prev = self.get_mut(prev_idx);
            prev.next = next_idx;
        }

        // Reinsert `start` at its new location.
        let start = self.get(start_idx);
        // While the mixed node is disconnected the length of the list is reduced by one.
        let len = self.len() - 1;
        let (prev, next) = if start.val >= 0 {
            let prev = self.next_nodes(start_idx).nth(start.val as usize % len).unwrap();
            let next = self.get(prev.next);
            (prev, next)
        } else {
            let next = self.prev_nodes(start_idx).nth(start.val.unsigned_abs() as usize % len).unwrap();
            let prev = self.get(next.prev);
            (prev, next)
        };
        let prev_idx = next.prev;
        let next_idx = prev.next;
        let start = self.get_mut(start_idx);
        start.prev = prev_idx;
        start.next = next_idx;
        let prev = self.get_mut(prev_idx);
        prev.next = start_idx;
        let next = self.get_mut(next_idx);
        next.prev = start_idx;
    }
}

impl From<&[Int]> for CircularList {
    fn from(vals: &[Int]) -> Self {
        let mut cl = CircularList::new();
        for &v in vals {
            cl.push(v);
        }
        cl
    }
}

impl fmt::Debug for CircularList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in 0..self.len() {
            if i != 0 {
                write!(f, " ")?;
            }
            let node = self.get(ListIndex(i));
            write!(f, "{}<({}:{})>{}", node.prev.0, i, node.val, node.next.0)?;
        }
        Ok(())
    }
}

struct NextNodes<'a> {
    list: &'a CircularList,
    cur: &'a Node,
}

impl<'a> Iterator for NextNodes<'a> {
    type Item = &'a Node;

    fn next(&mut self) -> Option<Self::Item> {
        let to_yield = self.cur;
        self.cur = self.list.get(self.cur.next);
        Some(to_yield)
    }
}

struct PrevNodes<'a> {
    list: &'a CircularList,
    cur: &'a Node,
}

impl<'a> Iterator for PrevNodes<'a> {
    type Item = &'a Node;

    fn next(&mut self) -> Option<Self::Item> {
        let to_yield = self.cur;
        self.cur = self.list.get(self.cur.prev);
        Some(to_yield)
    }
}

struct NodesFromZero<'a> {
    list: &'a CircularList,
    cur: &'a Node,
    yielded_zero: bool,
}

impl<'a> NodesFromZero<'a> {
    fn new(list: &'a CircularList, start: &'a Node) -> Self {
        NodesFromZero {
            list,
            cur: start,
            yielded_zero: false,
        }
    }
}

impl<'a> Iterator for NodesFromZero<'a> {
    type Item = &'a Node;

    fn next(&mut self) -> Option<Self::Item> {
        let to_yield = self.cur;
        if to_yield.val == 0 && self.yielded_zero {
            None
        } else {
            self.yielded_zero = true;
            self.cur = self.list.get(self.cur.next);
            Some(to_yield)
        }
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

fn part1(r: impl BufRead) -> Result<Int, Box<dyn Error>> {
    let a = read_ints(r)?;
    let mut cl: CircularList = a.as_slice().into();
    cl.mix();
    let sum = [1000usize, 2000, 3000]
        .iter()
        .map(|&v| {
            let node = cl.nodes_from_zero()
                .ok_or("list should contain zero")?
                .nth(v % cl.len())
                .unwrap();
            Ok(node.val)
        })
        .sum::<Result<_, Box<dyn Error>>>()?;
    Ok(sum)
}

fn part2(r: impl BufRead) -> Result<Int, Box<dyn Error>> {
    let mut a = read_ints(r)?;
    for v in a.iter_mut() {
        *v *= 811589153;
    }
    let mut cl: CircularList = a.as_slice().into();
    for _ in 0..10 {
        cl.mix();
    }
    let sum = [1000usize, 2000, 3000]
        .iter()
        .map(|&v| {
            let node = cl.nodes_from_zero()
                .ok_or("list should contain zero")?
                .nth(v % cl.len())
                .unwrap();
            Ok(node.val)
        })
        .sum::<Result<_, Box<dyn Error>>>()?;
    Ok(sum)
}

fn read_ints(r: impl BufRead) -> Result<Vec<Int>, Box<dyn Error>> {
    r.lines()
        .map(|line| {
            let line = line?;
            let n = line.parse::<Int>()?;
            Ok(n)
        })
        .collect::<Result<Vec<_>, _>>()
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = "\
1
2
-3
3
-2
0
4";

    #[test]
    fn test_part1() {
        assert_eq!(part1(EXAMPLE.as_bytes()).unwrap(), 3);
    }

    #[test]
    fn test_part2() {
        assert_eq!(part2(EXAMPLE.as_bytes()).unwrap(), 1623178306);
    }

    #[test]
    fn test_mix() {
        let mut cl: CircularList = vec![1, 2, -3, 3, -2, 0, 4].as_slice().into();
        cl.mix();
        assert_eq!(cl.as_vec().unwrap(), vec![0, 3, -2, 1, 2, -3, 4]);
    }

    fn mix_one(vec: Vec<Int>, idx: ListIndex) -> Vec<Int> {
        let mut cl: CircularList = vec.as_slice().into();
        //println!("{:?}", cl);
        cl.mix_one(idx);
        //println!("{:?}", cl);
        cl.as_vec().unwrap()
    }

    #[test]
    fn test_mix_one_zero() {
        let mixed = mix_one(vec![1, 2, -3, 3, -2, 0, 4], ListIndex(5));
        assert_eq!(mixed, vec![0, 4, 1, 2, -3, 3, -2]);
    }

    #[test]
    fn test_mix_one_forward_nowrap() {
        let mixed = mix_one(vec![1, 2, -3, 3, -2, 0, 4], ListIndex(0));
        assert_eq!(mixed, vec![0, 4, 2, 1, -3, 3, -2]);
    }

    #[test]
    fn test_mix_one_forward_wrap_before_start() {
        let mixed = mix_one(vec![1, 2, -3, 0, 3, 4, -2], ListIndex(5));
        assert_eq!(mixed, vec![0, 3, -2, 1, 2, -3, 4]);
    }

    #[test]
    fn test_mix_one_forward_wrap_to_start() {
        let mixed = mix_one(vec![1, 6, -3, 0, 3, 4, -2], ListIndex(1));
        assert_eq!(mixed, vec![0, 3, 4, -2, 1, 6, -3]);
    }

    #[test]
    fn test_mix_one_forward_wrap_after_start() {
        let mixed = mix_one(vec![1, 7, -3, 0, 3, 4, -2], ListIndex(1));
        assert_eq!(mixed, vec![0, 3, 4, -2, 1, -3, 7]);
    }

    #[test]
    fn test_mix_one_backward_nowrap() {
        let mixed = mix_one(vec![1, 2, -3, 3, -2, 0, 4], ListIndex(4));
        assert_eq!(mixed, vec![0, 4, 1, 2, -2, -3, 3]);
    }

    #[test]
    fn test_mix_one_backward_wrap_after_start() {
        let mixed = mix_one(vec![1, 2, -2, -3, 0, 3, 4], ListIndex(2));
        assert_eq!(mixed, vec![0, 3, 4, -2, 1, 2, -3]);
    }

    #[test]
    fn test_mix_one_backward_wrap_to_start() {
        let mixed = mix_one(vec![1, 2, -6, -3, 0, 3, 4], ListIndex(2));
        assert_eq!(mixed, vec![0, 3, 4, 1, 2, -6, -3]);
    }

    #[test]
    fn test_mix_one_backward_wrap_before_start() {
        let mixed = mix_one(vec![1, 2, -8, -3, 0, 3, 4], ListIndex(2));
        assert_eq!(mixed, vec![0, 3, 4, -8, 1, 2, -3]);
    }
}
