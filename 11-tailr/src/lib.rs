use std::{error::Error, str::FromStr};

use clap::{ArgGroup, Parser};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, PartialEq, Clone)]
enum TakeValue {
    // represents an argument of +0
    PlusZero,
    // represents a valid integer value
    TakeNum(i64),
}

#[derive(Debug, Parser)]
#[command(group(
        ArgGroup::new("flags")
            .required(false)
            .multiple(false)
            .args(["lines", "bytes"])
        ))]
pub struct Config {
    #[arg(required = true)]
    files: Vec<String>,

    #[arg(short = 'n', long, value_parser = parse_line, default_value = "-10")]
    lines: TakeValue,

    #[arg(short = 'c', long, value_parser = parse_byte )]
    bytes: Option<TakeValue>,

    #[arg(short = 'q', long)]
    quiet: bool,
}

fn parse_line(s: &str) -> Result<TakeValue, String> {
    FromStr::from_str(s).map_err(|_| format!("illegal line count -- {}", s))
}

fn parse_byte(s: &str) -> Result<TakeValue, String> {
    FromStr::from_str(s).map_err(|_| format!("illegal byte count -- {}", s))
}

#[derive(Debug, PartialEq, Eq)]
struct ParseTakeValueError;

impl FromStr for TakeValue {
    type Err = ParseTakeValueError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "+0" => Ok(TakeValue::PlusZero),
            pos if pos.starts_with("+") => Ok(TakeValue::TakeNum(
                s.parse::<i64>().map_err(|_| ParseTakeValueError)?,
            )),
            _ => Ok(TakeValue::TakeNum(
                s.parse::<i64>()
                    .map(|v| if v > 0 { v * -1 } else { v })
                    .map_err(|_| ParseTakeValueError)?,
            )),
        }
    }
}

// ---------------------------------------------------------------------------------
pub fn get_args() -> MyResult<Config> {
    Ok(Config::parse())
}

pub fn run(config: Config) -> MyResult<()> {
    dbg!(config);
    Ok(())
}
