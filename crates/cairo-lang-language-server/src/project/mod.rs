use anyhow::Result;
use cairo_lang_semantic::db::SemanticGroup;

pub use self::project_manifest_path::*;

mod project_manifest_path;
mod scarb;

pub type ProjectBox = Box<dyn Project>;

pub trait Project {
    fn update_crate_roots(&self, db: &mut dyn SemanticGroup);
}

pub fn load_project(manifest: &ProjectManifestPath) -> Result<ProjectBox> {
    match manifest {
        ProjectManifestPath::CairoProject(_) => todo!(),
        ProjectManifestPath::Scarb(manifest_path) => {
            Ok(Box::new(scarb::ScarbWorkspace::load(manifest_path)?))
        }
    }
}
