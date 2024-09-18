use std::collections::HashMap;

use rayon::{prelude::*, ThreadPool};

use crate::{
    deaths::Death, player_stats::PlayerStats, sorting::retain_top_elements,
    weapon_stats::WeaponStats,
};

/// A struct that holds the stats of the game.
pub struct Stats {
    total_deaths: usize,
    players: HashMap<String, PlayerStats>,
    weapons: HashMap<String, WeaponStats>,
}

impl Stats {
    /// Creates a new empty `Stats` instance.
    fn new() -> Self {
        Self {
            total_deaths: 0,
            players: HashMap::new(),
            weapons: HashMap::new(),
        }
    }

    /// Creates a new `Stats` instance from a parallel iterator of `Death` instances.
    /// The `pool` parameter is used to parallelize the processing of the deaths.
    pub fn from_deaths(deaths: impl ParallelIterator<Item = Death>, pool: &ThreadPool) -> Self {
        pool.install(|| {
            deaths
                .fold(Stats::new, |mut acc, death| {
                    acc.total_deaths += 1;

                    let death_distance = death.distance();
                    let killer_name = death.killer_name;
                    let killed_by = death.killed_by;
                    let killed_by_clone = killed_by.clone();

                    if let Some(killer_name) = killer_name {
                        acc.players
                            .entry(killer_name)
                            .or_insert_with(PlayerStats::new)
                            .add_death(killed_by_clone);
                    }

                    if let Some(killed_by) = killed_by {
                        acc.weapons
                            .entry(killed_by)
                            .or_insert_with(WeaponStats::new)
                            .add_death(death_distance);
                    }

                    acc
                })
                .reduce(Stats::new, |mut acc1, acc2| {
                    acc1.merge(acc2);
                    acc1
                })
        })
    }

    /// Merges another `Stats` instance into this one.
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

    /// Filters the top `player_count` players and the top `weapon_count` weapons of each player.
    /// The filtering is done in parallel using the `pool` parameter.
    pub fn filter_top_killers(
        &mut self,
        player_count: usize,
        weapon_count: usize,
        pool: &ThreadPool,
    ) {
        retain_top_elements(&mut self.players, player_count, pool);

        self.players.iter_mut().for_each(|(_, player_stats)| {
            player_stats.filter_top_weapons(weapon_count, pool);
        });
    }

    /// Filters the top `weapon_count` weapons.
    /// The filtering is done in parallel using the `pool` parameter.
    pub fn filter_top_weapons(&mut self, weapon_count: usize, pool: &ThreadPool) {
        retain_top_elements(&mut self.weapons, weapon_count, pool);
    }

