#[derive(Debug)]
pub enum Error {
    IoError(std::io::Error),
    PraserError,
}