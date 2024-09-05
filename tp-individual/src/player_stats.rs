use std::collections::HashMap;

use crate::{deaths::Death, sorting::find_top_elements};

pub type PlayerWeaponStats = HashMap<String, usize>;

pub struct PlayerStats {
    pub total: usize,
    pub weapons: PlayerWeaponStats,
}

impl PlayerStats {
    pub fn new(weapon: &String) -> Self {
        Self {
            total: 1,
            weapons: vec![(weapon.to_string(), 1)].into_iter().collect(),
        }
    }

    pub fn add_death(&mut self, weapon: &String) {
        self.total += 1;
        if let Some(count) = self.weapons.get_mut(weapon) {
            *count += 1;
        } else {
            self.weapons.insert(weapon.to_string(), 1);
        }
    }
}

pub fn player_stats_from_deaths(deaths: &[Death]) -> HashMap<&String, PlayerStats> {
    deaths
        .iter()
        .fold(
            HashMap::new(),
            |mut acc: HashMap<&String, PlayerStats>, death| {
                if let Some(stats) = acc.get_mut(&death.killer_name) {
                    stats.add_death(&death.killed_by);
                } else {
                    acc.insert(&death.killer_name, PlayerStats::new(&death.killed_by));
                }

                acc
            },
        )
        .into_iter()
        .collect()
}

pub fn get_top_killers(
    player_stats: HashMap<&String, PlayerStats>,
    player_count: usize,
    weapon_count: usize,
) -> HashMap<&String, PlayerStats> {
    find_top_elements(player_stats, player_count, |a, b| {
        let a = a.total;
        let b = b.total;
        b.cmp(&a)
    })
    .into_iter()
    .map(|(name, stats)| {
        let weapons = get_top_weapons(stats.weapons, weapon_count);
        (
            name,
            PlayerStats {
                total: stats.total,
                weapons,
            },
        )
    })
    .collect()
}

pub fn get_top_weapons(weapon_stats: PlayerWeaponStats, weapon_count: usize) -> PlayerWeaponStats {
    find_top_elements(weapon_stats, weapon_count, |a, b| {
        let a = *a;
        let b = *b;
        b.cmp(&a)
    })
}
