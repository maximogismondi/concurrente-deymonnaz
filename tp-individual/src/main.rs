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

    if let Err(e) = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
    {
        eprintln!("Error creating thread pool: {}", e);
        std::process::exit(1);
    }

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

#[cfg(test)]
mod tests {
    use crate::{
        deaths::Death,
        file_reading::read_csv_files,
        json_writting::save_as_json,
        player_stats::{self, PlayerStats},
        stats::Stats,
        weapon_stats::WeaponStats,
        PADRON,
    };
    use assert_json_diff::assert_json_eq;
    use rayon::prelude::*;
    use serde_json::json;
    use tempfile::NamedTempFile;

    const HEADER: &str = "killed_by,killer_name,killer_placement,killer_position_x,killer_position_y,map,match_id,time,victim_name,victim_placement,victim_position_x,victim_position_y";
    const DEATH_RECORD_1: &str = "AK47,Player1,1.0,0.0,0.0,map,match-id,123,Player2,1.0,100.0,0.0";
    const DEATH_RECORD_2: &str = "M4A4,Player2,1.0,0.0,0.0,map,match-id,123,Player1,1.0,50.0,0.0";

    #[test]
    fn test_empty_no_csv_files() {
        let csv_files = vec![];
        let deaths = read_csv_files(csv_files, |_: String| Ok(()));

        assert_eq!(deaths.count(), 0);
    }

    #[test]
    fn test_empty_csv_files() {
        let temp_file = NamedTempFile::new().unwrap();
        let csv_files = vec![temp_file.path().to_path_buf()];
        let deaths = read_csv_files(csv_files, |_: String| Ok(()));

        assert_eq!(deaths.count(), 0);
    }

    #[test]
    fn test_single_csv_file() {
        let temp_file = NamedTempFile::new().unwrap();
        std::fs::write(
            temp_file.path(),
            format!("{}\n{}", HEADER, DEATH_RECORD_1).as_bytes(),
        )
        .unwrap();

        let csv_files = vec![temp_file.path().to_path_buf()];
        let deaths = read_csv_files(csv_files, |line: String| Ok(line));

        assert_eq!(deaths.count(), 1);
    }

    #[test]
    fn test_multiline_csv_file() {
        let temp_file = NamedTempFile::new().unwrap();
        std::fs::write(
            temp_file.path(),
            format!("{}\n{}\n{}", HEADER, DEATH_RECORD_1, DEATH_RECORD_1).as_bytes(),
        )
        .unwrap();

        let csv_files = vec![temp_file.path().to_path_buf()];
        let deaths = read_csv_files(csv_files, |line: String| Ok(line));

        assert_eq!(deaths.count(), 2);
    }

    #[test]
    fn test_multiple_csv_files() {
        let temp_file_1 = NamedTempFile::new().unwrap();
        std::fs::write(
            temp_file_1.path(),
            format!("{}\n{}", HEADER, DEATH_RECORD_1).as_bytes(),
        )
        .unwrap();

        let temp_file_2 = NamedTempFile::new().unwrap();
        std::fs::write(
            temp_file_2.path(),
            format!("{}\n{}", HEADER, DEATH_RECORD_1).as_bytes(),
        )
        .unwrap();

        let csv_files = vec![
            temp_file_1.path().to_path_buf(),
            temp_file_2.path().to_path_buf(),
        ];
        let deaths = read_csv_files(csv_files, |line: String| Ok(line));

        assert_eq!(deaths.count(), 2);
    }

    fn json_from_file(file_path: &str) -> serde_json::Value {
        let reader = std::fs::File::open(file_path).unwrap();
        serde_json::from_reader(reader).unwrap()
    }

    fn stats_from_deaths(deaths: Vec<String>) -> Stats {
        Stats::from_deaths(
            deaths
                .into_par_iter()
                .map(|record| Death::from_csv_record(record).unwrap()),
        )
    }

    #[test]
    fn test_save_as_json_empty() {
        let temp_file = NamedTempFile::new().unwrap();
        let output_path = temp_file.path().to_str().unwrap();

        let deaths = vec![];
        let stats = stats_from_deaths(deaths);

        save_as_json(stats, output_path);

        let expected_json = json!({
            "padron": PADRON,
            "top_killers": {},
            "top_weapons": {},
        });

        let output_json = json_from_file(output_path);

        assert_json_eq!(expected_json, output_json);
    }

    #[test]
    fn test_save_as_json_single_death() {
        let temp_file = NamedTempFile::new().unwrap();
        let output_path = temp_file.path().to_str().unwrap();

        let deaths = vec![DEATH_RECORD_1.to_string()];
        let stats = stats_from_deaths(deaths);

        save_as_json(stats, output_path);

        let expected_json = json!({
            "padron": PADRON,
            "top_killers": {
                "Player1": {
                    "deaths": 1,
                    "weapons_percentage": {
                        "AK47": 100.0
                    }
                }
            },
            "top_weapons": {
                "AK47": {
                    "deaths_percentage": 100.0,
                    "average_distance": 100.0
                }
            },
        });

        let output_json = json_from_file(output_path);

        assert_json_eq!(expected_json, output_json);
    }

    #[test]
    fn test_save_as_json_multiple_players_and_weapons() {
        let temp_file = NamedTempFile::new().unwrap();
        let output_path = temp_file.path().to_str().unwrap();

        let deaths = vec![DEATH_RECORD_1.to_string(), DEATH_RECORD_2.to_string()];
        let stats = stats_from_deaths(deaths);

        save_as_json(stats, output_path);

        let expected_json = json!({
            "padron": PADRON,
            "top_killers": {
                "Player1": {
                    "deaths": 1,
                    "weapons_percentage": {
                        "AK47": 100.0
                    }
                },
                "Player2": {
                    "deaths": 1,
                    "weapons_percentage": {
                        "M4A4": 100.0
                    }
                }
            },
            "top_weapons": {
                "AK47": {
                    "deaths_percentage": 50.0,
                    "average_distance": 100.0
                },
                "M4A4": {
                    "deaths_percentage": 50.0,
                    "average_distance": 50.0
                }
            },
        });

        let output_json = json_from_file(output_path);

        assert_json_eq!(expected_json, output_json);
    }
}
