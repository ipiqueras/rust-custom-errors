#![allow(dead_code, unused_macros, unused_variables)]
use std::path::{Path, PathBuf};
use ini::{self};
use url::{self};
use std::error;
use std::fmt::{self, Display};
type Result<T> = std::result::Result<T, MyError>;

#[derive(Debug)]
pub enum MyError {

    ConfigNotFound {
        file: PathBuf,
    },
    KeyNotFound {
        key: String,
        file: PathBuf,
    },
    Parse(ini::ini::Error),
    UrlError(url::ParseError),
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MyError::ConfigNotFound { ref file } => write!(f, "'configuration' section not found on file {:?}", &file),
            MyError::KeyNotFound { ref key, ref file } => write!(f, "Key '{}' not found on file {:?}", &key, &file),
            MyError::Parse(ref err) => write!(f, "INI parse error: {}", err),
            MyError::UrlError(ref err) => write!(f, "URL parse error: {}", err),
        }
    }
}

impl error::Error for MyError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            MyError::ConfigNotFound { ref file } => None,
            MyError::KeyNotFound { ref key, ref file } => None,
            // The cause is the underlying implementation error type. Is implicitly
            // cast to the trait object `&error::Error`. This works because the
            // underlying type already implements the `Error` trait.
            MyError::Parse(ref e) => Some(e),
            MyError::UrlError(ref e) => Some(e),
        }
    }
}

// Implement the conversion from `ini::ini::Error` to `MyError`.
// This will be automatically called by `?` if a `ini::ini::Error`
// needs to be converted into a `MyError`.
impl From<ini::ini::Error> for MyError {
    fn from(err: ini::ini::Error) -> MyError {
        MyError::Parse(err)
    }
}

// Implement the conversion from `url::ParseError` to `MyError`.
// This will be automatically called by `?` if a `url::ParseError`
// needs to be converted into a `MyError`.
impl From<url::ParseError> for MyError {
    fn from(err: url::ParseError) -> MyError {
        MyError::UrlError(err)
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
        .ok_or(MyError::ConfigNotFound { file: filepath.to_owned() })?;
    let url = config.get("url").ok_or(MyError::KeyNotFound {
            file: filepath.to_owned(),
            key: String::from("url")
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
            assert!(err.to_string().starts_with("INI parse error: 7:0 Expecting"));
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
            assert_eq!(err.to_string(), "URL parse error: relative URL without a base");
        }
    }
}
