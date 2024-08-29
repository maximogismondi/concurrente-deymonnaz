mod deaths;

use deaths::Deaths;
use std::collections::HashMap;

const CSV_EXTENSION: &str = "csv";
const TOP_K: usize = 10;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    if args.len() != 4 {
        eprintln!(
            "Usage: {} <input-path> <num-threads> <output-file-name>",
            args[0]
        );
        std::process::exit(1);
    }

    let input_path = &args[1];
    let num_threads = match args[2].parse::<usize>() {
        Ok(num) => num,
        Err(_) => {
            eprintln!("Invalid number of threads: {}", args[2]);
            std::process::exit(1);
        }
    };

    let output_file_name = &args[3];

    // find csv files in the input path

    let csv_files = match std::fs::read_dir(input_path) {
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
    };

    let deaths = csv_files.iter().map(|file| csv_file_to_deaths(&file)).fold(
        Vec::new(),
        |mut acc, deaths| {
            acc.extend(deaths);
            acc
        },
    );

    println!(
        "Processing {} records with {} threads",
        deaths.len(),
        num_threads
    );

    // now i want to process the records i want to make first the most letal wapon and the most letal player in a functional way i want the TOP_K most letal weapons and the TOP_K most letal players

    let most_letal_weapon = deaths
        .iter()
        .fold(HashMap::new(), |mut acc, death| {
            *acc.entry(&death.killed_by).or_insert(0) += 1;
            acc
        })
        .into_iter()
        .collect::<Vec<_>>();

    let most_letal_player = deaths
        .iter()
        .fold(HashMap::new(), |mut acc, death| {
            *acc.entry(&death.killer_name).or_insert(0) += 1;
            acc
        })
        .into_iter()
        .collect::<Vec<_>>();

    let mut most_letal_weapon = most_letal_weapon.iter().collect::<Vec<_>>();
    most_letal_weapon.sort_by(|a, b| b.1.cmp(&a.1));
    let most_letal_weapon = most_letal_weapon.iter().take(TOP_K).collect::<Vec<_>>();

    let mut most_letal_player = most_letal_player.iter().collect::<Vec<_>>();
    most_letal_player.sort_by(|a, b| b.1.cmp(&a.1));
    let most_letal_player = most_letal_player.iter().take(TOP_K).collect::<Vec<_>>();

    // PRINT THE RESULTS

    println!("Most letal weapons:");
    for (weapon, count) in most_letal_weapon {
        println!("{}: {}", weapon, count);
    }

    println!("Most letal players:");
    for (player, count) in most_letal_player {
        println!("{}: {}", player, count);
    }
}

fn csv_file_to_deaths(file: &std::path::Path) -> Vec<Deaths> {
    let file_name = match file.file_name().and_then(|name| name.to_str()) {
        Some(name) => name,
        None => {
            eprintln!("Error getting file name");
            std::process::exit(1);
        }
    };

    // try to parse the file and if there is an error, print it and stop the parsing

    let file = match std::fs::read_to_string(&file) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error reading file {}: {}", file_name, e);
            std::process::exit(1);
        }
    };

    // i want to check if the records are valid before collecting them and if they are not valid, i want to print the error and continue to the next record

    let deaths = file
        .lines()
        .filter_map(|line| Deaths::from_csv_record(line.to_string()).ok())
        .collect::<Vec<_>>();

    println!("Parsed {} records from {}", deaths.len(), file_name);

    deaths
}
