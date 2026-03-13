use json::JsonValue;
use std::fs;

// Even though parser accepts single strings in json file
// it stores them in vector strings for convenience

#[derive(Debug, Clone)]
struct ApiFilterSettings {
    field: Vec<String>,
    values: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ApiSettings {
    pub source_url_key: Vec<String>,
    filter: Option<ApiFilterSettings>,
}

#[derive(Debug, Clone)]
pub struct Section {
    name: String,
    // date cron like format
    pub date: String,
    // images/image path or url
    pub source: String,
    // api fetching settings
    pub api: Option<ApiSettings>,
}

#[derive(Debug, Clone)]
pub struct ConfigSection {
    // default image path
    default: String,
    // how often should the "cron" date be rechecked
    pub refresh_rate: i64,
    pub retry_count: u16,
}

#[derive(Debug, Clone)]
pub struct Settings {
    pub config: ConfigSection,
    pub sections: Vec<Section>,
}

impl Settings {
    pub fn new() -> Settings {
        Settings {
            config: ConfigSection {
                default: "".to_string(),
                refresh_rate: 10,
                retry_count: 3,
            },
            sections: Vec::new(),
        }
    }

    // field is an atomic value: it is a string or array of strings
    fn save_field_to_vec(vec: &mut Vec<String>, field: &JsonValue) {
        if field.is_array() {
            for member in field.members() {
                vec.push(member.as_str().unwrap_or_else(|| "").to_string());
            }
        } else {
            vec.push(field.as_str().unwrap_or_else(|| "").to_string());
        }
    }

    // Fills settings attribute in place
    pub fn parse_settings(&mut self, settings_path: &String) {
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

        self.config = ConfigSection {
            default: file["default"].as_str().unwrap_or_else(|| "").to_string(),
            refresh_rate: file["refresh_rate"].as_i64().unwrap_or_else(|| 10),
            retry_count: file["retry_count"].as_u16().unwrap_or_else(|| 3),
        };

        let plan = file["plan"].members();
        for element in plan {
            if !element.has_key("name") || !element.has_key("source") || !element.has_key("date") {
                continue;
            }

            let api: Option<ApiSettings> = 'api: {
                let mut api = ApiSettings {
                    source_url_key: Vec::new(),
                    filter: None,
                };

                if !element.has_key("api") {
                    break 'api None;
                }

                let api_json = &element["api"];

                if !api_json.has_key("source_url_key") {
                    break 'api None;
                }

                Settings::save_field_to_vec(&mut api.source_url_key, &api_json["source_url_key"]);

                if api_json.has_key("filter") {
                    let mut filter_settings = ApiFilterSettings {
                        field: Vec::new(),
                        values: Vec::new(),
                    };

                    Settings::save_field_to_vec(
                        &mut filter_settings.field,
                        &api_json["filter"]["field"],
                    );

                    Settings::save_field_to_vec(
                        &mut filter_settings.values,
                        &api_json["filter"]["values"],
                    );

                    api.filter = Some(filter_settings);
                }

                Some(api)
            };

            let section = Section {
                name: element["name"].as_str().unwrap().to_string(),
                source: element["source"].as_str().unwrap().to_string(),
                date: element["date"].as_str().unwrap().to_string(),
                api,
            };

            self.sections.push(section);
        }
    }
}
