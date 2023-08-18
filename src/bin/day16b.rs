use std::cmp;
use std::collections::{HashMap, HashSet, BinaryHeap};
use std::error::Error;
use std::fmt;
use std::io::Read;
use std::str::FromStr;

use regex_lite::Regex;
use once_cell::unsync::Lazy;

const DEBUG: bool = false;

struct StateTree {
    states: Vec<State>,
    start: StateHandle,
    volcano: Volcano,
    shortest_paths: SquareArray,
}

struct State {
    parent: Option<StateHandle>,
    rooms: [RoomHandle; 2],
    choices: [Choice; 2],
    steps_left: u8,
    opened_valves: HashSet<RoomHandle>,
    pressure_released: usize,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Choice {
    Start,
    Move(RoomHandle, usize),
    OpenValve,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct StateHandle(usize);

impl StateTree {
    fn new(volcano: Volcano) -> Self {
        let start_room = volcano.handle_for["AA"];
        let states = vec![State {
            parent: None,
            rooms: [start_room, start_room],
            choices: [Choice::Start, Choice::Start],
            steps_left: 26,
            opened_valves: HashSet::new(),
            pressure_released: 0,
        }];

        let shortest_paths = shortest_paths(&volcano.graph);

        StateTree {
            states,
            start: StateHandle(0),
            volcano,
            shortest_paths,
        }
    }

    fn new_state(&self, parent: StateHandle, choices: [Choice; 2]) -> State {
        let parent_state = self.get(parent);
        let steps_left = parent_state.steps_left - 1;

        let mut rooms: [RoomHandle; 2] = parent_state.rooms;
        for i in 0..choices.len() {
            if let Choice::Move(next, dist) = choices[i] {
                if dist == 0 {
                    rooms[i] = next;
                }
            }
        }

        let mut opened_valves = parent_state.opened_valves.clone();
        let mut pressure_released = parent_state.pressure_released;
        for (&choice, &room) in choices.iter().zip(parent_state.rooms.iter()) {
            let Choice::OpenValve = choice else {
                continue;
            };
            opened_valves.insert(room);
            let flow = self.volcano.flow_for[&room];
            pressure_released += flow * (steps_left as usize)
        }

        State {
            parent: Some(parent),
            rooms,
            choices,
            steps_left,
            opened_valves,
            pressure_released,
        }
    }

    fn add(&mut self, state: State) -> StateHandle {
        self.states.push(state);
        StateHandle(self.states.len() - 1)
    }

    fn get(&self, sh: StateHandle) -> &State {
        &self.states[sh.0]
    }

    fn branch_and_bound(&mut self) -> StateHandle {
        let mut queue: BinaryHeap<(usize, StateHandle)> = BinaryHeap::new();
        let mut best: StateHandle = self.start;
        let mut best_score: usize = self.get(self.start).pressure_released;

        let mut choices_a: Vec<Choice> = Vec::new();
        let mut choices_b: Vec<Choice> = Vec::new();
        let mut combos: Vec<[Choice; 2]> = Vec::new();

        queue.push((self.upper_bound(self.get(self.start)), self.start));

        let mut nstates: usize = 0;
        while let Some((upper_bound, sh)) = queue.pop() {
            nstates += 1;
            let state = self.get(sh);
            if DEBUG {
                self.print_state(state, upper_bound, best_score);
            }

            // Prune low-scoring branches, since our best score may have changed since it was
            // pushed.
            if upper_bound <= best_score {
                continue;
            }

            // Update the best state, maybe.
            if state.pressure_released > best_score {
                best = sh;
                best_score = state.pressure_released;
            }

            // We can't do anything useful at this point.
            if state.steps_left == 1 {
                continue;
            }

            // Queue all possible new states.
            choices_a.clear();
            choices_b.clear();
            self.push_new_choices(&mut choices_a, state, 0);
            self.push_new_choices(&mut choices_b, state, 1);
            for &a in &choices_a {
                for &b in &choices_b {
                    if state.rooms[0] == state.rooms[1] {
                        // Don't have both agents start opening the same valve.
                        if a == Choice::OpenValve && b == Choice::OpenValve {
                            continue;
                        }

                        // If the agents are in the same room 1 moving to B and 2 moving to C is
                        // the same as 1 -> C and 2 -> B, so skip it.
                        if let (Choice::Move(_, _), Choice::Move(_, _)) = (a, b) {
                            if combos.iter().any(|&v| v == [b, a]) {
                                continue;
                            }
                        }
                    }
                    combos.push([a, b]);
                }
            }

            while let Some(choices) = combos.pop() {
                let new = self.new_state(sh, choices);
                let upper_bound = self.upper_bound(&new);
                if upper_bound <= best_score {
                    continue;  // Prune low-scoring branches.
                }
                let new_handle = self.add(new);
                queue.push((upper_bound, new_handle));
            }
        }
        if DEBUG {
            self.print_path(best);

        }
        println!("nstates={nstates}");
        best
    }

    fn print_state(&self, state: &State, upper_bound: usize, best: usize) {
        print!("[{}, {}] ", 
            self.volcano.name_for[&state.rooms[0]],
            self.volcano.name_for[&state.rooms[1]]);

        let print_choice = |choice| match choice {
            Choice::Start => print!("Start"),
            Choice::Move(rh, dist) => {
                let name = &self.volcano.name_for[&rh];
                print!("Move({name}, {dist})")
            },
            Choice::OpenValve => print!("OpenValve"),
        };
        print!("[");
        print_choice(state.choices[0]);
        print!(", ");
        print_choice(state.choices[1]);
        print!("] ");

        println!("steps_left={} upper={upper_bound} best={best} open={:?} rel={}",
            state.steps_left,
            state.opened_valves,
            state.pressure_released);
    }

    fn push_new_choices(&self, choices: &mut Vec<Choice>, state: &State, i: usize) {
        let room = state.rooms[i];

        // If we're in the middle of a multi-step move, we need to finish it.
        let choice = state.choices[i];
        if let Choice::Move(dst, dist) = choice {
            if dist > 0 {
                choices.push(Choice::Move(dst, dist - 1));
                return;
            }
        }

        if !state.opened_valves.contains(&room) && self.volcano.flow_for[&room] > 0 {
            choices.push(Choice::OpenValve);
        }
        for child in self.volcano.child_handles(room) {
            // Don't move back to the previous room without having done anything.
            if let Some(sh) = state.parent {
                let parent = self.get(sh);
                if parent.rooms[i] == child && parent.choices[i] != Choice::OpenValve {
                    continue;
                }
            }
            let dist = self.volcano.graph.get(state.rooms[i], child).unwrap();
            choices.push(Choice::Move(child, dist - 1));
        }
    }

    fn upper_bound(&self, state: &State) -> usize {
        let mut closed_valves: Vec<_> = self.volcano.flow_for
            .iter()
            .filter(|(rh, &flow)| flow > 0 && !state.opened_valves.contains(rh))
            .collect();
        // Sort by flow rate, descending.
        closed_valves.sort_by(|(_, a), (_, b)| b.cmp(a));

        let mut steps_left: usize = state.steps_left as usize;
        let mut released: usize = state.pressure_released;
        let mut closed_valves = closed_valves.iter();
        while steps_left > 0 {
            let mut valves_opened = 0;
            while valves_opened < 2 {
                let Some(&(&rh, flow)) = closed_valves.by_ref().next() else {
                    break;
                };
                let min_dist = cmp::min(
                    self.shortest_paths.get(state.rooms[0], rh).unwrap(),
                    self.shortest_paths.get(state.rooms[1], rh).unwrap(),
                );
                // Skip valves that are too far away.
                if min_dist >= steps_left {
                    continue;
                }
                released += cmp::min(
                    steps_left * flow,
                    (state.steps_left as usize - 1) * flow
                );
                valves_opened += 1;
            }
            // Open a valve and move to the next room.
            steps_left = steps_left.saturating_sub(2);
        }
        released
    }

    #[allow(unused)]
    fn print_path(&self, sh: StateHandle) {
        let mut states: Vec<&State> = Vec::new();
        let mut cur: Option<StateHandle> = Some(sh);
        while let Some(sh) = cur {
            let state = self.get(sh);
            states.push(state);
            cur = state.parent;
        }
        states.reverse();
        for state in states.iter() {
            self.print_state(state, self.upper_bound(state), 0);
        }
    }

    #[allow(unused)]
    fn print_path_choices(&self, sh: StateHandle) {
        let mut choices: Vec<[Choice; 2]> = Vec::new();
        let mut cur: Option<StateHandle> = Some(sh);
        while let Some(sh) = cur {
            let state = self.get(sh);
            choices.push(state.choices);
            cur = state.parent;
        }
        choices.reverse();
        let print_choice = |c| match c {
            Choice::Start => print!("start"),
            Choice::Move(rh, dist) => print!("Move({}, {dist})", self.volcano.name_for[&rh]),
            Choice::OpenValve => print!("OpenValve"),
        };
        for &[a, b] in choices.iter() {
            print_choice(a);
            print!(", ");
            print_choice(b);
            println!();
        }
    }
}


struct Volcano {
    graph: SquareArray,
    flow_for: HashMap<RoomHandle, usize>,
    name_for: HashMap<RoomHandle, String>,
    handle_for: HashMap<String, RoomHandle>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct RoomHandle(u8);

impl RoomHandle {
    fn as_usize(&self) -> usize {
        self.0 as usize
    }
}

impl fmt::Debug for RoomHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Volcano {
    #[allow(unused)]
    fn flow(&self, name: &str) -> usize {
        let i = self.handle_for[name];
        self.flow_for[&i]
    }

    #[allow(unused)]
    fn child_names(&self, name: &str) -> Vec<&str> {
        let src = self.handle_for[name];
        self.graph.row(src.as_usize()).iter().enumerate()
            .filter(|&(dst, w)| w.is_some() && src.as_usize() != dst)
            .map(|(i, _)| self.name_for[&RoomHandle(i as u8)].as_str())
            .collect::<Vec<_>>()
    }

    pub fn child_handles(&self, rh: RoomHandle) -> impl Iterator<Item=RoomHandle> + '_ {
        self.graph.row(rh.as_usize()).iter().enumerate()
            .filter(|(_, &w)| matches!(w, Some(w) if w > 0))
            .map(|(i, _)| RoomHandle(i as u8))
    }

    // Remove zero-flow rooms from the graph and update the weights of their neighbors
    // appropriately.
    fn compact(&mut self) {
        let start_room = self.handle_for["AA"];
        let zero_flow_rooms: Vec<RoomHandle> = self.flow_for.iter()
            .filter(|(&rh, &flow)| flow == 0 && rh != start_room)
            .map(|(&rh, _)| rh)
            .collect();
        let nrooms = self.graph.cols;
        let rooms = || (0..nrooms).map(|i| RoomHandle(i as u8));
        for zero in zero_flow_rooms {
            let other_rooms = || rooms().filter(|&room| room != zero);

            // Update weights of neighbors' edges.
            for room in other_rooms() {
                for child in other_rooms() {
                    if self.graph.get(zero, child).is_none() {
                        continue;
                    }
                    let direct = self.graph.get(room, child);
                    let b = self.graph.get(room, zero);
                    let c = self.graph.get(zero, child);
                    let mediated = if let (Some(b), Some(c)) = (b, c) {
                        Some(b + c)
                    } else {
                        None
                    };
                    self.graph.set(room, child, inner_min(direct, mediated));
                }
            }

            // Remove the zero-flow room.
            for child in rooms() {
                self.graph.set(zero, child, None);
                self.graph.set(child, zero, None);
            }
        }
    }
}

impl FromStr for Volcano {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // eg: Valve AA has flow rate=0; tunnels lead to valves DD, II, BB
        let line_re = Lazy::new(|| {
            Regex::new(r#"Valve ([A-Z]{2}) has flow rate=(\d+); tunnel(?:s)? lead(?:s)? to valve(?:s)? (.*)"#).unwrap()
        });
        let cols = s.lines().count();
        let mut graph = SquareArray::new(cols);
        let mut flow_for = HashMap::new();
        let mut name_for = HashMap::new();
        let mut handle_for: HashMap<String, RoomHandle> = HashMap::new();

        let get_handle = |name: &str, name_for: &mut HashMap<_, _>, handle_for: &mut HashMap<_, _>| {
            match handle_for.get(name) {
                Some(i) => *i,
                None => {
                    let i = RoomHandle(handle_for.len() as u8);
                    handle_for.insert(name.to_string(), i);
                    name_for.insert(i, name.to_string());
                    i
                }
            }
        };

        for line in s.lines() {
            let Some(caps) = line_re.captures(line) else {
                return Err(format!("unexpected line format: {line}").into());
            };
            let name: String = caps[1].to_string();
            let src = get_handle(&name, &mut name_for, &mut handle_for);

            let flow_rate: usize = caps[2].parse()?;
            flow_for.insert(src, flow_rate);

            graph.set(src, src, Some(0));

            let adjacent = &caps[3];
            for name in adjacent.split(", ") {
                let dst = get_handle(name, &mut name_for, &mut handle_for);
                graph.set(src, dst, Some(1));
            }
        }
        Ok(Volcano { graph, flow_for, name_for, handle_for })
    }
}


#[derive(Clone)]
struct SquareArray {
    cols: usize,
    data: Vec<Option<usize>>,
}

impl SquareArray {
    pub fn new(cols: usize) -> Self {
        let mut data = Vec::new();
        data.resize(cols * cols, None);
        Self { cols, data }
    }

    pub fn get_raw(&self, src: usize, dst: usize) -> Option<usize> {
        self.data[src * self.cols + dst]
    }

    pub fn get(&self, src: RoomHandle, dst: RoomHandle) -> Option<usize> {
        self.get_raw(src.as_usize(), dst.as_usize())
    }

    pub fn set_raw(&mut self, src: usize, dst: usize, v: Option<usize>) {
        self.data[src * self.cols + dst] = v;
    }

    pub fn set(&mut self, src: RoomHandle, dst: RoomHandle, v: Option<usize>) {
        self.set_raw(src.as_usize(), dst.as_usize(), v);
    }

    pub fn row(&self, y: usize) -> &[Option<usize>] {
        let start = y * self.cols;
        &self.data[start..(start + self.cols)]
    }
}

impl fmt::Display for SquareArray {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for x in 0..self.cols {
            for y in 0..self.cols {
                let w = self.get_raw(x, y);
                match w {
                    Some(w) => write!(f, "{:>2}", w)?,
                    _ => write!(f, "{:>2}", "-")?,
                }
            }
            writeln!(f)?
        }
        Ok(())
    }
}

fn shortest_paths(weights: &SquareArray) -> SquareArray {
    let mut min_weights = weights.clone();

    // extend_shortest_paths() kind of "squares" the matrix, so instead of needing to extend the
    // shortest paths for each neighbor (or n-1 times) to propagate weights fully, we instead only
    // need to square the weights lg(n -1) times.
    let mut i = 1;
    while i < min_weights.cols {
        i *= 2;
        extend_shortest_paths(&mut min_weights);
    }
    min_weights
}

// Do an analog of multiplying a matrix by itself, but with "min" instead. See Section 25.1 in
// Cormen et al's Introduction to Algorithms.
//
// It seems safe to update min_weights in place and avoid copies, since while operations in the
// same call to extend_shortest_paths() can depend on each other, the result converges, so taking
// advantage of intermediate result for some nodes but not others is fine: some nodes will just get
// to their smallest weight earlier.
fn extend_shortest_paths(min_weights: &mut SquareArray) {
    let n = min_weights.cols;
    for src in 0..n {
        for dst in 0..n {
            for mid in 0..n {  // "mid" is short for "middleman"
                let direct = min_weights.get_raw(src, dst);
                let b = min_weights.get_raw(src, mid);
                let c = min_weights.get_raw(mid, dst);
                let mediated = if let (Some(b), Some(c)) = (b, c) {
                    Some(b + c)
                } else {
                    None
                };
                let min = inner_min(direct, mediated);
                min_weights.set_raw(src, dst, min);
            }
        }
    }
}

fn inner_min<T: Ord>(a: Option<T>, b: Option<T>) -> Option<T> {
    match (a, b) {
        (Some(a), Some(b)) => Some(cmp::min(a, b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        _ => None,
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    Ok(println!("{}", part2(std::io::stdin().lock())?))
}

fn part2(r: impl Read) -> Result<usize, Box<dyn Error>> {
    let input = std::io::read_to_string(r)?;
    let mut volcano = Volcano::from_str(&input)?;
    volcano.compact();
    let mut state_tree = StateTree::new(volcano);
    let best = state_tree.branch_and_bound();
    Ok(state_tree.get(best).pressure_released)
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = "\
Valve AA has flow rate=0; tunnels lead to valves DD, II, BB
Valve BB has flow rate=13; tunnels lead to valves CC, AA
Valve CC has flow rate=2; tunnels lead to valves DD, BB
Valve DD has flow rate=20; tunnels lead to valves CC, AA, EE
Valve EE has flow rate=3; tunnels lead to valves FF, DD
Valve FF has flow rate=0; tunnels lead to valves EE, GG
Valve GG has flow rate=0; tunnels lead to valves FF, HH
Valve HH has flow rate=22; tunnel leads to valve GG
Valve II has flow rate=0; tunnels lead to valves AA, JJ
Valve JJ has flow rate=21; tunnel leads to valve II";

    #[test]
    fn test_volcano_from_str() {
        let volcano = Volcano::from_str(EXAMPLE).unwrap();
        assert_eq!(volcano.flow("BB"), 13);
        assert_eq!(volcano.flow("HH"), 22);
        assert_eq!(volcano.child_names("GG"), vec!["FF", "HH"]);
        assert_eq!(volcano.child_names("JJ"), vec!["II"]);
    }

    #[test]
    fn test_shortest_paths() {
        let volcano = Volcano::from_str(EXAMPLE).unwrap();
        let paths = shortest_paths(&volcano.graph);
        let want_matrix = vec![
        //  a  b  c  d  e  f  g  h  i  j
            0, 1, 2, 1, 2, 3, 4, 5, 1, 2,  // a
            1, 0, 1, 2, 3, 4, 5, 6, 2, 3,  // b
            2, 1, 0, 1, 2, 3, 4, 5, 3, 4,  // c
            1, 2, 1, 0, 1, 2, 3, 4, 2, 3,  // d
            2, 3, 2, 1, 0, 1, 2, 3, 3, 4,  // e
            3, 4, 3, 2, 1, 0, 1, 2, 4, 5,  // f
            4, 5, 4, 3, 2, 1, 0, 1, 5, 6,  // g
            5, 6, 5, 4, 3, 2, 1, 0, 6, 7,  // h
            1, 2, 3, 2, 3, 4, 5, 6, 0, 1,  // i
            2, 3, 4, 3, 4, 5, 6, 7, 1, 0,  // j
        ];
        let names = vec!["AA", "BB", "CC", "DD", "EE", "FF", "GG", "HH", "II", "JJ"];
        for (src_idx, src_name) in names.iter().enumerate() {
            for (dst_idx, dst_name) in names.iter().enumerate() {
                let got = dist(&volcano, &paths, src_name, dst_name);
                let want = want_matrix[src_idx * names.len() + dst_idx];
                assert_eq!(got, Some(want), "mismatch for {src_name}->{dst_name}");
            }
        }
    }

    fn dist(v: &Volcano, paths: &SquareArray, src: &str, dst: &str) -> Option<usize> {
        paths.get(v.handle_for[src], v.handle_for[dst])
    }

    #[test]
    fn test_compact() {
        let mut volcano = Volcano::from_str(EXAMPLE).unwrap();
        volcano.compact();

        let mut wants: HashMap<(&str, &str), usize> = HashMap::new();
        wants.insert(("AA", "AA"), 0);
        wants.insert(("AA", "DD"), 1);
        wants.insert(("AA", "BB"), 1);
        wants.insert(("AA", "JJ"), 2);
        wants.insert(("BB", "BB"), 0);
        wants.insert(("BB", "CC"), 1);
        wants.insert(("BB", "AA"), 1);
        wants.insert(("CC", "CC"), 0);
        wants.insert(("CC", "DD"), 1);
        wants.insert(("CC", "BB"), 1);
        wants.insert(("DD", "DD"), 0);
        wants.insert(("DD", "CC"), 1);
        wants.insert(("DD", "AA"), 1);
        wants.insert(("DD", "EE"), 1);
        wants.insert(("EE", "EE"), 0);
        wants.insert(("EE", "DD"), 1);
        wants.insert(("EE", "HH"), 3);
        wants.insert(("HH", "HH"), 0);
        wants.insert(("HH", "EE"), 3);
        wants.insert(("JJ", "JJ"), 0);
        wants.insert(("JJ", "AA"), 2);

        for src in volcano.name_for.values() {
            for dst in volcano.name_for.values() {
                let got = dist(&volcano, &volcano.graph, src, dst);
                let want = wants.get(&(src, dst)).copied();
                assert_eq!(got, want, "mismatch as ({src}, {dst})");
            }
        }
    }

    #[test]
    fn test_compact_shortest_paths() {
        let mut volcano = Volcano::from_str(EXAMPLE).unwrap();
        volcano.compact();
        let paths = shortest_paths(&volcano.graph);
        let want_matrix = vec![
        //  a  b  c  d  e  h  j
            0, 1, 2, 1, 2, 5, 2,  // a
            1, 0, 1, 2, 3, 6, 3,  // b
            2, 1, 0, 1, 2, 5, 4,  // c
            1, 2, 1, 0, 1, 4, 3,  // d
            2, 3, 2, 1, 0, 3, 4,  // e
            5, 6, 5, 4, 3, 0, 7,  // h
            2, 3, 4, 3, 4, 7, 0,  // j
        ];
        let names = vec!["AA", "BB", "CC", "DD", "EE", "HH", "JJ"];
        for (src_idx, src_name) in names.iter().enumerate() {
            for (dst_idx, dst_name) in names.iter().enumerate() {
                let got = dist(&volcano, &paths, src_name, dst_name);
                let want = want_matrix[src_idx * names.len() + dst_idx];
                assert_eq!(got, Some(want), "mismatch for {src_name}->{dst_name}");
            }
        }
    }

    #[test]
    fn test_part2() {
        let best = part2(EXAMPLE.as_bytes()).unwrap();
        assert_eq!(best, 1707);
    }
}
