use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, PartialEq, Debug)]
struct DirHandle(usize);

#[derive(Clone, Copy)]
struct FileHandle(usize);

struct Filesystem {
    dirs: Vec<Dir>,
    files: Vec<File>,
}

struct Dir {
    parent: Option<DirHandle>,
    name: String,
    dirs: Vec<DirHandle>,
    files: Vec<FileHandle>,
}

struct File {
    name: String,
    size: u32,
}

#[derive(PartialEq, Debug)]
struct DirSize<'a> {
    name: &'a str,
    size: u32,
}

impl Filesystem {
    pub fn new() -> Self {
        let mut fs = Self { dirs: Vec::new(), files: Vec::new() };
        let root = Dir {
            parent: None,
            name: "root".to_string(),
            dirs: Vec::new(),
            files: Vec::new(),
        };
        fs.dirs.push(root);
        fs
    }

    fn from_lines<I>(lines: I) -> Result<Self, String>
    where
        I: Iterator,
        I::Item: AsRef<str>,
    {
        let mut fs = Self::new();
        let mut wd = fs.root();
        for line in lines {
            let line = line.as_ref();
            let fields = line.split_whitespace().collect::<Vec<&str>>();
            match fields[..] {
                ["$", "cd", "/"] => {
                    wd = fs.root();
                },
                ["$", "cd", ".."] => {
                    wd = fs.dir_ref(wd).parent();
                },
                ["$", "cd", dir] => {
                    wd = fs.find_dir(wd, dir).expect("dir not found");
                },
                ["$", "ls"] => {},
                ["dir", dir] => {
                    fs.add_dir(wd, dir.to_string());
                }
                [size, file] => {
                    let size = size.parse::<u32>().unwrap();
                    fs.add_file(wd, file.to_string(), size);
                }
                _ => return Err(format!("unexpected line: {}", line)),
            }
        }
        Ok(fs)
    }


    pub fn root(&self) -> DirHandle {
        DirHandle(0)
    }

    pub fn add_dir(&mut self, parent: DirHandle, name: String) -> DirHandle {
        let handle = DirHandle(self.dirs.len());
        self.dirs.push(Dir::new(parent, name));
        self.dirs[parent.0].dirs.push(handle);
        handle
    }

    pub fn add_file(&mut self, parent: DirHandle, name: String, size: u32) -> FileHandle {
        let handle = FileHandle(self.files.len());
        self.files.push(File { name, size });
        self.dirs[parent.0].files.push(handle);
        handle
    }

    pub fn dir_sizes(&self) -> Vec<DirSize<'_>> {
        let mut sizes: Vec<DirSize> = Vec::new();
        let _ = self._dir_size(self.root(), &mut sizes);
        sizes
    }

    fn _dir_size<'a>(&'a self, dir_handle: DirHandle, sizes: &mut Vec<DirSize<'a>>) -> u32 {
        let dir = self.dir_ref(dir_handle);
        let mut size: u32 = dir.files.iter().map(|fh| self.file_ref(*fh).size).sum::<u32>();
        size += dir.dirs.iter().map(|dh| self._dir_size(*dh, sizes)).sum::<u32>();
        sizes.push(DirSize::new(&dir.name, size));
        size
    }

    pub fn dir_ref(&self, handle: DirHandle) -> &Dir {
        &self.dirs[handle.0]
    }

    pub fn file_ref(&self, handle: FileHandle) -> &File {
        &self.files[handle.0]
    }

    pub fn find_dir(&self, dir: DirHandle, name: &str) -> Option<DirHandle> {
        self.dir_ref(dir).dirs.iter()
            .find(|&&dh| self.dir_ref(dh).name == name)
            .copied()
    }
}

impl Display for Filesystem {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        enum Node {
            Dir(DirHandle),
            File(FileHandle),
        }

        let mut stack: Vec<(u32, Node)> = Vec::new();
        stack.push((0, Node::Dir(self.root())));
        while let Some((lvl, node)) = stack.pop() {
            let indent = " ".repeat(4 * lvl as usize);
            match node {
                Node::Dir(dh) => {
                    let dir = self.dir_ref(dh);
                    writeln!(f, "{}- {} (dir)", indent, dir.name).unwrap();
                    for &fh in self.dir_ref(dh).files.iter().rev() {
                        stack.push((lvl + 1, Node::File(fh)));
                    }
                    for &dh in self.dir_ref(dh).dirs.iter().rev() {
                        stack.push((lvl + 1, Node::Dir(dh)));
                    }
                },
                Node::File(fh) => {
                    let file = self.file_ref(fh);
                    writeln!(f, "{}- {} (file, size={})", indent, file.name, file.size).unwrap();
                }
            }
        }
        Ok(())
    }
}

