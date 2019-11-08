#![allow(dead_code, unused_macros, unused_variables)]
use std::path::{Path, PathBuf};
use ini::{self};
use url::{self};
#[macro_use] extern crate failure;
use failure::Error;

// type Result<T> = std::result::Result<T, MyError>;

#[derive(Debug, Fail)]
pub enum MyError {
    #[fail(display = "'configuration' section not found on file: {:?}", file)]
    ConfigNotFound {
        file: PathBuf,
    },
    #[fail(display = "Key '{}' not found on file: {:?}", key, file)]
    KeyNotFound {
        key: String,
        file: PathBuf,
    },
    #[fail(display = "ini parser error: {}", _0)]
    Parse(#[fail(cause)] ini::ini::Error),
    #[fail(display = "url error: {}", _0)]
    UrlError(#[fail(cause)] url::ParseError),
}

#[derive(Debug, Clone)]
pub struct Repository {

    pub name: String,
    pub url: url::Url,
}

pub fn from_file(filepath: &Path) -> Result<Repository, Error> {

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
