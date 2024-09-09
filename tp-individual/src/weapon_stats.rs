use std::collections::HashMap;

use crate::sorting::truncate_top_elements;

#[derive(Debug)]
pub struct WeaponStats {
    pub death_count: usize,
    pub death_count_with_distance: usize,
    pub total_distance: f64,
}

impl PartialEq for WeaponStats {
    fn eq(&self, other: &Self) -> bool {
        self.death_count == other.death_count
    }
}

impl PartialOrd for WeaponStats {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.death_count.cmp(&other.death_count))
    }
}

impl Eq for WeaponStats {}

impl Ord for WeaponStats {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.death_count.cmp(&other.death_count)
    }
}

impl WeaponStats {
    pub fn new() -> Self {
        Self {
            death_count: 0,
            death_count_with_distance: 0,
            total_distance: 0.0,
        }
    }

    pub fn add_death(&mut self, distance: Option<f64>) {
        self.death_count += 1;
        if let Some(distance) = distance {
            self.death_count_with_distance += 1;
            self.total_distance += distance;
        }
    }

    pub fn merge(&mut self, other: &Self) {
        self.death_count += other.death_count;
        self.total_distance += other.total_distance;
    }
}

pub fn filter_top_weapons(weapon_stats: &mut HashMap<String, WeaponStats>, weapon_count: usize) {
    truncate_top_elements(weapon_stats, weapon_count)
}
