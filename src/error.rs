#[derive(Debug)]
pub struct S5Exception {
    pub error_type: S5ErrorType,
    pub error_message: String,
}
#[derive(Debug)]
pub enum UnExpectedError {
    IoError(std::io::Error),
    Utf8Error(std::str::Utf8Error),
    S5Error(S5Exception),
}
impl std::fmt::Display for S5Exception {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} : {}",
            self.error_type.to_string(),
            self.error_message
        )
    }
}
impl std::error::Error for S5Exception {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}
impl From<std::io::Error> for UnExpectedError{
    fn from(s: std::io::Error) -> Self {
        UnExpectedError::IoError(s)
    }
}
impl From<std::str::Utf8Error> for UnExpectedError{
    fn from(s: std::str::Utf8Error) -> Self {
        UnExpectedError::Utf8Error(s)
    }
}
impl From<S5Exception> for UnExpectedError{
    fn from(s: S5Exception) -> Self {
        UnExpectedError::S5Error(s)
        
    }
}

#[derive(Debug)]
pub enum S5ErrorType {
    ProtocolError,
}
impl std::fmt::Display for UnExpectedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnExpectedError::IoError(ref e) => write!(f, "IoError: {}", e),
            UnExpectedError::S5Error(ref e) => write!(f, "S5Error: {}", e),
            UnExpectedError::Utf8Error(ref e) => write!(f, "Utf8Error: {}", e),
        }
    }
}
impl std::error::Error for UnExpectedError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self {
            UnExpectedError::S5Error(ref e) => Some(e),
            UnExpectedError::IoError(ref e) => Some(e),
            UnExpectedError::Utf8Error(ref e) => Some(e),
        }
    }
}

impl std::fmt::Display for S5ErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            S5ErrorType::ProtocolError => write!(f, "ProtocolError"),
        }
    }
}

