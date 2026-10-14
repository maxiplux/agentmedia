use crate::{error::Result, storage};
use std::path::Path;
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Queued,
    Generating,
    Assembling,
    Saving,
    Complete,
    Failed,
}
pub fn save(run: &Path, id: &str, stage: Stage) -> Result<()> {
    storage::save(&run.join("jobs").join(id).join("stage.yaml"), &stage)
}
pub fn active(run: &Path, ids: impl Iterator<Item = String>) -> String {
    ids.filter_map(|id| {
        storage::yaml::<Stage>(&run.join("jobs").join(&id).join("stage.yaml"))
            .ok()
            .map(|stage| {
                format!(
                    "{id}: {}",
                    match stage {
                        Stage::Queued => "queued",
                        Stage::Generating => "generating",
                        Stage::Assembling => "assembling",
                        Stage::Saving => "saving",
                        Stage::Complete => "complete",
                        Stage::Failed => "failed",
                    }
                )
            })
    })
    .collect::<Vec<_>>()
    .join(", ")
}
pub fn label(state: crate::contracts::State) -> &'static str {
    use crate::contracts::State;
    match state {
        State::Queued => "queued",
        State::Running => "generating",
        State::Cancelling => "cancelling",
        State::Succeeded => "complete",
        State::Partial => "partial",
        State::Failed => "failed",
        State::Cancelled => "cancelled",
        State::Blocked => "blocked",
        State::Unknown => "unknown",
    }
}
