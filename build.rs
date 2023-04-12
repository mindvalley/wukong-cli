fn main() {
    tonic_build::configure()
        .build_server(false)
        .out_dir("src/services/gcloud/api") // you can change the generated code's location
        .compile(
            &["proto/googleapis/google/logging/v2/logging.proto"],
            &["proto/googleapis"], // specify the root location to search proto dependencies
        )
        .unwrap();
}
