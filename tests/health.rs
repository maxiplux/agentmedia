use async_trait::async_trait;
use mediaagent::{
    contracts::Provider,
    error::{Error, Result},
    providers::{
        Transport, WireResponse,
        health::{self, Authentication, Status},
    },
};
use serde_json::{Value, json};
use std::{
    fs,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
struct Mock {
    status: u16,
    body: Option<Value>,
    error: bool,
    delay: Duration,
    calls: AtomicUsize,
    active: AtomicUsize,
    max: AtomicUsize,
    seen: Mutex<Vec<String>>,
}
impl Mock {
    fn new(status: u16, body: Option<Value>) -> Self {
        Self {
            status,
            body,
            error: false,
            delay: Duration::from_millis(20),
            calls: AtomicUsize::new(0),
            active: AtomicUsize::new(0),
            max: AtomicUsize::new(0),
            seen: Mutex::new(vec![]),
        }
    }
}
#[async_trait]
impl Transport for Mock {
    async fn send(&self, request: reqwest::Request) -> Result<WireResponse> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        assert!(request.method() == reqwest::Method::GET);
        assert!(request.body().is_none());
        let url = request.url();
        assert!(url.scheme() == "https");
        assert!(url.query().is_none_or(|q| q == "pageSize=1"
            || q == "pageSize=100"
            || q == "output_modalities=all"
            || q == "output_modalities=image&sort=most-popular"));
        let header = match url.host_str().unwrap() {
            "generativelanguage.googleapis.com" => "x-goog-api-key",
            "api.elevenlabs.io" => "xi-api-key",
            _ => "authorization",
        };
        assert!(request.headers()[header].is_sensitive());
        self.seen.lock().unwrap().push(url.to_string());
        let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
        self.max.fetch_max(active, Ordering::SeqCst);
        tokio::time::sleep(self.delay).await;
        self.active.fetch_sub(1, Ordering::SeqCst);
        if self.error {
            return Err(Error::unknown(
                "LEAKED_ACCOUNT_SECRET https://somewhere/?key=SECRET",
            ));
        }
        let body=self.body.clone().unwrap_or_else(||match url.host_str().unwrap() {
      "api.openai.com"=>json!({"data":[],"account":"PRIVATE"}),
      "generativelanguage.googleapis.com"=>json!({"models":[]}),
      "api.fish.audio"=>json!({"credit":"123.45","user_id":"PRIVATE"}),
      "openrouter.ai"=>json!({"data":{"label":"PRIVATE","limit":123.45}}),
      _=>json!({"user_id":"PRIVATE","xi_api_key_preview":"PRIVATE","subscription":{"credit":123.45}}),
    });
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("retry-after", "13".parse().unwrap());
        Ok(WireResponse {
            status: self.status,
            headers,
            bytes: serde_json::to_vec(&body).unwrap(),
        })
    }
}
fn root() -> tempfile::TempDir {
    let root = tempfile::tempdir().unwrap();
    fs::write(
        root.path().join(".env"),
        health::PROVIDERS
            .iter()
            .map(|p| format!("{}=OFFLINE_FIXTURE\n", p.key()))
            .collect::<String>(),
    )
    .unwrap();
    root
}
#[tokio::test]
async fn live_checks_use_all_authenticated_read_only_routes_concurrently_and_redact_metadata() {
    let root = root();
    let fake = Arc::new(Mock::new(200, None));
    let client = health::Client::with_transport(2, fake.clone()).unwrap();
    let report = client.check(root.path(), &[], true).await.unwrap();
    assert_eq!(report.exit(), 0);
    assert!(report.network_checked);
    assert!(!report.generation_performed);
    assert_eq!(report.credentials.len(), 5);
    assert_eq!(fake.calls.load(Ordering::SeqCst), 5);
    assert_eq!(fake.max.load(Ordering::SeqCst), 5);
    for row in &report.credentials {
        assert_eq!(row.status, Status::Ok);
        assert_eq!(row.authentication, Authentication::Accepted);
        assert_eq!(row.endpoint_reachable, Some(true));
    }
    let output = serde_json::to_string(&report).unwrap();
    assert!(!output.contains("PRIVATE"));
    assert!(!output.contains("123.45"));
    assert!(!output.contains("OFFLINE_FIXTURE"));
    let seen = fake.seen.lock().unwrap().clone();
    for suffix in [
        "/v1/models",
        "/v1beta/models?pageSize=1",
        "/wallet/self/api-credit",
        "/api/v1/key",
        "/v1/user",
    ] {
        assert!(seen.iter().any(|url| url.ends_with(suffix)));
    }
    assert!(!root.path().join("mediaagent").exists());
    let mixed = Arc::new(Mock::new(
        200,
        Some(json!({"data":[],"models":[],"credit":"0","user_id":"PRIVATE"})),
    ));
    let report = health::Client::with_transport(2, mixed.clone())
        .unwrap()
        .check(root.path(), &[], true)
        .await
        .unwrap();
    assert_eq!(report.exit(), 3);
    assert_eq!(
        report
            .credentials
            .iter()
            .filter(|row| row.status == Status::Ok)
            .count(),
        4
    );
    assert_eq!(mixed.calls.load(Ordering::SeqCst), 5);
    assert_eq!(
        report
            .credentials
            .iter()
            .find(|row| row.provider == Provider::Openrouter)
            .unwrap()
            .status,
        Status::InvalidResponse
    );
}
#[tokio::test]
async fn offline_diagnostics_never_send_and_provider_filters_deduplicate() {
    let root = root();
    let fake = Arc::new(Mock::new(200, None));
    let client = health::Client::with_transport(2, fake.clone()).unwrap();
    let report = client.check(root.path(), &[], false).await.unwrap();
    assert!(!report.network_checked);
    assert_eq!(fake.calls.load(Ordering::SeqCst), 0);
    assert!(
        report
            .credentials
            .iter()
            .all(|c| c.status == Status::Configured
                && c.authentication == Authentication::NotChecked)
    );
    let report = client
        .check(root.path(), &[Provider::Google, Provider::Google], true)
        .await
        .unwrap();
    assert_eq!(report.credentials.len(), 1);
    assert_eq!(fake.calls.load(Ordering::SeqCst), 1);
}
#[tokio::test]
async fn rejected_permission_rate_limit_and_service_errors_are_distinct_and_do_not_retry() {
    let root = root();
    for (http, body, status, auth) in [
        (
            401,
            json!({"detail":{"status":"invalid_api_key"},"message":"PRIVATE"}),
            Status::AuthenticationRejected,
            Authentication::Rejected,
        ),
        (
            401,
            json!({"detail":{"status":"missing_permissions","message":"PRIVATE"}}),
            Status::AccessDenied,
            Authentication::Inconclusive,
        ),
        (
            403,
            json!({"error":{"message":"PRIVATE"}}),
            Status::AccessDenied,
            Authentication::Inconclusive,
        ),
        (
            400,
            json!({"error":{"details":[{"reason":"API_KEY_INVALID","metadata":{"key":"PRIVATE"}}]}}),
            Status::AuthenticationRejected,
            Authentication::Rejected,
        ),
        (
            429,
            json!({"error":"PRIVATE"}),
            Status::RateLimited,
            Authentication::Inconclusive,
        ),
        (
            503,
            json!({"message":"PRIVATE"}),
            Status::ServiceUnavailable,
            Authentication::Inconclusive,
        ),
        (
            302,
            json!({}),
            Status::EndpointError,
            Authentication::Inconclusive,
        ),
        (
            404,
            json!({}),
            Status::EndpointError,
            Authentication::Inconclusive,
        ),
        (
            200,
            json!({"error":"PRIVATE"}),
            Status::InvalidResponse,
            Authentication::Inconclusive,
        ),
    ] {
        let fake = Arc::new(Mock::new(http, Some(body)));
        let report = health::Client::with_transport(2, fake.clone())
            .unwrap()
            .check(root.path(), &[Provider::Elevenlabs], true)
            .await
            .unwrap();
        let row = &report.credentials[0];
        assert_eq!(row.status, status);
        assert_eq!(row.authentication, auth);
        assert_eq!(row.http_status, Some(http));
        assert_eq!(row.retry_after_seconds, Some(13));
        assert_eq!(report.exit(), 3);
        assert_eq!(fake.calls.load(Ordering::SeqCst), 1);
        assert!(!serde_json::to_string(&report).unwrap().contains("PRIVATE"));
    }
}
#[tokio::test]
async fn response_shape_and_size_are_checked_without_leaking_provider_data() {
    let root = root();
    for provider in health::PROVIDERS {
        let fake = Arc::new(Mock::new(200, Some(json!({"unexpected":"PRIVATE"}))));
        let report = health::Client::with_transport(2, fake)
            .unwrap()
            .check(root.path(), &[provider], true)
            .await
            .unwrap();
        assert_eq!(report.credentials[0].status, Status::InvalidResponse);
    }
    let fake = Arc::new(Mock::new(
        200,
        Some(json!({"data":[],"extra":"PRIVATE".repeat(200000)})),
    ));
    let report = health::Client::with_transport(2, fake)
        .unwrap()
        .check(root.path(), &[Provider::Openai], true)
        .await
        .unwrap();
    assert_eq!(report.credentials[0].status, Status::InvalidResponse);
}
#[tokio::test]
async fn interrupted_or_timed_out_probes_do_not_claim_key_rejection_or_expose_error_text() {
    let root = root();
    let mut fake = Mock::new(200, None);
    fake.error = true;
    let report = health::Client::with_transport(2, Arc::new(fake))
        .unwrap()
        .check(root.path(), &[Provider::Fish], true)
        .await
        .unwrap();
    assert_eq!(report.credentials[0].status, Status::NetworkError);
    assert_eq!(
        report.credentials[0].authentication,
        Authentication::Inconclusive
    );
    assert!(!serde_json::to_string(&report).unwrap().contains("SECRET"));
    let mut fake = Mock::new(200, None);
    fake.delay = Duration::from_secs(2);
    let fake = Arc::new(fake);
    let report = health::Client::with_transport(1, fake.clone())
        .unwrap()
        .check(root.path(), &[Provider::Fish], true)
        .await
        .unwrap();
    assert_eq!(report.credentials[0].status, Status::Timeout);
    assert_eq!(fake.calls.load(Ordering::SeqCst), 1);
    assert!(health::Client::new(0).is_err());
    assert!(health::Client::new(61).is_err());
}

