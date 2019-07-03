use std::path::Path;

extern crate medioxide;
use medioxide::{
    http::{
        MediaServer,
        //Result
    },
    file_manager::{
        FileManager,
        Result
    }
};

use std::io::Read;

fn main() -> Result<()> {
    //MediaServer::new(Path::new("./media"))?.start();

    let fm = FileManager::new(Path::new("./media"), true)?;

    let b = "This string will be read".as_bytes();
    fm.add_file("test", "test.txt", b)?;

    let mut f = fm.get_file_by_id("test")?.unwrap();

    let mut s = String::new();
    f.read_to_string(&mut s).unwrap();

    println!("{}", s);
    Ok(())
}
