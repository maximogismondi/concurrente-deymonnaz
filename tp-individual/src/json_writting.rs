use std::collections::HashMap;

use serde_json::json;

use crate::{stats::Stats, PADRON};

pub fn save_as_json(stats: Stats, output_path: &str) {
    let players = stats.players;
    let weapons = stats.weapons;
    let total_deaths = stats.total_deaths;

    let json = json!({
        "padron": PADRON,
        "top_killers": players.iter().map(|(player_name, player_stats)| {
            let weapon_stats: HashMap<_, _> = player_stats.weapons.iter().map(|(weapon_name, weapon_death_count)| {
                (weapon_name.clone(), (*weapon_death_count as f64  / player_stats.deaths as f64  * 10000f64).round() / 100f64)
            }).collect();

            (player_name, json!({
                "total_kills": player_stats.deaths,
                "weapons": weapon_stats
            }))
        }).collect::<HashMap<_, _>>(),
        "top_weapons": weapons.iter().map(|(weapon_name, weapon_stats)| {
            (weapon_name, json!({
                "total_kills": (weapon_stats.death_count as f64 / total_deaths as f64 * 10000f64).round() / 100f64,
                "average_distance": (weapon_stats.total_distance / weapon_stats.death_count_with_distance as f64 * 100f64).round() / 100f64
            }))
        }).collect::<HashMap<_, _>>()
    });

    let json_str = serde_json::to_string_pretty(&json).unwrap();
    std::fs::write(output_path, json_str).unwrap();
}
