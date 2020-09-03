use crate::Result;
use crate::Error;
use crate::ffi;
use crate::archive::Archive;
use std::ptr::null_mut;
use crate::error::ZipErrorT;
use std::marker::PhantomData;
use std::io;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Encoding {
    Guess,
    Utf8,
    Cp437,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum OpenFlag {
    Compressed,
    Unchanged,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum LocateFlag {
    NoCase,
    NoDir,
    EncodingRaw,
    EncodingGuess,
    EncodingStrict,
}

#[derive(Debug)]
pub struct File<'a> {
    pub(crate) handle: *mut ffi::zip_file_t,
    pub(crate) phantom: PhantomData<&'a Archive>,
}

impl File<'_> {
    fn error(&mut self) -> ZipErrorT<&mut ffi::zip_error_t> {
        unsafe {
            let error = ffi::zip_file_get_error(self.handle);
            (&mut *error).into()
        }
    }

    pub fn close(mut self) -> Result<()> {
        if self.handle.is_null() {
            Ok(())
        } else {
            let result = unsafe {
                ffi::zip_fclose(self.handle)
            };
            self.handle = null_mut();
            if result == 0 {
                Ok(())
            } else {
                let error: ZipErrorT<_> = result.into();
                Err(error.into())
            }
        }
    }
}
impl Drop for File<'_> {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                ffi::zip_fclose(self.handle);
            }
        }
    }
}

impl io::Read for File<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let result = unsafe {
            ffi::zip_fread(self.handle, buf.as_mut_ptr() as _, buf.len() as _)
        };
        if result == -1 {
            let error: Error = self.error().into();
            Err(io::Error::new(io::ErrorKind::Other, error))
        } else {
            Ok(result as _)
        }
    }
}

impl io::Seek for File<'_> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        let result = unsafe {
            match pos {
                io::SeekFrom::Start(pos) => ffi::zip_fseek(self.handle, pos as _, ffi::SEEK_SET as _),
                io::SeekFrom::End(pos) => ffi::zip_fseek(self.handle, pos as _, ffi::SEEK_END as _),
                io::SeekFrom::Current(pos) => ffi::zip_fseek(self.handle, pos as _, ffi::SEEK_CUR as _),
            }
        };
        if result == -1 {
            let error: Error = self.error().into();
            Err(io::Error::new(io::ErrorKind::Other, error))
        } else {
            unsafe {
                // Assume this will work, otherwise the fseek would have already failed.
                Ok(ffi::zip_ftell(self.handle) as _)
            }
        }
    }
}
