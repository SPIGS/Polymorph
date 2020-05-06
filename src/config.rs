use serde::{Deserialize, Serialize};
use toml;
use std::io::Write;
use std::fs::File;
use std::fs;
use bracket_lib::prelude::BACKEND;

/// Wrapper for user settings. Includes the serialized structure of user settings from the config file as well as log and debugging information.
pub struct SettingsContext {
    pub settings : Settings,
    pub message : String,
    pub log_warning : bool,
    pub log_error : bool,
}

///Structure for user settings
#[derive(Deserialize, Default, Serialize)]
pub struct Settings {
    pub development : Development,
    pub other : Other,
    pub graphical : Graphical,
}

#[derive(Deserialize, Serialize)]
pub struct Other {
    pub screenshot_location : String,
}

impl Default for Other {
    fn default() -> Self {
        Other {
            screenshot_location : "/screenshots".to_string(),
        }
    }
}

#[derive(Deserialize, Serialize, Default)]
pub struct Development {
    pub debug : bool,
}

#[derive(Deserialize, Serialize)]
pub struct Graphical {
    pub fullscreen : bool,
    pub vsync : bool,
    pub post_processing : PostProcessing,
}

impl Default for Graphical {
    fn default() -> Self {
        Graphical {
            fullscreen : true,
            vsync : false,
            post_processing : Default::default(),
        }
    }
}

#[derive(Deserialize, Serialize, Default)]
pub struct PostProcessing {
    pub scan_lines : bool,
    pub screen_burn : bool,
}

/// Auto-detects the resolution of the monitor and returns dimensions of console in tiles, depending on the resolution.
pub fn auto_detect_resolution () -> (u32, u32) {
    //grab the resolution of the current monitor from the backend
    let monitor_size = BACKEND.lock().context_wrapper.as_ref().unwrap().wc.window().current_monitor().size();
    let resolution : (u32, u32) = (monitor_size.width, monitor_size.height);
    info!("Detected screen resolution: {} x {}", resolution.0, resolution.1);
    
    //convert the resolution of the monitor to tile dimensions
    let mut width = resolution.0 / 24;
    let mut height = resolution.1 /24;
    let width_remainder = width % 10;
    let height_remainder = height % 10;
    
    width = width - width_remainder;
    height = height - height_remainder;
    info!("Setting console tile resolution to: {} x {}", width, height);

    return (width, height);
}

///Attempts to load user settings from the config.toml file. If the file is not found, a new config file, with default values, will be created.
///If a file is found but contains syntax errors, default values will be used during runtime. Returns a SettingsContext. 
pub fn load_config_file () -> SettingsContext {
    let config_result = fs::read_to_string("config.toml");
    let mut config_file = "".to_string();
    let mut config_msg = "Loaded user settings from config.".to_string();
    let mut log_warning = false;
    let mut log_error = false;
    match config_result {
        Ok(t) => {
            config_file = t;
        },
        Err(e) => {
            config_msg = format!("Unable to find config file: {}; Making a new one with default settings", e);
            create_new_config_file(Default::default());
            log_warning = true;
        },
    }

    let settings : Settings = if log_warning {
        Default::default()
    } else {
        let deserialized_result  : Result<Settings, toml::de::Error> = toml::from_str(&config_file);
        let deserialized;
        match deserialized_result {
            Ok(t) => {
                deserialized = t;
            },
            Err(e) => {
                config_msg = format!("Error within config file : {}; Falling back to default settings.", e);
                log_error = true;
                deserialized = Default::default();
            },
        }
        deserialized
    };

    SettingsContext {
        settings : settings,
        message : config_msg,
        log_warning : log_warning,
        log_error : log_error,
    }
}

///Creates a new config file from a given Settings structure. 
pub fn create_new_config_file (settings : Settings) {
    let config_data = toml::to_string(&settings).unwrap();
    let mut new_file = File::create("config.toml").unwrap();
    write!(new_file, "{}", config_data).unwrap();
}