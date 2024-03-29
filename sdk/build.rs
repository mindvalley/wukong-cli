use std::env;

fn main() {
    let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    tonic_build::configure()
        .build_server(false)
        .out_dir("src/services/gcloud/api") // you can change the generated code's location
        .compile(
            &[
                format!(
                    "{}/proto/googleapis/google/logging/v2/logging.proto",
                    cargo_manifest_dir
                ),
                format!(
                    "{}/proto/googleapis/google/cloud/sql/v1/cloud_sql_instances.proto",
                    cargo_manifest_dir
                ),
                format!(
                    "{}/proto/googleapis/google/monitoring/v3/metric.proto",
                    cargo_manifest_dir
                ),
                format!(
                    "{}/proto/googleapis/google/monitoring/v3/metric_service.proto",
                    cargo_manifest_dir
                ),
            ],
            &[format!("{}/proto/googleapis", cargo_manifest_dir)], // specify the root location to search proto dependencies
        )
        .unwrap();
}
