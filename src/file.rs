pub mod file {
    use std::fs;
    use std::path;

    pub fn read_file(file_path: &str) -> Result<String, String> {
        let corrected_file_path = match file_path.starts_with("/") {
            true => &file_path[1..],
            false => file_path,
        };
        let path = path::Path::new(corrected_file_path);
        if !path.exists() {
            let error = String::from("File does not exist: ") + path.to_str().expect("");
            println!("{}", error);
            return Err(String::from("File does not exist"));
        }
        return match fs::read_to_string(path) {
            Ok(content) => Ok(content),
            Err(e) => {
                println!("{}", e);
                let error = String::from("Failed to read file: ") + file_path;
                return Err(error);
            }
        };
    }
}
