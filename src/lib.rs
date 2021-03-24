pub mod archive;
pub mod error;
pub mod file;
pub mod source;

use error::Error;
use error::Result;
use is_libzip_sys as ffi;

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;
    use std::ffi::{CStr, CString};
    use std::io::Read;

    use std::string::String;

    use tempdir::TempDir;

    #[test]
    fn round_trip() {
        let tempdir = TempDir::new("test").unwrap();
        let zip_path = CString::new(tempdir.path().join("file.zip").to_str().unwrap()).unwrap();
        let foo = "Lorem ipsum dolor sit amet";
        let bar = "sed do eiusmod tempor incididunt ut labore et dolore magna aliqua";

        {
            let file_source: source::Source<source::File> =
                (&zip_path as &CStr).try_into().unwrap();
            let mut archive = archive::Archive::open(
                file_source,
                [archive::OpenFlag::Create, archive::OpenFlag::Exclusive],
            )
            .unwrap();
            let foo_source: source::Source<&[u8]> = foo.as_bytes().try_into().unwrap();
            archive
                .add(
                    &CString::new("foo").unwrap(),
                    foo_source,
                    file::Encoding::Guess,
                    false,
                )
                .unwrap();
            let bar_source: source::Source<&[u8]> = bar.as_bytes().try_into().unwrap();
            archive
                .add(
                    &CString::new("bar").unwrap(),
                    bar_source,
                    file::Encoding::Guess,
                    false,
                )
                .unwrap();
            archive.close().unwrap();
        }

        {
            let file_source: source::Source<source::File> =
                (&zip_path as &CStr).try_into().unwrap();
            let mut archive = archive::Archive::open(
                file_source,
                [
                    archive::OpenFlag::CheckConsistency,
                    archive::OpenFlag::ReadOnly,
                ],
            )
            .unwrap();
            let mut foo_buf = String::new();
            archive
                .open_file(&CString::new("foo").unwrap(), [], [])
                .unwrap()
                .read_to_string(&mut foo_buf)
                .unwrap();
            assert_eq!(foo_buf, foo);
            let mut bar_buf = String::new();
            archive
                .open_file(&CString::new("bar").unwrap(), &[], &[])
                .unwrap()
                .read_to_string(&mut bar_buf)
                .unwrap();
            assert_eq!(bar_buf, bar);
            archive.close().unwrap();
        }
    }
}
