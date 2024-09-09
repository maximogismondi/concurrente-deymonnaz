use std::collections::HashMap;

use rayon::prelude::*;

use crate::{deaths::Death, player_stats::PlayerStats, weapon_stats::WeaponStats};

pub struct Stats {
    pub total_deaths: usize,
    pub players: HashMap<String, PlayerStats>,
    pub weapons: HashMap<String, WeaponStats>,
}

impl Stats {
    pub fn from_deaths(deaths: impl ParallelIterator<Item = Death>) -> Stats {
        deaths
            .fold(
                || Stats {
                    total_deaths: 0,
                    players: HashMap::new(),
                    weapons: HashMap::new(),
                },
                |mut acc, death| {
                    acc.total_deaths += 1;

                    let death_distance = death.distance();
                    let killer_name = death.killer_name;
                    let killed_by = death.killed_by;
                    let killer_by_clone = killed_by.clone();

                    acc.players
                        .entry(killer_name)
                        .or_insert_with(PlayerStats::new)
                        .add_death(killer_by_clone);

                    acc.weapons
                        .entry(killed_by)
                        .or_insert_with(WeaponStats::new)
                        .add_death(death_distance);

                    acc
                },
            )
            .reduce(
                || Stats {
                    total_deaths: 0,
                    players: HashMap::new(),
                    weapons: HashMap::new(),
                },
                |mut acc1, acc2| {
                    acc1.merge(acc2);
                    acc1
                },
            )
    }

    fn merge(&mut self, other: Stats) {
        self.total_deaths += other.total_deaths;

        for (name, other_player_stats) in other.players {
            if let Some(player_stats) = self.players.get_mut(&name) {
                player_stats.merge(other_player_stats);
            } else {
                self.players.insert(name, other_player_stats);
            }
        }

        for (name, other_weapon_stats) in other.weapons {
            self.weapons
                .entry(name)
                .and_modify(|main_weapon_stats| {
                    main_weapon_stats.merge(&other_weapon_stats);
                })
                .or_insert(other_weapon_stats);
        }
    }
}
