fn main() {
    #[cfg(feature = "c-api")]
    {
        let crate_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let output_file = std::path::Path::new(&crate_dir)
            .join("include")
            .join("jsonrepair.h");

        // Create include directory if it doesn't exist
        std::fs::create_dir_all(output_file.parent().unwrap()).unwrap();

        cbindgen::Builder::new()
            .with_crate(crate_dir)
            .with_config(cbindgen::Config::from_file("cbindgen.toml").unwrap())
            .generate()
            .expect("Unable to generate C bindings")
            .write_to_file(&output_file);

        println!("cargo:rerun-if-changed=src/ffi.rs");
        println!("cargo:rerun-if-changed=cbindgen.toml");
    }
}
