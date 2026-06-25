use std::{fs::{self, File, OpenOptions}};

use crate::{derive::deriver::Deriver, writer::{writer_structs::{FileData, WriterResult}}};
use std::io::Write;
use std::path::Path;

pub struct OutputWriter {
    to_write: WriterResult,
    file: File,
}
impl OutputWriter {
    pub fn new(to_write: WriterResult, file: File) -> Self {
        Self { to_write, file }
    }
    pub fn write(mut self) {
        let mut replaced = self.to_write;
        let last: FileData = replaced.file_data.pop().unwrap();
        write!(&mut self.file, "{}", last.data).expect("Unable to write");

        replaced.file_data.iter_mut().for_each(|result| {
            let path = Path::new(&result.path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("Unable to create directories");
            }
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&result.path)
                .expect(&format!("Unable to open or create file {}", result.path));

            file.write_all(result.data.as_bytes())
                .expect("Unable to write");
        });

        replaced.derives.iter().for_each(|derive| {
            if fs::exists(&derive.path).is_err() {
                panic!("Error: no such file {} for derive", derive.path)
            }
            let result = Deriver::derive(derive);
            let mut file = OpenOptions::new()
                .write(true)
                .create(false)
                .truncate(true)
                .open(&derive.path)
                .expect("Unable to open file");
            file.write_all(result.as_bytes())
                .expect("Unable to write to file!");
        });
    }
}
