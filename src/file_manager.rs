use std::io;
use std::io::{
    BufReader,
    BufRead,
    Write,
    Read
};
use std::fs::{
    File,
    create_dir,
};
use std::path::{
    Path,
    PathBuf,
};
use std::collections::HashMap;
use std::sync::{
    RwLock,
    RwLockReadGuard,
    RwLockWriteGuard,
    Arc,
};

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "Directory to manage \"{}\" does not exist ", _0)]
    DirectoryDoesNotExist(String),
    #[fail(display = "Directory could not be created because {}", _0)]
    CouldNotCreateDirectory(#[fail(cause)] io::Error),
    #[fail(display = "Directory could not be read from because {}", _0)]
    CouldNotReadFromDirectory(#[fail(cause)] io::Error),
    #[fail(display = "Could not load in-memory file index because {}", _0)]
    CouldNotLoadIndexIntoMemory(#[fail(cause)] io::Error),
    #[fail(display = "Could create new file because {}", _0)]
    CouldNotCreateNewFile(#[fail(cause)] io::Error),
    #[fail(display = "Entry in index was corrupted")]
    CorruptedIndexEntry,
    #[fail(display = "Could not aquire index lock")]
    CouldNotLockIndex,
    #[fail(display = "File {} already exists", _0)]
    FileAlreadyExists(String),
    #[fail(display = "File {} does not exist in directory, but was found in index", _0)]
    IndexedFileDoesNotExist(String),
}

pub type Result<T> = std::result::Result<T, Error>;
pub type FileMap = HashMap<String, PathBuf>;

pub struct IndexEntry {
    id: String,
    file_name: PathBuf,
}

impl IndexEntry {
    pub fn from_str(text: &str) -> Result<IndexEntry> {
        let tokens: Vec<&str> = text.split(' ').collect();
        
        if tokens.len() == 2 {
            Ok(IndexEntry { id: String::from(tokens[0]), file_name: PathBuf::from(tokens[1]) })
        } else {
            Err(Error::CorruptedIndexEntry)
        }
    }
}

#[derive(Clone, Debug)]
pub struct FileManager {
    folder: PathBuf,
    index: Arc<RwLock<FileMap>>,
}

impl FileManager {

    pub fn new(folder: &Path, create: bool) -> Result<FileManager> {

        if !folder.exists() {
            if create {
                create_dir(folder).map_err(Error::CouldNotCreateDirectory)?;
            } else {
                return Err(Error::DirectoryDoesNotExist(String::from(folder.to_string_lossy())));
            }
        }

        let file_map = FileManager::load_index_from_disk(folder, create)?;

        let index = Arc::new(RwLock::new(file_map));
        let folder = folder.to_path_buf();

        // If we're here it means the directory is there to use
        Ok(FileManager {
            folder,
            index
        })
    }

    fn load_index_from_disk(folder: &Path, create: bool) -> Result<FileMap> {
        if !folder.exists() {
            if create {
                create_dir(folder).map_err(Error::CouldNotCreateDirectory)?;
            } else {
                return Err(Error::DirectoryDoesNotExist(String::from(folder.to_string_lossy())));
            }
        }

        let mut index_map = HashMap::new();

        let mut index_path = folder.to_path_buf();
        index_path.push("/index.txt");
        let index_file_result = File::open(index_path.clone());

        // since the directory contained our index file, we need to load the index from disk
        match index_file_result {
            Ok(index_file) => {

                let br = BufReader::new(index_file);

                for line in br.lines() {
                    let line = line.map_err(Error::CouldNotLoadIndexIntoMemory)?;
                    let entry = IndexEntry::from_str(&line)?;

                    index_map.insert(entry.id, entry.file_name);
                }

                Ok(index_map)
            },
            Err(err) => {

                if err.kind() == io::ErrorKind::NotFound {
                    File::create(index_path);
                    Ok(index_map) // directory contained no index, so return empty HashMap
                } else {
                    Err(Error::CouldNotLoadIndexIntoMemory(err)) // some other issue occured so fail
                }

                
            }
        }
    }

    pub fn get_file_by_path(&self, path: &Path) -> Result<File> {
        unimplemented!() // TODO
    }

    pub fn get_file_by_id(&self, id: &str) -> Result<Option<File>> {
        let read_files_lck = self.lock_index_for_read()?;

        if let Some(entry) = read_files_lck.get(id) {
            if entry.exists() {
                Ok(Some(File::open(entry).map_err(Error::CouldNotReadFromDirectory)?))
            } else {
                Err(Error::IndexedFileDoesNotExist(String::from(entry.to_string_lossy())))
            }
        } else {
            Ok(None)
        }

    
    }

    pub fn get_path_by_id(&self, id: &str) -> Result<&'static Path> {
        unimplemented!() // TODO
    }

    pub fn get_id_from_path(&self, path: &Path) -> Result<&str> {
        unimplemented!() // TODO
    }

    pub fn file_exists_with_id(&self, id: &str) -> Result<bool> {
        let read_files_lck = self.lock_index_for_read()?;
        Ok(read_files_lck.contains_key(id))
    }

    pub fn file_exists_at_path(&self, path: &Path) -> Result<bool> {
        let read_files_lck = self.lock_index_for_read()?;
        
        for entry in read_files_lck.iter() {
            if entry.1 == path {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn add_file<S: Read>(&self, id: &str, file_name: &str, mut data: S) -> Result<PathBuf> {
        let mut read_files_lck = self.lock_index_for_write()?;

        let mut buffer = Vec::new();
        data.read_to_end(&mut buffer).map_err(Error::CouldNotCreateNewFile)?;

        let mut file_path = self.folder.clone();
        file_path.push(file_name);

        if file_path.exists() {
            Err(Error::FileAlreadyExists(String::from(file_name)))
        } else {
            let mut new_file = File::create(file_path.clone()).map_err(Error::CouldNotCreateNewFile)?;

            new_file.write_all(&buffer).map_err(Error::CouldNotCreateNewFile)?;

            read_files_lck.insert(String::from(id), file_path.clone());

            Ok(file_path)
        }

    }

    fn lock_index_for_read(&self) -> Result<RwLockReadGuard<'_, HashMap<String, PathBuf>>> {
        self.index.read().map_err(|_| Error::CouldNotLockIndex)
    }

    fn lock_index_for_write(&self) -> Result<RwLockWriteGuard<'_, HashMap<String, PathBuf>>> {
        self.index.write().map_err(|_| Error::CouldNotLockIndex)
    }

}