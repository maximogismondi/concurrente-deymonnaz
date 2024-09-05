use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
};

use rayon::prelude::*;

const CSV_EXTENSION: &str = "csv";

pub fn find_csv_in_dir(input_path: &str) -> Vec<PathBuf> {
    match std::fs::read_dir(input_path) {
        Ok(files) => files
            .filter_map(|file| {
                let file = file.ok()?;
                let path = file.path();
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

pub fn read_csv_files<T, K>(files: Vec<PathBuf>, process_line: T) -> Vec<K>
where
    T: Fn(String) -> Result<K, String> + Send + Sync,
    K: Send,
{
    files
        .par_iter() // Parallel iteration over the files
        .flat_map(|file| {
            let reader = read_csv_file(file);
            reader
                .lines()
                .skip(1) // Skip the header if necessary
                .par_bridge() // Parallelize line processing
        })
        .filter_map(|line| line.ok()) // Filter out any erroneous lines
        .filter_map(|line| process_line(line).ok()) // Apply the closure and handle the result
        .collect()
}
