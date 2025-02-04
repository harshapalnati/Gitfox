use std::fs;

fn main() {
    // Create the generated directory if it doesn't exist
    fs::create_dir_all("src/generated").unwrap();

    println!("cargo:rerun-if-changed=proto/ai_review.proto");

    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .out_dir("src/generated")
        .compile(&["proto/ai_review.proto"], &["proto"])
        .unwrap_or_else(|e| {
            eprintln!("Failed to compile protos: {}", e);
            std::process::exit(1);
        });
}
