use std::fs;
use std::path::Path;
use std::sync::Arc;

use anyhow::{bail, ensure, Context};
use cairo_lang_defs::ids::ModuleId;
use cairo_lang_filesystem::db::{
    CrateConfiguration, CrateSettings, Edition, ExperimentalFeaturesConfig, FilesGroupEx,
};
use cairo_lang_filesystem::ids::{CrateId, CrateLongId, Directory};
use cairo_lang_semantic::db::SemanticGroup;
use scarb_metadata::{Metadata, PackageMetadata};
use smol_str::SmolStr;
use tracing::warn;

pub fn update_crate_roots(metadata: &Metadata, db: &mut dyn SemanticGroup) {
    for compilation_unit in &metadata.compilation_units {
        for component in &compilation_unit.components {
            let crate_name: SmolStr = component.name.as_str().into();

            let package = metadata.packages.iter().find(|package| package.id == component.package);
            if package.is_none() {
                warn!("package for component is missing in Scarb metadata: {crate_name}");
            }

            let (root, file_stem) = match validate_and_chop_source_path(
                component.source_path.as_std_path(),
                &crate_name,
            ) {
                Ok(t) => t,
                Err(e) => {
                    warn!("{e:?}");
                    continue;
                }
            };

            let settings = CrateSettings {
                edition: scarb_package_edition(&package, &crate_name),
                cfg_set: scarb_cfg_set_to_cairo(
                    &component.cfg.as_ref().unwrap_or(&compilation_unit.cfg),
                    &crate_name,
                ),
                experimental_features: scarb_package_experimental_features(&package),
            };

            let crate_id = db.intern_crate(CrateLongId::Real(crate_name));
            db.set_crate_config(
                crate_id,
                Some(CrateConfiguration { root: Directory::Real(root.into()), settings }),
            );

            if file_stem != "lib" {
                inject_virtual_wrapper_lib(db, crate_id, &file_stem);
            }
        }
    }
}

/// Perform sanity checks on crate _source path_, and chop it into directory path and file stem.
fn validate_and_chop_source_path<'a>(
    source_path: &'a Path,
    crate_name: &str,
) -> anyhow::Result<(&'a Path, &'a str)> {
    let metadata = fs::metadata(source_path)
        .with_context(|| format!("io error when accessing source path of: {crate_name}"))?;

    ensure!(
        !metadata.is_dir(),
        "source path of component `{crate_name}` must not be a directory: {source_path}",
        source_path = source_path.display()
    );

    let Some(root) = source_path.parent() else {
        bail!(
            "unexpected fs root as a source path of component `{crate_name}`: {source_path}",
            source_path = source_path.display()
        );
    };

    let Some(file_stem) = source_path.file_stem() else {
        bail!(
            "failed to get file stem for component `{crate_name}`: {source_path}",
            source_path = source_path.display()
        );
    };

    let Some(file_stem) = file_stem.to_str() else {
        bail!("file stem is not utf-8: {source_path}", source_path = source_path.display());
    };

    Ok((root, file_stem))
}

/// Get the [`Edition`] from [`PackageMetadata`], or assume the default edition.
fn scarb_package_edition(package: &Option<&PackageMetadata>, crate_name: &str) -> Edition {
    package
        .and_then(|p| p.edition.clone())
        .and_then(|e| {
            serde_json::from_value(e.into())
                .with_context(|| format!("failed to parse edition of package: {crate_name}"))
                .inspect_err(|e| warn!("{e:?}"))
                .ok()
        })
        .unwrap_or_default()
}

/// Convert a slice of [`scarb_metadata::Cfg`]s to a [`cairo_lang_filesystem::cfg::CfgSet`].
///
/// The conversion is done the same way as in Scarb (but we do not panic here):
/// https://github.com/software-mansion/scarb/blob/9fe97c8eb8620a1e2103e7f5251c5a9189e75716/scarb/src/ops/metadata.rs#L295-L302
fn scarb_cfg_set_to_cairo(
    cfg_set: &[scarb_metadata::Cfg],
    crate_name: &str,
) -> Option<cairo_lang_filesystem::cfg::CfgSet> {
    serde_json::to_value(cfg_set)
        .and_then(serde_json::from_value)
        .inspect_err(|e| {
            warn!(
                "Scarb metadata Cfg did not convert identically to Cairo one for crate: \
                 {crate_name}"
            )
        })
        .ok()
}

/// Get [`ExperimentalFeaturesConfig`] from [`PackageMetadata`] fields.
fn scarb_package_experimental_features(
    package: &Option<&PackageMetadata>,
) -> ExperimentalFeaturesConfig {
    let contains = |feature: &str| -> bool {
        let Some(package) = package else { return false };
        package.experimental_features.iter().any(|f| f == feature)
    };

    ExperimentalFeaturesConfig {
        negative_impls: contains("negative_impls"),
        coupons: contains("coupons"),
    }
}

/// Generate a wrapper lib file for a compilation unit without a root `lib.cairo`.
///
/// This approach allows compiling crates that do not define `lib.cairo` file.
/// For example, single file crates can be created this way.
/// The actual single file module is defined as `mod` item in created lib file.
fn inject_virtual_wrapper_lib(db: &mut dyn SemanticGroup, crate_id: CrateId, file_stem: &str) {
    let module_id = ModuleId::CrateRoot(crate_id);
    let file_id = db.module_main_file(module_id).unwrap();
    // Inject virtual lib file wrapper.
    db.as_files_group_mut()
        .override_file_content(file_id, Some(Arc::new(format!("mod {file_stem};"))));
}
