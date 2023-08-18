use std::fmt;
use std::collections::HashMap;
use std::io::BufRead;
use std::error::Error;

use regex_lite::Regex;
use once_cell::unsync::Lazy;

use Res::*;

type Uint = u16;

struct Blueprint {
    ore_bot: BotCosts,
    clay_bot: BotCosts,
    obsidian_bot: BotCosts,
    geode_bot: BotCosts,
}

#[derive(Default)]
struct BotCosts {
    ore: Uint,
    clay: Uint,
    obsidian: Uint,
}

#[derive(Clone, Copy, Default)]
struct Global {
    nstates: usize,
    best: Uint,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
enum Res {
    Ore, Clay, Obsidian, Geode, Nothing,
}

#[derive(Clone, Default)]
struct State {
    ore: Uint,
    clay: Uint,
    obsidian: Uint,
    geode: Uint,
    ore_bot: Uint,
    clay_bot: Uint,
    obsidian_bot: Uint,
    geode_bot: Uint,
    ticks_left: u8,
}

impl State {
    fn start_part1() -> Self {
        State {
            ticks_left: 24,
            ore_bot: 1,
            ..Self::default()
        }
    }

    fn start_part2() -> Self {
        State {
            ticks_left: 32,
            ore_bot: 1,
            ..Self::default()
        }
    }

    fn collect(&mut self) {
        self.ore += self.ore_bot;
        self.clay += self.clay_bot;
        self.obsidian += self.obsidian_bot;
        self.geode += self.geode_bot;
        self.ticks_left -= 1;
    }

    fn make_bot(&self, res: Res, bp: &Blueprint) -> Option<Self> {
        match res {
            Ore => self.make_ore_bot(bp),
            Clay => self.make_clay_bot(bp),
            Obsidian => self.make_obsidian_bot(bp),
            Geode => self.make_geode_bot(bp),
            Nothing => self.make_nothing(bp),
        }
    }

    fn make_ore_bot(&self, bp: &Blueprint) -> Option<Self> {
        if bp.ore_bot.ore > self.ore {
            return None;
        }
        let mut new = self.clone();
        new.ore -= bp.ore_bot.ore;
        new.collect();
        new.ore_bot += 1;
        Some(new)
    }

    fn make_clay_bot(&self, bp: &Blueprint) -> Option<Self> {
        if bp.clay_bot.ore > self.ore {
            return None;
        }
        let mut new = self.clone();
        new.ore -= bp.clay_bot.ore;
        new.collect();
        new.clay_bot += 1;
        Some(new)
    }

    fn make_obsidian_bot(&self, bp: &Blueprint) -> Option<Self> {
        if bp.obsidian_bot.ore > self.ore || bp.obsidian_bot.clay > self.clay {
            return None;
        }
        let mut new = self.clone();
        new.ore -= bp.obsidian_bot.ore;
        new.clay -= bp.obsidian_bot.clay;
        new.collect();
        new.obsidian_bot += 1;
        Some(new)
    }

    fn make_geode_bot(&self, bp: &Blueprint) -> Option<Self> {
        if !self.can_make_geode_bot(bp) {
            return None;
        }
        let mut new = self.clone();
        new.ore -= bp.geode_bot.ore;
        new.clay -= bp.geode_bot.clay;
        new.obsidian -= bp.geode_bot.obsidian;
        new.collect();
        new.geode_bot += 1;
        Some(new)
    }

    fn can_make_geode_bot(&self, bp: &Blueprint) -> bool {
        self.ore >= bp.geode_bot.ore
            && self.clay >= bp.geode_bot.clay
            && self.obsidian >= bp.geode_bot.obsidian
    }

    fn make_nothing(&self, bp: &Blueprint) -> Option<Self> {
        if self.can_make_geode_bot(bp) {
            return None;
        }
        let mut new = self.clone();
        new.collect();
        Some(new)
    }
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ore={} clay={} obs={} geode={} r_ore={} r_clay={} r_obs={} r_geo={}",
            self.ore,
            self.clay,
            self.obsidian,
            self.geode,
            self.ore_bot,
            self.clay_bot,
            self.obsidian_bot,
            self.geode_bot)
    }
}

