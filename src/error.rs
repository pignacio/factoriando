use std::io;

#[derive(Debug)]
pub enum Error{
    Simple(String),
    Csv(csv::Error),
    Io{err: io::Error, path: String},
    SerdeJson(serde_json::Error),
}

impl Error {
    pub fn from<S: AsRef<str>>(err: io::Error, path: S) -> Self {
        Error::Io{err, path: path.as_ref().to_owned()}
    }
}

impl From<csv::Error> for Error {
    fn from(err: csv::Error) -> Self {
        Error::Csv(err)
    }
}


impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::SerdeJson(err)
    }
}