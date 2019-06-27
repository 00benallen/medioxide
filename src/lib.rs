use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream};
use std::path::{ Path, PathBuf };
use std::io;
use std::io::prelude::*;
use std::thread;
use std::fs::File;

use failure::Fail;

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "Directory {} doesn't exist", _0)]
    DirectoryDoesntExist(String),

    #[fail(display = "Could not start server because {}", _0)]
    CouldNotStartServer(#[fail(cause)] io::Error),

    #[fail(display = "Could not parse request because {}", _0)]
    CouldNotParseRequest(#[fail(cause)] io::Error),

    #[fail(display = "File {} doesn't exist", _0)]
    FileDoesntExist(String),

    #[fail(display = "Path {} is not a file", _0)]
    PathIsNotFile(String),
    
    #[fail(display = "Response could not be sent because {}", _0)]
    ResponseCouldNotBeSent(#[fail(cause)] io::Error),
    
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct ImageServer {
    image_folder: PathBuf
}

impl ImageServer {

    pub fn new(image_folder: &str) -> Result<ImageServer> {

        let image_folder = Path::new(image_folder);

        if image_folder.exists() {

            let image_folder = image_folder.to_path_buf();

            return Ok(ImageServer {
                image_folder
            });
        }

        Err(Error::DirectoryDoesntExist(String::from(image_folder.to_str().unwrap())))

    }

    pub fn start(&self, address: &str) -> Result<()> {

        let listener = TcpListener::bind(address).map_err(Error::CouldNotStartServer)?;
        
        // println!("Listening for connections on port {}", port);

        for stream in listener.incoming() {
            let mut image_folder = self.image_folder.clone();
            match stream {
                Ok(stream) => {
                    thread::spawn(move || {
                        let request = Request::new(stream.try_clone().unwrap()).unwrap();
                        image_folder.push(request.file_name);

                        let sender = FileSender::new(&image_folder).unwrap();
                        sender.write_to_stream(stream).unwrap();
                    });
                }
                Err(e) => println!("Unable to connect: {}", e),
            }
        }
        Ok(())

    }

}

struct Request {
    pub file_name: String,
}

impl Request {

    pub fn new(mut stream: TcpStream) -> Result<Request> {
        let mut buf = [0u8; 4096];
        match stream.read(&mut buf) {
            Ok(_) => {
                let req_str = String::from_utf8_lossy(&buf);
                let lines: Vec<&str> = req_str.split("\r\n").collect();
                let tokens_in_first_line: Vec<&str> = lines[0].split(' ').collect();
                let mut file_name = String::from(tokens_in_first_line[1]);
                file_name.remove(0);  //remove leading slash
                Ok(Request {
                    file_name
                })
            },
            Err(e) => {
                Err(Error::CouldNotParseRequest(e))
            }
        }
    }

}

struct FileSender {
    file_path: PathBuf
}

impl FileSender {
    pub fn new(path: &Path) -> Result<FileSender> {

        if path.exists() {

            if path.is_file() {

                Ok(FileSender {
                    file_path: path.to_path_buf()
                })

            } else {
                Err(Error::PathIsNotFile(path.to_string_lossy().to_string()))
            }

        } else {
            Err(Error::FileDoesntExist(path.to_string_lossy().to_string()))
        }
    }

    pub fn write_to_stream(&self, mut stream: TcpStream) -> Result<()> {

        let mut buf = Vec::new();
        let mut file = File::open(self.file_path.clone()).unwrap();
        file.read_to_end(&mut buf).unwrap();

        let headers = [
            "HTTP/1.1 200 OK",
            "Content-type: image/jpeg",
            "\r\n"
        ];
        let mut response = headers.join("\r\n")
            .to_string()
            .into_bytes();
        response.extend(buf);

        match stream.write(&response) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::ResponseCouldNotBeSent(e)),
        }
    }
}