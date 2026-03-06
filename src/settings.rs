use std::fs;

#[derive(Debug)]
struct Section {
    name: String,
    // date cron like format
    date: String,
    // images/image path or url
    source: String
}

#[derive(Debug)]
struct ConfigSection {
    // default image path
    default: String,
    // image fetch retry count
    retry_count: u8,
    // how often should the "cron" date be rechecked
    refresh_rate: u16
}

#[derive(Debug)]
pub struct Settings {
    config: ConfigSection,
    sections: Vec<Section>
}

impl Settings {
    pub fn new() -> Settings {
        Settings {
            config: ConfigSection { default: "".to_string(), retry_count: 3, refresh_rate: 10 },
            sections: Vec::new()
        }
    }

    // Fills settings attribute in place
    pub fn parse_settings(settings_path: &String, settings: &mut Settings) {        
        let contents = match fs::read_to_string(settings_path) {
            Ok(content) => content,
            Err(_) => {
                panic!("There was an error reading settings file");
            }
        };

        let file = match json::parse(&contents) {
            Ok(result) => result,
            Err(_) => {
                panic!("There was an error parsing json file");
            }
        };

        settings.config = ConfigSection {
            default: file["default"].as_str().unwrap_or_else(|| "").to_string(),
            retry_count: file["retry_count"].as_u8().unwrap_or_else(|| { 3 }),
            refresh_rate: file["refresh_rate"].as_u16().unwrap_or_else(|| { 10 })
        };

        let plan = file["plan"].members();
        for element in plan {
            if !element.has_key("name") || !element.has_key("source") || !element.has_key("date") {
                continue;
            }

            let section = Section {
                name: element["name"].as_str().unwrap().to_string(),
                source: element["source"].as_str().unwrap().to_string(),
                date: element["date"].as_str().unwrap().to_string()
            };

            settings.sections.push(section);
        }
    }
} 