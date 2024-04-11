use std::path::{Path, PathBuf};

use anyhow::Result;
use cairo_lang_semantic::db::SemanticGroup;
use scarb_metadata::Metadata;
use tracing::{trace, warn};

use crate::project::Project;

mod update_crate_roots;

pub struct ScarbWorkspace {
    /// Path to the top-most Scarb manifest file (i.e., the **workspace** one).
    manifest_path: PathBuf,

    /// Last known revision of `scarb metadata` command output.
    metadata: Option<Metadata>,
}

impl ScarbWorkspace {
    pub fn load(manifest_path: &Path) -> Result<Self> {
        Ok(Self { manifest_path: manifest_path.to_path_buf(), metadata: None })
    }
}

impl Project for ScarbWorkspace {
    #[tracing::instrument(level = "trace", skip_all, fields(manifest = %self.manifest_path))]
    fn update_crate_roots(&self, db: &mut dyn SemanticGroup) {
        let Some(metadata) = self.metadata.as_ref() else {
            trace!("metadata has not been yet loaded, skipping");
            return;
        };

        update_crate_roots::update_crate_roots(metadata, db);
    }
}
