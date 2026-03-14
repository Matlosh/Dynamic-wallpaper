use crate::settings::{ApiSettings, Section, Settings};
use chrono::{DateTime, Duration, Local, Utc};
use cron::Schedule;
use procfs::process::Process;
use rand::seq::IndexedRandom;
use reqwest::blocking::{Client, Response};
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
    client: Client,
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

            let wallpaper_tool = match Display::find_wallpaper_tool(exec_name.as_str()) {
                Some(t) => t,
                None => continue,
            };

            running_tool = Some(wallpaper_tool);
            break;
        }

        let tool = match running_tool {
            Some(t) => t,
            None => match Display::find_wallpaper_tool(&settings.config.wallpaper_tool) {
                Some(v) => v,
                None => panic!(
                    "Cannot find any of the supported wallpaper tools or the default one. Check out docs for more info."
                ),
            },
        };

        let mut plans: Vec<Plan> = Vec::new();

        for section in &settings.sections {
            plans.push(Plan {
                section: section.clone(),
                schedule: match Schedule::from_str(&section.date) {
                    Ok(s) => s,
                    Err(_) => continue,
                },
            });
        }

        let client = Client::builder()
            .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:148.0) Gecko/20100101 Firefox/148.0")
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("Couldn't construct a reqwest Client");

        Display {
            settings: settings,
            tool: tool,
            wallpaper_last_timestamp: Local::now(),
            plans,
            client,
        }
    }

    fn find_wallpaper_tool(exec_name: &str) -> Option<WallpaperTools> {
        match exec_name {
            "hyprpaper" => Some(WallpaperTools::Hyprpaper),
            "swww" => Some(WallpaperTools::Swww),
            "mpvpaper" => Some(WallpaperTools::Mpvpaper),
            "swaybg" => Some(WallpaperTools::Swaybg),
            _ => None,
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
        let resp = self.client.get(url).send()?;
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
        let resp: Response = self.client.get(url).send()?;
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

            let settings = api.clone().expect("Cloning ApiSettings failed");
            let mut content: Value = resp.json()?;

            for key in settings.source_url_key {
                if content.is_array() {
                    let pos = match key.parse::<usize>() {
                        Ok(val) => val,
                        Err(e) => {
                            return Err(e.into());
                        }
                    };
                    content = content.as_array().unwrap()[pos].clone();
                } else {
                    content = content[&key].clone();
                }
            }

            return self.read_api(content.as_str().unwrap(), api);
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

    fn display_image(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let interval = Duration::seconds(self.settings.config.refresh_rate);
        let past_datetime = self.wallpaper_last_timestamp - interval;

        for plan in &self.plans {
            let mut datetime = plan.schedule.after(&past_datetime).take(1);
            let datetime = match datetime.next() {
                Some(dt) => dt,
                None => continue,
            };

            if self.wallpaper_last_timestamp > datetime {
                let mut retry_count: u16 = 0;

                while retry_count < self.settings.config.retry_count {
                    let filepath = self.read_local(&plan.section.source);
                    if filepath.is_ok() {
                        self.set_wallpaper(filepath?.as_str());
                        break;
                    }

                    match self.read_api(&plan.section.source, &plan.section.api) {
                        Ok(path) => {
                            self.set_wallpaper(path.as_str());
                            break;
                        }
                        Err(e) => {
                            println!("An error occured during fetching source from API: {}", e);
                        }
                    };

                    retry_count += 1;
                }

                break;
            }
        }

        self.wallpaper_last_timestamp = Local::now();
        Ok(())
    }

    pub fn setup_automatic_display(&mut self) {
        let interval = Duration::seconds(self.settings.config.refresh_rate);
        let mut next_time = Utc::now() + interval;

        self.set_wallpaper(&self.settings.config.default);

        loop {
            next_time += interval;
            let now = Utc::now();
            if now < next_time {
                thread::sleep((next_time - now).to_std().unwrap());
                match self.display_image() {
                    Err(e) => {
                        println!("There was an error during image display: {}", e);
                        self.set_wallpaper(&self.settings.config.default);
                    }
                    _ => (),
                }
            }
        }
    }
}
