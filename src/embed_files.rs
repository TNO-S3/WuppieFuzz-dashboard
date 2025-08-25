use include_dir::Dir;
use std::fs;
use std::io::Write;
use std::path::Path;

// Writes data to a new file
pub fn write_embedded_file(path: &Path, content: &[u8]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("Failed to create directory");
    }
    let mut file = fs::File::create(path).expect("Failed to create file");
    file.write_all(content).expect("Failed to write file");
}

/// Extracts the embedded directory and its subdirectories into the target path
pub fn extract_embedded_dir(embedded_dir: &Dir, target_path: &Path) {
    for entry in embedded_dir.entries() {
        extract_entry(entry, target_path);
    }
}

/// Recursively processes each entry (file or directory)
pub fn extract_entry(entry: &include_dir::DirEntry, base_path: &Path) {
    let path = base_path.join(entry.path());

    if entry.as_dir().is_some() {
        fs::create_dir_all(&path).expect("Failed to create directory");
        if let Some(dir) = entry.as_dir() {
            for sub_entry in dir.entries() {
                extract_entry(sub_entry, base_path);
            }
        }
    } else if let Some(file) = entry.as_file() {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent directories");
        }
        let mut output_file = fs::File::create(&path).expect("Failed to create file");
        output_file
            .write_all(file.contents())
            .expect("Failed to write file");
    }
}

// Writes all embedded files to temp dir
pub fn setup_embedded_files(embedded_dir: &Dir, embedded_file: &str) -> std::path::PathBuf {
    let temp_dir: std::path::PathBuf = std::env::temp_dir().join("grafana_data");

    // Write files
    write_embedded_file(&temp_dir.join("grafana.ini"), embedded_file.as_bytes());
    extract_embedded_dir(embedded_dir, &temp_dir.join("provisioning"));

    println!("Files written to: {}", temp_dir.display());

    return temp_dir;
}
