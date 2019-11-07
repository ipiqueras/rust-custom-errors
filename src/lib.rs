#![allow(dead_code, unused_imports, unused_macros, unused_variables)]
use std::num::ParseIntError;
use std::result;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::io;
use std::num;

// We derive `Debug` because all types should probably derive `Debug`.
// This gives us a reasonable human readable description of `CliError` values.
#[derive(Debug)]
pub enum CliError {
    Io(io::Error),
    Parse(num::ParseIntError),
}


fn file_double<P: AsRef<Path>>(file_path: P) -> Result<i32, CliError> {

    let mut file = File::open(file_path).map_err(CliError::Io)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).map_err(CliError::Io)?;
    let n: i32 = contents.trim().parse().map_err(CliError::Parse)?;
    Ok(2 * n)
}


fn double_number(number_str: &str) -> Result<i32, ParseIntError> {
    number_str.parse::<i32>().map(|n| 2 * n)
}


#[cfg(test)]
mod tests {
    use tempfile::NamedTempFile;
    use std::io::{self, Write, Read};
    use std::path::Path;

    #[test]
    fn it_works() {

        assert_eq!(super::double_number("2"), Ok(4));
    }

    #[test]
    fn it_works_again() {

        match super::double_number("10") {
            Ok(n) => assert_eq!(n, 20),
            Err(err) => panic!("Test error")
        }
    }

    #[test]
    fn file_double() {
        let mut file1 = NamedTempFile::new().unwrap();
        let text = "8";
        file1.write_all(text.as_bytes()).unwrap();
        match super::file_double(file1.path()) {
            Ok(num) => assert_eq!(num, 16),
            Err(err) => assert!(false),
        }
    }

    #[test]
    fn file_double_parse_error() {
        let mut file1 = NamedTempFile::new().unwrap();
        let text = "p";
        file1.write_all(text.as_bytes()).unwrap();
        match super::file_double(file1.path()) {
            Ok(n) => assert!(false),
            Err(err) => match err {
                super::CliError::Io(ref err) => assert!(false),
                super::CliError::Parse(ref err) => assert!(true),
            },
        }
    }

    #[test]
    fn file_double_read_error() {
        match super::file_double(Path::new("Doenotexist")) {
            Ok(n) => assert!(false),
            Err(err) => match err {
                super::CliError::Io(ref err) => assert!(true),
                super::CliError::Parse(ref err) => assert!(false),
            },
        }
    }
}
