use chrono::NaiveTime;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::fs::File;
use std::io::{self, Write};
use toml::{self, to_string};
use url::{ParseError, Url};

#[derive(Debug, Deserialize, Serialize)]
pub struct Response {
    pub results: SunriseSunsetResults,
    pub status: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SunriseSunsetResults {
    pub date: String,
    pub sunrise: String,
    pub sunset: String,
    pub first_light: String,
    pub last_light: String,
    pub dawn: String,
    pub dusk: String,
    pub solar_noon: String,
    pub golden_hour: String,
    pub day_length: String,
    pub timezone: String,
    pub utc_offset: i32,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub default: Option<Default>,
}

#[derive(Debug, Deserialize)]
pub struct Default {
    pub base_url: String,
    pub lat: f64,
    pub lng: f64,
    pub output: String,
}

fn convert_to_24_hour(time_str: &str) -> Option<String> {
    NaiveTime::parse_from_str(time_str, "%I:%M:%S %p")
        .ok()
        .map(|time| time.format("%H:%M:%S").to_string())
}

fn convert_times_to_24_hour(body: &str) -> Result<Value, serde_json::Error> {
    let mut parsed: Value = serde_json::from_str(body)?;

    if let Some(results) = parsed.get_mut("results") {
        if let Some(results_obj) = results.as_object_mut() {
            let time_fields = vec![
                "sunrise",
                "sunset",
                "first_light",
                "last_light",
                "dawn",
                "dusk",
                "solar_noon",
                "golden_hour",
            ];

            for field in time_fields {
                if let Some(time_str) = results_obj.get_mut(field) {
                    if let Some(time_str_str) = time_str.as_str() {
                        *time_str = json!(convert_to_24_hour(time_str_str));
                    }
                }
            }
        }
    }
    Ok(parsed)
}

fn fetch_data(url: Url) -> Result<String, reqwest::Error> {
    let res = reqwest::blocking::get(url)?;
    let body = res.text()?;
    Ok(body)
}

fn parse_and_write_response(body: &str, output: String) -> Result<(), Box<dyn std::error::Error>> {
    let parsed = convert_times_to_24_hour(&body)?;
    let response: Response = serde_json::from_value(parsed)?;
    write_response(&response, output)?;
    Ok(())
}

fn string_to_url(url_str: String) -> Result<Url, ParseError> {
    Url::parse(&url_str)
}

fn write_response(response: &Response, output: String) -> io::Result<()> {
    let toml_str = match to_string(response) {
        Ok(s) => s,
        Err(e) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to serialize response: {}", e),
            ))
        }
    };

    let mut file = File::create(&output)?;

    file.write_all(toml_str.as_bytes()).unwrap();

    println!("File wrote to: {}", &output);

    Ok(())
}

pub fn compile_url(config: Config) -> Option<Url> {
    if let Some(default_config) = &config.default {
        let base_url_str = default_config.base_url.clone();
        let lat = default_config.lat;
        let lng = default_config.lng;
        let url_str = format!("{}/json?lat={}&lng={}", base_url_str, lat, lng);
        string_to_url(url_str).ok()
    } else {
        println!("Default configuration not found in config.toml");
        None
    }
}

pub fn fetch_and_process_data(url: Option<Url>, output: String) {
    if let Some(url) = url {
        match fetch_data(url) {
            Ok(body) => match parse_and_write_response(&body, output) {
                Ok(()) => println!("Data processed successfully"),
                Err(err) => eprintln!("Error processing data: {}", err),
            },
            Err(err) => eprintln!("Error fetching data: {}", err),
        }
    }
}

pub fn read_config_from_file(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let decoded: Config = toml::from_str(&content)?;

    Ok(decoded)
}
