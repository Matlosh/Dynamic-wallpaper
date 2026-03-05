use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct Date {
    day: String,
    month: String,
    year: String
}

#[derive(Deserialize)]
struct Section {
    name: String,
    // date cron like format
    date: Date,
    // images/image path or url
    source: String
}

struct ConfigSection {
    // default image path
    default: String,
    // image fetch retry count
    retry_count: u8,
    // how often should the "cron" date be rechecked
    refresh_rate: u16
}

pub struct Settings {
    config: ConfigSection,
    sections: Vec<Section>
}

impl Settings {
    // pub fn new(settings_path: &String) -> Settings {

    // }

    fn parse_settings(settings_path: &String) {
        let contents = match fs::read_to_string(settings_path) {
            Ok(content) => content,
            Err(_) => {
                panic!("There was an error reading settings file");
            }
        };

    }
} 