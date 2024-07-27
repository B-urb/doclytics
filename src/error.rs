use std::fmt;

#[derive(Debug)]
pub enum ResponseError {
    Io(std::io::Error),
    ParseBody(std::num::ParseIntError),
    RequestError(std::io::Error),
    Other(String),
}

// Step 2: Implement std::fmt::Display
impl fmt::Display for ResponseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ResponseError::Io(ref err) => write!(f, "IO error: {}", err),
            ResponseError::ParseBody(ref err) => write!(f, "Parse error: {}", err),
            ResponseError::RequestError(ref err) => write!(f, "Parse error: {}", err),
            ResponseError::Other(ref err) => write!(f, "Other error: {}", err),
        }
    }
}

impl std::error::Error for ResponseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            ResponseError::Io(ref err) => Some(err),
            ResponseError::ParseBody(ref err) => Some(err),
            ResponseError::RequestError(ref err) => Some(err),
            ResponseError::Other(_) => None,
        }
    }
}

