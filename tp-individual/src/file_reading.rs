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

pub fn read_csv_files<T, K>(files: Vec<PathBuf>, process_line: T) -> impl ParallelIterator<Item = K>
where
    T: Fn(String) -> Result<K, String> + Send + Sync + 'static,
    K: Send + 'static,
{
    files
        .into_par_iter() // Consumes the Vec<PathBuf>
        .flat_map(|file| {
            let reader = read_csv_file(&file); // Borrow file instead of consuming
            reader.lines().skip(1).par_bridge() // Use par_bridge for parallelism
        })
        .filter_map(|line| line.ok()) // Filter out errors from reading lines
        .filter_map(move |line| process_line(line).ok()) // Filter out errors from processing lines
}
