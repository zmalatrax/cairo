use std::path::{Path, PathBuf};
use std::{fmt, fs};

use cairo_lang_project::PROJECT_FILE_NAME;

const MAX_CRATE_DETECTION_DEPTH: usize = 20;
const SCARB_TOML: &str = "Scarb.toml";

/// An absolute path to a manifest file of a single Cairo project.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum ProjectManifestPath {
    /// `cairo_project.toml` file.
    CairoProject(PathBuf),

    /// `Scarb.toml` file.
    ///
    /// This could either be a single package or a workspace manifest.
    Scarb(PathBuf),
}

impl ProjectManifestPath {
    /// Look for a project manifest that **can** include source files at the given path.
    pub fn discover(path: &Path) -> Option<ProjectManifestPath> {
        return find_in_parent_dirs(path.to_path_buf(), PROJECT_FILE_NAME)
            .map(ProjectManifestPath::CairoProject)
            .or_else(|| {
                find_in_parent_dirs(path.to_path_buf(), SCARB_TOML).map(ProjectManifestPath::Scarb)
            });

        fn find_in_parent_dirs(mut path: PathBuf, target_file_name: &str) -> Option<PathBuf> {
            for _ in 0..MAX_CRATE_DETECTION_DEPTH {
                if !path.pop() {
                    return None;
                }

                let manifest_path = path.join(target_file_name);
                if fs::metadata(&manifest_path).is_ok() {
                    return Some(manifest_path);
                };
            }
            None
        }
    }
}

impl fmt::Display for ProjectManifestPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProjectManifestPath::CairoProject(path) | ProjectManifestPath::Scarb(path) => {
                fmt::Display::fmt(path, f)
            }
        }
    }
}
