use std::io;
use std::path::Path;

pub struct Socket(String);

impl Socket {
    pub fn new(socket: String) -> Self {
        Socket(socket)
    }

    pub fn value(&self) -> String {
        self.0.clone()
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn get_path(&self) -> &Path {
        std::path::Path::new(&self.0)
    }

    pub fn file_exists(&self) -> bool {
        self.get_path().exists()
    }

    pub fn remove_file(&self) -> io::Result<()> {
        let path = self.get_path();

        if !path.exists() {
            return Ok(());
        }

        std::fs::remove_file(path)
    }
}
