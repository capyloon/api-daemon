#[cfg(test)]
extern crate mkstemp;
use std::io::{Result, Write};

fn do_write(writer: &mut Write) -> Result<usize> {
    writer.write(b"test")
}

#[test]
fn mkstemp() {
    let mut path;
    {
        let templ = ::std::env::temp_dir().to_str().unwrap().to_string() + "/testXXXXXX";
        let rc = mkstemp::TempFile::new(&templ, true);
        assert!(rc.is_ok());
        let mut temp_file = rc.unwrap();
        assert!(do_write(&mut temp_file).is_ok());
        path = temp_file.path().to_string();
    }
    assert!(::std::fs::metadata(&path).is_err());

    {
        let templ = ::std::env::temp_dir().to_str().unwrap().to_string() + "/testXXXXXX";
        let rc = mkstemp::TempFile::new(&templ, false);
        assert!(rc.is_ok());
        let mut temp_file = rc.unwrap();
        assert!(do_write(&mut temp_file).is_ok());
        path = temp_file.path().to_string();
    }
    assert!(::std::fs::metadata(&path).is_ok());
    assert!(::std::fs::remove_file(&path).is_ok());
}
