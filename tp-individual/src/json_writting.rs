use serde_json::json;

use crate::{stats::Stats, PADRON};

/// Save the stats as a JSON file in the given path.
pub fn save_as_json(stats: Stats, output_path: &str) {
    let mut json_stats = stats.json_display();

    match json_stats.as_object_mut() {
        Some(obj) => {
            obj.insert("padron".to_string(), json!(PADRON));
        }
        None => {
            json_stats = json!({
                "padron": PADRON,
                "top_killers": {},
                "top_weapons": {},
            });
        }
    }

    let json_str = match serde_json::to_string_pretty(&json_stats) {
        Ok(json_str) => json_str,
        Err(err) => {
            eprintln!("Failed to serialize JSON: {}", err);
            return;
        }
    };

    match std::fs::write(output_path, json_str) {
        Ok(_) => println!("Stats saved as JSON in {}", output_path),
        Err(err) => eprintln!("Failed to save stats as JSON: {}", err),
    }
}