#[tokio::test]
async fn model_discovery_is_read_only_bounded_and_does_not_claim_entitlement() {
    let root = root();
    for (provider, body, model) in [
        (
            Provider::Openai,
            json!({"data":[{"id":"gpt-image-1","owned_by":"PRIVATE","shutdown_date":"2027-01-01"}]}),
            "gpt-image-1",
        ),
        (
            Provider::Google,
            json!({"models":[{"name":"models/gemini-2.5-flash-image"}],"nextPageToken":"PRIVATE"}),
            "gemini-2.5-flash-image",
        ),
        (
            Provider::Openrouter,
            json!({"data":[{"id":"openai/gpt-image-1","pricing":{"private":"PRIVATE"}}]}),
            "openai/gpt-image-1",
        ),
        (
            Provider::Elevenlabs,
            json!([{"model_id":"eleven_v3","description":"PRIVATE"}]),
            "eleven_v3",
        ),
    ] {
        let fake = Arc::new(Mock::new(200, Some(body)));
        let report = health::Client::with_transport(2, fake.clone())
            .unwrap()
            .models(root.path(), &[provider])
            .await
            .unwrap();
        assert_eq!(fake.calls.load(Ordering::SeqCst), 1);
        assert_eq!(report["catalogs"][0]["models"][0]["model"], model);
        assert_eq!(
            report["catalogs"][0]["account_generation_access"],
            "unverified"
        );
        assert!(!report.to_string().contains("PRIVATE"));
    }
    let fake = Arc::new(Mock::new(403, Some(json!({"error":"PRIVATE"}))));
    let report = health::Client::with_transport(2, fake.clone())
        .unwrap()
        .models(root.path(), &[Provider::Openai, Provider::Fish])
        .await
        .unwrap();
    assert_eq!(fake.calls.load(Ordering::SeqCst), 1);
    assert!(!report.to_string().contains("PRIVATE"));
}

#[tokio::test]
async fn model_popularity_is_provider_scoped_and_discovery_does_not_enable_unknown_models() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join(".env"), "OPENROUTER_API_KEY=fixture\n").unwrap();
    let fake = Arc::new(Mock::new(
        200,
        Some(
            json!({"data":[{"id":"openai/gpt-image-2.5-flare"},{"id":"provider/unreviewed-image"}]}),
        ),
    ));
    let report = health::Client::with_transport(2, fake.clone())
        .unwrap()
        .models_filtered(
            root.path(),
            &[Provider::Openrouter],
            Some(mediaagent::contracts::Media::Image),
            true,
        )
        .await
        .unwrap();
    assert_eq!(fake.calls.load(Ordering::SeqCst), 1);
    assert!(fake.seen.lock().unwrap()[0].contains("output_modalities=image&sort=most-popular"));
    assert_eq!(report["catalogs"][0]["models"][0]["current"], true);
    assert_eq!(report["catalogs"][0]["models"][1]["current"], false);
    assert_eq!(report["catalogs"][0]["models"][1]["supported"], false);
    assert_eq!(report["generation_performed"], false);
}
