use std::{
    fs::{self, File}, io::{BufReader, BufWriter}, process::exit,
};

use anyhow::Result;
use directories::ProjectDirs;

use crate::package_manager::web::structs::LogginResponse;

pub fn save_tok(context: LogginResponse) -> Result<()> {
    let dir = ProjectDirs::from("io", "rplace", "rplace");
    let dir = match dir {
        Some(dir) => dir,
        None => {
            println!("Unable to find path");
            exit(0);
        }
    };
    let config = dir.data_dir().join("tok.json");
    
    let file = if !config.exists() {
         File::create(&config)?
    } else {
        File::open(&config)?
    };
   
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &context)?;
    Ok(())
}
pub fn read_tok() -> Result<LogginResponse> {
    let dir = ProjectDirs::from("io", "rplace", "rplace");
    let dir = match dir {
        Some(dir) => dir,
        None => {
            println!("Unable to find path");
            exit(0);
        }
    };
    let config = dir.data_dir().join("tok.json");

    if config.parent().is_some() && !config.parent().unwrap().exists() {
        fs::create_dir_all(&config.parent().unwrap())?;
    }
    let file = File::open(&config)?;
    let reader = BufReader::new(file);
    let tok = serde_json::from_reader(reader)?;
    Ok(tok)
}
