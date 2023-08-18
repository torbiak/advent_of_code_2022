use std::cmp;
use std::collections::{HashMap, HashSet, BinaryHeap};
use std::error::Error;
use std::fmt;
use std::io::Read;
use std::str::FromStr;

use regex_lite::Regex;
use once_cell::unsync::Lazy;

struct StateTree {
    states: Vec<State>,
    start: StateHandle,
    volcano: Volcano,
    shortest_paths: SquareArray,
}

struct State {
    parent: Option<StateHandle>,
    room: RoomHandle,
    choice: Choice,
    steps_left: u8,
    opened_valves: HashSet<RoomHandle>,
    pressure_released: usize,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Choice {
    Start,
    Move(RoomHandle),
    OpenValve,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct StateHandle(usize);

impl StateTree {
    fn new(volcano: Volcano) -> Self {
        let states = vec![State {
            parent: None,
            room: volcano.handle_for["AA"],
            choice: Choice::Start,
            steps_left: 30,
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

    fn new_state(&self, parent: StateHandle, choice: Choice) -> State {
        let parent_state = self.get(parent);
        let steps_left = parent_state.steps_left - 1;
        State {
            parent: Some(parent),
            room: match choice {
                Choice::Move(room) => room,
                _ => parent_state.room,
            },
            choice,
            steps_left,
            opened_valves: match choice {
                Choice::OpenValve => {
                    let mut opened = parent_state.opened_valves.clone();
                    opened.insert(parent_state.room);
                    opened
                },
                _ => parent_state.opened_valves.clone(),
            },
            pressure_released: {
                let prev = parent_state.pressure_released;
                match choice {
                    Choice::OpenValve => {
                        let flow = self.volcano.flow_for[&parent_state.room];
                        prev + flow * (steps_left as usize)
                    },
                    _ => prev,
                }
            },
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
        let mut choices: Vec<Choice> = Vec::new();
        let start_room = self.get(self.start).room;

        // The upper bound just needs to be greater than 0, since we're popping this state right
        // away.
        queue.push((1, self.start));

        let mut nstates: usize = 0;
        while let Some((upper_bound, sh)) = queue.pop() {
            nstates += 1;
            //println!("room={:?} {:?} steps_left={} upper={} best={} open={:?} rel={}", state.room, state.choice, state.steps_left, upper_bound, best_score, state.opened_valves, state.pressure_released);

            // Prune low-scoring branches, since our best score may have changed since it was
            // pushed.
            if upper_bound <= best_score {
                continue;
            }

            let state = self.get(sh);

            // Update the best state, maybe.
            if state.pressure_released > best_score {
                best = sh;
                best_score = state.pressure_released;
            }

            // We can't do anything useful at this point.
            if state.steps_left == 1 {
                continue;
            }

            // Queue all possible new states from this room.
            //
            // Save up new states and push them all at once, since otherwise it's awkward to avoid
            // having both mutable and immutable borrows of self.
            if !state.opened_valves.contains(&state.room) && state.room != start_room {
                choices.push(Choice::OpenValve);
            }
            for child in self.volcano.child_handles(state.room) {
                // Don't move back to the previous room without having done anything.
                if let Some(sh) = state.parent {
                    let parent = self.get(sh);
                    if parent.room == child && parent.choice != Choice::OpenValve {
                        continue;
                    }
                }

                choices.push(Choice::Move(child));
            }
            while let Some(choice) = choices.pop() {
                let new = self.new_state(sh, choice);
                let upper_bound = self.upper_bound(&new);
                if upper_bound <= best_score {
                    continue;  // Prune low-scoring branches.
                }
                let new_handle = self.add(new);
                queue.push((upper_bound, new_handle));
            }
        }
        println!("nstates={nstates}");
        best
    }

    // To get an upper bound on the pressure released, we know how far closed valves are from our
    // current locations, but it's not feasible to take the distance between those rooms into
    // account because it's too complicated. (If it was feasible, we wouldn't need to use branch
    // and bound.) We're using two somewhat conflicting assumptions. We're pretending that the
    // rooms with closed valves are right next to each other in descending order of flow rate, so
    // it only takes 2 minutes to open a valve and move to the next one, but we're also acting like
    // we had to take the time to walk the minimum distance from the current room to a valve when
    // calculating the pressure released from it.
    fn upper_bound(&self, state: &State) -> usize {
        let mut closed_valves: Vec<_> = self.volcano.flow_for
            .iter()
            .filter(|(rh, &flow)| flow > 0 && !state.opened_valves.contains(rh))
            .map(|(&rh, &flow)| {
                let min_dist = self.shortest_paths.get(rh.0 as usize, state.room.0 as usize);
                (flow, min_dist)
            })
            .collect();
        // Sort by flow rate, descending.
        closed_valves.sort_by(|(a, _), (b, _)| b.cmp(a));

        let mut steps_left: usize = state.steps_left as usize;
        let mut released: usize = state.pressure_released;
        let mut closed_valves = closed_valves.iter();
        while steps_left > 0 {
            let Some(&(flow, min_dist)) = closed_valves.next() else {
                break;
            };

            // Skip valves that are too far away.
            if min_dist >= steps_left {
                continue;
            }

            // Move to the room with the valve, if needed.
            if min_dist > 0 {
                steps_left -= 1;
            }

            // Open the valve, if we have time.
            if steps_left > 0 {
                // The max a valve can contribute is the min of:
                // - if we went to it in order, assuming the valves are only 1 step apart, and used
                //   the time left after going to other valves. This overestimates most when
                //   there's a high-value valve far away, since it doesn't take their distance away
                //   into account.
                // - if we went to it first and used the time left after traveling directly to it.
                //   This overestimates most when there's many high-value valves nearby, since it
                //   doesn't take the time to close other valves into account.
                released += cmp::min(
                    steps_left * flow,
                    (state.steps_left as usize - min_dist - 1) * flow
                );
                steps_left -= 1;
            }
        }
        released
    }

    #[allow(unused)]
    fn print_path(&self, sh: StateHandle) {
        let mut choices: Vec<Choice> = Vec::new();
        let mut cur: Option<StateHandle> = Some(sh);
        while let Some(sh) = cur {
            let state = self.get(sh);
            choices.push(state.choice);
            cur = state.parent;
        }
        choices.reverse();
        for choice in choices.iter() {
            match choice {
                Choice::Start => println!("start"),
                Choice::Move(rh) => println!("Move({})", self.volcano.name_for[rh]),
                Choice::OpenValve => println!("OpenValve"),
            }
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
        self.graph.row(src.0 as usize).iter().enumerate()
            .filter(|(_, &w)| w == 1)
            .map(|(i, _)| self.name_for[&RoomHandle(i as u8)].as_str())
            .collect::<Vec<_>>()
    }

    pub fn child_handles(&self, rh: RoomHandle) -> impl Iterator<Item=RoomHandle> + '_ {
        self.graph.row(rh.0 as usize).iter().enumerate()
            .filter(|(_, &w)| w == 1)
            .map(|(i, _)| RoomHandle(i as u8))
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
        let mut graph = SquareArray::new(cols, usize::MAX);
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

            graph.set(src.0 as usize, src.0 as usize, 0);

            let adjacent = &caps[3];
            for name in adjacent.split(", ") {
                let dst = get_handle(name, &mut name_for, &mut handle_for);
                graph.set(src.0 as usize, dst.0 as usize, 1);
            }
        }
        Ok(Volcano { graph, flow_for, name_for, handle_for })
    }
}


#[derive(Clone)]
struct SquareArray {
    cols: usize,
    data: Vec<usize>,
}

impl SquareArray {
    pub fn new(cols: usize, initial_value: usize) -> Self {
        let mut data = Vec::new();
        data.resize(cols * cols, initial_value);
        Self { cols, data }
    }

    pub fn get(&self, x: usize, y: usize) -> usize {
        self.data[y * self.cols + x]
    }

    pub fn set(&mut self, x: usize, y: usize, v: usize) {
        self.data[y * self.cols + x] = v;
    }

    pub fn row(&self, y: usize) -> &[usize] {
        let start = y * self.cols;
        &self.data[start..(start + self.cols)]
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
                let min = cmp::min(
                    min_weights.get(src, dst),
                    min_weights.get(src, mid).saturating_add(min_weights.get(mid, dst))
                );
                min_weights.set(src, dst, min);
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    Ok(println!("{}", part1(std::io::stdin().lock())?))
}

fn part1(r: impl Read) -> Result<usize, Box<dyn Error>> {
    let input = std::io::read_to_string(r)?;
    let volcano = Volcano::from_str(&input)?;
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
                assert_eq!(got, want, "mismatch at {src_idx},{dst_idx}");
            }
        }
    }

    fn dist(v: &Volcano, paths: &SquareArray, src: &str, dst: &str) -> usize {
        let x = v.handle_for[src];
        let y = v.handle_for[dst];
        paths.get(x.0 as usize, y.0 as usize)
    }

    #[test]
    fn test_part1() {
        assert_eq!(part1(EXAMPLE.as_bytes()).unwrap(), 1651);
    }
}
