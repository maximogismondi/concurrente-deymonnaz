use std::collections::HashMap;

use crate::{deaths::Death, sorting::find_top_elements};

pub struct WeaponStats {
    pub count: usize,
    pub total_distance: f64,
}

impl WeaponStats {
    pub fn new(distance: f64) -> Self {
        Self {
            count: 1,
            total_distance: distance,
        }
    }

    pub fn add_death(&mut self, distance: f64) {
        self.count += 1;
        self.total_distance += distance;
    }
}

pub fn weapon_stats_from_deaths(deaths: &[Death]) -> HashMap<&String, WeaponStats> {
    deaths.iter().fold(
        HashMap::new(),
        |mut acc: HashMap<&String, WeaponStats>, death| {
            if let Some(stats) = acc.get_mut(&death.killed_by) {
                stats.add_death(death.distance());
            } else {
                acc.insert(&death.killed_by, WeaponStats::new(death.distance()));
            }

            acc
        },
    )
}

pub fn get_top_weapons(
    weapon_stats: HashMap<&String, WeaponStats>,
    weapon_count: usize,
) -> HashMap<&String, WeaponStats> {
    find_top_elements(weapon_stats, weapon_count, |a, b| {
        let a = a.count;
        let b = b.count;

        b.cmp(&a)
    })
}
