use crate::structs::{FileConfig, Value};

#[derive(Debug, Clone)]
pub struct ResValue{
    pub value:String,
}
pub enum FileWriteOptions{
    Override,
    Append,
}
pub struct FileData{
    pub data: String,
    pub path: String,
    pub options: Option<Vec<FileWriteOptions>>
}
pub struct Derive{
    pub path: String,
    pub vals: Vec<(String,Value)>,
}
pub struct WriterResult {
    pub file_data: Vec<FileData>,
    pub derives: Vec<Derive>,
}
impl WriterResult {
    pub fn new() -> Self{
        Self { file_data: Vec::new(), derives: Vec::new()}
    }
    pub fn append(&mut self, mut data: WriterResult){
        self.file_data.append(&mut data.file_data);
        self.derives.append(&mut data.derives);
    }
    pub fn push_elements(&mut self, data: String, path: String){
        self.file_data.push(FileData { data, path, options: None });
    }
}
impl IntoIterator for WriterResult {
    type Item = FileData;
    type IntoIter = std::vec::IntoIter<FileData>;

    fn into_iter(self) -> Self::IntoIter {
        self.file_data.into_iter()
    }
}