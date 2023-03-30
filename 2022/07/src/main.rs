use std::collections::HashMap;
use std::io::Read;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum CdCommand<'src> {
    Root,
    Parent,
    Named(&'src str),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct LsCommand;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum LsOutputLine<'src> {
    Dir(&'src str),
    File(u64, &'src str),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Input<'src> {
    Cd(CdCommand<'src>),
    Ls(LsCommand, Vec<LsOutputLine<'src>>),
}

#[derive(Debug, Default)]
struct Dir<'src>(HashMap<&'src str, FSItem<'src>>);

impl<'a> Dir<'a> {
    fn size(&self) -> u64 {
        self.0.values().map(|f| f.size()).sum()

    }
}

#[derive(Debug)]
enum FSItem<'src> {
    Dir(Dir<'src>),
    Ordinary(u64),
}

impl<'a> FSItem<'a> {
    fn size(&self) -> u64{
        match self {
            FSItem::Dir(d) => {
                d.size()
            },
            FSItem::Ordinary(i) => *i,
        }
    }
}

mod nom_parse {
    use super::*;
    use nom::{
        branch::alt,
        bytes::complete::tag,
        character::complete::{char, line_ending, satisfy, space0, u64},
        combinator::{map, recognize, value},
        multi::{many0, many1},
        sequence::{delimited, pair, preceded, terminated, tuple},
        IResult,
    };

    fn prompt(s: &str) -> IResult<&str, ()> {
        value((), pair(char('$'), space0))(s)
    }

    fn token(s: &str) -> IResult<&str, &str> {
        delimited(
            space0,
            recognize(many1(satisfy(|c| !c.is_ascii_whitespace()))),
            space0,
        )(s)
    }

    fn cd(s: &str) -> IResult<&str, Input> {
        fn detail(s: &str) -> IResult<&str, CdCommand> {
            map(preceded(tag("cd"), token), |s| match s {
                "/" => CdCommand::Root,
                ".." => CdCommand::Parent,
                dirname => CdCommand::Named(dirname),
            })(s)
        }
        map(delimited(prompt, detail, line_ending), Input::Cd)(s)
    }

    fn ls_output(s: &str) -> IResult<&str, Vec<LsOutputLine>> {
        fn line(s: &str) -> IResult<&str, LsOutputLine> {
            fn dir(s: &str) -> IResult<&str, LsOutputLine> {
                map(preceded(tag("dir"), token), LsOutputLine::Dir)(s)
            }
            fn file(s: &str) -> IResult<&str, LsOutputLine> {
                map(tuple((u64, token)), |(sz, name)| {
                    LsOutputLine::File(sz, name)
                })(s)
            }
            terminated(alt((dir, file)), line_ending)(s)
        }
        many0(line)(s)
    }
    fn ls(s: &str) -> IResult<&str, Input> {
        fn cmd(s: &str) -> IResult<&str, LsCommand> {
            value(LsCommand, tuple((prompt, tag("ls"), line_ending)))(s)
        }
        map(tuple((cmd, ls_output)), |(c, i)| Input::Ls(c, i))(s)
    }
    pub(crate) fn parse(s: &str) -> IResult<&str, Vec<Input>> {
        many1(alt((cd, ls)))(s)
    }

    #[cfg(test)]
    mod test {
        use super::*;
        #[test]
        fn test_cd() {
            assert_eq!(cd("$ cd /\n"), Ok(("", Input::Cd(CdCommand::Root))));
            assert_eq!(cd("$ cd ..\n"), Ok(("", Input::Cd(CdCommand::Parent))));
            assert_eq!(
                cd("$ cd dirname\n"),
                Ok(("", Input::Cd(CdCommand::Named("dirname"))))
            );
        }
    }
}

fn populate_file_system(s: Vec<Input>) -> Dir {
    let mut fs_stack = vec![("", Dir(HashMap::new()))];
    fn collapse<'a>(fs_stack: Vec<(&'a str, Dir<'a>)>) -> Dir<'a> {
        fs_stack
            .into_iter()
            .rev()
            .reduce(|(child_name, child), mut parent| {
                let old_dir = parent.1 .0.insert(child_name, FSItem::Dir(child));
                assert!(old_dir.is_none());
                parent
            })
            .expect("empty fs").1

    }
    for input in s {
        match input {
            Input::Cd(dir) => match dir {
                CdCommand::Root => {
                    fs_stack = vec![("", collapse(fs_stack))];
                }
                CdCommand::Parent => {
                    let child = fs_stack.pop().expect("empty fs");
                    let parent = match fs_stack.last_mut() {
                        Some(parent) => parent,
                        None => {
                            fs_stack = vec![child];
                            continue;
                        }
                    };
                    let (child_name, child) = child;
                    let old_dir = parent.1 .0.insert(child_name, FSItem::Dir(child));
                    assert!(old_dir.is_none(), "{old_dir:?}");
                }
                CdCommand::Named(dirname) => {
                    let current = fs_stack.last_mut().expect("empty fs");
                    let d = current.1.0.remove(&dirname).unwrap_or(FSItem::Dir(Dir::default()));
                    match d {
                        FSItem::Ordinary(_) => panic!("Bad dir structure"),
                        FSItem::Dir(d) => fs_stack.push((dirname, d)),
                    };
                }
            },
            Input::Ls(_, lines) => {
                let current_dir = fs_stack.last_mut().expect("empty fs");
                for line in lines {
                    match line {
                        LsOutputLine::Dir(name) => {
                            current_dir
                                .1
                                 .0
                                .entry(name)
                                .or_insert(FSItem::Dir(Dir(HashMap::new())));
                        }
                        LsOutputLine::File(sz, name) => {
                            let old_file = current_dir.1.0.insert(name, FSItem::Ordinary(sz));
                            if let Some(old_file) = old_file {
                                if let FSItem::Ordinary(old_sz) = old_file {
                                    assert_eq!(old_sz, sz)

                                } else {
                                    panic!("mismatch")

                                }
                            }
                        },
                    }
                }
            }
        }
    }
    collapse(fs_stack)
}

fn main() -> anyhow::Result<()> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;
    let (_, i) = nom_parse::parse(&input).map_err(|e| e.to_owned())?;
    let fs = populate_file_system(i);
    println!("q1: {}", q1(&fs));
    println!("q2: {}", q2(&fs));
    Ok(())
}

fn dir_sizes<'a>(root: &'a Dir) -> impl Iterator<Item=u64> + 'a {
    let mut dir_stack = vec![root];
    std::iter::from_fn(move || {
        let dir = dir_stack.pop()?;
        let dir_size = dir.size();
        dir_stack.extend(
            dir.0.values().filter_map(|item| {
                if let FSItem::Dir(dir) = item {
                    Some(dir)
                } else {
                    None
                }
            })
        );
        Some(dir_size)
    })

}

fn q1(root: &Dir) -> u64 {
    dir_sizes(root).filter(|&s| s <= 100_000).sum()
}

fn q2(root: &Dir) -> u64 {
    let total_size = root.size();
    let empty_space = 70_000_000u64.checked_sub(total_size).unwrap();
    let required_space = 30_000_000u64.checked_sub(empty_space).unwrap();
    dir_sizes(root).filter(|&x| x >= required_space).min().unwrap()
}
