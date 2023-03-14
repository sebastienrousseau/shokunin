use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
/// File struct to hold the name and content of a file.
pub(crate) struct File {
    pub(crate) name: String,
    pub(crate) content: String,
}
/// Add files to a vector.
pub(crate) fn add_files(path: &Path) -> io::Result<Vec<File>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.to_string())
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid filename"))?;
            let content = match fs::read_to_string(&path) {
                Ok(content) => content,
                Err(err) => {
                    println!("Skipping file {}: {}", name, err);
                    continue;
                }
            };
            files.push(File { name, content });
        }
    }
    Ok(files)
}
