use rayon::prelude::*;

use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

const CSV_EXTENSION: &str = "csv";

pub fn find_csv_in_dir(input_path: &str) -> Vec<PathBuf> {
    match std::fs::read_dir(input_path) {
        Ok(files) => files
            .filter_map(|file| {
                let path = file.ok()?.path();
                if path.extension()?.to_str()? == CSV_EXTENSION {
                    Some(path)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>(),
        Err(e) => {
            eprintln!("Error reading input path: {}", e);
            std::process::exit(1);
        }
    }
}

fn read_csv_file(file: &Path) -> BufReader<File> {
    match std::fs::File::open(file) {
        Ok(file) => BufReader::new(file),
        Err(e) => {
            eprintln!("Error reading file {:?}: {}", file, e);
            std::process::exit(1);
        }
    }
}

pub fn read_csv_files<F, T>(files: Vec<PathBuf>, process_line: F) -> impl ParallelIterator<Item = T>
where
    F: Fn(String) -> Result<T, String> + Send + Sync,
    T: Send,
{
    files
        .into_par_iter()
        .flat_map(|file| {
            let reader = read_csv_file(&file);
            reader.lines().skip(1).par_bridge()
        })
        .filter_map(|line| line.ok())
        .filter_map(move |line| process_line(line).ok())
}
