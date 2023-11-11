use std::error::Error;

use chrono::NaiveDate;
use clap::Parser;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Parser)]
struct Args {
    #[arg(short = 'y', long, name = "SHOW_YEAR", required = false)]
    show_current_year: bool,

    #[arg(short = 'm', long, name = "MONTH", required = false)]
    month: Option<u32>,

    #[arg(name = "YEAR")]
    year: Option<i32>,
}

#[derive(Debug)]
pub struct Config {
    month: Option<u32>,
    year: i32,
    today: NaiveDate,
}

pub fn get_args() -> MyResult<Config> {
    let args = Args::parse();

    Ok(Config {
        month: None,
        year: 1,
        today: chrono::prelude::Local::now().date_naive(),
    })
}

pub fn run(config: Config) -> MyResult<()> {
    println!("{:?}", config);

    Ok(())
}
