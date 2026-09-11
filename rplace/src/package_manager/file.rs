use anyhow::{bail, Result};
use directories::ProjectDirs;
use path_clean::PathClean;
use reqwest::{Client, StatusCode};
use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use crate::{
    constants::PROJECT_FILE,
    package_manager::{
        package_structs::{Dependency, PackageData},
        web::{
            fetch::{get_initial_data, get_package_file, get_version_paths},
            structs::PackageFile,
        },
    },
};
pub fn package_exists(path: &str) -> bool {
    let dir = ProjectDirs::from("io", "rplace", "rplace").unwrap();
    let dir = dir.data_dir();
    let dir = dir.join("packages");
    let path = match parse_package_path(path.to_string(), &dir) {
        Ok(path) => path,
        Err(_) => return false,
    };
    Path::new(&path).exists()
}
// get rplace.toml data
// get a list of all code and paths
// one by one copy to project file
pub async fn load_all_package_files(
    package_source: &str,
    package_data: &PackageData,
) -> Result<()> {

    let client = Client::new();
    let response = client.get(package_source).send().await;
    if response.is_err() || response.unwrap().status() != StatusCode::OK {
        println!("Package manager source {} offline!", package_source);
        return Ok(());
    }

    let dependencies = &package_data.dependencies;
    let dependencies: HashMap<String, Dependency> = match dependencies {
        Some(d) => d.clone(),
        None => return Ok(()),
    };
    // hashset to avoid duplicate imports
    let mut dep_map: HashSet<(String, String)> = HashSet::new();
    for (name, dep) in dependencies.iter() {
        dep_map.insert((name.to_string(), dep.get_version().to_string()));
    }
    // list of dependencies to import
    let mut dep: Vec<(String, Dependency)> = dependencies
        .iter()
        .map(|(name, dep)| {
            return (name.clone(), dep.clone());
        })
        .collect();
    let mut i: usize = 0;
    while i < dep.len() {
        let (package_name, dependency) = &dep[i].clone();
        let version_name = dependency.get_version();

        let package = load_single_package(package_source, &package_name, &version_name).await?;
        for file in package {
            // add nested dependencies to be imported
            if file.file_path == PROJECT_FILE {
                let toml = PackageData::from_toml(file.code.clone())?;
                if toml.dependencies.is_none() {
                    continue;
                }
                for (name, dependency) in toml.dependencies.unwrap() {
                    let data = (name.to_string(), dependency.get_version().to_string());
                    if !dep_map.contains(&data) {
                        dep_map.insert(data);
                        dep.push((name, dependency));
                    }
                }
            }
            let path =
                save_package_file_raw(&package_name, &version_name, &file.file_path, &file.code)?;
            println!("loaded dependency to path {}", path);
        }
        i += 1;
    }
    Ok(())
}

pub async fn load_single_package(
    package_source: &str,
    package_name: &str,
    package_version: &str,
) -> Result<Vec<PackageFile>> {
    let data = get_initial_data(package_source, package_name, package_version).await?;
    let version_id = data.version_id;
    //let header_file: super::web::structs::ResponseGetPackageFile = get_package_file(package_source, data.package_id, PROJECT_FILE).await?;
    let paths = get_version_paths(package_source, version_id).await?;

    let mut files = vec![];
    for path in paths.links {
        let file = get_package_file(package_source, version_id, &path).await?;
        files.push(file.into_package_file());
    }
    return Ok(files);
}
pub fn create_dependency_folder(name: &str) -> Result<PathBuf> {
    let dir = ProjectDirs::from("io", "rplace", "rplace").unwrap();
    let dir = dir.data_dir();
    let dir = dir.join("packages");
    let dir = dir.join(name);
    if dir.exists() {
        fs::remove_dir_all(&dir)?;
    }
    fs::create_dir_all(&dir)?;
    Ok(dir)
}
pub fn save_package_file_raw(
    package_name: &str,
    package_version_name: &str,
    path: &str,
    code: &str,
) -> Result<String> {
    let path = resolve_package_path(package_name, package_version_name, path)?;
    let path = PathBuf::from(path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = File::create(&path)?;
    file.write_all(code.as_bytes())?;
    Ok(path.to_str().unwrap().to_string())
}
pub fn resolve_package_path(
    package_name: &str,
    package_version_name: &str,
    path: &str,
) -> Result<String> {
    validate_package_component(package_name, "package name")?;
    validate_package_component(package_version_name, "package version")?;

    let dir = ProjectDirs::from("io", "rplace", "rplace").unwrap();
    let dir = dir.data_dir();
    let dir = dir.join("packages");
    let dir = dir.join(package_name);
    let dir = dir.join(package_version_name);
    parse_package_path(path.to_string(), &dir)
}
pub fn parse_package_path(path: String, base_dir: &PathBuf) -> Result<String> {
    let path = path.strip_prefix("package/").unwrap_or(&path);
    let target = Path::new(path).clean();

    if target.is_absolute() {
        if target.strip_prefix(base_dir).is_ok() {
            return Ok(target.to_string_lossy().into_owned());
        }
        bail!("package path escapes package directory: {path}");
    }

    if target
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        bail!("package path escapes package directory: {path}");
    }

    Ok(base_dir.join(target).to_string_lossy().into_owned())
}

fn validate_package_component(value: &str, label: &str) -> Result<()> {
    if value.is_empty() {
        return Ok(());
    }

    let mut components = Path::new(value).components();
    if !matches!(components.next(), Some(std::path::Component::Normal(_)))
        || components.next().is_some()
    {
        bail!("invalid {label}: {value}");
    }

    Ok(())
}
