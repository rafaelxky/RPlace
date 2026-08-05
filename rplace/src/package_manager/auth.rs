use std::{
    fs::{self, File}, io::{BufReader, BufWriter, Write}, process::exit,
};

use anyhow::{Ok, Result};
use directories::ProjectDirs;
use thiserror::Error;

use crate::{errors::NotLoggedInError, package_manager::web::structs::LogginResponse};

pub fn remove_tok() -> Result<()>{
 let dir = ProjectDirs::from("io", "rplace", "rplace");
    let dir = match dir {
        Some(dir) => dir,
        None => {
            return Err(NotLoggedInError::new("Unable to find path").into());
        }
    };
    let config = dir.data_dir().join("tok.json");

    let mut file = if !config.exists() {
         File::create(&config)?
    } else {
        File::open(&config)?
    };
    file.write_all("".as_bytes())?;
    Ok(())
}
pub fn save_tok(context: LogginResponse) -> Result<()> {
    let dir = ProjectDirs::from("io", "rplace", "rplace");
    let dir = match dir {
        Some(dir) => dir,
        None => {
            return Err(NotLoggedInError::new("Unable to find path").into());
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
            return Err(NotLoggedInError::new("Unable to find path").into());
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
