use std::collections::HashMap;
use std::env;
use std::fs;
use serde::Deserialize;

struct Config {
    settings_path: String
}

#[derive(Deserialize)]
struct Section {
    // date cron like format
    date: String,
    // images path
    images: String
}

#[derive(Deserialize)]
struct Settings {
    #[serde(flatten)]
    rest: HashMap<String, Section>
}

fn load_settings(mut settings: &mut Settings) {
    let config = Config {
        settings_path: env::args().nth(1).unwrap()
    };

    let settings_str = fs::read_to_string(config.settings_path).unwrap();

    // settings = toml::from_str(&settings_str).unwrap();
}

fn main() {
    if env::args().len() < 2 {
        println!("Usage: [executable] [settings file path]");
        return
    }

    // for (key, value) in settings.rest {
    //     println!("{} {}", key, value.images);
    // }
}