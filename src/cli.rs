use std::{env, process::Command};
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Args, Debug)]
pub struct StartArguments {
    #[arg(short, long, value_name = "report.db")]
    pub database: PathBuf,
}


#[derive(Subcommand, Debug)]
pub enum Commands {
    License,
    Version,
    Start(StartArguments),
    Stop,
}

// CLI argument parser
#[derive(Parser)]
#[command(version, about, long_about=None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

pub fn get_dashboard_version() -> String {
    let git_output = Command::new("git").arg("rev-parse").arg("HEAD").output();
    let git_hash = match git_output {
        Ok(v) => {
            if v.stdout.is_empty() {
                "".to_string()
            } else {
                "-".to_string().clone()
                    + &String::from_utf8(v.stdout)
                        .clone()
                        .unwrap_or("Invalid UTF8 output".to_string())
            }
        }
        Err(_) => "<could not get git hash>".to_string(),
    };

    clap::crate_version!().to_string() + &git_hash
}

pub fn print_version() {
    println!("WuppieFuzz Dashboard version: {}", get_dashboard_version())
}

pub fn print_license() {
    print_version();
    println!(
        "===============================================================================\
    \n                                LICENSE NOTICE\n\
    ===============================================================================\
    \n{}\n\
    ===============================================================================",
        include_str!("../LICENSE"),
    )
}