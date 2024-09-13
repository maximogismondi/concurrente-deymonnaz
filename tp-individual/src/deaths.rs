pub struct Death {
    pub killed_by: Option<String>,
    pub killer_name: Option<String>,
    killer_position_x: Option<f64>,
    killer_position_y: Option<f64>,
    victim_position_x: Option<f64>,
    victim_position_y: Option<f64>,
}

impl Death {
    pub fn from_csv_record(record: String) -> Result<Self, String> {
        let fields = record.split(',').collect::<Vec<_>>();

        if fields.len() != 12 {
            return Err(format!("Invalid number of fields: {}", fields.len()));
        }

        let killed_by = (!fields[0].is_empty()).then(|| fields[0].to_string());
        let killer_name = (!fields[1].is_empty()).then(|| fields[1].to_string());

        let killer_position_x = fields[3].parse::<f64>().ok();
        let killer_position_y = fields[4].parse::<f64>().ok();

        let victim_position_x = fields[10].parse::<f64>().ok();
        let victim_position_y = fields[11].parse::<f64>().ok();

        Ok(Self {
            killed_by,
            killer_name,
            killer_position_x,
            killer_position_y,
            victim_position_x,
            victim_position_y,
        })
    }

    pub fn distance(&self) -> Option<f64> {
        let killer_x = self.killer_position_x?;
        let killer_y = self.killer_position_y?;
        let victim_x = self.victim_position_x?;
        let victim_y = self.victim_position_y?;

        Some(((killer_x - victim_x).powi(2) + (killer_y - victim_y).powi(2)).sqrt())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const COMPLETE_RECORD: &str = "AK47,Player1,1.0,0.0,0.0,map,match-id,123,Player2,1.0,100.0,0.0";
    const NO_DISTANCE_RECORD: &str = "AK47,Player1,1.0,,,map,match-id,123,Player2,1.0,,";
    const NO_WEAPON_RECORD: &str = ",Player1,1.0,0.0,0.0,map,match-id,123,Player2,1.0,100.0,0.0";
    const NO_KILLER_RECORD: &str = "AK47,,1.0,0.0,0.0,map,match-id,123,Player2,1.0,100.0,0.0";

    #[test]
    fn test_from_complete_csv_record() {
        let record = COMPLETE_RECORD.to_string();
        let death = Death::from_csv_record(record).unwrap();

        assert_eq!(death.killed_by, Some("AK47".to_string()));
        assert_eq!(death.killer_name, Some("Player1".to_string()));
        assert_eq!(death.killer_position_x, Some(0.0));
        assert_eq!(death.killer_position_y, Some(0.0));
        assert_eq!(death.victim_position_x, Some(100.0));
        assert_eq!(death.victim_position_y, Some(0.0));
    }

    #[test]
    fn test_invalid_number_of_fields() {
        let record = "AK47,Player1,1.0,0.0,0.0,map,match-id,123,Player2,1.0,100.0".to_string();
        let death = Death::from_csv_record(record);

        assert!(death.is_err());
    }

    #[test]
    fn test_distance() {
        let record = COMPLETE_RECORD.to_string();
        let death = Death::from_csv_record(record).unwrap();

        assert_eq!(death.distance(), Some(100.0));
    }

    #[test]
    fn test_no_distance() {
        let record = NO_DISTANCE_RECORD.to_string();
        let death = Death::from_csv_record(record).unwrap();

        assert_eq!(death.distance(), None);
    }

    #[test]
    fn test_no_weapon() {
        let record = NO_WEAPON_RECORD.to_string();
        let death = Death::from_csv_record(record).unwrap();

        assert_eq!(death.killed_by, None);
    }

    #[test]
    fn test_no_killer() {
        let record = NO_KILLER_RECORD.to_string();
        let death = Death::from_csv_record(record).unwrap();

        assert_eq!(death.killer_name, None);
    }
}
