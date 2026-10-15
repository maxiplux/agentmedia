use mediaagent::{
    contracts::*,
    error::Error,
    operations::{self, Output, Request},
    storage,
};
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};
fn request(retry: bool) -> Request {
    Request {
        selection: Selection {
            provider: Provider::Openai,
            model: "gpt-6-luna".into(),
        },
        input: json!({"kind":"image_review","artifact":"fixture"}),
        attempts: 3,
        retry_unknown: retry,
    }
}
#[tokio::test]
async fn operation_success_is_reused_after_local_recording_interruption() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("operation.yaml");
    let calls = AtomicUsize::new(0);
    for _ in 0..2 {
        let operation = operations::perform(
            &path,
            request(false),
            || Ok(()),
            || async {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(Output {
                    value: json!({"quality":"passed"}),
                    request_id: Some("remote-1".into()),
                    usage: Some(json!({"total_tokens":12})),
                })
            },
        )
        .await
        .unwrap();
        assert_eq!(operation.state, State::Succeeded);
        assert_eq!(operation.attempts, 1);
        assert_eq!(operation.request_id.as_deref(), Some("remote-1"));
    }
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}
#[tokio::test]
async fn unknown_operations_require_explicit_retry_and_keep_provider_identity() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("operation.yaml");
    let calls = AtomicUsize::new(0);
    let result = operations::perform(
        &path,
        request(false),
        || Ok(()),
        || async {
            calls.fetch_add(1, Ordering::SeqCst);
            let mut error = Error::unknown("Interrupted response");
            error.request_id = Some("uncertain-id".into());
            Err(error)
        },
    )
    .await;
    assert_eq!(result.unwrap_err().exit, 7);
    let result = operations::perform(
        &path,
        request(false),
        || Ok(()),
        || async {
            calls.fetch_add(1, Ordering::SeqCst);
            unreachable!()
        },
    )
    .await;
    assert_eq!(result.unwrap_err().exit, 7);
    let saved: Operation = storage::yaml(&path).unwrap();
    assert_eq!(saved.request_id.as_deref(), Some("uncertain-id"));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    let result = operations::perform(
        &path,
        request(true),
        || Ok(()),
        || async {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(Output {
                value: json!({"quality":"passed"}),
                request_id: Some("resolved".into()),
                usage: None,
            })
        },
    )
    .await
    .unwrap();
    assert_eq!(result.attempts, 2);
    assert_eq!(
        result.history[0].request_id.as_deref(),
        Some("uncertain-id")
    );
    assert_eq!(result.history[0].state, State::Unknown);
    assert_eq!(result.history[1].request_id.as_deref(), Some("resolved"));
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}
#[tokio::test]
async fn budget_failure_precedes_submission_and_rate_limit_is_honored() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("operation.yaml");
    let error = operations::perform(
        &path,
        request(false),
        || Err(Error::limit("Exhausted")),
        || async { unreachable!() },
    )
    .await
    .unwrap_err();
    assert_eq!(error.exit, 6);
    assert_eq!(storage::yaml::<Operation>(&path).unwrap().attempts, 0);
    let error = operations::perform(
        &path,
        request(false),
        || Ok(()),
        || async {
            let mut error = Error::new("provider_rejected", "Rate limited", 3);
            error.retry_after_seconds = Some(60);
            Err(error)
        },
    )
    .await
    .unwrap_err();
    assert_eq!(error.code, "provider_rejected");
    let error = operations::perform(
        &path,
        request(false),
        || Ok(()),
        || async { unreachable!() },
    )
    .await
    .unwrap_err();
    assert_eq!(error.code, "retry_deferred");
}
#[tokio::test]
async fn lost_running_operation_and_changed_inputs_are_not_resubmitted() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("operation.yaml");
    let error = operations::perform(
        &path,
        request(false),
        || Ok(()),
        || async { Err(Error::unknown("lost")) },
    )
    .await
    .unwrap_err();
    assert_eq!(error.exit, 7);
    let mut record: Operation = storage::yaml(&path).unwrap();
    record.state = State::Running;
    storage::save(&path, &record).unwrap();
    assert_eq!(
        operations::perform(
            &path,
            request(false),
            || Ok(()),
            || async { unreachable!() }
        )
        .await
        .unwrap_err()
        .exit,
        7
    );
    let mut changed = request(true);
    changed.selection.model = "different".into();
    assert_eq!(
        operations::perform(&path, changed, || Ok(()), || async { unreachable!() })
            .await
            .unwrap_err()
            .exit,
        2
    );
}

#[tokio::test]
async fn status_reports_owned_work_and_marks_orphaned_attempts_unknown() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("operation.yaml");
    operations::perform(
        &path,
        request(false),
        || Ok(()),
        || async {
            let current = operations::status(&path)?;
            assert_eq!(current.state, State::Running);
            Ok(Output {
                value: json!({"quality":"passed"}),
                request_id: Some("receipt".into()),
                usage: None,
            })
        },
    )
    .await
    .unwrap();
    let mut saved: Operation = storage::yaml(&path).unwrap();
    saved.state = State::Running;
    saved.history[0].state = State::Running;
    storage::save(&path, &saved).unwrap();
    let orphan = operations::status(&path).unwrap();
    assert_eq!(orphan.state, State::Unknown);
    assert_eq!(orphan.history[0].state, State::Unknown);
    assert_eq!(orphan.request_id.as_deref(), Some("receipt"));
}

#[tokio::test]
async fn reconciliation_keeps_attempts_and_receipts_without_reserving_again() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("operation.yaml");
    let selection = Selection {
        provider: Provider::Google,
        model: "gemini-3.8-flash".into(),
    };
    let mut error = Error::unknown("Pending interaction");
    error.request_id = Some("interaction_review".into());
    let _ = operations::perform(
        &path,
        Request {
            selection,
            input: json!({"artifact":"one"}),
            attempts: 1,
            retry_unknown: false,
        },
        || Ok(()),
        || async { Err(error) },
    )
    .await;
    let pending = operations::reconcile(&path, |_| async { Ok(None) })
        .await
        .unwrap();
    assert_eq!(pending.state, State::Unknown);
    let complete = operations::reconcile(&path, |record| async move {
        assert_eq!(record.request_id.as_deref(), Some("interaction_review"));
        Ok(Some(Output {
            value: json!({"quality":"passed"}),
            request_id: record.request_id,
            usage: None,
        }))
    })
    .await
    .unwrap();
    assert_eq!(complete.state, State::Succeeded);
    assert_eq!(complete.attempts, 1);
    assert_eq!(complete.history.len(), 1);
    assert_eq!(complete.history[0].state, State::Succeeded);
}
