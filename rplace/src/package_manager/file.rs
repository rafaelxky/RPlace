use anyhow::Result;
use directories::ProjectDirs;
use path_clean::PathClean;
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};
// get rplace.toml data
// get a list of all code and paths
// one by one copy to project file
pub fn save_package_project(){
    todo!()
}
pub fn package_exists(path: &str) -> bool {
    let dir = ProjectDirs::from("io", "rplace", "rplace").unwrap();
    let dir = dir.data_dir();
    let dir = dir.join("packages");
    let path = parse_package_path(path.to_string(), &dir);
    Path::new(&path).exists()
}
pub fn save_package_file_raw(path: &str, code: &str) -> Result<String> {
    let dir = ProjectDirs::from("io", "rplace", "rplace").unwrap();
    let dir = dir.data_dir();
    let dir = dir.join("packages");
    let path = parse_package_path(path.to_string(), &dir);
    let path = PathBuf::from(path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = File::create(&path)?;
    file.write_all(code.as_bytes())?;
    Ok(path.to_str().unwrap().to_string())
}
pub fn parse_package_path(path: String, base_dir: &PathBuf) -> String {
    let mut path = path;
    if path.starts_with("package/") {
        path = path.strip_prefix("package/").unwrap().to_string();
    }
    let target = Path::new(&path).clean();
    if !target.starts_with(&base_dir) {
        path = base_dir.join(path).clean().to_str().unwrap().to_string();
    }
    return path;
}