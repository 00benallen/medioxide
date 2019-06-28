use std::net::{ TcpListener, TcpStream};
use std::path::{ Path, PathBuf };
use std::io;
use std::io::prelude::*;
use std::thread;
use std::fs::File;

use failure::Fail;

use httparse::{
    EMPTY_HEADER,
    Request
};

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "Directory {} doesn't exist", _0)]
    DirectoryDoesntExist(String),

    #[fail(display = "Could not start server because {}", _0)]
    CouldNotStartServer(#[fail(cause)] io::Error),

    #[fail(display = "Could not parse request because {}", _0)]
    CouldNotParseRequest(#[fail(cause)] httparse::Error),

    #[fail(display = "Could not read request from stream because {}", _0)]
    CouldNotReadRequest(#[fail(cause)] io::Error),

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
            match stream {
                Ok(stream) => {
                    let image_folder = self.image_folder.clone();
                    thread::spawn(move || {
                        ImageServer::handle_incoming(stream, image_folder).unwrap();
                    });
                }
                Err(e) => println!("Unable to connect: {}", e),
            }
        }
        Ok(())

    }

    fn handle_incoming(stream: TcpStream, mut image_folder: PathBuf) -> Result<()> {
        let mut headers = [EMPTY_HEADER; 16];
        let mut req = Request::new(&mut headers);
        let mut buf = [0u8; 4096];
        req = parse_req_from_stream(stream.try_clone().unwrap(), req, &mut buf).unwrap();

        let mut path = String::from(req.path.unwrap());
        path.remove(0);
        
        image_folder.push(path);

        let sender = FileSender::new(&image_folder).unwrap();
        sender.write_to_stream(stream).unwrap();
        Ok(())
    }

}

fn parse_req_from_stream<'a, 'b: 'a>(
    mut stream: TcpStream, 
    mut empty_req: Request<'a, 'b>, 
    buf: &'b mut [u8]) -> Result<Request<'a, 'b>> {

    match stream.read(buf) {
        Ok(_) => {
            empty_req.parse(buf).map_err(Error::CouldNotParseRequest)?;
            Ok(empty_req)
        },
        Err(e) => {
            Err(Error::CouldNotReadRequest(e))
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