    /// Returns the stats of the game in a JSON format.
    pub fn json_display(&self) -> serde_json::Value {
        let top_killers = self
            .players
            .iter()
            .map(|(player_name, player_stats)| (player_name, player_stats.json_display()))
            .collect::<HashMap<_, _>>();

        let top_weapons = self
            .weapons
            .iter()
            .map(|(weapon_name, weapon_stats)| {
                (weapon_name, weapon_stats.json_display(self.total_deaths))
            })
            .collect::<HashMap<_, _>>();

        serde_json::json!({
            "top_killers": top_killers,
            "top_weapons": top_weapons,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_json_diff::assert_json_eq;
    use rayon::ThreadPoolBuilder;
    use serde_json::json;

    const DEATH_RECORD_1: &str = "AK47,Player1,1.0,0.0,0.0,map,match-id,123,Player2,1.0,100.0,0.0";
    const DEATH_RECORD_2: &str = "AK47,Player2,1.0,0.0,0.0,map,match-id,123,Player1,1.0,100.0,0.0";
    const DEATH_RECORD_3: &str = "M4A4,Player1,1.0,0.0,0.0,map,match-id,123,Player2,1.0,100.0,0.0";

    fn pool() -> ThreadPool {
        ThreadPoolBuilder::new().num_threads(1).build().unwrap()
    }

    fn stats_from_deaths(deaths: Vec<&str>) -> Stats {
        let deaths = deaths
            .into_par_iter()
            .map(|record| Death::from_csv_record(record.to_string()).unwrap());

        Stats::from_deaths(deaths, &pool())
    }

    #[test]
    fn test_stats_from_deaths() {
        let stats = stats_from_deaths(vec![DEATH_RECORD_1]);

        assert_eq!(stats.total_deaths, 1);
        assert_eq!(stats.players.len(), 1);
        assert_eq!(stats.weapons.len(), 1);
    }

    #[test]
    fn test_stats_from_multiple_deaths() {
        let stats = stats_from_deaths(vec![DEATH_RECORD_1, DEATH_RECORD_1]);

        assert_eq!(stats.total_deaths, 2);
        assert_eq!(stats.players.len(), 1);
        assert_eq!(stats.weapons.len(), 1);
    }

    #[test]
    fn test_stats_from_multiple_players() {
        let stats = stats_from_deaths(vec![DEATH_RECORD_1, DEATH_RECORD_2]);

        assert_eq!(stats.total_deaths, 2);
        assert_eq!(stats.players.len(), 2);
        assert_eq!(stats.weapons.len(), 1);
    }

    #[test]
    fn test_stats_from_multiple_weapons() {
        let stats = stats_from_deaths(vec![DEATH_RECORD_1, DEATH_RECORD_3]);

        assert_eq!(stats.total_deaths, 2);
        assert_eq!(stats.players.len(), 1);
        assert_eq!(stats.weapons.len(), 2);
    }

    #[test]
    fn test_stats_merge() {
        let mut stats_1 = stats_from_deaths(vec![DEATH_RECORD_1]);
        let stats_2 = stats_from_deaths(vec![DEATH_RECORD_2, DEATH_RECORD_3]);

        stats_1.merge(stats_2);

        assert_eq!(stats_1.total_deaths, 3);
        assert_eq!(stats_1.players.len(), 2);
        assert_eq!(stats_1.weapons.len(), 2);
    }

    #[test]
    fn test_filter_top_killers() {
        let mut stats = stats_from_deaths(vec![DEATH_RECORD_1, DEATH_RECORD_2]);

        stats.filter_top_killers(1, 1, &pool());

        assert_eq!(stats.players.len(), 1);
        assert_eq!(stats.weapons.len(), 1);
    }

    #[test]
    fn test_filter_top_weapons() {
        let mut stats = stats_from_deaths(vec![DEATH_RECORD_1, DEATH_RECORD_1, DEATH_RECORD_3]);

        stats.filter_top_weapons(1, &pool());

        assert_eq!(stats.weapons.len(), 1);
    }

    #[test]
    fn test_filter_on_players_tie_resolve_alphabetically() {
        let mut stats = stats_from_deaths(vec![DEATH_RECORD_2, DEATH_RECORD_1]);

        stats.filter_top_killers(1, 1, &pool());

        assert_eq!(stats.players.len(), 1);
        assert!(stats.players.contains_key("Player1"));
        assert!(!stats.players.contains_key("Player2"));
    }

    #[test]
    fn test_filter_on_weapons_tie_resolve_alphabetically() {
        let mut stats = stats_from_deaths(vec![DEATH_RECORD_3, DEATH_RECORD_1]);

        stats.filter_top_weapons(1, &pool());

        assert_eq!(stats.weapons.len(), 1);
        assert!(stats.weapons.contains_key("AK47"));
    }

    #[test]
    fn test_json_display() {
        let stats = stats_from_deaths(vec![DEATH_RECORD_1, DEATH_RECORD_3]);

        let json_stats = stats.json_display();

        let expected_json = json!({
            "top_killers": {
                "Player1": {
                    "deaths": 2,
                    "weapons_percentage": {
                        "AK47": 50.0,
                        "M4A4": 50.0
                    }
                }
            },
            "top_weapons": {
                "AK47": {
                    "deaths_percentage": 50.0,
                    "average_distance": 100.0
                },
                "M4A4": {
                    "deaths_percentage": 50.0,
                    "average_distance": 100.0
                }
            }
        });

        assert_json_eq!(expected_json, json_stats);
    }
}
