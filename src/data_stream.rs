use std::{fs, path::Path, process::exit};

use walkdir::WalkDir;
use reqwest::blocking::get;

pub enum DataSouce {
    WEB,
    FILE,
}
pub fn get_data_stream(path: &str) -> (Box<dyn DataStream>, DataSouce) {
    if path.starts_with("http") {
        return (Box::new(WebDataStream::new(path.to_string())), DataSouce::WEB);
    }
    return (Box::new(FileDataStream::new(path.to_string())),DataSouce::FILE);
}
pub trait DataStream {
    fn next(&mut self) -> Option<(String, String)>;
}
pub struct FileDataStream {
    paths: Vec<String>,
    i: usize,
}
impl FileDataStream {
    pub fn new(path: String) -> Self {
        let mut paths: Vec<String> = Vec::new();
        for entry in WalkDir::new(path) {
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
}
pub struct WebDataStream{
    path: Vec<String>,
    i: usize,
}
impl WebDataStream {
    pub fn new(path: String) -> Self{
        Self { path: vec![path], i: 0 }
    }
}
impl DataStream for WebDataStream {
    fn next(&mut self) -> Option<(String,String)> {
        if self.i > 0 {
            return None;
        }
        self.i = self.i + 1;
        return Some((get_from_http(&self.path[0]), self.path[0].clone()));
    }
}
fn get_from_http(path: &str) -> String {
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