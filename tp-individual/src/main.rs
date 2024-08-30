mod deaths;

use deaths::Deaths;
use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
};

use rayon::prelude::*;

const CSV_EXTENSION: &str = "csv";
const TOP_K_PLAYERS: usize = 10;
const TOP_K_WEAPONS: usize = 10;
const TOP_K_WEAPONS_OF_PLAYER: usize = 3;

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

    let _output_file_name = &args[3];

    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .unwrap();

    // find csv files in the input path

    let start = std::time::Instant::now();

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

    let deaths = csv_files
        .par_iter() // Parallelize the iteration over csv_files
        .flat_map(|file| {
            let file = std::fs::File::open(file).unwrap();
            let reader = BufReader::new(file);
            reader.lines().skip(1).par_bridge()
        })
        .filter_map(|line| line.ok())
        .filter_map(|line| Deaths::from_csv_record(line).ok())
        .collect::<Vec<_>>();

    let end_parse = std::time::Instant::now();
    println!("End parsing: {:?}", end_parse - start);

    // i want to transform the records as a map of
    // player   -> weapon -> count
    //          -> total

    // weapon -> count
    //        -> total distance

    let mut player_stats = deaths
        .iter()
        .fold(HashMap::new(), |mut acc, death| {
            let player = acc.entry(&death.killer_name).or_insert((HashMap::new(), 0));
            *player.0.entry(&death.killed_by).or_insert(0) += 1;
            player.1 += 1;
            acc
        })
        .into_iter()
        .collect::<Vec<_>>();

    player_stats.sort_by(|a, b| {
        let a = a.1 .1;
        let b = b.1 .1;
        b.cmp(&a)
    });
    let most_letal_player = player_stats
        .iter()
        .take(TOP_K_PLAYERS)
        .map(|(player, stats)| {
            let total = stats.1;
            let mut weapons = stats.0.iter().collect::<Vec<_>>();
            weapons.sort_by(|a, b| {
                let a = a.1;
                let b = b.1;
                b.cmp(&a)
            });
            let player_lethal_weapon = weapons
                .iter()
                .take(TOP_K_WEAPONS_OF_PLAYER)
                .map(|(weapon, count)| (*weapon, *count))
                .collect::<Vec<_>>();

            (player, total, player_lethal_weapon)
        });

    let end_players = std::time::Instant::now();
    println!("End players: {:?}", end_players - end_parse);

    // save sum and count for each weapon

    let mut weapon_stats = deaths
        .iter()
        .fold(HashMap::new(), |mut acc, death| {
            let weapon = acc.entry(&death.killed_by).or_insert((0.0, 0));
            weapon.0 += death.distance();
            weapon.1 += 1;
            acc
        })
        .into_iter()
        .collect::<Vec<_>>();

    // sort by count
    weapon_stats.sort_by(|a, b| {
        let a = a.1 .1;
        let b = b.1 .1;
        b.cmp(&a)
    });

    let most_letal_weapon = weapon_stats
        .iter()
        .take(TOP_K_WEAPONS)
        .map(|(weapon, stats)| {
            let total = stats.1;
            let distance = stats.0 / total as f32;
            (weapon, total, distance)
        });

    let end_weapons = std::time::Instant::now();
    println!("End weapons: {:?}", end_weapons - end_players);

    // PRINT THE RESULTS

    println!("Most letal players:");
    for (player, total, weapons) in most_letal_player {
        println!("{}: {}", player, total);
        for (weapon, count) in weapons {
            let percentage = (*count as f32 / total as f32 * 100.0).round() / 100.0;
            println!("  {}: ({:.2}%)", weapon, percentage);
        }
    }

    println!("Most letal weapons:");
    for (weapon, total, distance) in most_letal_weapon {
        println!("{}: {}", weapon, total);
        println!("  Average distance: {:.2}", distance);
    }
    println!();
    println!("Total: {:?}", end_weapons - start);
}
