use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader},
};

use clap::{ArgGroup, Parser};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Parser, Debug)]
#[command(name = "catr")]
#[command(author = "Ali Ahmed")]
#[command(version = "1.0")]
#[command(about = "Rust cat", long_about = None)]
#[command(group(
        ArgGroup::new("flags")
            .required(false)
            .multiple(false)
            .args(["number_lines", "number_nonblank_lines"])
        ))]
pub struct Config {
    #[arg(help = "Input File(s)", required = false, default_value = "-")]
    files: Vec<String>,

    #[arg(help = "Number lines (blank and non-blank both)", short = 'n', long)]
    number_lines: bool,

    #[arg(help = "Number non-blank lines only", short = 'b', long)]
    number_nonblank_lines: bool,

    #[arg(
        help = "Squeeze multiple adjacent empty lines into a single empty line",
        short = 's',
        long
    )]
    squeeze_empty_lines: bool,
}

pub fn get_args() -> MyResult<Config> {
    Ok(Config::parse())
}

pub fn run(config: Config) -> MyResult<()> {
    for filename in &config.files {
        match open(&filename) {
            Err(err) => eprintln!("Failed to open {}: {}", filename, err),
            Ok(file) => {
                let mut count_of_blanks = 0;
                let mut is_previous_blank = false;
                let mut blanks_omitted = 0;

                for (index, line) in file.lines().enumerate() {
                    let line = line.unwrap();

                    let line_is_empty = line.is_empty();

                    let prefix =
                        determine_line_prefix(&config, index, line_is_empty, count_of_blanks, blanks_omitted);

                    if !line_is_empty
                        || (line_is_empty && !config.squeeze_empty_lines)
                        || (line_is_empty && config.squeeze_empty_lines && !is_previous_blank)
                    {
                        println!("{}{}", prefix, line);
                    }

                    if line_is_empty {
                        if !config.squeeze_empty_lines || !is_previous_blank {
                            count_of_blanks += 1;
                        } else {
                            blanks_omitted += 1;
                        }
                        is_previous_blank = true;
                    } else {
                        is_previous_blank = false;
                    }
                }
            }
        }
    }

    Ok(())
}

fn determine_line_prefix(
    config: &Config,
    index: usize,
    line_is_empty: bool,
    count_of_blanks: usize,
    blanks_omitted: usize
) -> String {
    if config.number_lines {
        format!("{:>6}\t", index + 1 - blanks_omitted)
    } else if config.number_nonblank_lines && !line_is_empty {
        format!("{:>6}\t", index + 1 - count_of_blanks - blanks_omitted)
    } else {
        String::from("")
    }
}

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
