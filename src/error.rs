


use std::fmt; 
use std::io;
use std::time::SystemTimeError; 


#[derive(Debug)]
pub enum Error {
    IoError(io::Error), 
    SystemTimeError(SystemTimeError), 
    TooSmallWindow(usize, usize), 
    UnknownWindowSize, 
    NotUtf8iInput(Vec<u8>), 
    ControllCharInText(char), 
}


impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*; 
        match self {
            IoError(err) => write!(f, "{}", err), 
            SystemTimeError(err) => write!(f, "{}", err), 
            TooSmallWindow(width, height) => write!(f,  
                "Your Screen size width: {}x height: {}", width, height), 
            UnknownWindowSize => write!(f, "Wow! Unable to detect Terminal Window size"),
            NotUtf8iInput(utf) => {
                write!(f, "Unable to handle non-UTF8 input"); 
                for byte in utf.iter() {
                    write!(f, "\\x{:x}", byte)?;
                }

                Ok(())
            }, 
            ControllCharInText(literal) => write!(f, "Invalid Character entered {:?}", literal)

        }
    }
}

//returned result from type for the editor -> T can be anything
pub type Result<T> = std::result::Result<T, Error>; 