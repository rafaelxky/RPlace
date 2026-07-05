use std::{
    fs::{self, File},
    io::{BufReader, BufWriter},
    path::Path,
    sync::{Arc, LazyLock, Mutex, RwLock},
};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerConfig {
    pub allow_lua: bool,
    pub allow_import: bool,
}
impl CompilerConfig {
    pub fn load(path: &Path) -> Self {
        let file = File::open(path);
        let file = match file {
            Ok(file) => file,
            Err(_e) => {
                println!("Could not load config from {}, loading default!", path.to_str().unwrap());
                return Self::default();
            }
        };
        let reader = BufReader::new(file);
        let config = serde_json::from_reader(reader);
        match config {
            Ok(config) => {
                return config;
            }
            Err(_e) => {
                println!("Could not deserialize config from {}, loading default!", path.to_str().unwrap());
                return Self::default();
            }
        }
    }
    pub fn default() -> Self {
        Self { allow_lua: false, allow_import: true, }
    }
}
pub static CONFIG: LazyLock<Arc<RwLock<CompilerConfig>>> = LazyLock::new(|| {
    let dir = ProjectDirs::from("io", "rplace", "rplace");
    let dir = match dir {
        Some(dir) => dir,
        None => {
            return Arc::new(RwLock::new(CompilerConfig::default()));
        }
    };

    let config = dir.config_dir().join("config.json");

    let conf = if !config.exists() {
        let conf = CompilerConfig::default();
        if config.parent().is_some() && !config.parent().unwrap().exists() {
            fs::create_dir_all(&config.parent().unwrap()).unwrap();
            println!("Created config folder {}", &config.parent().unwrap().to_str().unwrap());
        }
        
        let file = File::create(&config);
        let file = match file {
            Ok(file) => file,
            Err(e) => {panic!("{}: {}",e,&config.to_str().unwrap())}
        };
        let writer = BufWriter::new(file);
        let result = serde_json::to_writer_pretty(writer, &conf);
        if result.is_err() {
            println!("Failed to write json to file!");
            conf
        } else {
            println!("Created config file {}", &config.to_str().unwrap());
            conf
        }
    } else {
        println!("Loaded file from {}", config.to_str().unwrap());
        CompilerConfig::load(&config)
    };

    return Arc::new(RwLock::new(conf));
});
