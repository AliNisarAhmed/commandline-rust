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
    show_col1: bool,

    #[arg(help = "Suppress printing of column 2", short = '2', long)]
    show_col2: bool,

    #[arg(help = "Suppress printing of column 3", short = '3', long)]
    show_col3: bool,

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

    let print_output_line = |line1: &str, line2: &str, common: &str| {
        println!(
            "{}{}{}{}{}",
            line1, config.delimiter, line2, config.delimiter, common
        )
    };

    let _file1 = open(file1)?;
    let _file2 = open(file2)?;

    let mut file1_iter = _file1.lines().map(|l| l.unwrap());
    let mut file2_iter = _file2.lines().map(|l| l.unwrap());

    let mut file1_line = file1_iter.next();
    let mut file2_line = file2_iter.next();

    loop {
        match (&file1_line, &file2_line) {
            (Some(line1), Some(line2)) => match line1.cmp(&line2) {
                Ordering::Less => {
                    println!("{}", line1);
                    file1_line = file1_iter.next();
                }
                Ordering::Greater => {
                    println!("{}{}", &config.delimiter, line2);
                    file2_line = file2_iter.next();
                }
                Ordering::Equal => {
                    println!("{}{}{}", &config.delimiter, &config.delimiter, line1);
                    file1_line = file1_iter.next();
                    file2_line = file2_iter.next();
                }
            },
            (Some(line1), None) => {
                println!("{}", line1);
                file1_line = file1_iter.next();
            }
            (None, Some(line2)) => {
                println!("{}{}", &config.delimiter, line2);
                print_output_line("", &line2, "");
                file2_line = file2_iter.next();
            }
            (None, None) => break,
        }
    }

    Ok(())
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(
            File::open(filename).map_err(|e| format!("{}: {}", filename, e))?,
        ))),
    }
}
