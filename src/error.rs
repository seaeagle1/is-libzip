use crate::ffi;
use std::mem::zeroed;
use std::os::raw::c_int;
use std::ffi::CStr;
use std::ops::{Deref, DerefMut};
use std::borrow::{Borrow, BorrowMut};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum System {
    Sys(c_int),
    Zlib(c_int),
    Unknown(c_int),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Zip {
    Changed,
    Close,
    CompressionNotSupported,
    CompressedDataInvalid,
    Crc,
    Deleted,
    EncryptionNotSupported,
    Eof,
    Exists,
    Inconsistent,
    Internal,
    InUse,
    InvalidArgument,
    Memory,
    Multidisk,
    NoSuchFile,
    NoPassword,
    NotZip,
    Open,
    OperationNotSupported,
    ReadOnly,
    Read,
    Remove,
    Rename,
    Seek,
    Tell,
    TempFile,
    Write,
    WrongPassword,
    ZipClosed,
    Zlib,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Error {
    system: Option<System>,
    zip: Option<Zip>,
    message: String,
}

pub type Result<T> = std::result::Result<T, Error>;

/// A containment structure for a zip_error_t.
/// This can be either Default instantiated for an owned version or borrowed with its From
/// implementation.
#[derive(Debug)]
pub(crate) struct ZipErrorT<T: Borrow<ffi::zip_error_t> + BorrowMut<ffi::zip_error_t>>{
    error: T,
    cleanup: bool,
}

impl Default for ZipErrorT<ffi::zip_error_t> {
    fn default() -> Self {
        unsafe {
            let mut handle: ffi::zip_error_t = zeroed();
            ffi::zip_error_init(&mut handle);
            ZipErrorT{
                error: handle,
                cleanup: true,
            }
        }
    }
}

impl<'a> From<&'a mut ffi::zip_error_t> for ZipErrorT<&'a mut ffi::zip_error_t> {
    fn from(error: &'a mut ffi::zip_error_t) -> Self {
        ZipErrorT{
            error,
            cleanup: false,
        }
    }
}

impl<T> ZipErrorT<T> 
where T: Borrow<ffi::zip_error_t> + BorrowMut<ffi::zip_error_t> {
    pub fn system(&self) -> Option<System> {
        let system = unsafe {
            ffi::zip_error_system_type(self.deref())
        };
        match system as _ {
            ffi::ZIP_ET_NONE => None,
            _ => {
                let code = unsafe {
                    ffi::zip_error_code_system(self.deref())
                };
                Some(match system as _ {
                    ffi::ZIP_ET_SYS => System::Sys(code),
                    ffi::ZIP_ET_ZLIB => System::Zlib(code),
                    _ => System::Unknown(code),
                })
            }
        }
    }
    pub fn zip(&self) -> Option<Zip> {
        let code = unsafe {
            ffi::zip_error_code_zip(self.deref())
        };
        Some(match code as _ {
            ffi::ZIP_ER_CHANGED               => Zip::Changed,
            ffi::ZIP_ER_CLOSE                 => Zip::Close,
            ffi::ZIP_ER_COMPNOTSUPP           => Zip::CompressionNotSupported,
            ffi::ZIP_ER_COMPRESSED_DATA       => Zip::CompressedDataInvalid,
            ffi::ZIP_ER_CRC                   => Zip::Crc,
            ffi::ZIP_ER_DELETED               => Zip::Deleted,
            ffi::ZIP_ER_ENCRNOTSUPP           => Zip::EncryptionNotSupported,
            ffi::ZIP_ER_EOF                   => Zip::Eof,
            ffi::ZIP_ER_EXISTS                => Zip::Exists,
            ffi::ZIP_ER_INCONS                => Zip::Inconsistent,
            ffi::ZIP_ER_INTERNAL              => Zip::Internal,
            ffi::ZIP_ER_INUSE                 => Zip::InUse,
            ffi::ZIP_ER_INVAL                 => Zip::InvalidArgument,
            ffi::ZIP_ER_MEMORY                => Zip::Memory,
            ffi::ZIP_ER_MULTIDISK             => Zip::Multidisk,
            ffi::ZIP_ER_NOENT                 => Zip::NoSuchFile,
            ffi::ZIP_ER_NOPASSWD              => Zip::NoPassword,
            ffi::ZIP_ER_NOZIP                 => Zip::NotZip,
            ffi::ZIP_ER_OK                    => return None,
            ffi::ZIP_ER_OPEN                  => Zip::Open,
            ffi::ZIP_ER_OPNOTSUPP             => Zip::OperationNotSupported,
            ffi::ZIP_ER_RDONLY                => Zip::ReadOnly,
            ffi::ZIP_ER_READ                  => Zip::Read,
            ffi::ZIP_ER_REMOVE                => Zip::Remove,
            ffi::ZIP_ER_RENAME                => Zip::Rename,
            ffi::ZIP_ER_SEEK                  => Zip::Seek,
            ffi::ZIP_ER_TELL                  => Zip::Tell,
            ffi::ZIP_ER_TMPOPEN               => Zip::TempFile,
            ffi::ZIP_ER_WRITE                 => Zip::Write,
            ffi::ZIP_ER_WRONGPASSWD           => Zip::WrongPassword,
            ffi::ZIP_ER_ZIPCLOSED             => Zip::ZipClosed,
            ffi::ZIP_ER_ZLIB                  => Zip::Zlib,
            _ => Zip::Unknown,
        })
    }

    pub fn message(&mut self) -> &CStr {
        unsafe {
            let message = ffi::zip_error_strerror(self.deref_mut());
            assert!(!message.is_null());
            CStr::from_ptr(message)
        }
    }
}

impl<T> Drop for ZipErrorT<T> 
 where T: Borrow<ffi::zip_error_t> + BorrowMut<ffi::zip_error_t> {
    fn drop(&mut self) {
        if self.cleanup {
            unsafe {
                ffi::zip_error_fini(self.error.borrow_mut())
            }
        }
    }
}

impl<T> Deref for ZipErrorT<T> 
 where T: Borrow<ffi::zip_error_t> + BorrowMut<ffi::zip_error_t> {
    type Target = ffi::zip_error_t;
    fn deref(&self) -> &Self::Target {
        self.error.borrow()
    }
}

impl<T> DerefMut for ZipErrorT<T> 
 where T: Borrow<ffi::zip_error_t> + BorrowMut<ffi::zip_error_t> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.error.borrow_mut()
    }
}

impl<T> From<ZipErrorT<T>> for Error 
 where T: Borrow<ffi::zip_error_t> + BorrowMut<ffi::zip_error_t> {
    fn from(mut error: ZipErrorT<T>) -> Self {
        let system = error.system();
        let zip = error.zip();
        let message = error.message().to_string_lossy().into_owned();
        Error {
            system,
            zip,
            message,
        }
    }
}
