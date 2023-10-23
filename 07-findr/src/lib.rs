use crate::EntryType::*;
use clap::{ArgAction, Parser, ValueEnum};
use regex::Regex;
use std::{cmp::Ordering, error::Error};
use walkdir::{DirEntry, WalkDir};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum EntryType {
    Dir,
    File,
    Link,
}

#[derive(Clone, Debug)]
struct SizeFilter {
    ordering: Ordering,
    size: usize,
    unit: SizeUnit,
}

impl SizeFilter {
    fn get_size_in_bytes(&self) -> usize {
        match self.unit {
            SizeUnit::Bytes => self.size,
            SizeUnit::Kilobytes => self.size * 1024,
            SizeUnit::Megabytes => self.size * 1024 * 1024,
            SizeUnit::Gigabytes => self.size * 1024 * 1024 * 1024,
            SizeUnit::Terabytes => self.size * 1024 * 1024 * 1024 * 1024,
            SizeUnit::Petabytes => self.size * 1024 * 1024 * 1024 * 1024 * 1024,
        }
    }
}

#[derive(Debug, ValueEnum, Clone)]
enum SizeUnit {
    Bytes,
    Kilobytes,
    Megabytes,
    Gigabytes,
    Terabytes,
    Petabytes,
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

    #[arg(help = "filter on size", long, value_parser = parse_size)]
    size: Option<SizeFilter>,
}

fn parse_size(input: &str) -> Result<SizeFilter, String> {
    let re = Regex::new(r"([+-]?)(\d+)([ckMGTP]?)").unwrap();
    let caps = re.captures(input).unwrap();

    Ok(SizeFilter {
        ordering: parse_ordering(&caps[1]).unwrap(),
        size: parse_size_value(&caps[2]).unwrap(),
        unit: parse_unit(&caps[3]).unwrap(),
    })
}

fn parse_ordering(input: &str) -> Result<Ordering, String> {
    match input {
        "+" => Ok(Ordering::Greater),
        "-" => Ok(Ordering::Less),
        "" => Ok(Ordering::Equal),
        _ => Err(format!("illegal ordering option")),
    }
}

fn parse_unit(input: &str) -> Result<SizeUnit, String> {
    match input {
        "c" => Ok(SizeUnit::Bytes),
        "" => Ok(SizeUnit::Bytes),
        "k" => Ok(SizeUnit::Kilobytes),
        "M" => Ok(SizeUnit::Megabytes),
        "G" => Ok(SizeUnit::Gigabytes),
        "P" => Ok(SizeUnit::Petabytes),
        _ => Err(format!("illegal unit")),
    }
}

fn parse_size_value(input: &str) -> Result<usize, String> {
    input.parse().map_err(|_e| format!("illegal size value"))
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

    let size_filter = |entry: &DirEntry| {
        if let Some(size_config) = &config.size {
            let file_size = entry.metadata().unwrap().len() as usize;
            let size_in_filter = size_config.get_size_in_bytes();
            match size_config.ordering {
                Ordering::Equal => file_size == size_in_filter,
                Ordering::Less => file_size < size_in_filter,
                Ordering::Greater => file_size > size_in_filter,
            }
        } else {
            true
        }
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
            .filter(size_filter)
            .map(|entry| entry.path().display().to_string())
            .collect::<Vec<_>>();

        println!("{}", entries.join("\n"));
    }

    Ok(())
}