fn geode_upper_bound(state: &State, bp: &Blueprint) -> Uint {
    // Duplicate collected resources so that each type of bot has its own pool to draw from, so
    // that the bot-building decisions for different resources don't depend on each other. Also,
    // bots of different types can be built in the same tick.

    // Map: (have_resource, for_bot) -> count
    let mut resources: HashMap<(Res, Res), Uint> = HashMap::new();
    collect_resource(&mut resources, Ore, state.ore);
    collect_resource(&mut resources, Clay, state.clay);
    collect_resource(&mut resources, Obsidian, state.obsidian);
    collect_resource(&mut resources, Geode, state.geode);

    let mut bots: HashMap<Res, Uint> = HashMap::new();
    bots.insert(Ore, state.ore_bot);
    bots.insert(Clay, state.clay_bot);
    bots.insert(Obsidian, state.obsidian_bot);
    bots.insert(Geode, state.geode_bot);

    let mut new_bots: Vec<Res> = Vec::new();
    for _ in (1..=state.ticks_left).rev() {
        // Start building any bots that we can.
        if bp.ore_bot.ore <= *resources.entry((Ore, Ore)).or_default() {
            new_bots.push(Ore);
            resources.entry((Ore, Ore)).and_modify(|n| *n -= bp.ore_bot.ore);
        }
        if bp.clay_bot.ore <= *resources.entry((Ore, Clay)).or_default() {
            new_bots.push(Clay);
            resources.entry((Ore, Clay)).and_modify(|n| *n -= bp.clay_bot.ore);
        }
        if bp.obsidian_bot.ore <= *resources.entry((Ore, Obsidian)).or_default()
            && bp.obsidian_bot.clay <= *resources.entry((Clay, Obsidian)).or_default()
        {
            new_bots.push(Obsidian);
            resources.entry((Ore, Obsidian)).and_modify(|n| *n -= bp.obsidian_bot.ore);
            resources.entry((Clay, Obsidian)).and_modify(|n| *n -= bp.obsidian_bot.clay);
        }
        if bp.geode_bot.ore <= *resources.entry((Ore, Geode)).or_default()
            && bp.geode_bot.clay <= *resources.entry((Clay, Geode)).or_default()
            && bp.geode_bot.obsidian <= *resources.entry((Obsidian, Geode)).or_default()
        {
            new_bots.push(Geode);
            resources.entry((Ore, Geode)).and_modify(|n| *n -= bp.geode_bot.ore);
            resources.entry((Clay, Geode)).and_modify(|n| *n -= bp.geode_bot.clay);
            resources.entry((Obsidian, Geode)).and_modify(|n| *n -= bp.geode_bot.obsidian);
        }

        // Collect resources with bots existing at beginning of tick.
        for res in [Geode, Obsidian, Clay, Ore] {
            if let Some(&nbots) = bots.get(&res) {
                collect_resource(&mut resources, res, nbots);
            }
        }

        // Add newly-built bots to inventory at end of tick.
        while let Some(bot) = new_bots.pop() {
            bots.entry(bot).and_modify(|n| *n += 1).or_insert(1);
        }
    }
    *resources.entry((Geode, Geode)).or_default()
}

fn collect_resource(resources: &mut HashMap<(Res, Res), Uint>, resource: Res, n: Uint) {
    for dst in [Geode, Obsidian, Clay, Ore] {
        resources.entry((resource, dst)).and_modify(|have| *have += n).or_insert(n);
    }
}

fn cracked_geodes(state: State, bp: &Blueprint, global: &mut Global) -> Uint {
    if state.ticks_left == 0 {
        return state.geode;
    }
    // Use a Branch and Bound approach, implemented using recursion.
    [Geode, Obsidian, Clay, Ore, Nothing]
        .into_iter()
        .filter_map(|m| {
            let new = state.make_bot(m, bp);
            let Some(new) = new else {
                return None;
            };
            let upper = geode_upper_bound(&new, bp);
            if upper <= global.best {
                return None;
            }
            //println!("left={} do={m:?} upper={upper} best={} {new}", new.ticks_left, global.best);
            global.nstates += 1;
            global.best = global.best.max(new.geode);
            Some(cracked_geodes(new, bp, global))
        })
        .max().unwrap_or(0)
}

