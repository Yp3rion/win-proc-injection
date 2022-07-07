use std::error::Error;
use std::fmt::Display;
use std::result;

pub type Result<T> = result::Result<T, InjectionError>;

#[derive(Debug)]
pub enum InjectionError {
    NoDebugPriv(Option<Box<dyn Error>>),
    ProcNotFound(Option<Box<dyn Error>>),
    LibNotFound(Option<Box<dyn Error>>),
    ExeNotFound(Option<Box<dyn Error>>),
    ProcError(Option<Box<dyn Error>>),
    UnexpectedError(Option<Box<dyn Error>>),
}

impl Display for InjectionError {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InjectionError::NoDebugPriv(err_opt) => write!(f, "Failed to obtain debug privilege. {}", 
                err_opt.as_ref().map_or("".to_string(), |err| format!("Underlying error: {}", err))),
            InjectionError::ProcNotFound(err_opt) => write!(f, "Target process could not be found. {}", 
                err_opt.as_ref().map_or("".to_string(), |err| format!("Underlying error: {}", err))),
            InjectionError::LibNotFound(err_opt) => write!(f, "The library to be injected could not be found. {}",
                err_opt.as_ref().map_or("".to_string(), |err| format!("Underlying error: {}", err))),
            InjectionError::ExeNotFound(err_opt) => write!(f, "The target executable could not be found. {}",
                err_opt.as_ref().map_or("".to_string(), |err| format!("Underlying error: {}", err))),
            InjectionError::ProcError(err_opt) => write!(f, "An error occurred while attempting to inject the target process. {}",
                err_opt.as_ref().map_or("".to_string(), |err| format!("Underlying error: {}", err))),
            InjectionError::UnexpectedError(err_opt) => write!(f, "An unexpected error occurred. {}",
                err_opt.as_ref().map_or("".to_string(), |err| format!("Underlying error: {}", err))),
        }
    }

}

impl Error for InjectionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            InjectionError::NoDebugPriv(err_opt) => err_opt.as_ref().map(|err| err.as_ref()), 
            InjectionError::ProcNotFound(err_opt) => err_opt.as_ref().map(|err| err.as_ref()),
            InjectionError::LibNotFound(err_opt) => err_opt.as_ref().map(|err| err.as_ref()),
            InjectionError::ExeNotFound(err_opt) => err_opt.as_ref().map(|err| err.as_ref()),
            InjectionError::ProcError(err_opt) => err_opt.as_ref().map(|err| err.as_ref()),
            InjectionError::UnexpectedError(err_opt) => err_opt.as_ref().map(|err| err.as_ref()),
        }
    }
}