mod deaths;
mod file_reading;
mod json_writting;
mod player_stats;
mod weapon_stats;

use deaths::Deaths;
use file_reading::{find_csv_in_dir, read_csv_files};
use json_writting::save_as_json;
use player_stats::PlayerStats;
use std::collections::HashMap;
use weapon_stats::WeaponStats;

use rayon::prelude::*;

const ARGS: usize = 3;
const PADRON: usize = 110119;

const TOP_K_PLAYERS: usize = 10;
const TOP_K_WEAPONS: usize = 10;
const TOP_K_WEAPONS_OF_PLAYER: usize = 3;

fn main() {
    let args = std::env::args().collect::<Vec<String>>();

    if args.len() != ARGS + 1 {
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

    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .unwrap();

    // find csv files in the input path

    let start = std::time::Instant::now();

    let csv_files = find_csv_in_dir(input_path);
    let deaths = read_csv_files(csv_files, Deaths::from_csv_record);

    let end_parse = std::time::Instant::now();

    println!("End parsing: {:?}", end_parse - start);
    let player_stats: HashMap<String, PlayerStats> = deaths.iter().fold(
        HashMap::new(),
        |mut acc: HashMap<String, PlayerStats>, death| {
            if let Some(stats) = acc.get_mut(&death.killer_name) {
                stats.add_death(&death.killed_by);
            } else {
                acc.insert(
                    death.killer_name.clone(),
                    PlayerStats::new(&death.killed_by),
                );
            }

            acc
        },
    );

    let mut player_names: Vec<&String> = player_stats.keys().collect();

    player_names.sort_by(|a, b| {
        let a = player_stats.get(*a).unwrap().total;
        let b = player_stats.get(*b).unwrap().total;
        b.cmp(&a)
    });

    // create a hashmap with the top k players with their stats and weapons

    let most_lethal_players: HashMap<String, PlayerStats> = player_names
        .iter()
        .take(TOP_K_PLAYERS)
        .map(|name| {
            let stats = player_stats.get(*name).unwrap();
            let total = stats.total;

            let weapons: HashMap<String, usize> = stats
                .weapons
                .iter()
                .map(|(weapon, count)| (weapon.clone(), *count))
                .collect();

            let mut weapon_names: Vec<&String> = weapons.keys().collect();

            weapon_names.sort_by(|a, b| {
                let a = weapons.get(*a).unwrap();
                let b = weapons.get(*b).unwrap();
                b.cmp(&a)
            });

            let weapons: HashMap<String, usize> = weapon_names
                .iter()
                .take(TOP_K_WEAPONS_OF_PLAYER)
                .map(|weapon| (weapon.to_string(), *weapons.get(*weapon).unwrap()))
                .collect();

            (name.to_string(), PlayerStats { total, weapons })
        })
        .collect();

    let end_players = std::time::Instant::now();
    println!("End players: {:?}", end_players - end_parse);

    // save sum and count for each weapon

    let weapon_stats: HashMap<String, WeaponStats> = deaths.iter().fold(
        HashMap::new(),
        |mut acc: HashMap<String, WeaponStats>, death| {
            if let Some(stats) = acc.get_mut(&death.killed_by) {
                stats.add_death(death.distance());
            } else {
                acc.insert(death.killed_by.clone(), WeaponStats::new(death.distance()));
            }

            acc
        },
    );

    let mut weapon_names: Vec<&String> = weapon_stats.keys().collect();

    weapon_names.sort_by(|a, b| {
        let a = weapon_stats.get(*a).unwrap().count;
        let b = weapon_stats.get(*b).unwrap().count;
        b.cmp(&a)
    });

    let most_lethal_weapon: HashMap<String, WeaponStats> = weapon_names
        .iter()
        .take(TOP_K_WEAPONS)
        .map(|name| {
            let stats = weapon_stats.get(*name).unwrap();
            let count = stats.count;
            let total_distance = stats.total_distance;

            (
                name.to_string(),
                WeaponStats {
                    count,
                    total_distance,
                },
            )
        })
        .collect();

    let end_weapons = std::time::Instant::now();
    println!("End weapons: {:?}", end_weapons - end_players);

    // SAVE AS JSON

    save_as_json(
        most_lethal_players,
        most_lethal_weapon,
        output_file_name,
        deaths.len(),
    );
    println!("Total: {:?}", end_weapons - start);
}
