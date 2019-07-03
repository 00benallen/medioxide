use std::io;
use std::fs::File;
use std::path::{
    Path,
    PathBuf,
};
use std::thread;
use std::sync::Arc;

use rocket::{
    Rocket,
    Data
};



#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "Input was invalid UTF-8 at index {}", _0)]
    Utf8Error(usize),
    #[fail(display = "{}", _0)]
    Io(#[fail(cause)] io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct MediaServer {
    media_folder: PathBuf,
}

impl MediaServer {
    pub fn new(media_folder: &Path) -> Result<MediaServer> {
        let media_folder = media_folder.to_path_buf();
        Ok(MediaServer { media_folder })
    }

    pub fn start(&self) {
        let handle = thread::spawn(move || {
            rocket::ignite().mount("/media", routes![get_file, upload_pdf]).launch();
        });
        handle.join().expect("Rocket thread panicked!");
    }

    
}

// Static request handlers for MediaServer

#[get("/<file_id>")]
fn get_file(file_id: String) -> Result<File> {
    unimplemented!() // TODO
}

#[post("/upload/pdf/<file_id>", format = "multipart/form-data", data = "<data>")]
fn upload_pdf(file_id: String, data: Data) -> io::Result<String> {
    data.stream_to_file("./tmp/file_id.pdf").map(|n| n.to_string())
}