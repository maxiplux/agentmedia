//! Durable side operations share the same ambiguity rules as media generation.
use crate::{
    contracts::*,
    error::{Error, Result},
    storage::*,
};
use fs4::fs_std::FileExt;
use serde_json::Value;
use std::{future::Future, path::Path};

pub struct Output {
    pub value: Value,
    pub request_id: Option<String>,
    pub usage: Option<Value>,
}
pub struct Request {
    pub selection: Selection,
    pub input: Value,
    pub attempts: u32,
    pub retry_unknown: bool,
}
pub fn status(path: &Path) -> Result<Operation> {
    let guard = lock(&path.with_extension("lock"))?;
    let owned = guard.try_lock_exclusive()?;
    let mut record: Operation = yaml(path)?;
    version(record.schema_version)?;
    if owned && record.state == State::Running {
        record.state = State::Unknown;
        record.error = Some(
            "Process stopped before the response was recorded; reconcile before explicit retry"
                .into(),
        );
        if let Some(attempt) = record.history.last_mut() {
            attempt.state = State::Unknown;
            attempt.error = record.error.clone();
        }
        save(path, &record)?;
    }
    Ok(record)
}
pub async fn perform<R, C, F>(
    path: &Path,
    request: Request,
    reserve: R,
    call: C,
) -> Result<Operation>
where
    R: FnOnce() -> Result<()>,
    C: FnOnce() -> F,
    F: Future<Output = Result<Output>>,
{
    let guard = lock(&path.with_extension("lock"))?;
    if !guard.try_lock_exclusive()? {
        return Err(Error::new(
            "operation_active",
            "Another process owns this operation; inspect its record",
            3,
        ));
    }
    let mut record = if path.exists() {
        yaml::<Operation>(path)?
    } else {
        Operation {
            schema_version: 1,
            id: uuid::Uuid::new_v4().to_string(),
            selection: request.selection.clone(),
            input: request.input.clone(),
            state: State::Queued,
            attempts: 0,
            request_id: None,
            usage: None,
            error: None,
            retry_not_before: None,
            result: None,
            history: vec![],
        }
    };
    version(record.schema_version)?;
    if record.selection != request.selection || record.input != request.input {
        return Err(Error::input("Operation record belongs to different inputs"));
    }
    if record.state == State::Succeeded {
        return Ok(record);
    }
    if record.state == State::Running {
        record.state = State::Unknown;
        if let Some(attempt) = record.history.last_mut() {
            attempt.state = State::Unknown;
            attempt.error = Some("Process stopped before the response was recorded".into());
        }
        save(path, &record)?;
    }
    if matches!(record.state, State::Running | State::Unknown) && !request.retry_unknown {
        record.state = State::Unknown;
        record.error =
            Some("Submission outcome is unknown; explicit retry may incur another charge".into());
        save(path, &record)?;
        return Err(Error::unknown(format!(
            "Inspect {} before explicitly retrying the unknown operation",
            path.display()
        )));
    }
    let now = epoch();
    if record.retry_not_before.is_some_and(|time| time > now) {
        return Err(Error::new(
            "retry_deferred",
            "Provider retry guidance has not elapsed; inspect retry_not_before",
            3,
        ));
    }
    if record.attempts >= request.attempts {
        return Err(Error::limit("Operation attempt limit exhausted"));
    }
    crate::catalog::require_current(&request.selection)?;
    let _slots = crate::engine::slots(request.selection.provider).await?;
    record.state = State::Running;
    record.error = None;
    save(path, &record)?;
    if let Err(error) = reserve() {
        record.state = State::Blocked;
        record.error = Some(error.to_string());
        save(path, &record)?;
        return Err(error);
    }
    record.attempts += 1;
    record.history.push(OperationAttempt {
        attempt: record.attempts,
        state: State::Running,
        request_id: None,
        usage: None,
        error: None,
    });
    save(path, &record)?;
    match call().await {
        Ok(output) => {
            record.request_id = output.request_id;
            record.usage = output.usage;
            record.result = Some(output.value);
            record.retry_not_before = None;
            record.state = State::Succeeded;
            if let Some(attempt) = record.history.last_mut() {
                attempt.state = State::Succeeded;
                attempt.request_id = record.request_id.clone();
                attempt.usage = record.usage.clone();
            }
            save(path, &record)?;
            Ok(record)
        }
        Err(mut error) => {
            record.state = if error.exit == 7 {
                State::Unknown
            } else {
                State::Failed
            };
            record.request_id = error.request_id.clone();
            record.error = Some(error.to_string());
            record.retry_not_before = error
                .retry_after_seconds
                .map(|delay| epoch().saturating_add(delay));
            if let Some(attempt) = record.history.last_mut() {
                attempt.state = record.state;
                attempt.request_id = record.request_id.clone();
                attempt.error = record.error.clone();
            }
            save(path, &record)?;
            error.message =
                format!("{}; operation record: {}", error.message, path.display()).into_boxed_str();
            Err(error)
        }
    }
}
/// Retrieve a saved asynchronous result without consuming another attempt or request reservation.
pub async fn reconcile<C, F>(path: &Path, call: C) -> Result<Operation>
where
    C: FnOnce(Operation) -> F,
    F: Future<Output = Result<Option<Output>>>,
{
    let guard = lock(&path.with_extension("lock"))?;
    if !guard.try_lock_exclusive()? {
        return Err(Error::new(
            "operation_active",
            "Wait for the current operation before reconciliation",
            3,
        ));
    }
    let mut record: Operation = yaml(path)?;
    version(record.schema_version)?;
    if record.state == State::Succeeded {
        return Ok(record);
    }
    if !matches!(record.state, State::Unknown | State::Running) {
        return Err(Error::input(
            "Only uncertain submitted operations need provider reconciliation",
        ));
    }
    if record.request_id.is_none() {
        return Err(Error::unknown(
            "No provider interaction ID was recorded; inspect the provider account before explicit retry",
        ));
    }
    let _slots = crate::engine::slots(record.selection.provider).await?;
    record.state = State::Unknown;
    match call(record.clone()).await {
        Ok(Some(output)) => {
            record.state = State::Succeeded;
            record.result = Some(output.value);
            record.usage = output.usage;
            record.request_id = output.request_id.or(record.request_id);
            record.error = None;
            record.retry_not_before = None;
            if let Some(attempt) = record.history.last_mut() {
                attempt.state = State::Succeeded;
                attempt.request_id = record.request_id.clone();
                attempt.usage = record.usage.clone();
                attempt.error = None;
            }
        }
        Ok(None) => {
            record.error =
                Some("Provider interaction remains pending; no new request submitted".into())
        }
        Err(error) => {
            record.error = Some(error.to_string());
            if error.code == "provider_failed" {
                record.state = State::Failed;
            }
        }
    }
    save(path, &record)?;
    Ok(record)
}
fn epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
