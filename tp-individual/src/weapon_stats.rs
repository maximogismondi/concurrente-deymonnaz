use rayon::prelude::*;
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

    pub fn merge(&mut self, other: &mut WeaponStats) {
        self.count += other.count;
        self.total_distance += other.total_distance;
    }
}

pub fn weapon_stats_from_deaths(deaths: &Vec<Death>) -> Vec<(&String, WeaponStats)> {
    deaths
        .par_iter()
        .fold(
            || HashMap::new(),
            |mut acc, death| {
                acc.entry(&death.killed_by)
                    .or_insert_with(|| WeaponStats::new(death.distance()))
                    .add_death(death.distance());

                acc
            },
        )
        .reduce(
            || HashMap::new(),
            |mut acc, map| {
                // Merge the HashMaps from different threads
                for (weapon, mut stats) in map {
                    acc.entry(weapon)
                        .or_insert_with(|| WeaponStats::new(0.0))
                        .merge(&mut stats);
                }
                acc
            },
        )
        .into_iter()
        .collect()
}

pub fn get_top_weapons(
    mut weapon_stats: Vec<(&String, WeaponStats)>,
    weapon_count: usize,
) -> Vec<(&String, WeaponStats)> {
    weapon_stats.par_sort_by(|a, b| {
        let a = a.1.count;
        let b = b.1.count;

        b.cmp(&a)
    });

    weapon_stats.truncate(weapon_count);

    weapon_stats
}
