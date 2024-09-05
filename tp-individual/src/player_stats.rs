use rayon::prelude::*;
use std::collections::HashMap;

use crate::deaths::Death;

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

    pub fn merge(&mut self, other: &mut PlayerStats) {
        self.total += other.total;
        for (weapon, count) in other.weapons.iter() {
            if let Some(self_count) = self.weapons.get_mut(weapon) {
                *self_count += count;
            } else {
                self.weapons.insert(weapon.to_string(), *count);
            }
        }
    }
}

pub fn player_stats_from_deaths(deaths: &Vec<Death>) -> Vec<(&String, PlayerStats)> {
    deaths
        .par_iter()
        .fold(
            || HashMap::new(),
            |mut acc, death| {
                acc.entry(&death.killer_name)
                    .or_insert_with(|| PlayerStats::new(&death.killed_by))
                    .add_death(&death.killed_by);

                acc
            },
        )
        .reduce(
            || HashMap::new(),
            |mut acc, map| {
                for (player, mut stats) in map {
                    acc.entry(player)
                        .or_insert_with(|| PlayerStats {
                            total: 0,
                            weapons: HashMap::new(),
                        })
                        .merge(&mut stats);
                }
                acc
            },
        )
        .into_iter()
        .collect()

    // deaths.iter().fold(HashMap::new(), |mut acc, death| {
    //     if let Some(stats) = acc.get_mut(&death.killer_name) {
    //         stats.add_death(&death.killed_by);
    //     } else {
    //         acc.insert(
    //             death.killer_name.clone(),
    //             PlayerStats::new(&death.killed_by),
    //         );
    //     }

    //     acc
    // })
}

pub fn get_top_killers(
    mut player_stats: Vec<(&String, PlayerStats)>,
    player_count: usize,
    weapon_count: usize,
) -> Vec<(&String, PlayerStats)> {
    player_stats.par_sort_by(|a, b| {
        let a = a.1.total;
        let b = b.1.total;
        b.cmp(&a)
    });

    player_stats.truncate(player_count);

    player_stats
        .par_iter()
        .map(|(name, stats)| {
            let name = *name;
            let total = stats.total;
            let weapons = get_top_weapons(&stats.weapons, weapon_count);
            (name, PlayerStats { total, weapons })
        })
        .collect()
}

fn get_top_weapons(weapons_stats: &PlayerWeaponStats, weapon_count: usize) -> PlayerWeaponStats {
    let mut weapon_names: Vec<&String> = weapons_stats.keys().collect();

    weapon_names.par_sort_by(|a, b| {
        let a = weapons_stats.get(*a).unwrap();
        let b = weapons_stats.get(*b).unwrap();
        b.cmp(&a)
    });

    weapon_names
        .par_iter()
        .take(weapon_count)
        .map(|name| {
            let count = weapons_stats.get(*name).unwrap();
            (name.to_string(), *count)
        })
        .collect()
}
