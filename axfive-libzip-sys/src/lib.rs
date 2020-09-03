#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// Needed for tests
#[cfg(test)]
#[macro_use]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;
    use std::string::String;
    use std::vec::Vec;
    use std::ffi::CString;
    use std::os::raw::{c_int, c_void};
    use std::mem::zeroed;

    #[test]
    fn round_trip() {
        let tempdir = TempDir::new("test").unwrap();
        let zip_path = CString::new(tempdir.path().join("file.zip").to_str().unwrap()).unwrap();
        let foo = "Lorem ipsum dolor sit amet";
        let bar = "sed do eiusmod tempor incididunt ut labore et dolore magna aliqua";
        unsafe {
            let mut errorp: c_int = 0;
            let zip = zip_open(zip_path.as_ptr(), (ZIP_CREATE | ZIP_EXCL) as _, &mut errorp as _);
            assert!(!zip.is_null());

            let mut error = zeroed();
            ffi::zip_error_init(&mut error as _);
            let source = zip_source_buffer_create(foo.as_ptr() as _, foo.len() as zip_uint64_t, 0, &mut error as _);
            assert!(!source.is_null());

            assert_ne!(zip_file_add(zip, CString::new("foo").unwrap().as_ptr(), source, ZIP_FL_ENC_GUESS), -1);

            let source = zip_source_buffer_create(bar.as_ptr() as _, bar.len() as _, 0, &mut error as _);
            assert!(!source.is_null());

            assert_ne!(zip_file_add(zip, CString::new("bar").unwrap().as_ptr(), source, ZIP_FL_ENC_GUESS), -1);

            assert_eq!(zip_close(zip), 0);
        }

        unsafe {
            let mut errorp: c_int = 0;
            let zip = zip_open(zip_path.as_ptr(), (ZIP_RDONLY | ZIP_CHECKCONS) as _, &mut errorp as _);
            assert!(!zip.is_null());

            let foo_file = zip_fopen(zip, CString::new("foo").unwrap().as_ptr(), ZIP_FL_ENC_GUESS);
            assert!(!foo_file.is_null());
            let mut foo_content: Vec<u8> = Vec::with_capacity(1024);
            let new_len = zip_fread(foo_file, foo_content.as_mut_ptr() as _, foo_content.capacity() as _);
            assert_ne!(new_len, -1);
            assert_eq!(zip_fclose(foo_file), 0);
            foo_content.set_len(new_len as _);
            assert_eq!(String::from_utf8_unchecked(foo_content), foo);

            let bar_file = zip_fopen(zip, CString::new("bar").unwrap().as_ptr(), ZIP_FL_ENC_GUESS);
            assert!(!bar_file.is_null());
            let mut bar_content: Vec<u8> = Vec::with_capacity(1024);
            let new_len = zip_fread(bar_file, bar_content.as_mut_ptr() as _, bar_content.capacity() as _);
            assert_ne!(new_len, -1);
            assert_eq!(zip_fclose(bar_file), 0);
            bar_content.set_len(new_len as _);
            assert_eq!(String::from_utf8_unchecked(bar_content), bar);

            assert_eq!(zip_close(zip), 0);
        }
    }
}
