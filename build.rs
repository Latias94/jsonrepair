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

        // Fix cbindgen bug: remove leading space before function declarations
        // This is a workaround for cbindgen adding a leading space when cpp_compat = true
        let header_content = std::fs::read_to_string(&output_file).unwrap();
        let fixed_content = header_content
            .lines()
            .map(|line| {
                // Remove single leading space before type declarations
                // This fixes the cbindgen bug where function declarations get an extra space
                if line.starts_with(' ')
                    && !line.starts_with("  ")
                    && (line.contains("char *")
                        || line.contains("void ")
                        || line.contains("struct ")
                        || line.contains("int")
                        || line.contains("bool ")
                        || line.contains("uint")
                        || line.contains("size_t"))
                {
                    &line[1..]
                } else {
                    line
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"; // Add final newline
        std::fs::write(&output_file, fixed_content).unwrap();

        println!("cargo:rerun-if-changed=src/ffi.rs");
        println!("cargo:rerun-if-changed=cbindgen.toml");
    }
}
