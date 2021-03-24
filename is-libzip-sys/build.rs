
fn main () {
    println!("cargo:rustc-link-lib=zlib");
    println!("cargo:rustc-link-lib=zip");

    println!("cargo:rustc-link-search={}", env!("CARGO_MANIFEST_DIR"));

}