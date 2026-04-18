use std::{fs, path::Path, process::exit};

use reqwest::blocking::get;

pub enum DataSouce {
    WEB,
    FILE,
}
pub struct TextProvider {}
impl TextProvider {
    pub fn get_text(path: &str) -> (String, DataSouce) {
        if path.starts_with("https") || path.starts_with("http") {
            return Self::get_from_http(path);
        }
        return Self::get_from_file(path);
    }
    fn get_from_file(path: &str) -> (String, DataSouce) {
        let origin_path = Path::new(&path);
        if !Path::exists(origin_path) {
            println!("No such file {}", path);
            exit(1);
        }
        return (fs::read_to_string(path).unwrap(), DataSouce::FILE);
    }
    fn get_from_http(path: &str) -> (String, DataSouce) {
        let body = get(path);
        match body {
            Ok(response) => match response.text() {
                Ok(text) => (text,DataSouce::WEB),
                Err(e) => {
                    eprintln!("Failed to read body: {}", e);
                    exit(1);
                }
            },
            Err(e) => {
                eprintln!("Request failed {}", e);
                exit(1);
            }
        }
    }
}
