use std::env::{self, current_dir};

fn main() {
    let current_dir = current_dir().unwrap();
    // let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    tonic_build::configure()
        .build_server(false)
        .out_dir("src/services/gcloud/api") // you can change the generated code's location
        .compile(
            &[format!(
                "{}/proto/googleapis/google/logging/v2/logging.proto",
                current_dir.to_string_lossy()
            )],
            &[format!(
                "{}/proto/googleapis",
                current_dir.to_string_lossy()
            )], // specify the root location to search proto dependencies
        )
        .unwrap();
}
