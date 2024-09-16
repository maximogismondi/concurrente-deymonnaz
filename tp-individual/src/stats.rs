use std::collections::HashMap;

use rayon::prelude::*;

use crate::{
    deaths::Death, player_stats::PlayerStats, sorting::retain_top_elements,
    weapon_stats::WeaponStats,
};

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

    pub fn filter_top_killers(&mut self, player_count: usize, weapon_count: usize) {
        retain_top_elements(&mut self.players, player_count);

        self.players.iter_mut().for_each(|(_, stats)| {
            retain_top_elements(&mut stats.weapons, weapon_count);
        });
    }

    pub fn filter_top_weapons(&mut self, weapon_count: usize) {
        retain_top_elements(&mut self.weapons, weapon_count);
    }

    pub fn json_display(&self) -> serde_json::Value {
        // asume top_killers and top_weapons can be displayed as json as well

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
// implement display for Stats

#[cfg(test)]

mod tests {
    use super::*;
    use assert_json_diff::assert_json_eq;
    use serde_json::json;

    const DEATH_RECORD_1: &str = "AK47,Player1,1.0,0.0,0.0,map,match-id,123,Player2,1.0,100.0,0.0";
    const DEATH_RECORD_2: &str = "AK47,Player2,1.0,0.0,0.0,map,match-id,123,Player1,1.0,100.0,0.0";
    const DEATH_RECORD_3: &str = "M4A4,Player1,1.0,0.0,0.0,map,match-id,123,Player2,1.0,100.0,0.0";

    #[test]
    fn test_stats_from_deaths() {
        let deaths = vec![DEATH_RECORD_1.to_string()]
            .into_par_iter()
            .map(|record| Death::from_csv_record(record).unwrap());

        let stats = Stats::from_deaths(deaths);

        assert_eq!(stats.total_deaths, 1);
        assert_eq!(stats.players.len(), 1);
        assert_eq!(stats.weapons.len(), 1);
    }

    #[test]
    fn test_stats_from_multiple_deaths() {
        let deaths = vec![DEATH_RECORD_1.to_string(), DEATH_RECORD_1.to_string()]
            .into_par_iter()
            .map(|record| Death::from_csv_record(record).unwrap());

        let stats = Stats::from_deaths(deaths);

        assert_eq!(stats.total_deaths, 2);
        assert_eq!(stats.players.len(), 1);
        assert_eq!(stats.weapons.len(), 1);
    }

    #[test]
    fn test_stats_from_multiple_players() {
        let deaths = vec![DEATH_RECORD_1.to_string(), DEATH_RECORD_2.to_string()]
            .into_par_iter()
            .map(|record| Death::from_csv_record(record).unwrap());

        let stats = Stats::from_deaths(deaths);

        assert_eq!(stats.total_deaths, 2);
        assert_eq!(stats.players.len(), 2);
        assert_eq!(stats.weapons.len(), 1);
    }

    #[test]
    fn test_stats_from_multiple_weapons() {
        let deaths = vec![DEATH_RECORD_1.to_string(), DEATH_RECORD_3.to_string()]
            .into_par_iter()
            .map(|record| Death::from_csv_record(record).unwrap());

        let stats = Stats::from_deaths(deaths);

        assert_eq!(stats.total_deaths, 2);
        assert_eq!(stats.players.len(), 1);
        assert_eq!(stats.weapons.len(), 2);
    }

    #[test]
    fn test_stats_merge() {
        let deaths_1 = vec![DEATH_RECORD_1.to_string()]
            .into_par_iter()
            .map(|record| Death::from_csv_record(record).unwrap());

        let deaths_2 = vec![DEATH_RECORD_2.to_string(), DEATH_RECORD_3.to_string()]
            .into_par_iter()
            .map(|record| Death::from_csv_record(record).unwrap());

        let mut stats_1 = Stats::from_deaths(deaths_1);
        let stats_2 = Stats::from_deaths(deaths_2);

        stats_1.merge(stats_2);

        assert_eq!(stats_1.total_deaths, 3);
        assert_eq!(stats_1.players.len(), 2);
        assert_eq!(stats_1.weapons.len(), 2);
    }

    #[test]
    fn test_filter_top_killers() {
        let deaths = vec![DEATH_RECORD_1.to_string(), DEATH_RECORD_2.to_string()]
            .into_par_iter()
            .map(|record| Death::from_csv_record(record).unwrap());

        let mut stats = Stats::from_deaths(deaths);

        stats.filter_top_killers(1, 1);

        assert_eq!(stats.players.len(), 1);
        assert_eq!(stats.weapons.len(), 1);
    }

    #[test]
    fn test_filter_top_killers_weapons() {
        let deaths = vec![
            DEATH_RECORD_1.to_string(),
            DEATH_RECORD_1.to_string(),
            DEATH_RECORD_3.to_string(),
        ]
        .into_par_iter()
        .map(|record| Death::from_csv_record(record).unwrap());

        let mut stats = Stats::from_deaths(deaths);

        stats.filter_top_killers(1, 1);

        assert_eq!(stats.players.len(), 1);
        assert_eq!(stats.players.get("Player1").unwrap().weapons.len(), 1);
    }

    #[test]
    fn test_filter_top_weapons() {
        let deaths = vec![
            DEATH_RECORD_1.to_string(),
            DEATH_RECORD_1.to_string(),
            DEATH_RECORD_3.to_string(),
        ]
        .into_par_iter()
        .map(|record| Death::from_csv_record(record).unwrap());

        let mut stats = Stats::from_deaths(deaths);

        stats.filter_top_weapons(1);

        assert_eq!(stats.weapons.len(), 1);
    }

    #[test]
    fn test_json_display() {
        let deaths = vec![DEATH_RECORD_1.to_string(), DEATH_RECORD_3.to_string()]
            .into_par_iter()
            .map(|record| Death::from_csv_record(record).unwrap());

        let stats = Stats::from_deaths(deaths);

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
