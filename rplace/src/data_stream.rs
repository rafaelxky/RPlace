use std::{
    fs,
    path::{Path, PathBuf},
    process::exit,
};

use directories::ProjectDirs;
use reqwest::blocking::get;
use walkdir::WalkDir;

use crate::package_manager::file::parse_package_path;

pub enum DataSouce {
    WEB,
    FILE,
    Package,
}
pub fn get_data_stream(path: &str) -> (Box<dyn DataStream>, DataSouce) {
    if path.starts_with("http") {
        return (
            Box::new(WebDataStream::new(path.to_string())),
            DataSouce::WEB,
        );
    } else if path.starts_with("crate") {
        return (
            Box::new(PackageDataStream::new(path.to_string())),
            DataSouce::Package,
        );
    }
    return (
        Box::new(FileDataStream::new(path.to_string())),
        DataSouce::FILE,
    );
}
pub trait PathStream {
    fn next(&mut self) -> Option<String>;
}
pub trait DataStream {
    fn next(&mut self) -> Option<(String, String)>;
    fn to_path_stream(self) -> Box<dyn PathStream>;
    fn append(&mut self, paths: &mut Vec<String>);
}
pub struct FileDataStream {
    paths: Vec<String>,
    i: usize,
}
impl FileDataStream {
    pub fn new(path: String) -> Self {
        let mut paths: Vec<String> = Vec::new();
        for entry in WalkDir::new(path.clone()) {
            if entry.is_err() {
                panic!("No such file {}", path)
            }
            let entry = entry.unwrap();
            if entry.file_type().is_file() {
                paths.push(entry.path().to_str().unwrap().to_string());
            }
        }
        Self { paths, i: 0 }
    }
    pub fn get_from_file(&self, path: String) -> String {
        let origin_path = Path::new(&path);
        if !Path::exists(origin_path) {
            eprintln!("No such file {}", path);
            exit(1);
        }
        return fs::read_to_string(path).unwrap();
    }
}
impl DataStream for FileDataStream {
    fn next(&mut self) -> Option<(String, String)> {
        if self.i >= self.paths.len() {
            return None;
        }
        let path = self.paths[self.i].clone();
        self.i = self.i + 1;
        return Some((self.get_from_file(path.to_string()), path));
    }

    fn to_path_stream(self) -> Box<dyn PathStream> {
        return Box::new(self);
    }

    fn append(&mut self, paths: &mut Vec<String>) {
        self.paths.append(paths);
    }
}
impl PathStream for FileDataStream {
    fn next(&mut self) -> Option<String> {
        if self.i > 0 {
            return None;
        }
        self.i = self.i + 1;
        return Some(self.paths[self.i].clone());
    }
}
pub struct PackageDataStream {
    paths: Vec<String>,
    i: usize,
    dir: PathBuf,
}
impl PackageDataStream {
    pub fn new(path: String) -> Self {
        let dir = ProjectDirs::from("io", "rplace", "rplace");
        let binding = match dir {
            Some(dir) => dir,
            None => {
                println!("Unable to find path");
                exit(0);
            }
        };
        let dir = binding.data_dir();
        let dir = dir.join("packages");
        let dir = dir.to_path_buf();
        let path = parse_package_path(path, &dir);
        let mut paths: Vec<String> = Vec::new();
        for entry in WalkDir::new(path.clone()) {
            if entry.is_err() {
                panic!("No such file {}", path)
            }
            let entry = entry.unwrap();
            if entry.file_type().is_file() {
                paths.push(entry.path().to_str().unwrap().to_string());
            }
        }

        Self { paths, i: 0, dir }
    }
    pub fn get_from_file(&self, path: String) -> String {
        let origin_path = Path::new(&path);
        if !Path::exists(origin_path) {
            eprintln!("No such file {}", path);
            exit(1);
        }
        return fs::read_to_string(path).unwrap();
    }
}
impl DataStream for PackageDataStream {
    fn next(&mut self) -> Option<(String, String)> {
        if self.i >= self.paths.len() {
            return None;
        }
        let path = self.paths[self.i].clone();
        self.i = self.i + 1;
        return Some((self.get_from_file(path.to_string()), path));
    }

    fn to_path_stream(self) -> Box<dyn PathStream> {
        return Box::new(self);
    }

    fn append(&mut self, paths: &mut Vec<String>) {
        let dir = &self.dir;
        let mut paths: Vec<String> = paths
            .iter()
            .map(|path| {
                let mut path = path.clone();
                if path.starts_with("package/") {
                    path = path.strip_prefix("package/").unwrap().to_string();
                }
                let path = dir.join(path);
                path.to_str().unwrap().to_string()
            })
            .collect();
        self.paths.append(&mut paths);
    }
}
impl PathStream for PackageDataStream {
    fn next(&mut self) -> Option<String> {
        if self.i > 0 {
            return None;
        }
        self.i += 1;
        return Some(self.paths[0].clone());
    }
}
pub struct WebDataStream {
    paths: Vec<String>,
    i: usize,
}
impl WebDataStream {
    pub fn new(path: String) -> Self {
        Self {
            paths: vec![path],
            i: 0,
        }
    }
}
impl DataStream for WebDataStream {
    fn next(&mut self) -> Option<(String, String)> {
        if self.i > 0 {
            return None;
        }
        self.i = self.i + 1;
        return Some((get_from_http(&self.paths[0]), self.paths[0].clone()));
    }

    fn to_path_stream(self) -> Box<dyn PathStream> {
        return Box::new(self);
    }

    fn append(&mut self, paths: &mut Vec<String>) {
        self.paths.append(paths);
    }
}
impl PathStream for WebDataStream {
    fn next(&mut self) -> Option<String> {
        if self.i > 0 {
            return None;
        }
        self.i += 1;
        return Some(self.paths[0].clone());
    }
}
fn get_from_http(path: &str) -> String {
    // todo: make this work
    let body = get(path);
    match body {
        Ok(response) => match response.text() {
            Ok(text) => text,
            Err(e) => {
                //todo
                eprintln!("Failed to read body: {}", e);
                exit(1);
            }
        },
        Err(e) => {
            //todo
            eprintln!("Request failed {}", e);
            exit(1);
        }
    }
}
