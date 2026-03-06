use std::env;
use settings::Settings;

mod settings;

fn main() {
    if env::args().len() < 2 {
        println!("Usage: [executable] [settings file path]");
        return
    }

    let settings_path = match env::args().nth(1) {
        Some(path) => path,
        None => {
            panic!("Error processing settings path");
        }
    };

    let mut settings: Settings = Settings::new();
    Settings::parse_settings(&settings_path, &mut settings);
    println!("{settings:?}");
}