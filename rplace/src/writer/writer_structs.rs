use crate::structs::{Value, Var};

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
    pub vals: Vec<(Var,Value)>,
}
pub struct WriterResult {
    pub file_data: Vec<FileData>,
    pub derives: Vec<Derive>,
    pub to_parse: Vec<String>,
}
impl WriterResult {
    pub fn new() -> Self{
        Self { file_data: Vec::new(), derives: Vec::new(), to_parse: vec![]}
    }
    pub fn set_to_parse(&mut self, to_parse: Vec<String>){
        self.to_parse = to_parse;
    }
    pub fn push_to_parse(&mut self, path: String){
        self.to_parse.push(path);
    }
    pub fn append(&mut self, mut data: WriterResult){
        self.file_data.append(&mut data.file_data);
        self.derives.append(&mut data.derives);
        self.to_parse.append(&mut data.to_parse);
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