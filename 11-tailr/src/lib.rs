use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader},
    str::FromStr,
};

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
    #[arg(short = 'c', long, value_parser = parse_byte )]
    bytes: Option<TakeValue>,

    #[arg(short = 'n', long, value_parser = parse_line, default_value = "-10")]
    lines: TakeValue,

    #[arg(required = true)]
    files: Vec<String>,

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
                    .map(|v| i64::wrapping_neg(v))
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
    for filename in config.files {
        match File::open(&filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(_) => {
                let (total_lines, total_bytes) = count_lines_bytes(&filename)?;
                println!(
                    "{} has {} lines and {} bytes",
                    filename, total_lines, total_bytes
                );
            }
        }
    }
    Ok(())
}

// ----------------------------------------------------------------------------------
fn count_lines_bytes(filename: &str) -> MyResult<(i64, i64)> {
    let mut file = BufReader::new(File::open(filename)?);
    let mut num_lines = 0;
    let mut num_bytes = 0;
    let mut buf = Vec::new();
    loop {
        let bytes_read = file.read_until(b'\n', &mut buf)?;
        if bytes_read == 0 {
            break;
        }
        num_lines += 1;
        num_bytes += bytes_read as i64;
        buf.clear();
    }
    Ok((num_lines, num_bytes))
}

// -----------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use crate::count_lines_bytes;

    #[test]
    fn test_count_lines_bytes() {
        let res = count_lines_bytes("tests/inputs/one.txt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), (1, 24));

        let res = count_lines_bytes("tests/inputs/ten.txt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), (10, 49));
    }
}
