//! Resolve immutable local artifact receipts without provider calls or ancestor-project discovery.
use crate::{
    contracts::*,
    engine,
    error::{Error, Result},
    storage,
};
use std::{
    fs,
    path::{Path, PathBuf},
};
#[derive(Clone)]
struct Recorded {
    path: PathBuf,
    origin: AssetOrigin,
    quality: Quality,
}
pub struct Index {
    records: Vec<Recorded>,
}
impl Index {
    pub fn load(root: &Path) -> Result<Self> {
        let mut records = vec![];
        for run in engine::list(root)? {
            for job in run.jobs {
                for artifact in job.artifacts {
                    records.push(Recorded {
                        path: artifact.path,
                        quality: artifact.quality,
                        origin: AssetOrigin {
                            selection: job.input.selection.clone(),
                            settings: job.input.settings.clone(),
                            locale: job.input.locale.clone(),
                            sha256: artifact.sha256,
                            artifact: Some(artifact.id),
                        },
                    });
                }
            }
        }
        Ok(Self { records })
    }
    pub fn origin(
        &self,
        path: Option<&Path>,
        sha: &str,
        identity: bool,
    ) -> Result<Option<AssetOrigin>> {
        let exact = path.and_then(|p| fs::canonicalize(p).ok());
        let exact_matches = self
            .records
            .iter()
            .filter(|r| {
                exact
                    .as_ref()
                    .is_some_and(|p| fs::canonicalize(&r.path).is_ok_and(|recorded| &recorded == p))
            })
            .collect::<Vec<_>>();
        if exact_matches.iter().any(|r| r.origin.sha256 != sha) {
            return Err(Error::input(
                "Reference bytes changed after their original generation record; select a verified original asset",
            ));
        }
        let matches = if exact_matches.is_empty() {
            self.records
                .iter()
                .filter(|r| r.origin.sha256 == sha)
                .collect::<Vec<_>>()
        } else {
            exact_matches
        };
        let Some(first) = matches.first() else {
            return Ok(None);
        };
        if matches
            .iter()
            .any(|r| r.origin.selection != first.origin.selection)
        {
            return Err(Error::input(
                "Identical reference bytes have different recorded models; select an exact original artifact path or provide unambiguous origin metadata",
            ));
        }
        if identity && matches.iter().all(|r| r.quality == Quality::NeedsRevision) {
            return Err(Error::input(
                "This source was rejected as needs_revision; use the selected original identity instead",
            ));
        }
        Ok(Some(first.origin.clone()))
    }
}
pub fn check(reference: &Reference) -> Result<()> {
    if let Some(origin) = &reference.origin {
        if origin.sha256 != reference.sha256 {
            return Err(Error::input(
                "Reference origin hash does not match the actual media bytes",
            ));
        }
        crate::validation::model(&origin.selection)?;
    }
    Ok(())
}
pub fn attach(
    reference: &mut Reference,
    path: Option<&Path>,
    declared: Option<AssetOrigin>,
    index: Option<&Index>,
) -> Result<()> {
    reference.origin = declared;
    check(reference)?;
    if let Some(index) = index
        && let Some(recorded) = index.origin(
            path,
            &reference.sha256,
            matches!(reference.role, ReferenceRole::Identity),
        )?
    {
        if reference
            .origin
            .as_ref()
            .is_some_and(|declared| declared.selection != recorded.selection)
        {
            return Err(Error::input(
                "Declared reference model conflicts with its original generation receipt",
            ));
        }
        reference.origin = Some(recorded);
    }
    check(reference)
}
pub fn reference(
    base: &Path,
    path: &Path,
    role: ReferenceRole,
    declared: Option<AssetOrigin>,
    index: Option<&Index>,
) -> Result<Reference> {
    let resolved = storage::relative(base, path)?;
    let mut reference = crate::validation::reference(base, path)?;
    reference.role = role;
    attach(&mut reference, Some(&resolved), declared, index)?;
    Ok(reference)
}
