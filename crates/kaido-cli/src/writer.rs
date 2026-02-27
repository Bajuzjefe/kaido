use std::fs;
use std::path::{Path, PathBuf};

use kaido_core::generator::RenderResult;

/// Write a RenderResult to disk, returning the list of created file paths
pub fn write_project(result: &RenderResult, output_dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut paths = Vec::new();

    for file in &result.files {
        let full_path = output_dir.join(&file.path);

        // Create parent directories
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&full_path, &file.content)?;
        paths.push(full_path);
    }

    Ok(paths)
}
