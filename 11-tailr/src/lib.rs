use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader, Read, Seek, SeekFrom},
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
    let num_files = config.files.len();
    for (file_num, filename) in config.files.iter().enumerate() {
        match File::open(&filename) {
            Err(err) => eprintln!("{}: {}", filename, err),
            Ok(file) => {
                if !config.quiet && num_files > 1 {
                    println!(
                        "{}==> {} <==",
                        if file_num > 0 { "\n" } else { "" },
                        filename
                    )
                }

                let (total_lines, total_bytes) = count_lines_bytes(&filename)?;
                let file = BufReader::new(file);
                if let Some(num_bytes) = &config.bytes {
                    print_bytes(file, num_bytes, total_bytes)?;
                } else {
                    print_lines(file, &config.lines, total_lines)?
                }
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

fn print_lines(mut file: impl BufRead, num_lines: &TakeValue, total_lines: i64) -> MyResult<()> {
    if let Some(start) = get_starting_index(num_lines, total_lines) {
        let mut line_num = 0;
        let mut buf = Vec::new();
        loop {
            let bytes_read = file.read_until(b'\n', &mut buf)?;

            if bytes_read == 0 {
                break;
            }
            if line_num >= start {
                print!("{}", String::from_utf8_lossy(&buf));
            }
            line_num += 1;
            buf.clear();
        }
    }
    Ok(())
}

fn print_bytes<T: Read + Seek>(
    mut file: T,
    num_bytes: &TakeValue,
    total_bytes: i64,
) -> MyResult<()> {
    if let Some(start) = get_starting_index(&num_bytes, total_bytes) {
        file.seek(SeekFrom::Start(start))?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;
        if !buffer.is_empty() {
            print!("{}", String::from_utf8_lossy(&buffer));
        }
    }

    Ok(())
}

fn get_starting_index(take_val: &TakeValue, total: i64) -> Option<u64> {
    match take_val {
        TakeValue::PlusZero if total == 0 => None,
        TakeValue::PlusZero => Some((total - 1) as u64),
        TakeValue::TakeNum(k) if total == 0 || *k > total => None,
        TakeValue::TakeNum(k) if *k < 0 && k.abs() > total => Some(0),
        TakeValue::TakeNum(k) if *k < 0 => Some((total + *k) as u64),
        TakeValue::TakeNum(k) => Some((*k - 1) as u64),
    }
}

// -----------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use crate::{count_lines_bytes, get_starting_index, TakeValue};

    #[test]
    fn test_count_lines_bytes() {
        let res = count_lines_bytes("tests/inputs/one.txt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), (1, 24));

        let res = count_lines_bytes("tests/inputs/ten.txt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), (10, 49));
    }

    #[test]
    fn test_get_start_index() {
        // +0 from an empty file (0 lines/bytes) returns None
        assert_eq!(get_starting_index(&TakeValue::PlusZero, 0), None);

        // +0 from a nonempty file returns an index that
        // is one less than the number of lines/bytes
        assert_eq!(get_starting_index(&TakeValue::PlusZero, 1), Some(0));

        // Taking 0 lines/bytes returns None
        assert_eq!(get_starting_index(&TakeValue::TakeNum(1), 0), None);

        // Taking more lines/bytes than is available returns None
        assert_eq!(get_starting_index(&TakeValue::TakeNum(2), 1), None);

        // When starting line/byte is less than total lines/bytes,
        // return one less that the starting number
        assert_eq!(get_starting_index(&TakeValue::TakeNum(1), 10), Some(0));
        assert_eq!(get_starting_index(&TakeValue::TakeNum(2), 10), Some(1));
        assert_eq!(get_starting_index(&TakeValue::TakeNum(3), 10), Some(2));

        // When starting line/byte is negative and less than total,
        // return total - start
        assert_eq!(get_starting_index(&TakeValue::TakeNum(-1), 10), Some(9));
        assert_eq!(get_starting_index(&TakeValue::TakeNum(-2), 10), Some(8));
        assert_eq!(get_starting_index(&TakeValue::TakeNum(-3), 10), Some(7));

        // When starting line/byte is negative and more than total,
        // return 0 to print the whole file
        assert_eq!(get_starting_index(&TakeValue::TakeNum(-20), 10), Some(0));
    }
}
