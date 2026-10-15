# Implementation status

Development snapshot, October 14, 2026. The first-version command families are implemented within the provider boundaries below. Visual character consistency has failed user acceptance and must not be described as complete. Native validation is separate from media-quality acceptance; public binary availability is recorded on the [Releases page](https://github.com/maxiplux/agentmedia/releases/latest).

Implemented: Rust package/library/CLI, versioned contracts and schemas, offline preserving initialization, embedded skills/templates and explicit bundled-file refresh, portable character import/export and reference acceptance, direct/job/batch validation and explicit job overrides, infographic compilation, text planning with contained validated authoring bundles, concurrent durable runs, dependencies, finite limits, verified cache, submission idempotency, worker startup/loss handling, cancellation/resume, native audio assembly, review records, durable image AI review receipts/reuse/retry history, ElevenLabs descriptive voice previews and explicit saved-voice creation/binding/verification refresh, child revisions, targeted podcast revisions, consent-gated recorded-voice cloning, portable exact-transcript voice samples, Fish binary reference synthesis, Google audio AI review/retrieval, Google generation reconciliation, bounded paginated/filterable voice discovery, a terminal configuration wizard, and explicit user host concurrency controls. Reference media is decoded before acceptance or generation. Failed cached publication preserves independent jobs.

| Provider | Adapter paths implemented | Selected live verification |
| --- | --- | --- |
| OpenAI | PNG generation/editing, saved/built-in voice speech, text planning, image review | Image generation/editing, built-in speech, job planning, image review passed |
| Google | Interactions image/reference generation, speech, Lyria music, text planning, filterable paginated prebuilt/custom voice catalog, audio review, saved-interaction retrieval | Image generation and JPEG recovery, speech, instrumental clip, filtered catalog, short audio review passed; retrieval has mocked coverage |
| Fish Audio | Speech with explicit saved voice binding/model header or authorized binary reference samples, filterable paginated owned/licensed public catalogs | Licensed public voice speech and two-page filtered public catalog passed; inline samples have mocked coverage |
| OpenRouter | PNG image generation, text planning, image review | Image generation passed |
| ElevenLabs | Saved voice speech, instrumental/vocal music, sound effects, filterable paginated account voice catalog, descriptive preview design, explicit saved-voice creation/binding and verification lookup, consent-gated recorded cloning | Premade voice speech, short instrumental music, sound effect, catalog, three preview candidates passed; creation/binding/cloning tested with mocks only; two-page filtered catalog passed |

See [live evidence](LIVE_TESTS.md) for exact models, runs, decoded durations, one earlier rejected placeholder voice request, and untested paths. Live tests prove selected API/file handling, not human-perceived quality, exact script pronunciation, character consistency, or all model variants. No saved provider voice was created or cloned, and no candidates were accepted automatically. The descriptive preview test returned three candidates; provider-side creation/binding/verification refresh has offline coverage, with creation/binding untested live.

Ordinary tests use mocked adapters/transports and small local fixtures. They verify contracts, concurrency, limits, identity resolution, authoring validation, audio assembly, cache, revisions, and recovery without paid calls even when credentials exist. The baseline had 100 passing offline tests on macOS; remediation adds independent contract/recovery/storage tests. See the remediation evidence below for the current count. Historical local packages passed their isolated offline checks, including initialization and embedded skill export without the checkout; current 0.3.0 evidence is below. See [packaging](PACKAGING.md). Native CI covers Linux, macOS and Windows. See the 0.3.0 verification notes below for current results.

Verification still requiring external inputs: selected authorized speaker recordings for live cloning/reference synthesis, human listening and repeated character scenes. These do not become passed merely because implementation or mock tests succeeded.

Provider boundaries: voice preview design and saved-voice creation/cloning are implemented for ElevenLabs; other providers can reuse their accessible bindings. Fish supports stateless synthesis from authorized recordings with exact transcripts. Google supports audio review with a 20 MiB inline request ceiling; larger jobs can review individual clips. Google Interactions has retrieval by saved ID; the other adapter endpoints do not expose an implemented media-result retrieval path, so uncertain calls require account reconciliation before explicit retry. Provider professional/biometric verification must be completed by the speaker through the provider's supported account flow. No placeholder verification submission or video adapter is advertised. Public publishing, signing, and notarization are outside the requested local implementation.

The opt-in smoke script makes at most one bounded request per invocation and reports unavailable credentials as skipped. Image identity, voice quality, vocal lyrics, and infographic typography/data accuracy still require review using repeated scenes/clips and the exact intended source content.

Official implementation references: [OpenAI image API](https://developers.openai.com/api/docs/guides/image-generation), [OpenAI speech](https://developers.openai.com/api/reference/resources/audio/subresources/speech/methods/create), [Google images](https://ai.google.dev/gemini-api/docs/image-generation), [Google speech](https://ai.google.dev/gemini-api/docs/speech-generation), [Google music](https://ai.google.dev/gemini-api/docs/music-generation), [Fish TTS](https://docs.fish.audio/api-reference/endpoint/openapi-v1/text-to-speech), [OpenRouter images](https://openrouter.ai/docs/guides/overview/multimodal/image-generation), [ElevenLabs speech](https://elevenlabs.io/docs/api-reference/text-to-speech/convert), [music](https://elevenlabs.io/docs/api-reference/music/compose), [effects](https://elevenlabs.io/docs/api-reference/text-to-sound-effects/convert), [recorded cloning](https://elevenlabs.io/docs/api-reference/voices/ivc/create), [Google audio](https://ai.google.dev/gemini-api/docs/audio), [background retrieval](https://ai.google.dev/gemini-api/docs/background-execution), and [ElevenLabs speech limits](https://elevenlabs.io/docs/overview/models).


The bundled skill includes every implemented command/flag generated from Clap,
all current schemas, denied-permission consent scaffolds, clone/sample templates,
configuration/concurrency guidance, and recovery/review procedures. Project skills
are refreshed explicitly; existing credentials and unrelated assets are preserved.


Character-consistency correction: the first paired forest scene was rejected by
the user for changed identities and is recorded as `needs_revision`. The CLI now
supports explicit identity-reference roles in image jobs and repeated
`--identity-reference` flags. On supported OpenAI models it records and sends high
input fidelity, rejects low-fidelity identity requests before submission, and keeps
reference labels/order in the effective snapshot. Offline tests validate those
controls and the actual multipart body. These controls are prerequisites, not proof
that the generated face/body matches. Corrected visual output still needs user
acceptance; model/provider changes remain explicit.


Original-model continuity is now enforced before submission. Local original
receipts and declared portable origins pin identity references to their source
provider/model and carry compatible settings; conflicting model overrides fail.
Initialization supports complete config files or offline terminal setup, including
models, settings, voices, locales, consistency policy, and limits, before enabling
the initial persona profiles. New template and skill guidance cover these fields.
These changes have offline regression coverage; no additional paid generation was
performed to verify them. The earlier explicit alternative-model comparison is a
historical candidate, not evidence that changing the original model satisfies this
contract. Visual likeness still needs user acceptance.


Read-only API diagnostics are implemented through `doctor --live`, with provider
selection, bounded parallel GET checks, timeouts, secret/account-data redaction,
structured diagnostics schema, and actionable failure classes. Default doctor stays
offline. A bounded live authentication check passed for all five provider endpoints
with HTTP 200; media generation/model access and billing readiness were not tested
by this check. See [connection checks](CONNECTIONS.md) and [live evidence](LIVE_TESTS.md).


## Remediation implementation (0.2.0)

Implemented: v2 job/config/run readers with v1 compatibility and explicit backed-up
migration; ordered casts with canonical references and pinned-model conflict
rejection; selected character/asset planner bundles with immutable reserved inputs;
preview-hash-bound verification suites; read-only bounded model catalogs; user-attested
recovered-response attachment; usage reports including separate voice/review operations;
hash-addressed reference blobs, lightweight metadata indexes and conservative orphan
cleanup; optional podcast pauses/gain/EBU R128 normalization; reference comparison
reports, identity templates and complete generated skill command/schema documentation.

The project now has a local baseline commit and explicit generation defaults.
No remote or publication was added. The initial defaults are previously tested
selections, not a new assertion of current account access.

Prepared acceptance inputs are under `mediaagent/verification/`. The original-model
reference-sheet preview resolves two requests; complete-media acceptance resolves
13 requests; designed-voice preview is a separate one-request operation. The approved
two-image reference-sheet batch has now run successfully, one attempt per image;
both outputs were marked `needs_revision` after agent visual comparison for identity
drift and cropping. Human acceptance remains pending. Other paid remediation batches
remain unexecuted. Repeated scenes correctly remain blocked until references
are selected; saved-voice reuse remains blocked until explicit creation/binding.
Cloning/reference synthesis still needs authorized recordings and consent evidence.

Fresh Codex/Claude automatic-discovery sessions remain pending. Windows native execution is covered by the current CI evidence below.
Generic process-caller and isolated package tests are separate from those gates.
See [acceptance](ACCEPTANCE.md) and [remediation workflows](REMEDIATION.md).


Remediation local evidence: 119 offline tests pass on macOS, with formatting and
strict all-target/all-feature Clippy checks passing. The 0.2.0 Apple Silicon package
passes 12 isolated offline checks. No paid requests were made during implementation.
Subsequent approved character acceptance testing made two image requests, documented
in [acceptance](ACCEPTANCE.md). No provider voice creation, cloned voices, or canonical
reference acceptance were performed.


Final Linux x86-64 verification also passes 119 offline tests, formatting, strict
Clippy and 12 isolated package checks, executing under Docker Desktop emulation.
The verified source manifest matches the current Rust, test, asset and packaging
sources. Mac and Linux 0.2.0 archives/checksums are under `target/package-dist/`;
`target/remediation-verification.json` records their exact hashes. Windows was pending in that historical local snapshot; current native results follow. Neither historical local archive was publicly published or signed.

## Guided setup and current stable models (0.3.0)

The terminal initializer now loops through a searchable, colored checklist, supports explicit draft saving/resumption and masks missing project credential entry. Its default preset selects OpenAI GPT Image 2.5 Flare for images/infographics, Fish s2.1-pro for speech/podcasts and OpenAI GPT-6 Luna for planning. Optional features remain disabled until selected. `config status --json` shares the same local readiness checks; initialization remains offline.

Only reviewed current stable model families may submit new provider requests, including retries and side operations. Historical configuration, model locks, receipts, cached results and response recovery remain readable. Existing retired-model identities need an explicit new character version with human-accepted references. The bundled catalog has dated official-source metadata; live discovery is bounded and cannot authorize execution. GPT Image 2.5 request translation has mocked coverage, including exact model IDs/snapshots, references and supported size/quality settings; paid live acceptance was not performed for this change.

Local macOS formatting, strict Clippy and 140 offline tests pass, including real terminal navigation, masked credentials, draft/resume, cancellation and current-model policy tests. All 140 offline tests, formatting and strict Clippy also pass on native Windows, Linux and macOS runners: [Rust checks](https://github.com/maxiplux/agentmedia/actions/runs/38670199603). Each platform archive passes 13 isolated offline workflows: [native packages and downloadable artifacts](https://github.com/maxiplux/agentmedia/actions/runs/38670199579). Downloaded archive/executable hashes were verified, including the Windows executable’s 8 MiB stack reserve. These runs tested code revision `3395a58`; subsequent changes only update documentation. No claim of human identity/quality acceptance is added.

## Init destination correction (0.3.1)

Reproduced `File exists (os error 17)` when initialization runs beside a downloaded executable named `mediaagent`: its path conflicts with the required `mediaagent/` configuration directory. Setup now checks the destination before collecting choices or writing files and offers another project folder in a terminal. Noninteractive callers receive `init_path_conflict` with corrective arguments. The executable and existing credentials are preserved. `--project PATH init` now selects the destination when the positional path is omitted; an explicit `init PATH` takes precedence. Setup displays the full destination before the checklist and save review.

Formatting, strict Clippy and all 144 offline tests pass on native Windows, Linux and macOS runners: [CI results](https://github.com/maxiplux/agentmedia/actions/runs/38673081222). This includes a real terminal regression that redirects from the colliding folder and saves an unfinished project with both skills. The local macOS package passes 14 isolated checks, including the downloaded-executable collision. The [native package workflow and downloads](https://github.com/maxiplux/agentmedia/actions/runs/38673081221) record each platform’s package checks. These runs test code revision `5318faa`; subsequent changes only update documentation. No paid provider calls were made.


## User installation and updates (0.4.0)

Added offline `install`, read-only `update --check`, stable public-release updates, exact versions, explicit downgrades and offline rollback. Installation uses user-local directories, preserves unrelated executables unless replacement is explicit, and optionally configures user PATH. Updates verify GitHub's asset digest, size and executable version before replacement, keep a previous executable, and preserve project state and skills. Unix shell profile changes preserve existing text; Windows uses the user PATH rather than machine settings. No provider keys or paid calls are used.

Public shell/PowerShell installers select the native target, pin one stable release, verify SHA-256 and version, and call the binary's installer. CI adds native Intel Mac coverage. Tagged release publication validates all native archives and raw executables against the exact clean source commit, uploads a draft, verifies GitHub's stored digests, then publishes it as latest. Checksums are not code signing or notarization.

Local formatting, strict Clippy and 157 offline tests pass, plus 16 isolated native macOS package checks. The auxiliary ignored test entry is invoked in isolated subprocesses by the update/rollback tests. Native CI and tagged release workflows carry per-platform results. Live public download/update verification follows release publication; no media-provider testing is implied. See [installation](INSTALLATION.md).
