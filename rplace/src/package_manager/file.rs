use std::{
    fs::{self, File}, io::Write, path::PathBuf,
};
use anyhow::{Ok, Result};
use directories::ProjectDirs;

pub fn save_package_file_raw(path: &str, code: &str) -> Result<()>{
    let dir = ProjectDirs::from("io", "rplace", "rplace").unwrap();
    let dir = dir.data_dir();
    let dir = dir.join("packages");
    let path = parse_package_path(path.to_string(), &dir);
    let path = PathBuf::from(path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = File::create(path)?;
    file.write_all(code.as_bytes())?;
    Ok(())
}
pub fn parse_package_path(path: String, base_dir: &PathBuf) -> String {
    let mut path = path;
    if path.starts_with("package") {
        path = path.strip_prefix("package/").unwrap().to_string();
    }
    let target = fs::canonicalize(&path).unwrap();
    if !target.starts_with(&base_dir) {
        path = base_dir.join(path).to_str().unwrap().to_string();
    }
    return path;
}
