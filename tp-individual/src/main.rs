mod args_reading;
mod deaths;
mod file_reading;
mod json_writting;
mod player_stats;
mod sorting;
mod stats;
mod time_tracking;
mod weapon_stats;

use args_reading::read_args;
use deaths::Death;
use file_reading::{find_csv_in_dir, read_csv_files};
use json_writting::save_as_json;
use stats::Stats;
use time_tracking::Timer;

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

    let mut timer = Timer::new();

    // READ CSV FILES AND PROCESS DEATHS INTO STATS

    let csv_files = find_csv_in_dir(&input_path);
    let deaths = read_csv_files(csv_files, Death::from_csv_record);

    let mut stats = Stats::from_deaths(deaths);
    timer.print_lap("Processing deaths");

    // GET TOP KILLERS AND ITS BEST WEAPONS

    stats.filter_top_killers(TOP_PLAYERS_COUNT, TOP_WEAPONS_OF_PLAYER_COUNT);
    timer.print_lap("Filtering top killers");

    // GET TOP WEAPONS

    stats.filter_top_weapons(TOP_WEAPONS_COUNT);
    timer.print_lap("Filtering top weapons");

    // SAVE AS JSON

    save_as_json(stats, &output_file_name);
    timer.print_lap("Saving as JSON");

    timer.print_total();
}
