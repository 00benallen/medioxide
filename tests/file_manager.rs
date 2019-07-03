#[cfg(test)]
mod tests {

    use medioxide::file_manager::{
        FileManager,
        Result
    };

    use std::io::*;
    use std::path::*;
    use std::fs;

    fn remove_tmp_dir() {
        fs::remove_dir_all(Path::new("./tmp")).expect("Temp directory could not be removed");
    }

    #[test]
    fn file_manager_saves_and_retrieves_files() -> Result<()> {
        remove_tmp_dir(); //just in case


        let fm = FileManager::new(Path::new("./tmp"), true)?;

        let test_data = "This string will be read";
        let b = test_data.as_bytes();
        fm.add_file("test", "test.txt", b)?;

        let mut f = fm.get_file_by_id("test")?.expect("File was not found in FileManager");

        let mut s = String::new();
        f.read_to_string(&mut s).expect("Could not read file data to string");

        assert_eq!(test_data, s, "Data read from file did not match expected, data was {}", s);

        remove_tmp_dir();

        Ok(())
    }
}