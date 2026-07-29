use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use anyhow::{Ok, Result};
use rplace::data_stream::{DataStream, FileDataStream};

pub fn setup(file_path: &str, code: &str) -> Result<()> {
    let path = Path::new(file_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = File::create(path)?;
    file.write_all(code.as_bytes())?;
    Ok(())
}

#[test]
pub fn test_file_data_stream() -> Result<()> {
    let file_path = "./test_files/file_data_stream_test.txt";
    let code = "hello world";
    setup(file_path, code)?;

    let mut source = FileDataStream::new(file_path.to_string());
    let maybe_path: Option<(String, String)> = source.next();
    let (code_res, path_res) = match maybe_path {
        Some(r) => r,
        None => panic!("source next yielded None"),
    };
    assert_eq!(code_res, code);
    assert_eq!(path_res, file_path);
    Ok(())
}
#[test]
pub fn test_multiple_files_data_stream() -> Result<()> {
    let file_path_a = "./test_files/multiple_file_data_stream_test_a.txt";
    let file_path_b = "./test_files/multiple_file_data_stream_test_b.txt";
    let code_a = format!("//- include {}: hello", file_path_b);
    let code_b = "world";
    setup(file_path_a, &code_a)?;
    setup(file_path_b, code_b)?;

    let mut source = FileDataStream::new(file_path_a.to_string());
    source.append(&mut vec![file_path_b.to_string()]);
    let maybe_path: Option<(String, String)> = source.next();
    let (code_res, path_res) = match maybe_path {
        Some(r) => r,
        None => panic!("source next yielded None"),
    };
    assert_eq!(code_res, code_a);
    assert_eq!(path_res, file_path_a);
    let maybe_path: Option<(String, String)> = source.next();
    let (code_res, path_res) = match maybe_path {
        Some(r) => r,
        None => panic!("source next yielded None"),
    };
    assert_eq!(code_res, code_b);
    assert_eq!(path_res, file_path_b);
    Ok(())
}
#[test]
pub fn test_folder_file_data_stream() -> Result<()> {
    let folder_path = "./test_files/folder";
    let file_path_a = "./test_files/folder/test_data_stream_folder_a.txt";
    let file_path_b = "./test_files/folder/test_data_stream_folder_b.txt";
    let code_a = format!("//- include {}: hello", file_path_b);
    let code_b = "world";
    setup(file_path_a, &code_a)?;
    setup(file_path_b, code_b)?;

    let mut source = FileDataStream::new(folder_path.to_string());

    let maybe_path: Option<(String, String)> = source.next();
    let (code_res, path_res) = match maybe_path {
        Some(r) => r,
        None => panic!("source next yielded None"),
    };
    assert_eq!(code_res, code_b);
    assert_eq!(path_res, file_path_b);
    let maybe_path: Option<(String, String)> = source.next();
    let (code_res, path_res) = match maybe_path {
        Some(r) => r,
        None => panic!("source next yielded None"),
    };
    assert_eq!(code_res, code_a);
    assert_eq!(path_res, file_path_a);
    Ok(())
}

