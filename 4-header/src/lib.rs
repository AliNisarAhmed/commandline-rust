use clap::{ArgGroup, Parser};
use std::cmp::max;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Parser, Debug)]
#[command(name = "headr")]
#[command(author = "Ali Ahmed")]
#[command(version = "1.0")]
#[command(about = "Rust cat", long_about = None)]
#[command(group(
        ArgGroup::new("flags")
            .required(false)
            .multiple(false)
            .args(["lines", "bytes"])
        ))]
pub struct Config {
    #[arg(help = "Input file(s)", required = false, default_value = "-")]
    files: Vec<String>,

    #[arg(
        help = "Number of lines to print",
        short = 'n',
        default_value_t = 10,
        allow_hyphen_values = true
    )]
    lines: isize,

    #[arg(help = "Number of bytes to print", short = 'c', value_parser = parse_positive_int)]
    bytes: Option<usize>,
}

pub fn get_args() -> MyResult<Config> {
    Ok(Config::parse())
}

pub fn run(config: Config) -> MyResult<()> {
    if config.files.len() > 1 {
        print!("{}", print_multiple_files(&config))
    } else {
        let filename = &config.files.first().unwrap();
        print!("{}", print_single_file(filename, &config))
    }
    Ok(())
}

// ----

fn print_multiple_files(config: &Config) -> String {
    config
        .files
        .iter()
        .map(|filename| {
            let mut file_header = make_file_header(&filename);
            file_header.push_str(&print_single_file(filename, &config).to_string());
            file_header
        })
        .collect::<Vec<String>>()
        .join("\n")
}

fn make_file_header(filename: &String) -> String {
    format!("==> {} <==\n", filename)
}

fn print_single_file(filename: &String, config: &Config) -> String {
    match open(&filename) {
        Err(err) => {
            eprintln!("{}: {}", filename, err);
            String::new()
        }
        Ok(mut file) => {
            if let Some(n) = config.bytes {
                let mut buf = vec![];
                let mut chunk = file.take(n as u64);
                chunk.read_to_end(&mut buf).unwrap();
                String::from_utf8_lossy(&mut buf).to_string()
            } else {
                let mut result = String::new();
                let mut line = String::new();
                let mut lines_to_read = config.lines;

                loop {
                    let bytes = file.read_line(&mut line).unwrap();
                    // when read_line reaches EOF it returns 0 bytes
                    if bytes == 0 || (config.lines > 0 && lines_to_read <= 0) {
                        break;
                    }
                    result.push_str(&line);
                    line.clear();
                    lines_to_read -= 1;
                }

                if config.lines > 0 {
                    return result;
                }

                let num_lines: isize = result.clone().lines().count() as isize;
                let lines_to_take = max(0, num_lines.wrapping_add(config.lines as isize)) as usize;
                let mut result = result
                    .lines()
                    .take(lines_to_take)
                    .collect::<Vec<&str>>()
                    .join("\n");
                if lines_to_take > 0 {
                    result.push_str("\n");
                }
                result
            }
        }
    }
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}

pub fn parse_positive_int(val: &str) -> Result<usize, String> {
    match val.parse() {
        Ok(n) if n > 0 => Ok(n),
        Ok(n) if n <= 0 => Err(format!("value must be positive")),
        _ => Err(format!("Illegal count -- {}", val)),
    }
}
