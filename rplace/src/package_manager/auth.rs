use std::{
    fs::{self, File, OpenOptions},
    io::{BufReader, BufWriter, Write},
    process::exit,
};

use anyhow::{Ok, Result};
use directories::ProjectDirs;
use thiserror::Error;

use crate::{errors::NotLoggedInError, package_manager::web::structs::LoginResponse};

pub fn remove_tok() -> Result<()> {
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
const TOK_FILE: &str = "tok.json";
pub fn save_tok(context: LoginResponse) -> Result<()> {
    let dir = ProjectDirs::from("io", "rplace", "rplace");
    let dir = match dir {
        Some(dir) => dir,
        None => {
            return Err(NotLoggedInError::new("Unable to find path").into());
        }
    };
    let config = dir.data_dir().join(TOK_FILE);

    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&config)?;

    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &context)?;
    Ok(())
}
pub fn read_tok() -> Result<LoginResponse> {
    let dir = ProjectDirs::from("io", "rplace", "rplace");
    let dir = match dir {
        Some(dir) => dir,
        None => {
            return Err(NotLoggedInError::new("Unable to find path").into());
        }
    };
    let config = dir.data_dir().join(TOK_FILE);

    if config.parent().is_some() && !config.parent().unwrap().exists() {
        fs::create_dir_all(&config.parent().unwrap())?;
    }
    let file = File::open(&config)?;
    let reader = BufReader::new(file);
    let tok = serde_json::from_reader(reader)?;
    Ok(tok)
}
