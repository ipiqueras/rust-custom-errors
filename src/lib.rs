#![allow(dead_code, unused_macros, unused_variables)]
use std::path::{Path, PathBuf};
use ini::{self};
use url::{self};
use std::error;
use std::fmt::{self, Display};

type Result<T> = std::result::Result<T, MyError>;

pub struct MyError {
    repr: Repr,
}

impl fmt::Debug for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.repr, f)
    }
}

impl fmt::Display for MyError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.repr {
            Repr::Custom(ref c) => c.error.fmt(fmt),
            Repr::Simple(kind) => write!(fmt, "{}", kind.as_str()),
        }
    }
}

enum Repr {
    Simple(ErrorKind),
    Custom(Box<Custom>),
}

impl fmt::Debug for Repr {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Repr::Custom(ref c) => fmt::Debug::fmt(&c, fmt),
            Repr::Simple(kind) => fmt.debug_tuple("Kind").field(&kind).finish(),
        }
    }
}

#[derive(Debug)]
struct Custom {
    kind: ErrorKind,
    error: Box<dyn error::Error+Send+Sync>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[allow(deprecated)]
pub enum ErrorKind {
    ConfigNotFound,
    KeyNotFound,
    Parse,
    UrlError
}

impl ErrorKind {
    pub(crate) fn as_str(&self) -> &'static str {
        match *self {
            ErrorKind::ConfigNotFound => "'configuration' section not found",
            ErrorKind::KeyNotFound => "Key not found",
            ErrorKind::Parse => "INI parse error",
            ErrorKind::UrlError => "URL parse error",
        }
    }
}

impl From<ErrorKind> for MyError {
    #[inline]
    fn from(kind: ErrorKind) -> MyError {
        MyError {
            repr: Repr::Simple(kind)
        }
    }
}

impl MyError {
    pub fn new<E>(kind: ErrorKind, error: E) -> MyError
        where E: Into<Box<dyn error::Error+Send+Sync>>
    {
        Self::_new(kind, error.into())
    }

    fn _new(kind: ErrorKind, error: Box<dyn error::Error+Send+Sync>) -> MyError {
        MyError {
            repr: Repr::Custom(Box::new(Custom {
                kind,
                error,
            }))
        }
    }

    pub fn get_mut(&mut self) -> Option<&mut (dyn error::Error+Send+Sync+'static)> {
        match self.repr {
            Repr::Simple(..) => None,
            Repr::Custom(ref mut c) => Some(&mut *c.error),
        }
    }

    pub fn into_inner(self) -> Option<Box<dyn error::Error+Send+Sync>> {
        match self.repr {
            Repr::Simple(..) => None,
            Repr::Custom(c) => Some(c.error)
        }
    }

    pub fn kind(&self) -> ErrorKind {
        match self.repr {
            Repr::Simple(kind) => kind,
            Repr::Custom(ref c) => c.kind,
        }
    }
}

impl error::Error for MyError {
    fn description(&self) -> &str {
        match self.repr {
            Repr::Simple(..) => self.kind().as_str(),
            Repr::Custom(ref c) => c.error.description(),
        }
    }

    #[allow(deprecated)]
    fn cause(&self) -> Option<&dyn error::Error> {
        match self.repr {
            Repr::Simple(..) => None,
            Repr::Custom(ref c) => c.error.cause(),
        }
    }

    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self.repr {
            Repr::Simple(..) => None,
            Repr::Custom(ref c) => c.error.source(),
        }
    }
}

// Implement the conversion from `ini::ini::Error` to `MyError`.
// This will be automatically called by `?` if a `ini::ini::Error`
// needs to be converted into a `MyError`.
impl From<ini::ini::Error> for MyError {
    fn from(err: ini::ini::Error) -> MyError {
        MyError::new(ErrorKind::Parse, err)
    }
}

// Implement the conversion from `url::ParseError` to `MyError`.
// This will be automatically called by `?` if a `url::ParseError`
// needs to be converted into a `MyError`.
impl From<url::ParseError> for MyError {
    fn from(err: url::ParseError) -> MyError {
        MyError::new(ErrorKind::UrlError, err)
    }
}

#[derive(Debug, Clone)]
pub struct Repository {

    pub name: String,
    pub url: url::Url,
}

pub fn from_file(filepath: &Path) -> Result<Repository> {

    let ini_content = ini::Ini::load_from_file(filepath)?;
    let config = ini_content.section(Some("configuration"))
        .ok_or({
            let err_str = format!("'configuration' section not found on file: {:?}", &filepath);
            MyError::new(ErrorKind::ConfigNotFound, err_str)
        })?;
    let url = config.get("url").ok_or({
            let err_str = format!("Key 'url' not found on file: {:?}", &filepath);
            MyError::new(ErrorKind::KeyNotFound, err_str)
        })?;
    let url = url::Url::parse(url)?;
    Ok(Repository {
        name: String::from("algo"),
        url,
    })
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn one_result() {

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"[Globals]

[configuration]
url         = http://tttechsvn.vie.at.tttech.ttt
access      = svn
variable    = SVNROOT
layout      = tbtn
responsible = MSF
").unwrap();
        let repository = from_file(file.path()).unwrap();
        assert_eq!(repository.url.to_string(), "http://tttechsvn.vie.at.tttech.ttt/");
    }

    #[test]
    fn fail_parse() {

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"[Globals

url         = http://tttechsvn.vie.at.tttech.ttt
access      = svn
variable    = SVNROOT
layout      = tbtn
responsible = MSF
").unwrap();
        let repository = from_file(file.path()).err();
        if let Some(err) = repository {
            assert!(err.to_string().starts_with("7:0 Expecting"));
        }
    }

    #[test]
    fn fail_section() {

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"[Globals]

[no correct name]
url         = http://tttechsvn.vie.at.tttech.ttt
access      = svn
variable    = SVNROOT
layout      = tbtn
responsible = MSF
").unwrap();
        let repository = from_file(file.path()).err();
        if let Some(err) = repository {
            assert!(err.to_string().starts_with("'configuration' section not found"));
        }
    }

    #[test]
    fn fail_key() {

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"[Globals]

[configuration]
access      = svn
variable    = SVNROOT
layout      = tbtn
responsible = MSF
").unwrap();
        let repository = from_file(file.path()).err();
        if let Some(err) = repository {
            assert!(err.to_string().starts_with("Key 'url' not found"));
        }
    }

    #[test]
    fn fail_parse_url() {

        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"[Globals]

[configuration]
url         = tttechsvn.vie.at.tttech.ttt

").unwrap();
        let repository = from_file(file.path()).err();
        if let Some(err) = repository {
            assert_eq!(err.to_string(), "relative URL without a base");
        }
    }
}
