use std::{
    fs::{self, File}, io::{BufReader, BufWriter}, path::{Path, PathBuf}, sync::{Arc, LazyLock, RwLock},
};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManagerCompilerConfig{
    pub allow_lua: Option<bool>,
    pub allow_import: Option<bool>,
    pub package_source: Option<String>,
}
impl PackageManagerCompilerConfig {
    pub fn new() -> Self{
        Self { allow_lua: None, allow_import: None, package_source: None }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerConfig {
    pub allow_lua: bool,
    pub allow_import: bool,
    pub package_source: String,
}
impl CompilerConfig {
    pub fn load(path: &Path) -> Self {
        let file = File::open(path);
        let file = match file {
            Ok(file) => file,
            Err(_e) => {
                println!(
                    "Could not load config from {}, loading default!",
                    path.to_str().unwrap()
                );
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
                println!(
                    "Could not deserialize config from {}, loading default!",
                    path.to_str().unwrap()
                );
                return Self::default();
            }
        }
    }
    pub fn default() -> Self {
        let package_source = "".to_string();
        Self {
            allow_lua: false,
            allow_import: true,
            package_source: package_source,
        }
    }
    pub fn reload() {}
}
pub static CONFIG: LazyLock<Arc<RwLock<CompilerConfig>>> = LazyLock::new(|| {
    let dir = ProjectDirs::from("io", "rplace", "rplace");
    let dir = match dir {
        Some(dir) => dir,
        None => {
            println!("Unable to find config path");
            return Arc::new(RwLock::new(CompilerConfig::default()));
        }
    };

    let config = dir.config_dir().join("config.json");
    let config = load_config(config);

    return Arc::new(RwLock::new(config));
});

pub fn reload_config(path: PathBuf) -> CompilerConfig{
    let config = path;
    let conf = CompilerConfig::default();

    if config.parent().is_some() && !config.parent().unwrap().exists() {
        fs::create_dir_all(&config.parent().unwrap()).unwrap();
        println!(
            "Created config folder {}",
            &config.parent().unwrap().to_str().unwrap()
        );
    }

    let file = File::create(&config);
    let file = match file {
        Ok(file) => file,
        Err(e) => {
            panic!("{}: {}", e, &config.to_str().unwrap())
        }
    };
    let writer = BufWriter::new(file);
    let result = serde_json::to_writer_pretty(writer, &conf);
    if result.is_err() {
        println!("Failed to write json to file!");
        return conf;
    } else {
        println!("Created config file {}", &config.to_str().unwrap());
        return conf;
    }
}

pub fn load_config(path: PathBuf) -> CompilerConfig {
    let config = path;
    let _conf = if !config.exists() {
        return reload_config(config);
    } else {
        return CompilerConfig::load(&config);
    };
}