fn read_blueprints(r: impl BufRead) -> Result<Vec<Blueprint>, Box<dyn Error>> {
    // eg: Blueprint 1: Each ore robot costs 4 ore. Each clay robot costs 4 ore. Each obsidian robot costs 4 ore and 18 clay. Each geode robot costs 4 ore and 9 obsidian.
    let line_re = Lazy::new(|| {
        Regex::new(r#"Blueprint (?:\d+): Each ore robot costs (\d+) ore. Each clay robot costs (\d+) ore. Each obsidian robot costs (\d+) ore and (\d+) clay. Each geode robot costs (\d+) ore and (\d+) obsidian."#).unwrap()

    });
    let no_cost = BotCosts::default();
    r.lines()
        .map(|line| {
            let line = line?;
            let Some(captures) = line_re.captures(&line) else {
                return Err("unexpected blueprint line format".into());
            };
            let strs: [&str; 6] = captures.extract().1;
            let nums: Vec<Uint> = strs.iter().map(|s| s.parse::<Uint>()).collect::<Result<Vec<_>, _>>()?;
            let [ore_ore, clay_ore, obs_ore, obs_clay, geo_ore, geo_obs] = nums[..] else {
                return Err("missing expected captures".into());
            };
            Ok(Blueprint {
                ore_bot: BotCosts { ore: ore_ore, ..no_cost },
                clay_bot: BotCosts { ore: clay_ore, ..no_cost },
                obsidian_bot: BotCosts { ore: obs_ore, clay: obs_clay, ..no_cost },
                geode_bot: BotCosts { ore: geo_ore, obsidian: geo_obs, ..no_cost },
            })
        })
        .collect::<Result<Vec<_>, _>>()
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

fn part1(r: impl BufRead) -> Result<Uint, Box<dyn Error>> {
    let blueprints = read_blueprints(r)?;
    let sum = blueprints.iter().enumerate().map(|(i, bp)| {
        let mut global = Global::default();
        let geodes = cracked_geodes(State::start_part1(), bp, &mut global);
        let quality = (i as Uint + 1) * geodes;
        #[allow(clippy::let_and_return)]
        quality
    }).sum();
    Ok(sum)
}

fn part2(r: impl BufRead) -> Result<Uint, Box<dyn Error>> {
    let blueprints = read_blueprints(r)?;
    let product = blueprints.iter().take(3).map(|bp| {
        let mut global = Global::default();
        cracked_geodes(State::start_part2(), bp, &mut global)
    }).product();
    Ok(product)
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = "\
Blueprint 1: Each ore robot costs 4 ore. Each clay robot costs 2 ore. Each obsidian robot costs 3 ore and 14 clay. Each geode robot costs 2 ore and 7 obsidian.
Blueprint 2: Each ore robot costs 2 ore. Each clay robot costs 3 ore. Each obsidian robot costs 3 ore and 8 clay. Each geode robot costs 3 ore and 12 obsidian.";

    fn make_bluprint1() -> Blueprint {
        let no_cost = BotCosts::default();
        Blueprint {
            ore_bot: BotCosts { ore: 4, ..no_cost },
            clay_bot: BotCosts { ore: 2, ..no_cost },
            obsidian_bot: BotCosts { ore: 3, clay: 14, ..no_cost },
            geode_bot: BotCosts { ore: 2, obsidian: 7, ..no_cost },
        }
    }

    #[test]
    fn test_geode_upper_bound() {
        let state = State {
            geode_bot: 1,
            ticks_left: 5,
            ..State::default()
        };
        let blueprint = make_bluprint1();
        assert_eq!(geode_upper_bound(&state, &blueprint), 5);
    }

    #[test]
    fn test_cracked_geodes() {
        let start = State::start_part1();
        let blueprint = make_bluprint1();
        let mut global = Global::default();
        let max = cracked_geodes(start, &blueprint, &mut global);
        assert_eq!(max, 9);
    }

    #[test] #[ignore]
    fn test_part1() {
        assert_eq!(part1(EXAMPLE.as_bytes()).unwrap(), 33);
    }

    #[test] #[ignore]
    fn test_part2() {
        assert_eq!(part2(EXAMPLE.as_bytes()).unwrap(), 56 * 62);
    }
}
