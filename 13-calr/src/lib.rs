use std::{error::Error, str::FromStr};

use ansi_term::Style;
use chrono::{Datelike, Local, NaiveDate};
use clap::Parser;
use itertools::izip;

const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const LINE_WIDTH: usize = 22;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Parser)]
struct Args {
    #[arg(short = 'y', long = "year", name = "SHOW YEAR", required = false, conflicts_with_all = ["YEAR", "MONTH"])]
    show_current_year: bool,

    #[arg(
        short = 'm', 
        long, 
        name = "MONTH",
        help = "Month name or number (1-12)",
        required = false,
        value_parser = parse_month
    )]
    month: Option<u32>,

    #[arg(name = "YEAR", help = "Year (1-9999)", value_parser = parse_year)]
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

    let today = Local::now();
    let year = if args.show_current_year || args.year.is_none() {
        today.year()
    } else {
        args.year.unwrap()
    };


    Ok(Config {
        month: args.month,
        year, 
        today: today.date_naive(),
    })
}

pub fn run(config: Config) -> MyResult<()> {
    match config.month {
        Some(month) => {
            let lines = format_month(config.year, month, true, config.today);
            println!("{}", lines.join("\n"));
        }
        None => {
            println!("{:>32}", config.year);
            let months: Vec<_> = (1..=12)
                .into_iter()
                .map(|month| {
                    format_month(config.year, month, false, config.today)
                })
                .collect();

            for (i, chunk) in months.chunks(3).enumerate() {
                if let [m1, m2, m3] = chunk {
                    for lines in izip!(m1, m2, m3) {
                        println!("{}{}{}", lines.0, lines.1, lines.2);
                    }
                    if i < 3 {
                        println!();
                    }
                }
            }
        }
    }
    Ok(())
}

// ----------------------------------------------------------------------------

fn format_month(
    year: i32,
    month: u32,
    print_year: bool,
    today: NaiveDate
    ) -> Vec<String> {
    
    let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let mut days: Vec<String> = (1..first.weekday().number_from_sunday())
            .into_iter()
            .map(|_| "  ".to_string())
            .collect();

    let is_today = |day: u32| {
        year == today.year() && month == today.month() && day == today.day()
    };

    let last = last_day_in_month(year, month);

    days.extend((first.day()..=last.day())
                .into_iter()
                .map(|num| {
                    let fmt = format!("{:>2}", num);
                    if is_today(num) {
                        Style::new().reverse().paint(fmt).to_string()
                    } else {
                        fmt
                    }
                }));
    
    let month_name = MONTH_NAMES[month as usize - 1];
    let mut lines = Vec::with_capacity(8);
    lines.push(format!(
            "{:^20}",
            if print_year {
                format!("{} {}", month_name, year)
            } else {
                month_name.to_string()
            }
            ));

    lines.push("Su Mo Tu We Th Fr Sa  ".to_string());

    for week in days.chunks(7) {
        lines.push(format!(
                "{:width$}  ",
                week.join(" "),
                width = LINE_WIDTH - 2
                ));
    }

    while lines.len() < 8 {
        lines.push(" ".repeat(LINE_WIDTH));
    }

    lines
}

fn parse_int<T: FromStr>(val: &str) -> Result<T, String> {
    val.parse::<T>()
        .map_err(|_| format!("Invalid integer \"{}\"", val))
}

fn parse_year(year: &str) -> Result<i32, String> {
    match parse_int::<i32>(year) {
        Ok(n) if n >= 1 && n <= 9999 => Ok(n),
        Ok(n) => Err(format!("year \"{}\" not in the range 1 through 9999", n)),
        Err(e) => Err(e),
    }
}

fn parse_month(month: &str) -> Result<u32, String> {
    match parse_int::<u32>(month) {
        Ok(n) => {
            if (1..=12).contains(&n) {
                Ok(n)
            } else {
                Err(format!("month \"{}\" not in the range 1 through 12", month))
            }
        }
        _ => {
            let lower = &month.to_lowercase();
            let matches: Vec<_> = MONTH_NAMES
                .iter()
                .enumerate()
                .filter_map(|(i, name)| {
                    if name.to_lowercase().starts_with(lower) {
                        Some(i + 1)
                    } else {
                        None
                    }
                })
                .collect();
            if matches.len() == 1 {
                Ok(matches[0] as u32)
            } else {
                Err(format!("Invalid month \"{}\"", month))
            }
        }
    }
}

pub fn last_day_in_month(year: i32, month: u32) -> NaiveDate {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => {
            NaiveDate::from_ymd_opt(year, month, 31).unwrap()
        },
        4 | 6 | 9 | 11 => {
            NaiveDate::from_ymd_opt(year, month, 30).unwrap()
        },
        2 if is_leap_year(year) => {
            NaiveDate::from_ymd_opt(year, month, 29).unwrap()
        },
        _ => {
            NaiveDate::from_ymd_opt(year, month, 28).unwrap()
        }
    }
}

fn is_leap_year(year: i32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}


// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use crate::{parse_int, parse_month, parse_year, last_day_in_month};

    #[test]
    fn test_parse_int() {
        let res = parse_int::<usize>("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1usize);

        let res = parse_int::<i32>("-1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), -1i32);

        let res = parse_int::<i64>("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Invalid integer \"foo\"");
    }

    #[test]
    fn test_parse_year() {
        let res = parse_year("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1i32);

        let res = parse_year("9999");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 9999i32);

        let res = parse_year("0");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "year \"0\" not in the range 1 through 9999"
        );

        let res = parse_year("10000");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "year \"10000\" not in the range 1 through 9999"
        );

        let res = parse_year("foo");
        assert!(res.is_err());
    }

    #[test]
    fn test_parse_month() {
        let res = parse_month("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1u32);
        let res = parse_month("12");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 12u32);
        let res = parse_month("jan");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1u32);
        let res = parse_month("0");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "month \"0\" not in the range 1 through 12"
        );
        let res = parse_month("13");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "month \"13\" not in the range 1 through 12"
        );
        let res = parse_month("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Invalid month \"foo\"");
    }

    #[test]
    fn test_last_day_in_month() {
        assert_eq!(
            last_day_in_month(2020, 1),
            NaiveDate::from_ymd_opt(2020, 1, 31).unwrap()
            );
        assert_eq!(
            last_day_in_month(2020, 2),
            NaiveDate::from_ymd_opt(2020, 2, 29).unwrap()
            );
        assert_eq!(
            last_day_in_month(2020, 4),
            NaiveDate::from_ymd_opt(2020, 4, 30).unwrap()
            );
    }
}
