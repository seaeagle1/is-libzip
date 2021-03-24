# is-libzip
Rust libzip wrapper, based on axfive-libzip.

## Usage
* make sure libzip.so/zip.lib and libz.so/z.lib are available in a linker path (static MS Visual Studio libs are provided in the package)
* add dependency to your Cargo.toml:
```
[dependencies]
is-libzip = { git = "https://github.com/seaeagle1/is-libzip.git", branch = "release" }
```
* Use Archive.open() to open/create a zip file!
