use std::env;
use std::path::PathBuf;

fn main() {
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .impl_debug(true)
        .derive_debug(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let output = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(output.join("bindings.rs"))
	.expect("Unable to write bindings");
}
