use std::{
    cmp::Ordering,
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader},
};

use clap::Parser;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Parser, Debug)]
#[command(name = "commr")]
#[command(author = "Ali Ahmed")]
#[command(version = "1.0")]
#[command(about = "Rust commr", long_about = None)]
pub struct Config {
    #[arg(help = "First file, can be STDIN if file2 is not")]
    file1: String,

    #[arg(help = "Second file, can be STDIN if file1 is not")]
    file2: String,

    #[arg(help = "Suppress printing of column 1", short = '1', long)]
    suppress_col1: bool,

    #[arg(help = "Suppress printing of column 2", short = '2', long)]
    suppress_col2: bool,

    #[arg(help = "Suppress printing of column 3", short = '3', long)]
    suppress_col3: bool,

    #[arg(help = "Case-insensitive comparison of lines", short = 'i', long)]
    insensitive: bool,

    #[arg(help = "Output delimiter", short = 'd', long, default_value = "\t")]
    delimiter: String,
}

pub fn get_args() -> MyResult<Config> {
    Ok(Config::parse())
}

pub fn run(config: Config) -> MyResult<()> {
    let file1 = &config.file1;
    let file2 = &config.file2;

    if file1 == "-" && file2 == "-" {
        return Err(From::from("Both input files cannot be STDIN (\"-\")"));
    }

    let _file1 = open(file1)?;
    let _file2 = open(file2)?;

    let mut file1_iter = _file1.lines().map(|l| l.unwrap());
    let mut file2_iter = _file2.lines().map(|l| l.unwrap());

    let mut file1_line = file1_iter.next();
    let mut file2_line = file2_iter.next();

    loop {
        match (&file1_line, &file2_line) {
            (Some(line1), Some(line2)) => match compare_lines(line1, line2, config.insensitive) {
                Ordering::Less => {
                    print_line_1(line1, &config);
                    file1_line = file1_iter.next();
                }
                Ordering::Greater => {
                    print_line_2(line2, &config);
                    file2_line = file2_iter.next();
                }
                Ordering::Equal => {
                    print_line_3(line1, &config);
                    file1_line = file1_iter.next();
                    file2_line = file2_iter.next();
                }
            },
            (Some(line1), None) => {
                print_line_1(line1, &config);
                file1_line = file1_iter.next();
            }
            (None, Some(line2)) => {
                print_line_2(line2, &config);
                file2_line = file2_iter.next();
            }
            (None, None) => break,
        }
    }

    Ok(())
}

fn print_line_1(line1: &str, config: &Config) {
    if !config.suppress_col1 {
        println!("{}", line1)
    }
}

fn print_line_2(line2: &str, config: &Config) {
    if !config.suppress_col2 {
        if config.suppress_col1 {
            println!("{}", line2);
        } else {
            println!("{}{}", config.delimiter, line2);
        }
    }
}

fn print_line_3(line3: &str, config: &Config) {
    if !config.suppress_col3 {
        if config.suppress_col1 && config.suppress_col2 {
            println!("{}", line3);
        } else if config.suppress_col1 || config.suppress_col2 {
            println!("{}{}", config.delimiter, line3)
        } else {
            println!("{}{}{}", config.delimiter, config.delimiter, line3)
        }
    }
}

fn compare_lines(line1: &str, line2: &str, insensitive: bool) -> Ordering {
    if insensitive {
        line1.to_lowercase().cmp(&line2.to_lowercase())
    } else {
        line1.cmp(&line2)
    }
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(
            File::open(filename).map_err(|e| format!("{}: {}", filename, e))?,
        ))),
    }
}
