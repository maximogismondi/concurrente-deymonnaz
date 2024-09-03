mod deaths;

use deaths::Deaths;
use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
};

use rayon::prelude::*;
use serde_json::json;

const PADRON: usize = 110119;

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

    let output_file_name = &args[3];

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

    let mut player_stats: Vec<_> = deaths
        .iter()
        .fold(HashMap::new(), |mut acc, death| {
            let player = acc.entry(&death.killer_name).or_insert((HashMap::new(), 0));
            *player.0.entry(&death.killed_by).or_insert(0) += 1;
            player.1 += 1;
            acc
        })
        .into_iter()
        .map(|(player, (weapons, total))| (player.to_string(), (weapons, total)))
        .collect();

    player_stats.sort_by(|a, b| b.1 .1.cmp(&a.1 .1));

    let most_lethal_player: Vec<_> = player_stats
        .iter()
        .take(TOP_K_PLAYERS)
        .map(|(player, stats)| {
            let total = stats.1;
            let mut weapons: Vec<_> = stats.0.iter().collect();
            weapons.sort_by(|a, b| b.1.cmp(a.1));
            let player_lethal_weapon = weapons
                .iter()
                .take(TOP_K_WEAPONS_OF_PLAYER)
                .map(|(&weapon, &count)| {
                    let weapon = weapon.to_string();
                    let percentage = count as f32 / total as f32 * 100.0;
                    (weapon, percentage)
                })
                .collect();
            (player.clone(), total, player_lethal_weapon)
        })
        .collect();

    let end_players = std::time::Instant::now();
    println!("End players: {:?}", end_players - end_parse);

    // save sum and count for each weapon

    let mut weapon_stats: Vec<_> = deaths
        .iter()
        .fold(HashMap::new(), |mut acc, death| {
            let weapon = acc.entry(&death.killed_by).or_insert((0.0, 0));
            weapon.0 += death.distance();
            weapon.1 += 1;
            acc
        })
        .into_iter()
        .map(|(weapon, (total_distance, count))| {
            (
                weapon.to_string(),
                count as f32 / deaths.len() as f32 * 100.0,
                total_distance / count as f32,
            )
        })
        .collect();

    weapon_stats.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let most_lethal_weapon = weapon_stats
        .iter()
        .take(TOP_K_WEAPONS)
        .cloned()
        .collect::<Vec<_>>();

    let end_weapons = std::time::Instant::now();
    println!("End weapons: {:?}", end_weapons - end_players);

    // SAVE AS JSON

    save_as_json(most_lethal_player, most_lethal_weapon, output_file_name);

    // PRINT THE RESULTS

    // println!();
    // println!("Most letal players:");
    // for (player, total, weapons) in most_letal_player {
    //     println!("{}", player);
    //     println!("  Total kills: {}", total);
    //     println!("  Weapons:");
    //     for (weapon, percentage) in weapons {
    //         println!("    {}: {:.2}%", weapon, percentage);
    //     }
    // }

    // println!();
    // println!("Most letal weapons:");
    // for (weapon, kills_percentaje, distance) in most_letal_weapon {
    //     println!("{}", weapon);
    //     println!("  Kills percentage: {:.2}%", kills_percentaje);
    //     println!("  Average distance: {:.2}", distance);
    // }
    // println!();
    println!("Total: {:?}", end_weapons - start);
}

fn save_as_json(
    most_letal_player: Vec<(String, usize, Vec<(String, f32)>)>,
    most_letal_weapon: Vec<(String, f32, f32)>,
    output_path: &str,
) {
    let json = json!({
        "padron": PADRON,
        "top_killers": most_letal_player.iter().map(|(player, total, weapons)| {
            let weapon_percentages: HashMap<_, _> = weapons.iter().map(|(weapon, percentage)| {
                (weapon.to_string(), format!("{:.2}", percentage))
            }).collect();

            (player.to_string(), json!({
                "deaths": total,
                "weapons_percentage": weapon_percentages
            }))
        }).collect::<HashMap<_, _>>(),
        "top_weapons": most_letal_weapon.iter().map(|(weapon, deaths_percentage, average_distance)| {
            (weapon.to_string(), json!({
                "deaths_percentage": format!("{:.2}", deaths_percentage),
                "average_distance": format!("{:.2}", average_distance)
            }))
        }).collect::<HashMap<_, _>>()
    });

    let json_str = serde_json::to_string_pretty(&json).unwrap();
    std::fs::write(output_path, json_str).unwrap();
}
