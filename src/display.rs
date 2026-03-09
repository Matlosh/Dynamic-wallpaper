use crate::settings::Settings;
use procfs::process::Process;
use std::io::Error;
use std::{fs, process::Command};

const ACCEPTABLE_EXTS: [&str; 4] = ["jpg", "png", "webp", "jpeg"];

#[derive(Debug)]
enum WallpaperTools {
    Hyprpaper,
    Swww,
    Mpvpaper,
    Swaybg,
}

#[derive(Debug)]
pub struct Display {
    settings: Settings,
    tool: WallpaperTools,
}

impl Display {
    pub fn new(settings: Settings) -> Display {
        // try to find matching wallpaper tool
        let all_processes: Vec<Process> = procfs::process::all_processes()
            .expect("Can't load /proc")
            .filter_map(|p| match p {
                Ok(p) => Some(p),
                Err(e) => match e {
                    x => {
                        println!("Can't read process due to an error: {x:?}");
                        None
                    }
                },
            })
            .collect();

        let mut running_tool: Option<WallpaperTools> = None;
        for process in all_processes {
            let exec_name = match process.stat() {
                Ok(s) => s.comm,
                Err(_) => "".to_string(),
            };

            let wallpaper_tool: Option<WallpaperTools> = match exec_name.as_str() {
                "hyprpaper" => Some(WallpaperTools::Hyprpaper),
                "swww" => Some(WallpaperTools::Swww),
                "mpvpaper" => Some(WallpaperTools::Mpvpaper),
                "swaybg" => Some(WallpaperTools::Swaybg),
                _ => None,
            };

            if wallpaper_tool.is_some() {
                running_tool = wallpaper_tool;
                break;
            }
        }

        if running_tool.is_none() {
            panic!(
                "Cannot find any of the supported wallpaper tools. Check out docs for more info."
            );
        }

        Display {
            settings,
            tool: running_tool.unwrap(),
        }
    }

    fn is_file_acceptable(&self, path: &str) -> bool {
        match fs::read(path) {
            Ok(data) => match infer::get(&data) {
                Some(kind) => ACCEPTABLE_EXTS.contains(&kind.extension()),
                None => false,
            },
            Err(_) => false,
        }
    }

    // returns path to a file or (in case path is a directory) finds random applicable source file path
    // that is placed directly in the provided directory
    // fn read_local(&self, path: &str) -> Result<String, Error> {
    //     let is_dir = fs::metadata(path)?.is_dir();

    //     if is_dir {
    //         let dir = fs::read_dir(path)?;

    //         for entry in dir {
    //             let entry = entry?;
    //         }
    //     } else {
    //         if self.is_file_acceptable(path) {
    //             self.set_wallpaper(path);
    //         }
    //     }

    //     Error::new(std::io::ErrorKind::InvalidFilename, );
    // }

    fn read_api(&self, url: &str) {}

    pub fn set_wallpaper(&self, filepath: &str) {
        let result = match self.tool {
            WallpaperTools::Hyprpaper => Command::new("hyprctl")
                .arg("hyprpaper")
                .arg("wallpaper")
                .arg(filepath)
                .spawn(),
            WallpaperTools::Swww => Command::new("swww").arg("img").arg(filepath).spawn(),
            WallpaperTools::Mpvpaper => Command::new("mpvpaper").arg("ALL").arg(filepath).spawn(),
            WallpaperTools::Swaybg => Command::new("swaybg").arg("-i").arg(filepath).spawn(),
        };

        if result.is_err() {
            println!("Failed to set wallpaper for {filepath}");
        }
    }

    pub fn display_image(&self) {}
}
