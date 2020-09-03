use std::fs::File;
use std::env;
use std::path::PathBuf;
use std::io::prelude::*;

fn main() {
    println!("cargo:rustc-link-lib=zip");
    println!("cargo:rerun-if-changed=wrapper.h");
    if let Ok(_) = std::env::var("DOCS_RS") {
        let mut output = File::create(PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs")).unwrap();
        output.write_all(include_bytes!("docs_zip.rs")).unwrap();
    } else {
        let bindings = bindgen::Builder::default()
            .header("wrapper.h")
            .parse_callbacks(Box::new(bindgen::CargoCallbacks))
            .generate()
            .expect("Unable to generate bindings");

        let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
        bindings
            .write_to_file(out_path.join("bindings.rs"))
            .expect("Couldn't write bindings!");
    }
}
