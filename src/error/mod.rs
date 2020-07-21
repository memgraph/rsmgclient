use std::fmt;

pub struct MgError {
    message: String,
}

impl fmt::Display for MgError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl MgError {
    pub fn new(message: String) -> MgError {
        MgError {
            message,
        }
    }
}
