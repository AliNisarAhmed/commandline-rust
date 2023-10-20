use crate::EntryType::*;
use clap::{ArgAction, Parser, ValueEnum};
use regex::Regex;
use std::error::Error;
use walkdir::{DirEntry, WalkDir};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum EntryType {
    Dir,
    File,
    Link,
}

#[derive(Parser, Debug)]
pub struct Config {
    #[arg(help = "Search paths", required = false, default_value = ".", action = ArgAction::Append)]
    paths: Vec<String>,

    #[arg(help = "Name", required = false, short = 'n', long = "name", value_parser = parse_name, num_args(1..))]
    names: Vec<Regex>,

    #[arg(help = "Entry type", value_enum, short = 't', long = "type", value_parser = parse_entry_type, num_args(1..))]
    entry_types: Vec<EntryType>,

    #[arg(help = "Descend at most this levels", long = "max-depth")]
    max_depth: Option<usize>,

    #[arg(
        help = "Do not apply any tests or actions at levels less than this",
        long = "min-depth"
    )]
    min_depth: Option<usize>,
}

fn parse_name(name: &str) -> Result<Regex, String> {
    Regex::new(&name).map_err(|_| format!("Invalid --name \"{}\"", name))
}

fn parse_entry_type(str: &str) -> Result<EntryType, String> {
    match str {
        "f" => Ok(File),
        "d" => Ok(Dir),
        "l" => Ok(Link),
        _ => Err(format!("[possible values: d, f, l]")),
    }
}

pub fn get_args() -> MyResult<Config> {
    Ok(Config::parse())
}

pub fn run(config: Config) -> MyResult<()> {
    let type_filter = |entry: &DirEntry| {
        config.entry_types.is_empty()
            || config
                .entry_types
                .iter()
                .any(|entry_type| match entry_type {
                    File => entry.file_type().is_file(),
                    Dir => entry.file_type().is_dir(),
                    Link => entry.file_type().is_symlink(),
                })
    };

    let name_filter = |entry: &DirEntry| {
        config.names.is_empty()
            || config
                .names
                .iter()
                .any(|re| re.is_match(&entry.file_name().to_string_lossy()))
    };

    for path in &config.paths {
        let mut walkdir = WalkDir::new(path);

        match (config.max_depth, config.min_depth) {
            (Some(max_depth), Some(min_depth)) if max_depth < min_depth => return Ok(()),
            (Some(max_depth), Some(min_depth)) => {
                walkdir = walkdir.min_depth(min_depth).max_depth(max_depth);
            }
            (Some(max_depth), None) => {
                walkdir = walkdir.max_depth(max_depth);
            }
            (None, Some(min_depth)) => {
                walkdir = walkdir.min_depth(min_depth);
            }
            _ => (),
        }

        let entries = walkdir
            .into_iter()
            .filter_map(|e| match e {
                Err(e) => {
                    eprintln!("{}", e);
                    None
                }
                Ok(entry) => Some(entry),
            })
            .filter(type_filter)
            .filter(name_filter)
            .map(|entry| entry.path().display().to_string())
            .collect::<Vec<_>>();

        println!("{}", entries.join("\n"));
    }

    Ok(())
}
