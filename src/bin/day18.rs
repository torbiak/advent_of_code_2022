#![allow(dead_code)]

use std::collections::HashSet;
use std::str::FromStr;
use std::io::BufRead;
use std::error::Error;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct Point {
    x: i32,
    y: i32,
    z: i32,
}

impl Point {
    fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y , z }
    }
}

impl FromStr for Point {
    type Err = Box<dyn Error>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut coords: [i32; 3] = [0; 3];
        let mut fields = s.split(',');
        for v in coords.as_mut_slice() {
            let Some(v_str) = fields.next() else {
                return Err("s should have more commas".into());
            };
            *v = v_str.parse()?;
        }
        Ok(Self::new(coords[0], coords[1], coords[2]))
    }
}

const SIDES: [Point; 6] = [
    Point { x: 0, y: 0, z: 1 },
    Point { x: 0, y: 0, z: -1 },
    Point { x: -1, y: 0, z: 0 },
    Point { x: 1, y: 0, z: 0 },
    Point { x: 0, y: 1, z: 0 },
    Point { x: 0, y: -1, z: 0 },
];

fn part1(r: impl BufRead) -> Result<usize, Box<dyn Error>> {
    let voxels = read_voxels(r)?;
    let mut surface_area: usize = 0;
    for p in voxels.iter() {
        for side in SIDES {
            let neighbor = Point::new(p.x + side.x, p.y + side.y, p.z + side.z);
            if !voxels.contains(&neighbor) {
                surface_area += 1;
            }
        }
    }
    Ok(surface_area)
}

fn read_voxels(r: impl BufRead) -> Result<HashSet<Point>, Box<dyn Error>> {
    let mut voxels: HashSet<Point> = HashSet::new();
    for line in r.lines() {
        let p = Point::from_str(&line?)?;
        voxels.insert(p);
    }
    Ok(voxels)
}

fn part2(r: impl BufRead) -> Result<usize, Box<dyn Error>> {
    let voxels = read_voxels(r)?;
    let mut space = Space::new(&voxels);
    let mut surface_area: usize = 0;
    for p in voxels.iter() {
        for side in SIDES {
            let neighbor = Point::new(p.x + side.x, p.y + side.y, p.z + side.z);
            if !voxels.contains(&neighbor) && !space.is_contained(neighbor) {
                surface_area += 1;
            }
        }
    }
    Ok(surface_area)
}

struct Space<'a> {
    voxels: &'a HashSet<Point>,
    uncontained: HashSet<Point>,
    min: Point,
    max: Point,
}

impl<'a> Space<'a> {
    fn new(voxels: &'a HashSet<Point>) -> Self {
        let min_max_init = (i32::MAX, i32::MAX, i32::MAX, i32::MIN, i32::MIN, i32::MIN);
        let (lx, ly, lz, hx, hy, hz) = voxels.iter().fold(min_max_init, |(lx, ly, lz, hx, hy, hz), p| {
            (lx.min(p.x), ly.min(p.y), lz.min(p.z), hx.max(p.x), hy.max(p.y), hz.max(p.z))
        });
        let min = Point::new(lx, ly, lz);
        let max = Point::new(hx, hy, hz);
        Space {
            voxels,
            uncontained: HashSet::new(),
            min,
            max,
        }
    }

    fn is_outside_bounds(&self, p: Point) -> bool {
        p.x < self.min.x
        || p.y < self.min.y
        || p.z < self.min.z
        || p.x > self.max.x
        || p.y > self.max.y
        || p.z > self.max.z
    }

    // Do a stack-based depth-first search to see if we can find a way out of the bounds of the
    // given points.
    fn is_contained(&mut self, p: Point) -> bool {
        let mut stack: Vec<Point> = Vec::new();
        let mut pushed: HashSet<Point> = HashSet::new();
        stack.push(p);
        while let Some(p) = stack.pop() {
            for d in SIDES {
                let new = Point::new(p.x + d.x, p.y + d.y, p.z + d.z);
                if self.is_outside_bounds(new) || self.uncontained.contains(&new) {
                    for v in pushed.iter() {
                        self.uncontained.insert(*v);
                    }
                    return false;
                } else if pushed.contains(&new) || self.voxels.contains(&new) {
                    continue;
                } else {
                    stack.push(new);
                    pushed.insert(new);
                }
            }
        }
        true
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

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = "\
2,2,2
1,2,2
3,2,2
2,1,2
2,3,2
2,2,1
2,2,3
2,2,4
2,2,6
1,2,5
3,2,5
2,1,5
2,3,5";

    const SIMPLE_PART2: &str = "\
2,0,0
1,0,0
-1,0,0
0,1,0
0,-1,0
0,0,1
0,0,-1";

    #[test]
    fn test_part1() {
        assert_eq!(part1(EXAMPLE.as_bytes()).unwrap(), 64);

    }

    #[test]
    fn test_simple_part2() {
        assert_eq!(part2(SIMPLE_PART2.as_bytes()).unwrap(), 34);
    }

    #[test]
    fn test_part2() {
        assert_eq!(part2(EXAMPLE.as_bytes()).unwrap(), 58);
    }
}
