use crate::file::Encoding;
use crate::Result;
use crate::Error;
use crate::ffi;
use std::ffi::CStr;
use std::ptr::null_mut;
use crate::source::Source;
use crate::error::ZipErrorT;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum OpenFlag {
    CheckConsistency,
    Create,
    Exclusive,
    Truncate,
    ReadOnly,
}

#[derive(Debug)]
pub struct Archive {
    handle: *mut ffi::zip_t,
}

impl Archive {
    pub fn open<S, F>(mut source: Source<S>, flags: F) -> Result<Archive>
        where F: AsRef<[OpenFlag]> 
    {
        let mut flags_value = 0;
        for flag in flags.as_ref() {
            match flag {
                OpenFlag::CheckConsistency => flags_value |= ffi::ZIP_CHECKCONS,
                OpenFlag::Create => flags_value |= ffi::ZIP_CREATE,
                OpenFlag::Exclusive => flags_value |= ffi::ZIP_EXCL,
                OpenFlag::Truncate => flags_value |= ffi::ZIP_TRUNCATE,
                OpenFlag::ReadOnly => flags_value |= ffi::ZIP_RDONLY,
            }
        }

        unsafe {
            let mut error = ZipErrorT::default();
            let handle = ffi::zip_open_from_source(source.handle_mut(), flags_value as _, &mut *error);

            if handle.is_null() {
                Err(error.into())
            } else {
                source.taken();
                Ok(Archive {
                    handle,
                })
            }
        }
    }

    fn error(&mut self) -> ZipErrorT<&mut ffi::zip_error_t> {
        unsafe {
            let error = ffi::zip_get_error(self.handle);
            (&mut *error).into()
        }
    }

    /// Closes and consumes a zip file.  If this fails, an error and the failed-to-close zipfile
    /// will be returned
    fn close_mut(&mut self) -> Result<()> {
        if self.handle.is_null() {
            Ok(())
        } else {
            let result = unsafe {
                ffi::zip_close(self.handle)
            };
            if result == 0 {
                self.handle = null_mut();
                Ok(())
            } else {
                Err(self.error().into())
            }
        }
    }

    /// Closes and consumes a zip file.  If this fails, an error and the failed-to-close zipfile
    /// will be returned
    pub fn close(mut self) -> std::result::Result<(), (Self, Error)> {
        match self.close_mut() {
            Ok(()) => Ok(()),
            Err(e) => Err((self, e)),
        }
    }

    fn discard_mut(&mut self) {
        unsafe {
            ffi::zip_discard(self.handle)
        }
    }

    /// Discard and consume the zipfile.  This will never fail.
    pub fn discard(mut self) {
        self.discard_mut()
    }

    /// Add a file to the zip archive.
    /// Returns the index of the new file.
    pub fn add<N, S>(&mut self, name: N, mut source: Source<S>, encoding: Encoding, overwrite: bool) -> Result<u64> 
        where N: AsRef<CStr>
    {
        let mut flags = match encoding {
            Encoding::Guess => ffi::ZIP_FL_ENC_GUESS,
            Encoding::Utf8 => ffi::ZIP_FL_ENC_UTF_8,
            Encoding::Cp437 => ffi::ZIP_FL_ENC_CP437,
        };
        if overwrite {
            flags |= ffi::ZIP_FL_OVERWRITE;
        }

        let response = unsafe {
            ffi::zip_file_add(self.handle, name.as_ref().as_ptr(), source.handle_mut(), flags as _)
        };
        if response == -1 {
            Err(self.error().into())
        } else {
            source.taken();
            Ok(response as _)
        }
    }

    /// Replace a file in the zip archive.
    pub fn replace<S>(&mut self, index: u64, mut source: Source<S>) -> Result<()> 
    {
        let response = unsafe {
            ffi::zip_file_replace(self.handle, index as _, source.handle_mut(), 0)
        };
        if response == -1 {
            Err(self.error().into())
        } else {
            source.taken();
            Ok(())
        }
    }
}

/// Closes the archive, silently discarding on error.
/// It's strongly recommended to use the [Archive::close] method instead and validate that no
/// errors have occurred.
impl Drop for Archive {
    fn drop(&mut self) {
        if let Err(_) = self.close_mut() {
            self.discard_mut()
        }
    }
}
