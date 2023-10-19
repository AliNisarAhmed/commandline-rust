use clap::Parser;
use std::{
    error::Error,
    fs::File,
    io::{self, BufRead, BufReader, Write},
};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Parser, Debug)]
#[command(version, author, about, long_about = None)]
pub struct Config {
    #[arg(help = "Input file", required = false, default_value = "-")]
    in_file: String,

    #[arg(help = "Output file")]
    out_file: Option<String>,

    #[arg(help = "Show counts", long, short = 'c', required = false)]
    count: bool,

    #[arg(help = "only print diplcate lines, one for each group", short = 'd')]
    repeated: bool,
}

pub fn get_args() -> MyResult<Config> {
    Ok(Config::parse())
}

pub fn run(config: Config) -> MyResult<()> {
    let mut file = open(&config.in_file).map_err(|e| format!("{}: {}", config.in_file, e))?;

    let mut out_file: Box<dyn Write> = match &config.out_file {
        Some(output_file_name) => Box::new(File::create(output_file_name)?),
        _ => Box::new(io::stdout()),
    };

    let mut print = |count: u64, text: &str| -> MyResult<()> {
        if count > 0 {
            if config.count {
                if config.repeated && count > 1 {
                    write!(out_file, "{:>4} {}", count, text)?;
                } else {
                    write!(out_file, "{:>4} {}", count, text)?;
                }
            } else {
                if config.repeated {
                    if count > 1 {
                        write!(out_file, "{}", text)?;
                    }
                } else {
                    write!(out_file, "{}", text)?;
                }
            }
        };
        Ok(())
    };

    let mut line = String::new();
    let mut previous_line = String::new();
    let mut count: u64 = 0;

    loop {
        let bytes = file.read_line(&mut line)?;

        if bytes == 0 {
            break;
        }

        if line.trim_end() != previous_line.trim_end() {
            print(count, &previous_line)?;
            previous_line = line.clone();
            count = 0;
        }

        count += 1;
        line.clear();
    }

    print(count, &previous_line)?;

    Ok(())
}

// ------------------

fn open(filename: &str) -> MyResult<Box<dyn BufRead>> {
    match filename {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        _ => Ok(Box::new(BufReader::new(File::open(filename)?))),
    }
}
