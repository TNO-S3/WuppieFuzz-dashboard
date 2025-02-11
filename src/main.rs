use include_dir::{include_dir, Dir};
use std::env;
use std::path::Path;

// Custom module imports
pub mod docker_util;
pub mod embed_files;

// Global variables
pub const CONTAINER_NAME: &str = "grafana-dashboard";
pub const GRAFANA_INI: &str = include_str!("../grafana.ini");
pub static PROVISIONING_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/provisioning");

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage:");
        eprintln!("  {} start <path_to_report.db>", args[0]);
        eprintln!("  {} stop", args[0]);
        std::process::exit(1);
    }

    let command = args[1].as_str();

    let docker = docker_util::connect_docker();

    match command {
        "start" => {
            if args.len() < 3 {
                eprintln!("Error: Missing path to report.db");
                std::process::exit(1);
            }
            let report_db_path = Path::new(&args[2]);
            docker_util::start_container(
                &docker,
                &CONTAINER_NAME,
                report_db_path,
                &PROVISIONING_DIR,
                &GRAFANA_INI,
            )
            .await;
        }
        "stop" => docker_util::stop_and_remove_container(&docker, &CONTAINER_NAME).await,
        _ => {
            eprintln!("Invalid command. Use 'start' or 'stop'.");
            std::process::exit(1);
        }
    }
}
