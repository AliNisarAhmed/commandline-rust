use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader},
    num::NonZeroUsize,
    ops::Range,
};

use clap::{arg, ArgGroup, Parser};
use regex::Regex;

type MyResult<T> = Result<T, Box<dyn Error>>;
type PositionList = Vec<Range<usize>>;

#[derive(Debug, Clone)]
pub enum Extract {
    Fields(PositionList),
    Bytes(PositionList),
    Chars(PositionList),
}

#[derive(Parser, Debug)]
#[command(name = "cutr")]
#[command(author = "Ali Ahmed")]
#[command(version = "1.0")]
#[command(about = "Rust cut utility", long_about = None)]
#[command(group(
        ArgGroup::new("flags")
            .required(true)
            .multiple(false)
            .args(["chars", "bytes", "fields"])
        ))]
pub struct Args {
    #[arg(help = "Input File(s) [default: -]", default_value = "-")]
    files: Vec<String>,

    #[arg(help = "Field delimiter [default:  ]", 
          short = 'd',
          long,
          default_value = "\t", 
          value_parser = parse_delimiter)]
    delimiter: u8,

    #[arg(help = "Selected bytes", short = 'b', value_parser = parse_bytes, required = false)]
    bytes: Option<Extract>,

    #[arg(help = "Selected Characters", short = 'c', value_parser = parse_chars, required = false)]
    chars: Option<Extract>,

    #[arg(help = "Selected fields", short = 'f', value_parser = parse_fields, required = false)]
    fields: Option<Extract>,
}
#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    delimiter: u8,
    extract: Extract,
}

fn parse_delimiter(input: &str) -> Result<u8, String> {
    let delim_bytes = input.as_bytes();
    if delim_bytes.len() != 1 {
        return Err(From::from(format!(
            "--delim \"{}\" must be a single byte",
            input
        )));
    }
    Ok(*delim_bytes.first().unwrap())
}

fn parse_bytes(input: &str) -> Result<Extract, String> {
    let pos_list = parse_pos(input).unwrap();

    Ok(Extract::Bytes(pos_list))
}

fn parse_chars(input: &str) -> Result<Extract, String> {
    Ok(Extract::Chars(parse_pos(input).unwrap()))
}

fn parse_fields(input: &str) -> Result<Extract, String> {
    Ok(Extract::Fields(parse_pos(input).unwrap()))
}

fn parse_index(input: &str) -> Result<usize, String> {
    let value_error = || format!("illegal list value: {}", input);

    input
        .starts_with("+")
        .then(|| Err(value_error()))
        .unwrap_or_else(|| {
            input
                .parse::<NonZeroUsize>()
                .map(|n| usize::from(n) - 1)
                .map_err(|_| value_error())
        })
}

fn parse_pos(range: &str) -> Result<PositionList, String> {
    let range_re = Regex::new(r"^(\d+)-(\d+)$").unwrap();

    range
        .split(',')
        .into_iter()
        .map(|val| {
            parse_index(val).map(|n| n..n + 1).or_else(|e| {
                range_re.captures(val).ok_or(e).and_then(|captures| {
                    let n1 = parse_index(&captures[1])?;
                    let n2 = parse_index(&captures[2])?;
                    if n1 >= n2 {
                        return Err(format!(
                            "First number in range ({}) \
                                        must be lower than the second number ({})",
                            n1 + 1,
                            n2 + 1
                        ));
                    }

                    Ok(n1..n2 + 1)
                })
            })
        })
        .collect::<Result<_, _>>()
        .map_err(From::from)
}

pub fn get_args() -> MyResult<Config> {
    let args = Args::parse();

    Ok(Config {
        files: args.files,
        delimiter: args.delimiter,
        extract: args.bytes.or(args.chars).or(args.fields).unwrap(),
    })
}

pub fn run(config: Config) -> MyResult<()> {
    for filename in &config.files {
        match open(filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(file) => match &config.extract {
                Extract::Bytes(bytes_pos) => {
                    for line in file.lines() {
                        println!("{}", extract_bytes(&line?, bytes_pos))
                    }
                }
                Extract::Chars(char_pos) => {
                    for line in file.lines() {
                        println!("{}", extract_chars(&line?, char_pos))
                    }
                }
                Extract::Fields(field_pos) => {}
            },
        }
    }

    Ok(())
}

pub fn extract_chars(line: &str, char_pos: &[Range<usize>]) -> String {
    let chars: Vec<_> = line.chars().collect();

    char_pos
        .iter()
        .cloned()
        .flat_map(|range| range.filter_map(|i| chars.get(i)))
        .collect()
}

pub fn extract_bytes(line: &str, byte_pos: &[Range<usize>]) -> String {
    let bytes = line.as_bytes();

    let selected: Vec<_> = byte_pos
        .iter()
        .cloned()
        .flat_map(|range| range.filter_map(|i| bytes.get(i)))
        .copied()
        .collect();

    String::from_utf8_lossy(&selected).into_owned()
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}