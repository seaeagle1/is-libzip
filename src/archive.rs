use crate::error::ZipErrorT;
use crate::ffi;
use crate::file::{Encoding, File, LocateFlag, OpenFlag as FileOpenFlag, Encryption};
use crate::source::Source;
use crate::Error;
use crate::Result;
use std::ffi::CStr;
use std::marker::PhantomData;
use std::ptr::null_mut;
use std::ptr;

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
    where
        F: AsRef<[OpenFlag]>,
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
            let handle =
                ffi::zip_open_from_source(source.handle_mut(), flags_value as _, &mut *error);

            if handle.is_null() {
                Err(error.into())
            } else {
                source.taken();
                Ok(Archive { handle })
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
            let result = unsafe { ffi::zip_close(self.handle) };
            if result == 0 {
                self.handle = null_mut();
                Ok(())
            } else {
                Err(self.error().into())
            }
        }
    }

    /// Closes and consumes a zip file.
    /// If this fails, the failed-to-close zipfile and an error will be returned.
    pub fn close(mut self) -> std::result::Result<(), (Self, Error)> {
        match self.close_mut() {
            Ok(()) => Ok(()),
            Err(e) => Err((self, e)),
        }
    }

    /// Internal non-consuming discard, to facilitate drop
    fn discard_mut(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                ffi::zip_discard(self.handle);
            }
        }
    }

    /// Discard and consume the zipfile.  This will never fail.
    pub fn discard(mut self) {
        self.discard_mut()
    }

    /// Add a file to the zip archive.
    /// Returns the index of the new file.
    pub fn add<N, S>(
        &mut self,
        name: N,
        mut source: Source<S>,
        encoding: Encoding,
        overwrite: bool,
    ) -> Result<u64>
    where
        N: AsRef<CStr>,
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
            ffi::zip_file_add(
                self.handle,
                name.as_ref().as_ptr(),
                source.handle_mut(),
                flags as _,
            )
        };
        if response == -1 {
            Err(self.error().into())
        } else {
            source.taken();
            Ok(response as _)
        }
    }

    /// Replace a file in the zip archive.
    pub fn replace<S>(&mut self, index: u64, mut source: Source<S>) -> Result<()> {
        let response =
            unsafe { ffi::zip_file_replace(self.handle, index as _, source.handle_mut(), 0) };
        if response == -1 {
            Err(self.error().into())
        } else {
            source.taken();
            Ok(())
        }
    }

    // Set encryption flag for a file.
    pub fn set_encryption_on_file(&mut self, encryption: Encryption, file_index: u64 ) -> Result<()> {
        let mode = match encryption {
            Encryption::None => ffi::ZIP_EM_NONE,
            Encryption::AES128 => ffi::ZIP_EM_AES_128,
            Encryption::AES192 => ffi::ZIP_EM_AES_192,
            Encryption::AES256 => ffi::ZIP_EM_AES_256,
            Encryption::PkWare => ffi::ZIP_EM_TRAD_PKWARE,
        };
        let response = unsafe {
            ffi::zip_file_set_encryption(
                self.handle,
                file_index,
                mode as _,
                ptr::null::<i8>(),
            )
        };
        if response == -1 {
            Err(self.error().into())
        } else {
            Ok(())
        }
    }

    // set archive default encryption password
    pub fn set_encryption_password<N>(&mut self, password: N) -> Result<()>
        where N: AsRef<CStr>
    {
        let response = unsafe {
            ffi::zip_set_default_password(
                self.handle,
                password.as_ref().as_ptr()
            )
        };
        if response == -1 {
            Err(self.error().into())
        } else {
            Ok(())
        }
    }


    /// Add a file to the zip archive.
    /// Returns the index of the new file.
    pub fn open_file<N, O, L>(
        &mut self,
        name: N,
        open_flags: O,
        locate_flags: L,
    ) -> Result<File<'_>>
    where
        N: AsRef<CStr>,
        O: AsRef<[FileOpenFlag]>,
        L: AsRef<[LocateFlag]>,
    {
        let mut flags_value = 0;
        for flag in open_flags.as_ref() {
            match flag {
                FileOpenFlag::Compressed => flags_value |= ffi::ZIP_FL_COMPRESSED,
                FileOpenFlag::Unchanged => flags_value |= ffi::ZIP_FL_UNCHANGED,
            }
        }
        for flag in locate_flags.as_ref() {
            match flag {
                LocateFlag::NoCase => flags_value |= ffi::ZIP_FL_NOCASE,
                LocateFlag::NoDir => flags_value |= ffi::ZIP_FL_NODIR,
                LocateFlag::EncodingRaw => flags_value |= ffi::ZIP_FL_ENC_RAW,
                LocateFlag::EncodingGuess => flags_value |= ffi::ZIP_FL_ENC_GUESS,
                LocateFlag::EncodingStrict => flags_value |= ffi::ZIP_FL_ENC_STRICT,
            }
        }
        let handle =
            unsafe { ffi::zip_fopen(self.handle, name.as_ref().as_ptr(), flags_value as _) };
        if handle.is_null() {
            Err(self.error().into())
        } else {
            Ok(File {
                handle,
                phantom: PhantomData,
            })
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
