use serde_json::json;

use crate::{stats::Stats, PADRON};

pub fn save_as_json(stats: Stats, output_path: &str) {
    let mut json_stats = stats.json_display();

    json_stats
        .as_object_mut()
        .unwrap()
        .insert("padron".to_string(), json!(PADRON));

    let json_str = serde_json::to_string_pretty(&json_stats).expect("Failed to serialize to JSON");
    std::fs::write(output_path, json_str).expect("Failed to write JSON to file");
}
