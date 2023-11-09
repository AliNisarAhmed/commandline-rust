use std::{
    error::Error,
    ffi::OsStr,
    fs::{self, File},
    io::{BufRead, BufReader, Read},
    path::PathBuf,
};

use clap::Parser;
use regex::{Regex, RegexBuilder};
use walkdir::WalkDir;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(help = "Input files or directories", name = "FILE", required = true)]
    sources: Vec<String>,

    #[arg(short = 'm', long, name = "PATTERN", help = "Pattern")]
    pattern: Option<String>,

    #[arg(short = 'i', long, help = "Case-insensitive pattern matching")]
    insensitive: bool,

    #[arg(short = 's', long, name = "SEED", help = "Random seed")]
    seed: Option<u64>,
}

#[derive(Debug)]
pub struct Config {
    sources: Vec<String>,
    pattern: Option<Regex>,
    seed: Option<u64>,
}

#[derive(Debug)]
struct Fortune {
    // filename containing the record
    source: String,
    // contents of the record up to but not including
    // the terminating % sign
    text: String,
}

pub fn get_args() -> MyResult<Config> {
    let args = Args::parse();

    let pattern = match args.pattern {
        None => None,
        Some(s) => Some(
            RegexBuilder::new(&s)
                .case_insensitive(args.insensitive)
                .build()
                .map_err(|_| format!("Invalid --pattern \"{}\"", s))?,
        ),
    };

    Ok(Config {
        sources: args.sources,
        pattern,
        seed: args.seed,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    let files = find_files(&config.sources)?;
    let fortunes = read_fortunes(&files)?;
    println!("{:#?}", fortunes.last());
    Ok(())
}

// -------------------------------------------------------------------------

fn find_files(paths: &[String]) -> MyResult<Vec<PathBuf>> {
    let dat = OsStr::new("dat");
    let mut result = vec![];

    for path in paths {
        let metadata = fs::metadata(path)?;
        if metadata.is_dir() {
            for entry in WalkDir::new(path)
                .into_iter()
                .flatten()
                .filter(|e| e.file_type().is_file() && e.path().extension() != Some(dat))
            {
                result.push(entry.into_path());
            }
        } else if metadata.is_file() {
            result.push(PathBuf::from(path));
        }
    }

    result.sort();
    result.dedup();

    Ok(result)
}

fn read_fortunes(paths: &[PathBuf]) -> MyResult<Vec<Fortune>> {
    let mut result = vec![];
    let mut buffer = vec![];

    for path in paths {
        let basename = path.file_name().unwrap().to_string_lossy().into_owned();
        let file = File::open(path)
            .map_err(|e| format!("{}: {}", path.to_string_lossy().into_owned(), e))?;

        for line in BufReader::new(file).lines().filter_map(Result::ok) {
            if line == "%" {
                if !buffer.is_empty() {
                    result.push(Fortune {
                        source: basename.clone(),
                        text: buffer.join("\n"),
                    });
                    buffer.clear();
                }
            } else {
                buffer.push(line.to_string());
            }
        }
    }

    Ok(result)
}

fn pick_fortune(fortunes: &[Fortune], seed: Option<u64>) -> Option<String> {
    unimplemented!();
}

// -------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::{find_files, read_fortunes};

    #[test]
    fn test_find_files() {
        // verify that it finds a file that's known to exist
        let res = find_files(&["./tests/inputs/jokes".to_string()]);
        assert!(res.is_ok());

        let files = res.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(
            files.get(0).unwrap().to_string_lossy(),
            "./tests/inputs/jokes"
        );

        // Fails to find a bad file
        let res = find_files(&["/path/does/not/exist".to_string()]);
        assert!(res.is_err());

        // Find all the input files, excludes ".dat"
        let res = find_files(&["./tests/inputs".to_string()]);
        assert!(res.is_ok());

        // Check number and order of files
        let files = res.unwrap();
        dbg!(&files);
        assert_eq!(files.len(), 5);
        let first = files.get(0).unwrap().display().to_string();
        assert!(first.contains("ascii-art"));
        let last = files.last().unwrap().display().to_string();
        assert!(last.contains("quotes"));

        // Test for multiple sources, path must be unique and sorted
        let res = find_files(&[
            "./tests/inputs/jokes".to_string(),
            "./tests/inputs/ascii-art".to_string(),
            "./tests/inputs/jokes".to_string(),
        ]);
        assert!(res.is_ok());
        let files = res.unwrap();
        assert_eq!(files.len(), 2);
        if let Some(filename) = files.first().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "ascii-art".to_string())
        }
        if let Some(filename) = files.last().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "jokes".to_string())
        }
    }

    #[test]
    fn test_read_fortunes() {
        // One input file
        let res = read_fortunes(&[PathBuf::from("./tests/inputs/jokes")]);
        assert!(res.is_ok());

        if let Ok(fortunes) = res {
            // Correct number & sorting
            assert_eq!(fortunes.len(), 6);
            assert_eq!(
                fortunes.first().unwrap().text,
                "Q. What do you call a head of lettuce in a shirt and tie?\n\
                A. Collared greens."
            );
            assert_eq!(
                fortunes.last().unwrap().text,
                "Q: What do you call a deer wearing an eye patch?\n\
                A: A bad idea (bad-eye deer)."
            );
        }

        // Multiple input files
        let res = read_fortunes(&[
            PathBuf::from("./tests/inputs/jokes"),
            PathBuf::from("./tests/inputs/quotes"),
        ]);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 11);
    }
}
