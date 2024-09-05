mod args_reading;
mod deaths;
mod file_reading;
mod json_writting;
mod player_stats;
mod sorting;
mod weapon_stats;

use args_reading::read_args;
use deaths::Death;
use file_reading::{find_csv_in_dir, read_csv_files};
use json_writting::save_as_json;
use player_stats::{get_top_killers, player_stats_from_deaths};
use weapon_stats::{get_top_weapons, weapon_stats_from_deaths};

const PADRON: usize = 110119;

const TOP_PLAYERS_COUNT: usize = 10;
const TOP_WEAPONS_COUNT: usize = 10;
const TOP_WEAPONS_OF_PLAYER_COUNT: usize = 3;

fn main() {
    let (input_path, num_threads, output_file_name) = read_args();

    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .unwrap();

    // READ CSV FILES

    let start = std::time::Instant::now();

    let csv_files = find_csv_in_dir(&input_path);
    let deaths = read_csv_files(csv_files, Death::from_csv_record);

    let end_parse = std::time::Instant::now();
    println!("End parsing: {:?}", end_parse - start);

    // PROCESS DEATHS
    // Get top players and favorite weapons

    let player_stats = player_stats_from_deaths(&deaths);
    let most_lethal_players =
        get_top_killers(player_stats, TOP_PLAYERS_COUNT, TOP_WEAPONS_OF_PLAYER_COUNT);

    let end_players = std::time::Instant::now();
    println!("End players: {:?}", end_players - end_parse);

    // Get top weapons

    let weapon_stats = weapon_stats_from_deaths(&deaths);
    let most_lethal_weapon = get_top_weapons(weapon_stats, TOP_WEAPONS_COUNT);

    let end_weapons = std::time::Instant::now();
    println!("End weapons: {:?}", end_weapons - end_players);

    // SAVE AS JSON

    save_as_json(
        most_lethal_players,
        most_lethal_weapon,
        &output_file_name,
        deaths.len(),
    );
    println!("Total: {:?}", end_weapons - start);
}
