pub struct Deaths {
    pub killed_by: String,
    pub killer_name: String,
    // killer_placement: f32,
    killer_position_x: f32,
    killer_position_y: f32,
    // map: String,
    // match_id: String,
    // time: usize,
    // victim_name: String,
    // victim_placement: f32,
    victim_position_x: f32,
    victim_position_y: f32,
}

impl Deaths {
    pub fn from_csv_record(record: String) -> Result<Self, String> {
        let fields = record.split(',').collect::<Vec<_>>();

        if fields.len() != 12 {
            return Err(format!("Invalid number of fields: {}", fields.len()));
        }

        let killed_by = fields[0].to_string();
        let killer_name = fields[1].to_string();
        // let killer_placement = fields[2].parse::<f32>().map_err(|e| e.to_string())?;
        let killer_position_x = fields[3].parse::<f32>().map_err(|e| e.to_string())?;
        let killer_position_y = fields[4].parse::<f32>().map_err(|e| e.to_string())?;
        // let map = fields[5].to_string();
        // let match_id = fields[6].to_string();
        // let time = fields[7].parse::<usize>().map_err(|e| e.to_string())?;
        // let victim_name = fields[8].to_string();
        // let victim_placement = fields[9].parse::<f32>().map_err(|e| e.to_string())?;
        let victim_position_x = fields[10].parse::<f32>().map_err(|e| e.to_string())?;
        let victim_position_y = fields[11].parse::<f32>().map_err(|e| e.to_string())?;

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

    pub fn distance(&self) -> f32 {
        let dx = self.killer_position_x - self.victim_position_x;
        let dy = self.killer_position_y - self.victim_position_y;
        (dx * dx + dy * dy).sqrt()
    }
}
