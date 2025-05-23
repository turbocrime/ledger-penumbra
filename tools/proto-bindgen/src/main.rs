extern crate bindgen;

use std::fs;
use std::path::Path;
use walkdir::WalkDir;

fn main() {
    // Print current directory for debugging
    println!("Current dir: {:?}", std::env::current_dir().unwrap());

    let protobuf_dir = Path::new("../../app/src/protobuf");
    let output_dir = Path::new("../../app/rust/src/protobuf_h");

    // Get absolute paths
    let protobuf_dir = protobuf_dir
        .canonicalize()
        .expect("Failed to resolve protobuf directory");
    let output_dir = output_dir
        .canonicalize()
        .expect("Failed to resolve output directory");

    // Print resolved paths for debugging
    println!("Resolved protobuf dir: {:?}", protobuf_dir);
    println!("Resolved output dir: {:?}", output_dir);

    // Check if directories exist
    if !protobuf_dir.exists() {
        eprintln!("Protobuf directory does not exist: {:?}", protobuf_dir);
        return;
    }

    // Ensure the output directory exists
    fs::create_dir_all(&output_dir).expect("Failed to create output directory");

    // Recursively walk through the protobuf directory
    for entry in WalkDir::new(&protobuf_dir) {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();

        // Skip directories
        if path.is_dir() {
            continue;
        }

        println!("Found: {:?}", path);

        // Check for .pb.h files
        if path.extension().and_then(|s| s.to_str()) == Some("h")
            && path
                .file_stem()
                .and_then(|s| s.to_str())
                .map(|s| s.ends_with(".pb"))
                .unwrap_or(false)
        {
            let file_name = path.file_stem().unwrap().to_str().unwrap();
            // Replace .pb with _pb in the output filename
            let file_name = file_name.replace(".pb", "_pb");
            let output_file = output_dir.join(format!("{}.rs", file_name));

            println!("Processing: {:?} -> {:?}", path, output_file);

            // Generate bindings using bindgen
            let bindings = bindgen::Builder::default()
                .header(path.to_str().unwrap())
                .clang_arg("-I../../app/src/nanopb_tiny") // Include path for nanopb
                .clang_arg("-I../../app/src/protobuf") // Include path for protobuf
                .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
                .allowlist_var(r"^([A-Za-z0-9_]+_tag.*|[A-Z0-9_]+)$") // Include defines with _tag or all caps
                .generate()
                .expect("Unable to generate bindings");

            // Write the bindings to the output file
            bindings
                .write_to_file(output_file)
                .expect("Couldn't write bindings!");
        }
    }
}
