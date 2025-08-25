use include_dir::{include_dir, Dir};
use std::path::{Path, absolute};
use clap::Parser;

// Custom module imports
pub mod docker_util;
pub mod embed_files;
pub mod cli;

// Global variables
pub const CONTAINER_NAME: &str = "grafana-dashboard";
pub const GRAFANA_INI: &str = include_str!("../grafana.ini");
pub static PROVISIONING_DIR: Dir = include_dir!("$CARGO_MANIFEST_DIR/provisioning");


#[tokio::main]
async fn main() {
    let args = cli::Cli::parse();

    println!("\nWuppieFuzz dashboard");
    println!("--------------------\n");

    let docker = docker_util::connect_docker();

    match &args.command {
        cli::Commands::License => cli::print_license(),
        cli::Commands::Version => cli::print_version(),
        cli::Commands::Start(cmd_args) => {

            // Convert the relative path to an absolute path
            let absolute_report_db_path = match absolute(&cmd_args.database) {
                Ok(path) => path,
                Err(err) => panic!("{}", err),
            };

            let report_db_path = Path::new(&absolute_report_db_path);
            docker_util::start_container(
                &docker,
                &CONTAINER_NAME,
                report_db_path,
                &PROVISIONING_DIR,
                &GRAFANA_INI,
            )
            .await;
        },
        &cli::Commands::Stop => docker_util::stop_and_remove_container(&docker, &CONTAINER_NAME).await,
    };
}
