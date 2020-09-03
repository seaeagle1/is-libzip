use crate::error::ZipErrorT;
use crate::ffi;
use crate::Error;
use crate::Result;
use std::convert::{TryFrom, TryInto};
use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::path::Path;
use std::ptr::null_mut;

/// A simple marker enum, used to indicate that the source holds an open file handle.
pub enum File {}

/// Borrows or owns a source
#[derive(Debug)]
pub struct Source<T> {
    handle: *mut ffi::zip_source_t,
    phantom: PhantomData<T>,
}

impl<T> Source<T> {
    /// Indicate that the ownership has been taken by zip_open_from_source, zip_file_add, or
    /// zip_file_replace, and therefore shouldn't be freed.
    pub(crate) fn taken(mut self) {
        self.handle = null_mut();
    }

    pub(crate) fn handle_mut(&mut self) -> *mut ffi::zip_source_t {
        self.handle
    }
}

impl<'a> TryFrom<&'a [u8]> for Source<&'a [u8]> {
    type Error = Error;

    fn try_from(buffer: &[u8]) -> Result<Source<&[u8]>> {
        let mut error = ZipErrorT::default();
        let handle = unsafe {
            ffi::zip_source_buffer_create(buffer.as_ptr() as _, buffer.len() as _, 0, &mut *error)
        };
        if handle.is_null() {
            Err(error.into())
        } else {
            Ok(Source {
                handle,
                phantom: PhantomData,
            })
        }
    }
}

impl TryFrom<&CStr> for Source<File> {
    type Error = Error;

    fn try_from(filename: &CStr) -> Result<Source<File>> {
        let mut error = ZipErrorT::default();
        let handle = unsafe { ffi::zip_source_file_create(filename.as_ptr(), 0, 0, &mut *error) };
        if handle.is_null() {
            Err(error.into())
        } else {
            Ok(Source {
                handle,
                phantom: PhantomData,
            })
        }
    }
}

/// Open a zip file from a path.
/// This is less efficient than the &CStr variant, so that should be preferred when you can
/// construct a &CStr type directly or cache one.  If you would just be converting a path to a
/// cstring and then discarding it, this method might be preferable because that's all this does.
/// This will also panic if the Path contains any null bytes.
impl TryFrom<&Path> for Source<File> {
    type Error = Error;

    fn try_from(filename: &Path) -> Result<Source<File>> {
        let filename = CString::new(filename.to_string_lossy().into_owned())
            .expect("The path could not be converted into a CString");
        filename.as_ref().try_into()
    }
}

impl<T> Drop for Source<T> {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                ffi::zip_source_free(self.handle);
            }
        }
    }
}
