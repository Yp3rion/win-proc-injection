use std::error::Error;
use std::fmt::Display;
use std::result;

pub type Result<T> = result::Result<T, InjectionError>;

// We want our injector to behave differently based on the semantics of the injection error
// (e.g. not being able to obtain debug privilege is not a blocker per se, while if we encounter
// an error during manipulation of the target process we may want to ask the user to pick a different
// target). Unfortunately, since there is no 1:1 mapping between each type of injection error and 
// the possible underlying errors, it is not possible to implement From for this type and it is 
// thus necessary to rely on map_err. The error variants wrap an Option<Box<dyn Error>> because due
// to how the windows crate works, the underlying error condition may be represented by 
// a type which does not implement the Error trait, such as BOOL.
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