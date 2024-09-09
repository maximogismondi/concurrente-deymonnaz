pub struct Death {
    pub killed_by: Option<String>,
    pub killer_name: Option<String>,
    // killer_placement: f64,
    killer_position_x: Option<f64>,
    killer_position_y: Option<f64>,
    // map: String,
    // match_id: String,
    // time: usize,
    // victim_name: String,
    // victim_placement: f64,
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
        // let killer_placement = fields[2].parse::<f64>().map_err(|e| e.to_string())?;
        let killer_position_x = fields[3].parse::<f64>().ok();
        let killer_position_y = fields[4].parse::<f64>().ok();
        // let map = fields[5].to_string();
        // let match_id = fields[6].to_string();
        // let time = fields[7].parse::<usize>().map_err(|e| e.to_string())?;
        // let victim_name = fields[8].to_string();
        // let victim_placement = fields[9].parse::<f64>().map_err(|e| e.to_string())?;
        let victim_position_x = fields[10].parse::<f64>().ok();
        let victim_position_y = fields[11].parse::<f64>().ok();

        Ok(Self {
            killed_by,
            killer_name,
            // killer_placement,
            killer_position_x,
            killer_position_y,
            // map,
            // match_id,
            // time,
            // victim_name,
            // victim_placement,
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
