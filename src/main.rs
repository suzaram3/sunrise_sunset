use sunrise_sunset::*;

fn main() {
    let config_path = "/etc/sunrise_sunset.toml";
    let config = match read_config_from_file(&config_path) {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("Error reading config.toml: {}", err);
            return;
        }
    };

    let output = match &config.default {
        Some(default_config) => default_config.output.clone(),
        None => {
            eprintln!("Error: No default output path specified in config.");
            return;
        }
    };

    let url = compile_url(config);

    fetch_and_process_data(url, output)
}
