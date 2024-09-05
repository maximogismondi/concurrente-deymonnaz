use std::collections::HashMap;

use crate::deaths::Death;

pub struct WeaponStats {
    pub count: usize,
    pub total_distance: f32,
}

impl WeaponStats {
    pub fn new(distance: f32) -> Self {
        Self {
            count: 1,
            total_distance: distance,
        }
    }

    pub fn add_death(&mut self, distance: f32) {
        self.count += 1;
        self.total_distance += distance;
    }
}

pub fn weapon_stats_from_deaths(deaths: &Vec<Death>) -> HashMap<String, WeaponStats> {
    deaths.iter().fold(HashMap::new(), |mut acc, death| {
        if let Some(stats) = acc.get_mut(&death.killed_by) {
            stats.add_death(death.distance());
        } else {
            acc.insert(death.killed_by.clone(), WeaponStats::new(death.distance()));
        }

        acc
    })
}

pub fn get_top_weapons(
    weapon_stats: HashMap<String, WeaponStats>,
    weapon_count: usize,
) -> HashMap<String, WeaponStats> {
    let mut weapon_names: Vec<&String> = weapon_stats.keys().collect();

    weapon_names.sort_by(|a, b| {
        let a = weapon_stats.get(*a).unwrap().count;
        let b = weapon_stats.get(*b).unwrap().count;
        b.cmp(&a)
    });

    let most_lethal_weapons = weapon_names
        .iter()
        .take(weapon_count)
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

    most_lethal_weapons
}
