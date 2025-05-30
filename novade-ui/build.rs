extern crate glib_build_tools;

fn main() {
    // Compile GResource file
    // The path to resources.xml is relative to the Cargo.toml directory (i.e., novade-ui/)
    glib_build_tools::compile_resources(
        &["src/gresources/"],    // Path to the directory containing resources.xml and resource files
        "src/gresources/resources.xml", // Path to the resources.xml manifest
        "novade_ui.gresource",          // Output gresource binary file name
        None                            // Optional: C source file for embedding (not needed for Rust)
    );
}
