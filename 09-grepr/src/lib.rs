use std::{error::Error, fs};

use clap::Parser;
use regex::{Regex, RegexBuilder};
use walkdir::WalkDir;

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(help = "Pattern to match")]
    pattern: String,

    #[arg(help = "Input files, defaults to STDIN", default_value = "-")]
    files: Vec<String>,

    #[arg(help = "search files in directories", short = 'r', long)]
    recursive: bool,

    #[arg(help = "show count of matches", short = 'c', long)]
    count: bool,

    #[arg(help = "invert matches", short = 'v', long)]
    invert_match: bool,

    #[arg(help = "case insensitive search", short = 'i', long)]
    insensitive: bool,
}

#[derive(Debug)]
pub struct Config {
    pattern: Regex,
    files: Vec<String>,
    recursive: bool,
    count: bool,
    invert_match: bool,
}

pub fn get_args() -> MyResult<Config> {
    let args = Args::parse();

    let pattern = RegexBuilder::new(&args.pattern)
        .case_insensitive(args.insensitive)
        .build()
        .map_err(|_| format!("Invalid pattern \"{}\"", args.pattern))?;

    Ok(Config {
        pattern,
        files: args.files,
        recursive: args.recursive,
        count: args.count,
        invert_match: args.invert_match,
    })
}

pub fn run(config: Config) -> MyResult<()> {
    dbg!(config);
    Ok(())
}

fn find_files(paths: &[String], recursive: bool) -> Vec<MyResult<String>> {
    let mut results = vec![];

    for path in paths {
        match path.as_str() {
            "-" => results.push(Ok(path.to_string())),
            _ => match fs::metadata(path) {
                Ok(metadata) => {
                    if metadata.is_dir() {
                        if recursive {
                            for entry in WalkDir::new(path)
                                .into_iter()
                                .flatten()
                                .filter(|e| e.file_type().is_file())
                            {
                                results.push(Ok(entry.path().display().to_string()));
                            }
                        } else {
                            results.push(Err(From::from(format!("{} is a directory", path))))
                        }
                    }
                }

                Err(e) => {
                    results.push(Err(From::from(format!("{}: {}", path, e))));
                }
            },
        }
    }

    results
}

// ---------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::find_files;
    use rand::{distributions::Alphanumeric, Rng};
    #[test]
    fn test_find_files() {
        // Verify that the function finds a file known to exist
        let files = find_files(&["./tests/inputs/fox.txt".to_string()], false);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].as_ref().unwrap(), "./tests/inputs/fox.txt");

        // The function should reject a directory without the recursive option
        let files = find_files(&["./tests/inputs".to_string()], false);
        assert_eq!(files.len(), 1);
        if let Err(e) = &files[0] {
            assert_eq!(e.to_string(), "./tests/inputs is a directory");
        }

        // Verify the function recurses to find four files in the directory
        let res = find_files(&["./tests/inputs".to_string()], true);
        let mut files: Vec<String> = res
            .iter()
            .map(|r| r.as_ref().unwrap().replace("\\", "/"))
            .collect();
        files.sort();
        assert_eq!(files.len(), 4);
        assert_eq!(
            files,
            vec![
                "./tests/inputs/bustle.txt",
                "./tests/inputs/empty.txt",
                "./tests/inputs/fox.txt",
                "./tests/inputs/nobody.txt",
            ]
        );
        // Generate a random string to represent a nonexistent file
        let bad: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();
        // Verify that the function returns the bad file as an error
        let files = find_files(&[bad], false);
        assert_eq!(files.len(), 1);
        assert!(files[0].is_err());
    }
}
