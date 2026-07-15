pub struct Package {
    pub name: String,
    pub version: String,
    pub code: String,
}
impl Package {
    pub fn new(name: String, version: String, code: String) -> Self{
        Self { name, version, code }
    }
}