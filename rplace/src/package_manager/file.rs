use anyhow::Result;
use directories::ProjectDirs;
use path_clean::PathClean;
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use crate::{constants::PROJECT_FILE, package_manager::{package_structs::PackageData, web::fetch::{get_initial_data, get_package_file}}};
pub fn package_exists(path: &str) -> bool {
    let dir = ProjectDirs::from("io", "rplace", "rplace").unwrap();
    let dir = dir.data_dir();
    let dir = dir.join("packages");
    let path = parse_package_path(path.to_string(), &dir);
    Path::new(&path).exists()
}
// get rplace.toml data
// get a list of all code and paths
// one by one copy to project file
pub async fn load_all_package_files(package_source: &str, package_data: &PackageData){

}
// todo: this bellow
pub async fn load_single_package(package_source: &str, package_name: &str, package_version: &str) -> Result<()>{
    let data = get_initial_data(package_source, package_name, package_version).await?;
    let header_file = get_package_file(package_source, data.package_id, PROJECT_FILE).await?;
    // fetch all files
    // server must allow you to enumerate all version files
    // if file already exists, check version, if same, skip
    todo!();
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