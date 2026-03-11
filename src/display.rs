use crate::settings::{ApiSettings, Section, Settings};
use chrono::{DateTime, Duration, Local, Utc};
use cron::Schedule;
use procfs::process::Process;
use rand::seq::IndexedRandom;
use reqwest::blocking::Response;
use serde_json::Value;
use std::env;
use std::io::{Error, Write};
use std::str::FromStr;
use std::thread;
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
struct Plan {
    section: Section,
    schedule: Schedule,
}

#[derive(Debug)]
pub struct Display {
    settings: Settings,
    tool: WallpaperTools,
    // when wallpaper was last set
    wallpaper_last_timestamp: DateTime<Local>,
    // wallpaper cron schedules
    plans: Vec<Plan>,
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

        let mut plans: Vec<Plan> = Vec::new();

        for section in &settings.sections {
            plans.push(Plan {
                section: section.clone(),
                schedule: match Schedule::from_str(&section.date) {
                    Ok(s) => s,
                    Err(e) => continue,
                },
            });
        }

        Display {
            settings: settings,
            tool: running_tool.unwrap(),
            wallpaper_last_timestamp: Local::now(),
            plans,
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
    fn read_local(&self, path: &str) -> Result<String, Error> {
        let is_dir = fs::metadata(path)?.is_dir();

        if is_dir {
            let dir = fs::read_dir(path)?;
            let mut file_paths: Vec<String> = Vec::new();
            let mut rng = rand::rng();

            for entry in dir {
                let entry = match entry {
                    Ok(e) => e,
                    Err(_) => continue,
                };
                let path_buf = entry.path();
                let path = path_buf.to_str().unwrap_or_else(|| "");

                if path.len() == 0 {
                    continue;
                }

                if self.is_file_acceptable(path) {
                    file_paths.push(path.to_string());
                }
            }

            let random_path = file_paths.choose(&mut rng);
            if random_path.is_some() {
                return Ok(random_path.unwrap().to_string());
            }
        } else {
            if self.is_file_acceptable(path) {
                return Ok(path.to_string());
            }
        }

        Err(Error::new(
            std::io::ErrorKind::InvalidFilename,
            "Couldn't load any file/dir",
        ))
    }

    fn download_to_tmp(&self, url: &str) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
        let resp = reqwest::blocking::get(url)?;
        let bytes = resp.bytes()?;

        let tmp_dir = env::temp_dir();
        let filename = tmp_dir.join("dynamic_wallpaper");
        let mut file = std::fs::File::create(&filename)?;
        file.write_all(&bytes)?;

        Ok(filename)
    }

    // checks if url is a image source or api (json)
    // if json then tries to read and find the image url in fetched request
    fn read_api(
        &self,
        url: &str,
        api: &Option<ApiSettings>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let resp: Response = reqwest::blocking::get(url)?;
        let headers = resp.headers();
        let content_type = match headers.get("content-type") {
            Some(h) => h,
            None => {
                return Err(
                    Error::new(std::io::ErrorKind::InvalidData, "Content-type missing").into(),
                );
            }
        }
        .to_str()
        .unwrap_or("");

        if content_type.contains("json") {
            if api.is_none() {
                return Err(Error::new(
                    std::io::ErrorKind::InvalidData,
                    "url resolves to json data, but API settings are missing",
                )
                .into());
            }

            let settings = api.clone().unwrap();
            let mut content: Value = resp.json()?;

            println!("{content:#?}");
            let mut source_url = String::new();
            for key in settings.source_url_key {
                let value = content[&key].clone();
                if value.is_object() {
                    content = content[&key].clone();
                } else {
                    source_url = value.as_str().unwrap_or("").to_string();
                }
            }

            println!("{source_url:#?}");
            return self.read_api(&source_url, &None);
        }

        let filename = self.download_to_tmp(url)?;
        let filename = match filename.to_str() {
            Some(name) => name,
            None => {
                return Err(Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Failed parsing downloaded file's filename",
                )
                .into());
            }
        };

        return Ok(filename.to_string());
    }

    fn set_wallpaper(&self, filepath: &str) {
        let result = match self.tool {
            WallpaperTools::Hyprpaper => Command::new("hyprctl")
                .arg("hyprpaper")
                .arg("wallpaper")
                .arg(filepath)
                .spawn(),
            WallpaperTools::Swww => Command::new("swww").arg("img").arg(filepath).spawn(),
            WallpaperTools::Mpvpaper => Command::new("mpvpaper").arg("ALL").arg(filepath).spawn(),
            WallpaperTools::Swaybg => {
                Command::new("pkill").arg("swaybg").output().unwrap();
                Command::new("swaybg")
                    .arg("-i")
                    .arg(filepath)
                    .arg("-m")
                    .arg("fit")
                    .spawn()
            }
        };

        if result.is_err() {
            println!("Failed to set wallpaper for {filepath}");
        }
    }

    fn display_image(&mut self) {
        let interval = Duration::seconds(self.settings.config.refresh_rate);
        let past_datetime = self.wallpaper_last_timestamp - interval;

        for plan in &self.plans {
            let mut datetime = plan.schedule.after(&past_datetime).take(1);
            let datetime = match datetime.next() {
                Some(dt) => dt,
                None => continue,
            };

            println!("{} {datetime:#?}", self.wallpaper_last_timestamp);
            if self.wallpaper_last_timestamp > datetime {
                let filepath = self.read_local(&plan.section.source);
                if filepath.is_ok() {
                    self.set_wallpaper(filepath.unwrap().as_str());
                    break;
                }

                let filepath = self.read_api(&plan.section.source, &plan.section.api);
                if filepath.is_ok() {
                    self.set_wallpaper(filepath.unwrap().as_str());
                } else {
                    println!(
                        "An error occured during fetching source from API: {}",
                        filepath.err().unwrap()
                    );
                }

                break;
            }
        }

        self.wallpaper_last_timestamp = Local::now();
    }

    pub fn setup_automatic_display(&mut self) {
        let interval = Duration::seconds(self.settings.config.refresh_rate);
        let mut next_time = Utc::now() + interval;

        loop {
            next_time += interval;
            let now = Utc::now();
            if now < next_time {
                thread::sleep((next_time - now).to_std().unwrap());
                self.display_image();
            }
        }
    }
}
