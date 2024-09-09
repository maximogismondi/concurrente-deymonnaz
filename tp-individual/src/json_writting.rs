use std::collections::HashMap;

use serde_json::json;

use crate::{player_stats::PlayerStats, stats::Stats, weapon_stats::WeaponStats, PADRON};

pub fn save_as_json(stats: Stats, output_path: &str) {
    let Stats {
        players,
        weapons,
        total_deaths,
    } = stats;

    let top_killers = players
        .iter()
        .map(|(player_name, player_stats)| {
            let PlayerStats {
                deaths_count,
                weapons,
            } = player_stats;

            let weapon_stats = weapons
                .iter()
                .map(|(weapon_name, weapon_death_count)| {
                    (
                        weapon_name,
                        calculate_percentage(*weapon_death_count, *deaths_count),
                    )
                })
                .collect::<HashMap<_, _>>();

            (
                player_name,
                json!({
                    "deaths": deaths_count,
                    "weapons_percentage": weapon_stats
                }),
            )
        })
        .collect::<HashMap<_, _>>();

    let top_weapons = weapons.iter().map(|(weapon_name, weapon_stats)| {
        let WeaponStats {
            death_count,
            death_count_with_distance,
            total_distance,
        } = weapon_stats;

        (weapon_name, json!({
            "deaths_percentage": calculate_percentage(*death_count, total_deaths),
            "average_distance": calculate_average(*total_distance, *death_count_with_distance),
        }))
    }).collect::<HashMap<_, _>>();

    let json_data = json!({
        "padron": PADRON,
        "top_killers": top_killers,
        "top_weapons": top_weapons,
    });

    let json_str = serde_json::to_string_pretty(&json_data).expect("Failed to serialize to JSON");
    std::fs::write(output_path, json_str).expect("Failed to write JSON to file");
}

fn calculate_percentage(count: usize, total: usize) -> f64 {
    if total == 0 {
        return 0.0;
    } else {
        (count as f64 / total as f64 * 10000f64).round() / 100f64
    }
}

fn calculate_average(distance: f64, count: usize) -> f64 {
    if count == 0 {
        0.0
    } else {
        (distance / count as f64 * 100f64).round() / 100f64
    }
}
