use std::collections::HashMap;

use crate::sorting::truncate_top_elements;

pub type PlayerWeaponStats = HashMap<String, usize>;

#[derive(Eq, PartialEq)]
pub struct PlayerStats {
    pub deaths: usize,
    pub weapons: PlayerWeaponStats,
}

impl PartialOrd for PlayerStats {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.deaths.cmp(&other.deaths))
    }
}

impl Ord for PlayerStats {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.deaths.cmp(&other.deaths)
    }
}

impl PlayerStats {
    pub fn new() -> Self {
        Self {
            deaths: 0,
            weapons: HashMap::new(),
        }
    }

    pub fn add_death(&mut self, weapon: String) {
        self.deaths += 1;
        if let Some(count) = self.weapons.get_mut(&weapon) {
            *count += 1;
        } else {
            self.weapons.insert(weapon, 1);
        }
    }

    pub fn merge(&mut self, other: Self) {
        self.deaths += other.deaths;
        for (weapon, count) in other.weapons.into_iter() {
            if let Some(existing_count) = self.weapons.get_mut(&weapon) {
                *existing_count += count;
            } else {
                self.weapons.insert(weapon, count);
            }
        }
    }
}

pub fn filter_top_killers(
    player_stats: &mut HashMap<String, PlayerStats>,
    player_count: usize,
    weapon_count: usize,
) {
    truncate_top_elements(player_stats, player_count);

    player_stats.iter_mut().for_each(|(_, stats)| {
        get_top_weapons(&mut stats.weapons, weapon_count);
    });
}

pub fn get_top_weapons(weapon_stats: &mut PlayerWeaponStats, weapon_count: usize) {
    truncate_top_elements(weapon_stats, weapon_count);
}
