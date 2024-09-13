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
}

#[cfg(test)]

mod tests {
    use super::*;

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
}
