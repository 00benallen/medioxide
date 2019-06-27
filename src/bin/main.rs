extern crate medioxide;
pub use medioxide::{
    Result,
    ImageServer,
};

fn main() -> Result<()> {
    ImageServer::new("./files")?.start("127.0.0.1:8080")?;
    Ok(())
}
