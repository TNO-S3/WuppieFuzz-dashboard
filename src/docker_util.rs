use crate::embed_files;
use dockworker::container::ContainerFilters;
use dockworker::{ContainerCreateOptions, ContainerHostConfig, Docker, PortBindings};
use include_dir::Dir;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

pub fn connect_docker() -> Docker {
    Docker::connect_with_defaults().expect("[-] Failed to connect to Docker")
}

/// Check if a Docker image exists locally
async fn image_exists(docker: &Docker, image_name: &str) -> bool {
    match docker.images(false).await {
        Ok(images) => images
            .iter()
            .any(|img| img.RepoTags.iter().any(|tag| tag.contains(image_name))),
        Err(_) => false,
    }
}

/// Build the custom Grafana image if it doesn't exist
async fn build_grafana_image_if_needed(docker: &Docker, image_name: &str) {
    if image_exists(docker, image_name).await {
        println!("[*] Using cached Grafana image: {}", image_name);
        return;
    }

    println!("[*] Building custom Grafana image...");
    let output = Command::new("docker")
        .args(["build", "-t", image_name, "docker/grafana"])
        .output()
        .expect("[-] Failed to execute docker build");

    if !output.status.success() {
        eprintln!("[-] Error building image:");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        std::process::exit(1);
    }

    println!("[*] Grafana image built successfully: {}", image_name);
}

/// Start the Grafana container
pub async fn start_container<'a>(
    docker: &Docker,
    container_name: &str,
    report_db_path: &Path,
    embedded_dir: &Dir<'a>,
    embedded_file: &str,
) {
    if !report_db_path.exists() {
        eprintln!("[*] Error: The specified report.db path does not exist.");
        std::process::exit(1);
    }

    if is_container_running(docker, container_name).await {
        println!("[*] Dashboard is already running.");
        return;
    }

    // Remove a container if it is dead
    if is_container_dead(docker, container_name).await {
        println!("[*] Grafana container is dead.");

        let container_id = get_dead_container_id(docker, container_name).await.unwrap();

        remove_container(docker, container_id).await;
    }

    // Extract embedded files from the binary and write them to a temp directory
    let temp_dir = embed_files::setup_embedded_files(embedded_dir, embedded_file);

    let mut host_config = ContainerHostConfig::new();

    // Set up port bindings
    let port_bindings = PortBindings(vec![(3000, "tcp".to_string(), 3000)]);
    host_config.port_bindings(port_bindings);

    // Set up volume bindings from the temp directory
    let binds = vec![
        format!(
            "{}:/home/grafana/report.db",
            report_db_path.to_str().unwrap().to_string()
        ),
        format!(
            "{}:/var/lib/grafana/dashboards",
            temp_dir.join("provisioning/dashboards").to_str().unwrap()
        ),
        format!(
            "{}:/etc/grafana/grafana.ini",
            temp_dir.join("grafana.ini").to_str().unwrap()
        ),
        format!(
            "{}:/etc/grafana/provisioning",
            temp_dir.join("provisioning").to_str().unwrap()
        ),
    ];
    host_config.binds(binds);

    // Build the custom Grafana image if it doesn't exist
    let image_name = "wuppiefuzz-grafana:latest";
    build_grafana_image_if_needed(docker, image_name).await;

    // Create the container
    let mut create_options = ContainerCreateOptions::new(image_name);
    create_options
        .hostname(container_name.to_string())
        .host_config(host_config);

    let container_id_result = docker
        .create_container(Some(container_name), &create_options)
        .await;

    let container_id = match container_id_result {
        Ok(result) => result.id,
        Err(e) => {
            println!("[-] Error: {:?}\n\tIs the Docker engine not running?", e);
            panic!("")
        }
    };

    // Start the container
    docker.start_container(&container_id).await.unwrap();

    println!(
        "[*] Grafana container started successfully. It can be accessed at http://localhost:3000"
    );
}

pub async fn remove_container(docker: &Docker, container_id: String) {
    println!("[*] Removing Grafana container...");
    docker
        .remove_container(&container_id, None, None, None)
        .await
        .unwrap();
    println!("[*] Grafana container removed.");
}

// Stop and remove the Grafana container
pub async fn stop_and_remove_container(docker: &Docker, container_name: &str) {
    if let Some(container_id) = get_running_container_id(docker, container_name).await {
        println!("[*] Stopping Grafana container...");
        docker
            .stop_container(&container_id, Duration::new(5, 0))
            .await
            .unwrap();

        remove_container(docker, container_id).await;
    } else {
        println!("[-] Grafana container is not running.");
    }
}

// Check if the Grafana container is running
pub async fn is_container_running(docker: &Docker, container_name: &str) -> bool {
    get_running_container_id(docker, container_name)
        .await
        .is_some()
}

pub async fn is_container_dead(docker: &Docker, container_name: &str) -> bool {
    get_dead_container_id(docker, container_name)
        .await
        .is_some()
}

// Get the container ID of the running Grafana container
pub async fn get_running_container_id(docker: &Docker, container_name: &str) -> Option<String> {
    let mut filters = ContainerFilters::new();
    filters.status(dockworker::container::ContainerStatus::Running);

    docker
        .list_containers(None, None, None, filters)
        .await
        .unwrap()
        .iter()
        .find(|&c| c.Names.contains(&format!("/{}", container_name)))
        .map(|c| c.Id.clone())
}

// Get the container ID of any dead Grafana containers
pub async fn get_dead_container_id(docker: &Docker, container_name: &str) -> Option<String> {
    let mut filters = ContainerFilters::new();
    filters.status(dockworker::container::ContainerStatus::Exited);
    filters.status(dockworker::container::ContainerStatus::Dead);

    docker
        .list_containers(None, None, None, filters)
        .await
        .unwrap()
        .iter()
        .find(|&c| c.Names.contains(&format!("/{}", container_name)))
        .map(|c| c.Id.clone())
}
