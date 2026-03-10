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

    let display: Display = Display::new(settings.clone());
    // display.set_wallpaper(
    //     display
    //         .read_local("/home/matlosh/wallpapers")
    //         .unwrap()
    //         .as_str(),
    // );

    let api = settings
        .clone()
        .sections
        .clone()
        .get(0)
        .unwrap()
        .api
        .clone();
    display.set_wallpaper(
        display
            .read_api(
                "https://danbooru.donmai.us/posts.json?limit=1&page=[random]&tags=random%3A100",
                api,
            )
            .unwrap()
            .as_str(),
    );
    println!("{display:#?}");
}
