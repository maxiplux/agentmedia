//! Explicit read-only authentication probes. Response bodies/account data never leave this module.
use super::{Transport, WireResponse};
use crate::contracts::Media;
use crate::{
    contracts::Provider,
    credentials,
    error::{Error, Result},
};
use async_trait::async_trait;
use reqwest::header::HeaderValue;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

pub const PROVIDERS: [Provider; 5] = [
    Provider::Openai,
    Provider::Google,
    Provider::Fish,
    Provider::Openrouter,
    Provider::Elevenlabs,
];
const MAX_BYTES: usize = 1024 * 1024;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Configured,
    MissingCredentials,
    InvalidConfiguration,
    Ok,
    AuthenticationRejected,
    AccessDenied,
    RateLimited,
    ServiceUnavailable,
    EndpointError,
    InvalidResponse,
    Timeout,
    NetworkError,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Authentication {
    NotChecked,
    Accepted,
    Rejected,
    Inconclusive,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Check {
    pub provider: Provider,
    pub variable: String,
    pub configured: bool,
    pub endpoint: String,
    pub status: Status,
    pub authentication: Authentication,
    pub network_checked: bool,
    pub endpoint_reachable: Option<bool>,
    pub http_status: Option<u16>,
    pub elapsed_ms: Option<u64>,
    pub retry_after_seconds: Option<u64>,
    pub message: String,
    pub action: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Report {
    #[schemars(range(min = 1, max = 1))]
    pub schema_version: u32,
    pub project: PathBuf,
    pub initialized: bool,
    pub live_requested: bool,
    pub network_checked: bool,
    pub generation_performed: bool,
    pub credentials: Vec<Check>,
    pub scope: String,
}
impl Report {
    pub fn exit(&self) -> u8 {
        if self.live_requested && self.credentials.iter().any(|c| c.status != Status::Ok) {
            3
        } else {
            0
        }
    }
    pub fn human(&self) -> String {
        let mut out = if self.live_requested {
            "API connection checks\n"
        } else {
            "Local credential checks (offline)\n"
        }
        .to_owned();
        for c in &self.credentials {
            let label = serde_json::to_value(c.provider).expect("Provider serializes");
            let status = serde_json::to_value(c.status).expect("Status serializes");
            out += &format!(
                "{}: {} — {}\n",
                label.as_str().unwrap(),
                status.as_str().unwrap(),
                c.message
            );
            if let Some(action) = &c.action {
                out += &format!("  Next: {action}\n");
            }
        }
        out += &self.scope;
        out
    }
}
fn endpoint(provider: Provider) -> &'static str {
    match provider {
        Provider::Openai => "https://api.openai.com/v1/models",
        Provider::Google => "https://generativelanguage.googleapis.com/v1beta/models?pageSize=1",
        Provider::Fish => "https://api.fish.audio/wallet/self/api-credit",
        Provider::Openrouter => "https://openrouter.ai/api/v1/key",
        Provider::Elevenlabs => "https://api.elevenlabs.io/v1/user",
    }
}
struct LiveTransport {
    client: reqwest::Client,
}
#[async_trait]
impl Transport for LiveTransport {
    async fn send(&self, request: reqwest::Request) -> Result<WireResponse> {
        let mut response = self.client.execute(request).await.map_err(network_error)?;
        let status = response.status().as_u16();
        let headers = response.headers().clone();
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(network_error)? {
            if bytes.len() + chunk.len() > MAX_BYTES {
                return Err(Error::new(
                    "diagnostic_response_too_large",
                    "Read-only check exceeded its response limit",
                    3,
                ));
            }
            bytes.extend_from_slice(&chunk);
        }
        Ok(WireResponse {
            status,
            headers,
            bytes,
        })
    }
}
fn network_error(error: reqwest::Error) -> Error {
    if error.is_timeout() {
        Error::new("diagnostic_timeout", "Read-only check timed out", 3)
    } else {
        Error::new(
            "diagnostic_network",
            "Read-only check could not establish or finish a secure connection",
            3,
        )
    }
}
pub struct Client {
    client: reqwest::Client,
    transport: Arc<dyn Transport>,
    timeout: Duration,
}
impl Client {
    pub fn new(timeout_seconds: u64) -> Result<Self> {
        Self::build(timeout_seconds, None)
    }
    /// Injection keeps ordinary tests offline even when real credentials are present.
    pub fn with_transport(timeout_seconds: u64, transport: Arc<dyn Transport>) -> Result<Self> {
        Self::build(timeout_seconds, Some(transport))
    }
    fn build(timeout_seconds: u64, transport: Option<Arc<dyn Transport>>) -> Result<Self> {
        if !(1..=60).contains(&timeout_seconds) {
            return Err(Error::input("Diagnostic timeout must be 1–60 seconds"));
        }
        let timeout = Duration::from_secs(timeout_seconds);
        let client = reqwest::Client::builder()
            .https_only(true)
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(timeout.min(Duration::from_secs(5)))
            .timeout(timeout)
            .build()
            .map_err(|_| Error::input("Cannot initialize secure diagnostic client"))?;
        let transport = transport.unwrap_or_else(|| {
            Arc::new(LiveTransport {
                client: client.clone(),
            })
        });
        Ok(Self {
            client,
            transport,
            timeout,
        })
    }
    pub async fn check(&self, root: &Path, selected: &[Provider], live: bool) -> Result<Report> {
        let selected: BTreeSet<_> = if selected.is_empty() {
            PROVIDERS.into_iter().collect()
        } else {
            selected.iter().copied().collect()
        };
        let mut credentials = Vec::new();
        let mut pending = tokio::task::JoinSet::new();
        for provider in selected {
            let mut row = Check {
                provider,
                variable: provider.key().into(),
                configured: false,
                endpoint: endpoint(provider).into(),
                status: Status::MissingCredentials,
                authentication: Authentication::NotChecked,
                network_checked: false,
                endpoint_reachable: None,
                http_status: None,
                elapsed_ms: None,
                retry_after_seconds: None,
                message: "Credential is unavailable; no request sent".into(),
                action: Some(format!(
                    "Set {} in the process environment or target project's .env",
                    provider.key()
                )),
            };
            let key = match credentials::resolve(provider, root) {
                Ok(key) => key,
                Err(error) => {
                    if error.code != "missing_credentials" {
                        row.status = Status::InvalidConfiguration;
                        row.message =
                            "Cannot parse target project credentials; no request sent".into();
                        row.action = Some("Correct the target project's .env syntax".into());
                    }
                    credentials.push(row);
                    continue;
                }
            };
            row.configured = true;
            if !live {
                row.status = Status::Configured;
                row.message = "Credential is present; authentication has not been tested".into();
                row.action = Some(
                    "Run mediaagent doctor --live for read-only endpoint/authentication checks"
                        .into(),
                );
                credentials.push(row);
                continue;
            }
            let (header, value) = match provider {
                Provider::Google => ("x-goog-api-key", key),
                Provider::Elevenlabs => ("xi-api-key", key),
                _ => ("authorization", format!("Bearer {key}")),
            };
            let mut value = match HeaderValue::from_str(&value) {
                Ok(value) => value,
                Err(_) => {
                    row.status = Status::InvalidConfiguration;
                    row.message =
                        "Credential contains invalid HTTP header characters; no request sent"
                            .into();
                    row.action = Some(format!(
                        "Correct {} without including line breaks",
                        provider.key()
                    ));
                    credentials.push(row);
                    continue;
                }
            };
            value.set_sensitive(true);
            let request = self
                .client
                .get(endpoint(provider))
                .header(header, value)
                .header("accept", "application/json")
                .timeout(self.timeout)
                .build()
                .map_err(|_| Error::input("Cannot construct read-only diagnostic request"))?;
            let transport = self.transport.clone();
            let timeout = self.timeout;
            // At most five independent read-only requests. No retry, redirect, pagination, or generation.
            pending.spawn(async move {
                let start=Instant::now();row.network_checked=true;
                match tokio::time::timeout(timeout,transport.send(request)).await {
                    Ok(Ok(response))=>classify(&mut row,response),
                    outcome=>{
                        let code=match &outcome {Ok(Err(error))=>error.code,_=>"diagnostic_timeout"};
                        row.status=match code {"diagnostic_timeout"=>Status::Timeout,"diagnostic_response_too_large"=>Status::InvalidResponse,_=>Status::NetworkError};
                        row.authentication=Authentication::Inconclusive;
                        row.endpoint_reachable=if row.status==Status::NetworkError || row.status==Status::Timeout {None} else {Some(true)};
                        row.message=match row.status {Status::Timeout=>"Connection check timed out; key validity is unknown",Status::InvalidResponse=>"Response exceeded the 1 MiB diagnostic limit; authentication is unconfirmed",_=>"DNS, TLS, proxy, or network connection failed; key validity is unknown"}.into();
                        row.action=Some("Check the network/proxy and provider status, then rerun the read-only check".into());
                    }
                }
                row.elapsed_ms=Some(start.elapsed().as_millis().min(u64::MAX as u128) as u64);row
            });
        }
        while let Some(row) = pending.join_next().await {
            credentials.push(row.map_err(|_| {
                Error::new("diagnostic_worker", "Connection check worker failed", 3)
            })?);
        }
        credentials.sort_by_key(|c| c.provider);
        Ok(Report {schema_version:1,project:root.into(),initialized:root.join("mediaagent/config.yaml").exists(),live_requested:live,network_checked:credentials.iter().any(|c|c.network_checked),generation_performed:false,credentials,scope:if live {"Checks only the listed read-only authentication endpoints. Media generation endpoints, model/voice entitlements, billing readiness, and output quality remain untested. No media was generated."} else {"No network calls were made. Key presence does not establish key validity."}.into()})
    }
}
fn classify(row: &mut Check, response: WireResponse) {
    row.endpoint_reachable = Some(true);
    row.http_status = Some(response.status);
    row.authentication = Authentication::Inconclusive;
    row.retry_after_seconds = response
        .headers
        .get("retry-after")
        .and_then(|h| h.to_str().ok()?.parse().ok());
    let parsed: Option<Value> = if response.bytes.len() <= MAX_BYTES {
        serde_json::from_slice(&response.bytes).ok()
    } else {
        None
    };
    let v = parsed.as_ref().unwrap_or(&Value::Null);
    // Compare allowlisted machine codes only; never echo provider bodies, key previews, account or billing fields.
    let missing_permission =
        v.pointer("/detail/status").and_then(Value::as_str) == Some("missing_permissions");
    let invalid_google = v
        .pointer("/error/details")
        .and_then(Value::as_array)
        .is_some_and(|details| {
            details.iter().any(|d| {
                matches!(
                    d["reason"].as_str(),
                    Some("API_KEY_INVALID" | "API_KEY_EXPIRED")
                )
            })
        });
    let rejected = invalid_google
        || v.pointer("/error/code").and_then(Value::as_str) == Some("invalid_api_key");
    let (status, message, action) = match response.status {
        200..=299 if !rejected && expected(row.provider, v) => {
            row.authentication = Authentication::Accepted;
            (
                Status::Ok,
                "Read-only endpoint responded and accepted authentication",
                None,
            )
        }
        200..=299 => (
            Status::InvalidResponse,
            "Unexpected response; authentication is unconfirmed",
            Some("Check provider status and whether this documented endpoint has changed"),
        ),
        _ if rejected || response.status == 401 && !missing_permission => {
            row.authentication = Authentication::Rejected;
            (
                Status::AuthenticationRejected,
                "Provider rejected authentication",
                Some(
                    "Check the selected key, expiry, revocation, and account/project restrictions in the provider console",
                ),
            )
        }
        401 | 403 => (
            Status::AccessDenied,
            "This endpoint is forbidden; a restricted key may still work for generation",
            Some(
                "Check read permission, API enablement, project/IP/region restrictions, and account access; key validity is unconfirmed",
            ),
        ),
        429 => (
            Status::RateLimited,
            "Read-only check was rate limited; key validity is unconfirmed",
            Some("Honor retry_after_seconds when present and retry this read-only check later"),
        ),
        500..=599 => (
            Status::ServiceUnavailable,
            "Provider service is unavailable; key validity is unconfirmed",
            Some("Check provider status and retry this read-only check later"),
        ),
        _ => (
            Status::EndpointError,
            "Read-only endpoint returned an unexpected HTTP status; authentication is unconfirmed",
            Some(
                "Check provider documentation, network/proxy restrictions, and the reported endpoint; redirects are disabled",
            ),
        ),
    };
    row.status = status;
    row.message = message.into();
    row.action = action.map(str::to_owned);
}
fn expected(provider: Provider, v: &Value) -> bool {
    if v.get("error").is_some_and(|error| !error.is_null()) {
        return false;
    }
    match provider {
        Provider::Openai => v["data"].is_array(),
        Provider::Google => v["models"].is_array() || v.as_object().is_some_and(|o| o.is_empty()),
        Provider::Fish => v["credit"].is_string() || v["credit"].is_number(),
        Provider::Openrouter => v["data"].is_object(),
        Provider::Elevenlabs => v["user_id"].as_str().is_some_and(|id| !id.is_empty()),
    }
}

impl Client {
    /// One catalog GET per provider, no retries or pagination. A listed model is not a generation test.
    pub async fn models(&self, root: &Path, selected: &[Provider]) -> Result<Value> {
        self.models_filtered(root, selected, None, false).await
    }
    pub async fn models_filtered(
        &self,
        root: &Path,
        selected: &[Provider],
        media: Option<Media>,
        popular: bool,
    ) -> Result<Value> {
        if popular && selected != [Provider::Openrouter] {
            return Err(Error::input(
                "Popularity ranking is scoped to an explicitly selected OpenRouter catalog",
            ));
        }
        let selected: BTreeSet<_> = if selected.is_empty() {
            PROVIDERS.into_iter().collect()
        } else {
            selected.iter().copied().collect()
        };
        let mut rows = Vec::new();
        for provider in selected {
            if provider == Provider::Openrouter && matches!(media, Some(Media::Music | Media::Sfx))
            {
                rows.push(serde_json::json!({"provider":provider,"status":"catalog_visible","models":[],"generation_tested":false,"action":"No implemented OpenRouter catalog modality for music or sound effects"}));
                continue;
            }
            let endpoint = match provider {
                Provider::Openai => "https://api.openai.com/v1/models",
                Provider::Google => {
                    "https://generativelanguage.googleapis.com/v1beta/models?pageSize=100"
                }
                Provider::Openrouter => "https://openrouter.ai/api/v1/models",
                Provider::Elevenlabs => "https://api.elevenlabs.io/v1/models",
                Provider::Fish => {
                    rows.push(serde_json::json!({"provider":provider,"status":"bundled_catalog","verified_at":crate::catalog::VERIFIED_AT,"models":crate::catalog::models().into_iter().filter(|m|m.provider==provider&&media.is_none_or(|v|crate::catalog::target(v)==m.media)).collect::<Vec<_>>(),"action":"Fish has no implemented live TTS-model catalog; these are bundled documentation choices, not account access verification"}));
                    continue;
                }
            };
            let response:Result<Value>=async {
                let key=credentials::resolve(provider,root)?;
                let (header,key)=match provider {Provider::Google=>("x-goog-api-key",key),Provider::Elevenlabs=>("xi-api-key",key),_=>("authorization",format!("Bearer {key}"))};
                let mut key=HeaderValue::from_str(&key).map_err(|_|Error::input("Invalid credential header"))?;key.set_sensitive(true);
                let mut builder=self.client.get(endpoint).header(header,key);
                if provider==Provider::Openrouter {builder=builder.query(&[("output_modalities",media.map(crate::catalog::target).map_or("all",|v|match v {Media::Image=>"image",Media::Speech=>"speech",_=>"text"}))]);if popular{builder=builder.query(&[("sort","most-popular")]);}}
                let req=builder.build().map_err(|_|Error::input("Cannot build model catalog request"))?;
                let response=tokio::time::timeout(self.timeout,self.transport.send(req)).await.map_err(|_|Error::new("timeout","Catalog lookup timed out",3))??;
                if response.status!=200 {return Err(Error::new("catalog_unavailable",format!("Catalog HTTP {}; account generation access is inconclusive",response.status),3))}
                if response.bytes.len()>MAX_BYTES{return Err(Error::new("response_limit","Catalog response exceeded 1 MiB",3))}
                let value:Value=serde_json::from_slice(&response.bytes).map_err(|_|Error::input("Invalid catalog JSON"))?;
                let (list,field)=match provider {Provider::Google=>(&value["models"],"name"),Provider::Elevenlabs=>(&value,"model_id"),_=>(&value["data"],"id")};
                let list=list.as_array().ok_or_else(||Error::input("Invalid catalog shape"))?;
                let mut models=Vec::new();
                for (rank,item) in list.iter().enumerate() {
                    let id=item[field].as_str().ok_or_else(||Error::input("Invalid model identifier"))?;
                    let id=id.strip_prefix("models/").unwrap_or(id);
                    if id.len()>200 || !id.bytes().all(|b|b.is_ascii_alphanumeric()||b"-_/.:".contains(&b)){return Err(Error::input("Invalid model identifier"))}
                    let shutdown=item["shutdown_date"].as_str().filter(|v|v.len()==10&&v.bytes().all(|b|b.is_ascii_digit()||b==b'-'));
                    let known=crate::catalog::models().into_iter().find(|m|m.provider==provider&&m.model==id);
                    if media.is_some_and(|v|known.as_ref().is_some_and(|m|m.media!=crate::catalog::target(v))) {continue;}
                    models.push(serde_json::json!({"model":id,"popularity_rank":if popular{Some(rank+1)}else{None},"registry_supported":super::registry().iter().any(|c|c.provider==provider&&c.model==id),"shutdown_date":shutdown,"current":known.as_ref().is_some_and(|m|m.current),"supported":known.as_ref().is_some_and(|m|m.supported),"recommended":known.as_ref().is_some_and(|m|m.current&&m.recommended),"lifecycle":known.as_ref().map_or("unreviewed",|m|m.lifecycle.as_str()),"media":known.as_ref().map(|m|m.media)}));
                }
                Ok(serde_json::json!({"provider":provider,"status":"catalog_visible","models":models,"has_more":value.get("nextPageToken").is_some(),"account_generation_access":"unverified","generation_tested":false}))
            }.await;
            rows.push(match response {Ok(v)=>v,Err(e)=>serde_json::json!({"provider":provider,"status":"inconclusive","reason":e.code,"action":"Check credentials/endpoint permissions; do not infer generation readiness"})});
        }
        Ok(
            serde_json::json!({"catalogs":rows,"generation_performed":false,"ranking_scope":if popular{Some("OpenRouter provider-reported popularity")}else{None},"fetched_at_unix":std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs(),"scope":"One bounded read-only catalog page per provider; absent models may be unlisted. Fish uses its bundled documented catalog. No registry updates or model substitution."}),
        )
    }
}
