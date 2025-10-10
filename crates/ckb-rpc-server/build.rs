use std::fs;
use std::path::Path;
use sysinfo::System;

fn main() {
    // Check if running tests with cargo test instead of nextest
    if is_running_tests() && !is_running_nextest() {
        panic!(
            "\n\n\
            ERROR: Tests must be run with 'cargo nextest run', not 'cargo test'.\n\
            \n\
            Install nextest: cargo install cargo-nextest\n\
            Run tests: cargo nextest run\n\
            Run specific package: cargo nextest run -p ckb-rpc-server\n\
            \n\
            "
        );
    }

    // Skip version increment if running under cargo test
    if is_running_tests() {
        return;
    }

    // Read the Cargo.toml file
    let cargo_toml_path = Path::new("Cargo.toml");
    let contents = fs::read_to_string(cargo_toml_path)
        .expect("Failed to read Cargo.toml");

    // Find the version line
    let mut lines: Vec<String> = contents.lines().map(|s| s.to_string()).collect();
    let mut version_updated = false;
    let mut old_version = String::new();
    let mut new_version = String::new();

    for (_i, line) in lines.iter_mut().enumerate() {
        if line.starts_with("version = ") {
            // Extract current version
            let version_str = line.trim_start_matches("version = \"").trim_end_matches("\"").to_string();
            let parts: Vec<&str> = version_str.split('.').collect();

            if parts.len() == 3 {
                // Parse the patch version
                if let Ok(patch) = parts[2].parse::<u32>() {
                    // Increment patch version
                    let new_patch = patch + 1;
                    old_version = version_str.clone();
                    new_version = format!("{}.{}.{}", parts[0], parts[1], new_patch);

                    // Update the line
                    *line = format!("version = \"{}\"", new_version);
                    version_updated = true;
                    break;
                }
            }
        }
    }

    // Write back to Cargo.toml if version was updated
    if version_updated {
        let new_contents = lines.join("\n");
        fs::write(cargo_toml_path, new_contents)
            .expect("Failed to write updated Cargo.toml");
        // Only shown with --verbose flag
            println!("Version updated from {} to {}", old_version, new_version);
    } else {
        println!("cargo:warning=Version line not found or could not be parsed");
    }
}

fn is_running_tests() -> bool {
    let mut sys = System::new();
    sys.refresh_all();

    // Get current process PID
    if let Ok(current_pid) = sysinfo::get_current_pid() {
        // Get current process
        if let Some(process) = sys.process(current_pid) {
            // Get parent PID
            if let Some(parent_pid) = process.parent() {
                // Get parent process
                if let Some(parent_process) = sys.process(parent_pid) {
                    // Check if parent command contains "cargo test"
                    let cmd = parent_process.cmd().join(" ");
                    if cmd.contains("cargo test") {
                        return true;
                    }
                }
            }
        }
    }

    false
}

fn is_running_nextest() -> bool {
    std::env::var("NEXTEST").is_ok()
}