impl Dir {
    pub fn new(parent: DirHandle, name: String) -> Dir {
        Dir { parent: Some(parent), name, dirs: Vec::new(), files: Vec::new() }
    }

    pub fn parent(&self) -> DirHandle {
        self.parent.expect("tried to ascend past root")
    }
}

impl<'a> DirSize<'a> {
    pub fn new(name: &'a str, size: u32) -> DirSize {
        DirSize { name, size }
    }
}

fn part1<I>(lines: I) -> Result<u32, String>
where
    I: Iterator,
    I::Item: AsRef<str>,
{
    let fs = Filesystem::from_lines(lines)?;
    let dir_sizes = fs.dir_sizes();
    let sum = dir_sizes.iter().filter(|ds| ds.size <= 100000).map(|ds| ds.size).sum();
    Ok(sum)
}

fn part2<I>(lines: I) -> Result<u32, String>
where
    I: Iterator,
    I::Item: AsRef<str>,
{
    const TOTAL_SPACE: u32 = 70000000;
    const NEEDED_SPACE: u32 = 30000000;
    let fs = Filesystem::from_lines(lines)?;
    let dir_sizes = fs.dir_sizes();
    let used = dir_sizes.iter().find(|ds| ds.name == "root").unwrap().size;
    let available = TOTAL_SPACE - used;
    let need_to_free = NEEDED_SPACE - available;
    let mut big_enough: Vec<&DirSize> = dir_sizes.iter()
        .filter(|ds| ds.size >= need_to_free).collect();
    big_enough.sort_by_key(|&ds| ds.size);
    Ok(big_enough.first().unwrap().size)
}

const USAGE: &str = "\
day7 <opts> part1|part2

-h|--help
    show help
";

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(|a| a.as_str()).collect();
    if args.iter().any(|&a| a == "-h" || a == "--help") {
        print!("{}", USAGE);
        return Ok(());
    }
    match args[..] {
        ["part1"] => {
            let sum = part1(std::io::stdin().lines().map(|l| l.unwrap()))?;
            println!("{}", sum);
        },
        ["part2"] => {
            let size = part2(std::io::stdin().lines().map(|l| l.unwrap()))?;
            println!("{}", size);
        },
        _ => {
            print!("{}", USAGE);
            return Err("Must specify part1|part2".to_string());
        },
    };
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE: &str = "\
$ cd /
$ ls
dir a
14848514 b.txt
8504156 c.dat
dir d
$ cd a
$ ls
dir e
29116 f
2557 g
62596 h.lst
$ cd e
$ ls
584 i
$ cd ..
$ cd ..
$ cd d
$ ls
4060174 j
8033020 d.log
5626152 d.ext
7214296 k";

    fn filesystem() -> Filesystem {
        let mut fs = Filesystem::new();
        let a = fs.add_dir(fs.root(), "a".to_string());
        fs.add_file(a, "af1".to_string(), 3);
        fs.add_file(a, "af2".to_string(), 4);
        let b = fs.add_dir(a, "b".to_string());
        fs.add_file(b, "bf1".to_string(), 6);
        fs
    }

    #[test]
    fn find_dir() {
        let fs = filesystem();
        let a = fs.find_dir(fs.root(), "a").unwrap();
        assert_eq!(&fs.dir_ref(a).name, "a");
    }

    #[test]
    fn parent() {
        let fs = filesystem();
        let a = fs.find_dir(fs.root(), "a").unwrap();
        let parent = fs.dir_ref(a).parent.unwrap();
        assert_eq!(parent, fs.root());
    }

    #[test]
    fn dir_sizes() {
        let fs = Filesystem::from_lines(EXAMPLE.lines()).unwrap();
        let mut dir_sizes = fs.dir_sizes();
        dir_sizes.sort_by_key(|ds| ds.name);
        let mut iter = dir_sizes.iter();
        assert_eq!(iter.next(), Some(&DirSize::new("a", 94853)));
        assert_eq!(iter.next(), Some(&DirSize::new("d", 24933642)));
        assert_eq!(iter.next(), Some(&DirSize::new("e", 584)));
        assert_eq!(iter.next(), Some(&DirSize::new("root", 48381165)));
    }
}
