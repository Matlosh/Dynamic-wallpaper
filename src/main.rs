use display::Display;
use settings::Settings;
use std::env;

mod display;
mod settings;

fn main() {
    if env::args().len() < 2 {
        println!("Usage: [executable] [settings file path]");
        return;
    }

    let settings_path = match env::args().nth(1) {
        Some(path) => path,
        None => {
            panic!("Error processing settings path");
        }
    };

    let mut settings: Settings = Settings::new();
    settings.parse_settings(&settings_path);

    let display: Display = Display::new(settings);
    display.set_wallpaper("/home/matlosh/wallpapers/yukino3.jpg");
    println!("{display:#?}");
}
