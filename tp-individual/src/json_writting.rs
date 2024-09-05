use std::collections::HashMap;

use serde_json::json;

use crate::{player_stats::PlayerStats, weapon_stats::WeaponStats, PADRON};

pub fn save_as_json(
    top_killers: Vec<(&String, PlayerStats)>,
    most_letal_weapons: Vec<(&String, WeaponStats)>,
    output_path: &str,
    total_deaths: usize,
) {
    let json = json!({
        "padron": PADRON,
        "top_killers": top_killers.iter().map(|(player_name, stats)| {
            let weapon_percentages: HashMap<_, _> = stats.weapons.iter().map(|(weapon, count)| {
                (weapon.clone(), format!("{:.2}", *count as f32 / stats.total as f32))
            }).collect();

            (player_name, json!({
                "total_kills": stats.total,
                "weapons": weapon_percentages
            }))
        }).collect::<HashMap<_, _>>(),
        "top_weapons": most_letal_weapons.iter().map(|(name, weapon)| {
            (name, json!({
                "total_kills": format!("{:.2}", weapon.count as f32 / total_deaths as f32),
                "average_distance": format!("{:.2}", weapon.total_distance / weapon.count as f32)
            }))
        }).collect::<HashMap<_, _>>()
    });

    let json_str = serde_json::to_string_pretty(&json).unwrap();
    std::fs::write(output_path, json_str).unwrap();
}
