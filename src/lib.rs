pub mod error;
pub mod source;
pub mod archive;
pub mod file;

use error::Error;
use error::Result;

use axfive_libzip_sys as ffi;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
